use crate::error::V86Clone;
use crate::model_enums::Framework;
use crate::models::{Environment, File, Instance, Object, ObjectLink, ObjectRole, StateLink};
use crate::storage::StorageHandler;
use log::{error, info};
use sqlx::PgConnection;
use std::path::Path;
use uuid::Uuid;

#[allow(clippy::too_many_lines, clippy::missing_errors_doc)]
#[tracing::instrument(skip(conn, indexer))]
pub async fn clone_v86_machine(
    conn: &mut PgConnection,
    instance_id: Uuid,
    state_id: Uuid,
    storage_root: &str,
    depth: u8,
    indexer: &impl crate::search::SearchIndexer,
) -> Result<Uuid, V86Clone> {
    use crate::inc_metric;
    use std::process::Command;
    inc_metric!(conn, v86_clones, 1);
    let instance = Instance::get_by_id(conn, instance_id)
        .await?
        .ok_or(V86Clone::InstanceNotFound(instance_id))?;

    let env = Environment::get_by_id(conn, instance.environment_id)
        .await?
        .ok_or(V86Clone::EnvironmentNotFound(instance.environment_id))?;
    if env.environment_framework != Framework::V86 {
        return Err(V86Clone::WrongEnvironmentType);
    }
    let state = StateLink::get_by_id(conn, state_id)
        .await?
        .ok_or(V86Clone::StateNotFound(state_id))?;
    if state.instance_id != instance_id {
        return Err(V86Clone::WrongInstanceForState);
    }
    let state_file_path = format!("{storage_root}/{}", state.file_dest_path);
    let objects = ObjectLink::get_all_for_instance_id(conn, instance_id).await?;
    let mut env_json = env
        .environment_config
        .ok_or(V86Clone::EnvironmentInvalid(instance.environment_id))?
        .to_string();
    for obj in &objects {
        let file_path = format!("{storage_root}/{}", obj.file_dest_path);
        if let ObjectRole::Content = obj.object_role {
            let idx = obj.object_role_index;
            env_json = env_json.replace(&format!("$CONTENT{idx}"), &file_path);
            if idx == 0 {
                env_json = env_json.replace("$CONTENT\"", &format!("{file_path}\""));
            }
        }
    }
    env_json = env_json.replace("seabios.bin", "web-dist/v86/bios/seabios.bin");
    env_json = env_json.replace("vgabios.bin", "web-dist/v86/bios/vgabios.bin");
    info!("Input {env_json}\n{state_file_path}");
    let now = std::time::Instant::now();
    let proc_output = Command::new("node")
        .arg("v86dump/index.js")
        .arg(env_json)
        .arg(state_file_path)
        .output()?;
    let delta = now.elapsed().as_secs();
    if delta > 0 {
        // This is fine since delta is > 0 and smaller than 2^63
        #[allow(clippy::cast_possible_wrap)]
        let delta = delta as i64;
        inc_metric!(conn, v86_clone_dump_time, delta);
    }
    let err = String::from_utf8(proc_output.stderr)?;
    info!("{err}");
    let output = String::from_utf8(proc_output.stdout)?;
    info!("Output {output}");
    if !proc_output.status.success() {
        return Err(V86Clone::V86Dump(err));
    }
    // create the new instance
    let mut instance = instance;
    instance.created_on = chrono::Utc::now();
    instance.derived_from_instance = Some(instance.instance_id);
    instance.derived_from_state = Some(state_id);
    instance.instance_id = Uuid::new_v4();
    let new_id = instance.instance_id;
    let mut temp_folder = None;
    Instance::insert(conn, instance, indexer).await?;
    // add the requisite objects and link them
    // TODO: the ? inside of this loop should get caught and I should delete the outFGSFDS/ folder either way after.
    let mut content_index = 0;
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let (drive, diskpath) = match line.find(':').ok_or(V86Clone::V86Dump(
            "Invalid output from v86dump:{line}".to_owned(),
        )) {
            Ok(split) => line.split_at(split + 1),
            Err(err) => {
                if let Some(temp) = temp_folder {
                    std::fs::remove_dir_all(temp)?;
                }
                return Err(err);
            }
        };
        temp_folder = Some(
            Path::new(diskpath)
                .parent()
                .ok_or_else(|| V86Clone::DiskNotFound(diskpath.to_string()))?,
        );
        info!("Linking {drive}{diskpath} as CONTENT{content_index}");
        let file_name = Path::new(diskpath)
            .to_path_buf()
            .file_name()
            .ok_or_else(|| V86Clone::DiskNotFound(diskpath.to_string()))?
            .to_string_lossy()
            .to_string();
        let file_size =
            i64::try_from(std::fs::metadata(diskpath)?.len()).map_err(V86Clone::DiskTooBig)?;
        let mut file_record = crate::models::File {
            file_id: Uuid::new_v4(),
            file_hash: StorageHandler::get_file_hash(diskpath)?,
            file_filename: file_name.clone(),
            file_source_path: String::new(),
            file_dest_path: String::new(),
            file_size,
            file_compressed_size: None,
            created_on: chrono::Utc::now(),
        };
        let object = Object {
            object_id: Uuid::new_v4(),
            file_id: file_record.file_id,
            object_description: Some(file_name),
            created_on: chrono::Utc::now(),
        };
        let file_info = StorageHandler::write_file_to_uuid_folder(
            storage_root,
            depth,
            file_record.file_id,
            &file_record.file_filename,
            diskpath,
        )
        .await?;
        info!(
            "Wrote file {} to {}",
            file_info.dest_filename, file_info.dest_path
        );
        let obj_uuid = object.object_id;
        let file_uuid = file_record.file_id;
        file_record.file_dest_path = file_info.dest_path;
        let file_insert = File::insert(conn, file_record).await;
        let obj_insert = Object::insert(conn, object).await;
        if file_insert.as_ref().and(obj_insert.as_ref()).is_ok() {
            if let Err(link_err) = Object::link_object_to_instance(
                conn,
                obj_uuid,
                new_id,
                ObjectRole::Content,
                content_index,
            )
            .await
            {
                error!("Could not link object {link_err:?}");
                if let Some(temp) = temp_folder {
                    std::fs::remove_dir_all(temp)?;
                }
                return Err(link_err.into());
            }
            content_index += 1;
        } else {
            error!("Could not insert either file or object:\nf:{file_insert:?}\no:{obj_insert:?}");
            StorageHandler::delete_file_with_uuid(
                storage_root,
                depth,
                file_uuid,
                &file_info.dest_filename,
            )
            .await?;
            if let Some(temp) = temp_folder {
                std::fs::remove_dir_all(temp)?;
            }
            return Err(V86Clone::IncompleteClone(new_id));
        }
    }
    if let Some(temp) = temp_folder {
        std::fs::remove_dir_all(temp)?;
    }
    Ok(new_id)
}
