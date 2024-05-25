mod args;
mod cliconfig;

use crate::args::CreateScreenshot;
use crate::cliconfig::CLIConfig;
use anyhow::Result;
use args::{
    BaseSubcommand, Commands, CreateCreator, CreateEnvironment, CreateImage, CreateInstance,
    CreateObject, CreateReplay, CreateSave, CreateState, CreateWork, DeleteRecord, GISSTCli,
    GISSTCliError,
};
use clap::Parser;
use gisst::{
    model_enums::Framework,
    models::{ObjectRole, Screenshot, StateLink},
};
use gisst::{
    models::{
        Creator, DBHashable, DBModel, Environment, Image, Instance, Object, ObjectLink, Replay,
        Save, State, Work,
    },
    storage::StorageHandler,
};
use log::{debug, error, info, warn};
use sqlx::pool::PoolOptions;
use sqlx::types::chrono;
use sqlx::PgPool;
use std::path::{Path, PathBuf};
use std::{fs, io};
use uuid::{uuid, Uuid};
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<(), GISSTCliError> {
    let args = GISSTCli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    info!(
        "Found config file at path: {}",
        args.gisst_config_path.to_string()
    );
    let cli_config: CLIConfig = CLIConfig::new(&args.gisst_config_path)?;

    info!(
        "Connecting to database: {}",
        cli_config.database.database_url.to_string()
    );
    let db: PgPool = get_db_by_url(cli_config.database.database_url.to_string()).await?;
    info!("DB connection successful.");
    let storage_root = cli_config.storage.root_folder_path.to_string();
    info!(
        "Storage root is set to: {}",
        cli_config.storage.root_folder_path.to_string()
    );

    match dbg!(args).command {
        Commands::Link {
            record_type,
            source_uuid,
            target_uuid,
            role,
            role_index,
        } => link_record(&record_type, source_uuid, target_uuid, db, role, role_index).await?,
        Commands::Object(object) => match object.command {
            BaseSubcommand::Create(create) => create_object(create, db, storage_root).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => {
                delete_file_record::<Object>(delete, db, storage_root).await?
            }
            BaseSubcommand::Export(_export) => (),
        },
        Commands::Creator(creator) => match creator.command {
            BaseSubcommand::Create(create) => create_creator(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_record::<Creator>(delete, db).await?,
            BaseSubcommand::Export(_export) => (),
        },
        Commands::Environment(environment) => match environment.command {
            BaseSubcommand::Create(create) => create_environment(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_record::<Environment>(delete, db).await?,
            BaseSubcommand::Export(_export) => (),
        },
        Commands::Image(image) => match image.command {
            BaseSubcommand::Create(create) => create_image(create, db, storage_root).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => {
                delete_file_record::<Image>(delete, db, storage_root).await?
            }
            BaseSubcommand::Export(_export) => (),
        },
        Commands::Instance(instance) => match instance.command {
            BaseSubcommand::Create(create) => create_instance(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_record::<Instance>(delete, db).await?,
            BaseSubcommand::Export(_export) => (),
        },
        Commands::Work(work) => match work.command {
            BaseSubcommand::Create(create) => create_work(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_record::<Work>(delete, db).await?,
            BaseSubcommand::Export(_export) => (),
        },
        Commands::State(state) => match state.command {
            BaseSubcommand::Create(create) => create_state(create, db, storage_root).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => {
                delete_file_record::<State>(delete, db, storage_root).await?
            }
            BaseSubcommand::Export(_export) => (),
        },
        Commands::Save(save) => match save.command {
            BaseSubcommand::Create(create) => create_save(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => {
                delete_file_record::<Save>(delete, db, storage_root).await?
            }
            BaseSubcommand::Export(_export) => (),
        },
        Commands::Replay(replay) => match replay.command {
            BaseSubcommand::Create(create) => create_replay(create, db, storage_root).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => {
                delete_file_record::<Replay>(delete, db, storage_root).await?
            }
            BaseSubcommand::Export(_export) => (),
        },
        Commands::Screenshot(screenshot) => match screenshot.command {
            BaseSubcommand::Create(create) => create_screenshot(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_record::<Screenshot>(delete, db).await?,
            BaseSubcommand::Export(_export) => (),
        },
        Commands::CloneV86 {
            instance,
            state,
            depth,
        } => {
            clone_v86_machine(db, instance, state, storage_root, depth).await?;
        }
    }
    Ok(())
}

async fn clone_v86_machine(
    db: PgPool,
    instance_id: Uuid,
    state_id: Uuid,
    storage_root: String,
    depth: u8,
) -> Result<Uuid, GISSTCliError> {
    let mut conn = db.acquire().await?;
    let instance = Instance::get_by_id(&mut conn, instance_id)
        .await?
        .ok_or(GISSTCliError::RecordNotFound(instance_id))?;

    let env = Environment::get_by_id(&mut conn, instance.environment_id)
        .await?
        .ok_or(GISSTCliError::RecordNotFound(instance.environment_id))?;
    if env.environment_framework != Framework::V86 {
        return Err(GISSTCliError::InvalidRecordType(
            "Can only clone v86 instances".to_string(),
        ));
    }
    let state = StateLink::get_by_id(&mut conn, state_id)
        .await?
        .ok_or(GISSTCliError::RecordNotFound(state_id))?;
    if state.instance_id != instance_id {
        return Err(GISSTCliError::InvalidRecordType(
            "Can only clone state belonging to the given instance".to_string(),
        ));
    }
    let state_file_path = format!(
        "{storage_root}/{}/{}-{}",
        state.file_dest_path, state.file_hash, state.file_filename
    );
    let objects = ObjectLink::get_all_for_instance_id(&mut conn, instance_id).await?;
    // This unwrap is safe since we know it's a v86 framework environment
    let mut env_json = env.config_for_v86(&objects).unwrap().to_string();
    for obj in objects.iter() {
        let file_path = format!(
            "{storage_root}/{}/{}-{}",
            obj.file_dest_path, obj.file_hash, obj.file_filename
        );
        match obj.object_role {
            ObjectRole::Content => {
                let idx = obj.object_role_index;
                env_json = env_json.replace(&format!("$CONTENT{idx}"), &file_path);
                if idx == 0 {
                    env_json = env_json.replace("$CONTENT\"", &format!("{file_path}\""));
                }
            }
            ObjectRole::Dependency => { /* nop */ }
            ObjectRole::Config => { /*nop*/ }
        }
    }
    env_json = env_json.replace("seabios.bin", "web-dist/v86/bios/seabios.bin");
    env_json = env_json.replace("vgabios.bin", "web-dist/v86/bios/vgabios.bin");
    use std::process::Command;
    println!("Input {env_json}\n{state_file_path}");
    let output = Command::new("node")
        .arg("v86dump/index.js")
        .arg(env_json)
        .arg(state_file_path)
        .output()?;
    let err = String::from_utf8(output.stderr).expect("stderr not utf8");
    println!("{err}");
    let output = String::from_utf8(output.stdout).expect("disk image names not utf-8");
    println!("Output {output}");
    // create the new instance
    let mut instance = instance;
    instance.created_on = chrono::Utc::now();
    instance.derived_from_instance = Some(instance.instance_id);
    instance.derived_from_state = Some(state_id);
    instance.instance_id = Uuid::new_v4();
    let new_id = instance.instance_id;
    Instance::insert(&mut conn, instance)
        .await
        .map_err(GISSTCliError::NewModel)?;

    // add the requisite objects and link them
    // TODO: the ? inside of this loop should get caught and I should delete the outFGSFDS/ folder either way after.
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let (drive, diskpath) = line.split_at(
            line.find(':')
                .unwrap_or_else(|| panic!("Invalid output from v86dump:{line}"))
                + 1,
        );
        let index = match drive {
            "fda:" => 0,
            "fdb:" => 1,
            "hda:" => 2,
            "hdb:" => 3,
            "cdrom:" => 4,
            _ => panic!("Unrecognized drive type {drive}"),
        };
        println!("Linking {drive}{diskpath} as {index}");
        let file_name = Path::new(diskpath)
            .to_path_buf()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let file_size = std::fs::metadata(diskpath)?.len() as i64;
        let mut file_record = gisst::models::File {
            file_id: Uuid::new_v4(),
            file_hash: StorageHandler::get_file_hash(diskpath)?,
            file_filename: file_name.clone(),
            file_source_path: String::new(),
            file_dest_path: Default::default(),
            file_size,
            created_on: chrono::Utc::now(),
        };
        let object = Object {
            object_id: Uuid::new_v4(),
            file_id: file_record.file_id,
            object_description: Some(file_name),
            created_on: chrono::Utc::now(),
        };
        let file_info = StorageHandler::write_file_to_uuid_folder(
            &storage_root,
            depth,
            file_record.file_id,
            &file_record.file_filename,
            diskpath,
        )
        .await?;
        println!(
            "Wrote file {} to {}",
            file_info.dest_filename, file_info.dest_path
        );
        let obj_uuid = object.object_id;
        let file_uuid = file_record.file_id;
        file_record.file_dest_path = file_info.dest_path;
        let file_insert = gisst::models::File::insert(&mut conn, file_record).await;
        let obj_insert = Object::insert(&mut conn, object).await;
        if file_insert.as_ref().and(obj_insert.as_ref()).is_ok() {
            Object::link_object_to_instance(
                &mut conn,
                obj_uuid,
                new_id,
                ObjectRole::Content,
                index,
            )
            .await?;
        } else {
            println!(
                "Could not insert either file or object:\nf:{file_insert:?}\no:{obj_insert:?}"
            );
            StorageHandler::delete_file_with_uuid(
                &storage_root,
                depth,
                file_uuid,
                &file_info.dest_filename,
            )
            .await?;
        }
    }
    Ok(new_id)
}

async fn link_record(
    record_type: &str,
    source_id: Uuid,
    target_id: Uuid,
    db: PgPool,
    role: Option<ObjectRole>,
    role_index: Option<usize>,
) -> Result<(), GISSTCliError> {
    match record_type {
        "object" => {
            let mut conn = db.acquire().await?;
            Object::link_object_to_instance(
                &mut conn,
                source_id,
                target_id,
                role.unwrap(),
                role_index.unwrap_or(0),
            )
            .await?;
        }
        _ => {
            return Err(GISSTCliError::InvalidRecordType(format!(
                "{} is not a valid record type",
                record_type
            )))
        }
    }
    Ok(())
}

async fn create_object(
    CreateObject {
        recursive,
        ignore_description: ignore,
        depth,
        link,
        role,
        role_index,
        file,
        force_uuid,
        ..
    }: CreateObject,
    db: PgPool,
    storage_path: String,
) -> Result<(), GISSTCliError> {
    let mut valid_paths: Vec<PathBuf> = Vec::new();

    for path in file {
        let p = Path::new(&path);

        if p.exists() {
            if p.is_dir() {
                if !recursive {
                    error!("Recursive flag must be set for directory ingest.");
                    return Err(GISSTCliError::CreateObject(
                        "Recursive flag must be set for directory ingest.".to_string(),
                    ));
                }

                for entry in WalkDir::new(p) {
                    let dir_entry = entry?;
                    if !dir_entry.path().is_dir() {
                        valid_paths.push(dir_entry.path().to_path_buf());
                    }
                }
            } else {
                valid_paths.push(p.to_path_buf());
            }
        } else {
            error!("File not found: {}", &p.to_string_lossy());
            return Err(GISSTCliError::CreateObject(format!(
                "File not found: {}",
                &p.to_string_lossy()
            )));
        }
    }

    let mut conn = db.acquire().await?;

    for path in &valid_paths {
        let mut source_path = PathBuf::from(path);
        source_path.pop();
        let file_size = std::fs::metadata(path)?.len() as i64;
        let mut file_record = gisst::models::File {
            file_id: Uuid::new_v4(),
            file_hash: StorageHandler::get_file_hash(path)?,
            file_filename: path
                .to_path_buf()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            file_source_path: source_path.to_string_lossy().to_string().replace("./", ""),
            file_dest_path: Default::default(),
            file_size,
            created_on: chrono::Utc::now(),
        };

        let mut object = Object {
            object_id: Uuid::new_v4(),
            file_id: file_record.file_id,
            object_description: Some(
                path.to_path_buf()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            ),
            created_on: chrono::Utc::now(),
        };

        // DEBUG ONLY!! Need to find a more elegant solution
        if valid_paths.len() == 1 {
            object.object_id = force_uuid;
        }

        if let Some(found_hash) = Object::get_by_hash(&mut conn, &file_record.file_hash).await? {
            let found_file = gisst::models::File::get_by_id(&mut conn, found_hash.file_id)
                .await?
                .unwrap();
            warn!(
                "Found object {}:{} with matching hash value {} to object {}:{}, skipping...",
                found_hash.object_id,
                found_file.file_filename,
                found_file.file_hash,
                object.object_id,
                file_record.file_filename,
            );
            continue;
        }

        if !ignore {
            let mut description = Default::default();
            println!(
                "Please enter an object description for file: {}",
                &path.to_string_lossy()
            );
            io::stdin().read_line(&mut description).ok();
            object.object_description = Some(description.trim().to_string());
        }

        match StorageHandler::write_file_to_uuid_folder(
            &storage_path,
            depth,
            file_record.file_id,
            &file_record.file_filename,
            path,
        )
        .await
        {
            Ok(file_info) => {
                info!(
                    "Wrote file {} to {}",
                    file_info.dest_filename, file_info.dest_path
                );
                let obj_uuid = object.object_id;
                let file_uuid = file_record.file_id;
                file_record.file_dest_path = file_info.dest_path;

                if gisst::models::File::insert(&mut conn, file_record)
                    .await
                    .is_ok()
                    && Object::insert(&mut conn, object).await.is_ok()
                {
                    if let Some(link) = link {
                        Object::link_object_to_instance(
                            &mut conn, obj_uuid, link, role, role_index,
                        )
                        .await?;
                    }
                } else {
                    StorageHandler::delete_file_with_uuid(
                        &storage_path,
                        depth,
                        file_uuid,
                        &file_info.dest_filename,
                    )
                    .await?;
                }
            }
            Err(e) => {
                error!("Error writing object file to database, aborting...\n{e}");
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn delete_record<T: DBModel>(d: DeleteRecord, db: PgPool) -> Result<(), GISSTCliError> {
    let mut conn = db.acquire().await?;
    T::delete_by_id(&mut conn, d.id)
        .await
        .map_err(GISSTCliError::Sql)?;
    info!("Deleted record with uuid {}", d.id);
    Ok(())
}

async fn delete_file_record<T: DBModel + DBHashable>(
    d: DeleteRecord,
    db: PgPool,
    storage_path: String,
) -> Result<(), GISSTCliError> {
    let mut conn = db.acquire().await?;
    let model = T::get_by_id(&mut conn, d.id)
        .await?
        .ok_or(GISSTCliError::RecordNotFound(d.id))?;
    let linked_file_record = gisst::models::File::get_by_id(&mut conn, *model.file_id())
        .await?
        .ok_or(GISSTCliError::RecordNotFound(*model.file_id()))?;

    T::delete_by_id(&mut conn, d.id)
        .await
        .map_err(GISSTCliError::Sql)?;
    info!("Deleted record with uuid {}", d.id);

    gisst::models::File::delete_by_id(&mut conn, linked_file_record.file_id)
        .await
        .map_err(GISSTCliError::Sql)?;

    info!("Deleted file record with uuid {}", d.id);

    debug!(
        "File path depth is set to {}",
        StorageHandler::get_folder_depth_from_path(
            Path::new(&linked_file_record.file_dest_path),
            None
        )
    );

    StorageHandler::delete_file_with_uuid(
        &storage_path,
        StorageHandler::get_folder_depth_from_path(
            Path::new(&linked_file_record.file_dest_path),
            None,
        ),
        linked_file_record.file_id,
        &StorageHandler::get_dest_filename(
            &linked_file_record.file_hash,
            &linked_file_record.file_filename,
        ),
    )
    .await
    .map_err(GISSTCliError::Io)?;

    info!(
        "Deleted file at path:{}",
        Path::new(&linked_file_record.file_dest_path)
            .join(Path::new(&StorageHandler::get_dest_filename(
                &linked_file_record.file_hash,
                &linked_file_record.file_filename
            )))
            .to_string_lossy()
    );
    Ok(())
}

// async fn update_object(_u: &UpdateObject, _db: PgPool) -> Result<(), GISSTCliError> {
//     Ok(())
// }

async fn create_instance(
    CreateInstance {
        json_file,
        json_string,
        instance_config_json_file,
        instance_config_json_string,
        ..
    }: CreateInstance,
    db: PgPool,
) -> Result<(), GISSTCliError> {
    let instance_from_json: Option<Instance> = match (json_file, json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(GISSTCliError::Io)?;
            Some(serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?)
        }
        (None, Some(json_str)) => {
            Some(serde_json::from_str(&json_str).map_err(GISSTCliError::JsonParse)?)
        }
        (_, _) => None,
    };

    let instance_config_json: Option<serde_json::Value> =
        match (&instance_config_json_file, &instance_config_json_string) {
            (Some(file_path), None) => {
                let json_data = fs::read_to_string(file_path).map_err(GISSTCliError::Io)?;
                Some(serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?)
            }
            (None, Some(json_str)) => {
                Some(serde_json::from_str(json_str).map_err(GISSTCliError::JsonParse)?)
            }
            (_, _) => None,
        };

    match (instance_from_json, instance_config_json) {
        (Some(instance), instance_config) => {
            let mut conn = db.acquire().await?;
            Instance::insert(
                &mut conn,
                Instance {
                    instance_config,
                    ..instance
                },
            )
            .await
            .map_err(GISSTCliError::NewModel)?;
            Ok(())
        }
        _ => Err(GISSTCliError::CreateInstance(
            "Need to provide a JSON string or file to create instance record.".to_string(),
        )),
    }
}

async fn create_environment(
    CreateEnvironment {
        json_file,
        json_string,
        environment_config_json_file,
        environment_config_json_string,
        ..
    }: CreateEnvironment,
    db: PgPool,
) -> Result<(), GISSTCliError> {
    let environment_from_json: Option<Environment> = match (json_file, json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(GISSTCliError::Io)?;
            Some(serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?)
        }
        (None, Some(json_str)) => {
            Some(serde_json::from_str(&json_str).map_err(GISSTCliError::JsonParse)?)
        }
        (_, _) => None,
    };

    let environment_config_json: Option<serde_json::Value> = match (
        &environment_config_json_file,
        &environment_config_json_string,
    ) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(GISSTCliError::Io)?;
            Some(serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?)
        }
        (None, Some(json_str)) => {
            Some(serde_json::from_str(json_str).map_err(GISSTCliError::JsonParse)?)
        }
        (_, _) => None,
    };

    match (environment_from_json, environment_config_json) {
        (Some(environment), environment_config) => {
            let mut conn = db.acquire().await?;
            Environment::insert(
                &mut conn,
                Environment {
                    environment_config,
                    ..environment
                },
            )
            .await
            .map_err(GISSTCliError::NewModel)?;
            Ok(())
        }
        _ => Err(GISSTCliError::CreateInstance(
            "Need to provide a JSON string or file to create environment record.".to_string(),
        )),
    }
}

async fn create_creator(
    CreateCreator {
        json_file,
        json_string,
        ..
    }: CreateCreator,
    db: PgPool,
) -> Result<(), GISSTCliError> {
    let creator_from_json: Option<Creator> = match (json_file, json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(GISSTCliError::Io)?;
            Some(serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?)
        }
        (None, Some(json_value)) => {
            Some(serde_json::from_str(&json_value).map_err(GISSTCliError::JsonParse)?)
        }
        (_, _) => unreachable!(),
    };

    match creator_from_json {
        Some(creator) => {
            let mut conn = db.acquire().await?;
            Creator::insert(&mut conn, creator).await?;
            Ok(())
        }
        None => Err(GISSTCliError::CreateCreator(
            "Please provide JSON to parse for creating a creator record.".to_string(),
        )),
    }
}

async fn create_work(
    CreateWork {
        json_file,
        json_string,
        ..
    }: CreateWork,
    db: PgPool,
) -> Result<(), GISSTCliError> {
    let work_from_json: Option<Work> = match (&json_file, &json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(GISSTCliError::Io)?;
            Some(serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?)
        }
        (None, Some(json_str)) => {
            Some(serde_json::from_str(json_str).map_err(GISSTCliError::JsonParse)?)
        }
        (_, _) => unreachable!(),
    };

    match work_from_json {
        Some(work) => {
            let mut conn = db.acquire().await?;
            Work::insert(&mut conn, work).await?;
            Ok(())
        }
        None => Err(GISSTCliError::CreateWork(
            "Please provide JSON to parse for creating a work record.".to_string(),
        )),
    }
}

async fn create_image(
    CreateImage {
        ignore_description: ignore,
        depth,
        link,
        file,
        force_uuid,
        ..
    }: CreateImage,
    db: PgPool,
    storage_path: String,
) -> Result<(), GISSTCliError> {
    let mut valid_paths: Vec<PathBuf> = Vec::new();

    for path in file {
        let p = Path::new(&path);

        if p.exists() {
            valid_paths.push(p.to_path_buf());
        } else {
            error!("File not found: {}", &p.to_string_lossy());
            return Err(GISSTCliError::CreateImage(format!(
                "File not found: {}",
                &p.to_string_lossy()
            )));
        }
    }

    let mut conn = db.acquire().await?;

    for path in &valid_paths {
        let mut source_path = PathBuf::from(path);
        source_path.pop();
        let file_size = std::fs::metadata(path)?.len() as i64;
        let mut file_record = gisst::models::File {
            file_id: Uuid::new_v4(),
            file_hash: StorageHandler::get_file_hash(path)?,
            file_filename: path
                .to_path_buf()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            file_source_path: source_path.to_string_lossy().to_string().replace("./", ""),
            file_dest_path: Default::default(),
            file_size,
            created_on: chrono::Utc::now(),
        };
        let mut image = Image {
            image_id: Uuid::new_v4(),
            file_id: file_record.file_id,
            image_description: Some(path.to_path_buf().to_string_lossy().to_string()),
            image_config: None,
            created_on: chrono::Utc::now(),
        };

        // DEBUG ONLY!! Need to find a more elegant solution
        if valid_paths.len() == 1 {
            image.image_id = force_uuid;
        }

        if let Some(found_hash) = Image::get_by_hash(&mut conn, &file_record.file_hash).await? {
            let found_file = gisst::models::File::get_by_id(&mut conn, found_hash.file_id)
                .await?
                .unwrap();
            warn!(
                "Found image {}:{} with matching hash value {} to image {}:{}, skipping...",
                found_hash.image_id,
                found_file.file_filename,
                found_file.file_hash,
                image.image_id,
                file_record.file_filename,
            );
            continue;
        }

        if !ignore {
            let mut description = Default::default();
            println!(
                "Please enter an image description for file: {}",
                &path.to_string_lossy()
            );
            io::stdin().read_line(&mut description).ok();
            image.image_description = Some(description.trim().to_string());
        }

        match StorageHandler::write_file_to_uuid_folder(
            &storage_path,
            depth,
            file_record.file_id,
            &file_record.file_filename,
            path,
        )
        .await
        {
            Ok(file_info) => {
                info!(
                    "Wrote file {} to {}",
                    file_info.dest_filename, file_info.dest_path
                );
                let image_uuid = image.image_id;
                let file_uuid = file_record.file_id;
                file_record.file_dest_path = file_info.dest_path;

                if gisst::models::File::insert(&mut conn, file_record)
                    .await
                    .is_ok()
                    && Image::insert(&mut conn, image).await.is_ok()
                {
                    if let Some(link) = link {
                        Image::link_image_to_environment(&mut conn, image_uuid, link).await?;
                    }
                } else {
                    StorageHandler::delete_file_with_uuid(
                        &storage_path,
                        depth,
                        file_uuid,
                        &file_info.dest_filename,
                    )
                    .await?;
                }
            }
            Err(e) => {
                error!("Error writing image file to database, aborting...\n{e}");
            }
        }
    }
    Ok(())
}

async fn create_replay(
    CreateReplay {
        link,
        depth,
        force_uuid,
        file,
        creator_id,
        replay_forked_from,
        replay_name,
        replay_description,
        created_on,
    }: CreateReplay,
    db: PgPool,
    storage_path: String,
) -> Result<(), GISSTCliError> {
    let file = Path::new(&file);
    if !file.exists() || !file.is_file() {
        return Err(GISSTCliError::CreateReplay(format!(
            "File not found: {}",
            file.to_string_lossy()
        )));
    }
    let created_on = created_on
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|e| GISSTCliError::CreateReplay(e.to_string()))
                .ok()
        })
        .map(chrono::DateTime::<chrono::Utc>::from)
        .unwrap_or(chrono::Utc::now());
    let mut conn = db.acquire().await?;
    let file_size = std::fs::metadata(file)?.len() as i64;
    let hash = StorageHandler::get_file_hash(file)?;
    let file_id = if let Some(file) = gisst::models::File::get_by_hash(&mut conn, &hash).await? {
        info!("File exists in DB already");
        file.file_id
    } else {
        let uuid = Uuid::new_v4();

        let filename = file.file_name().unwrap().to_string_lossy().to_string();

        let file_info =
            StorageHandler::write_file_to_uuid_folder(&storage_path, depth, uuid, &filename, file)
                .await?;
        info!(
            "Wrote file {} to {}",
            file_info.dest_filename, file_info.dest_path
        );

        let mut source_path = PathBuf::from(file);
        source_path.pop();

        let file_record = gisst::models::File {
            file_id: uuid,
            file_hash: hash,
            file_filename: filename,
            file_source_path: source_path.to_string_lossy().to_string().replace("./", ""),
            file_dest_path: file_info.dest_path,
            file_size,
            created_on,
        };
        gisst::models::File::insert(&mut conn, file_record).await?;
        uuid
    };
    info!("File ID: {file_id}");
    let replay = Replay {
        replay_id: force_uuid.unwrap_or_else(Uuid::new_v4),
        instance_id: link,
        creator_id: creator_id.unwrap_or_else(|| uuid!("00000000-0000-0000-0000-000000000000")),
        replay_name: replay_name.unwrap_or_else(|| "a replay".to_string()),
        replay_description: replay_description
            .unwrap_or_else(|| "a replay description".to_string()),
        replay_forked_from,
        file_id,
        created_on,
    };
    Replay::insert(&mut conn, replay)
        .await
        .map(|_| ())
        .map_err(GISSTCliError::NewModel)
}

async fn create_screenshot(
    CreateScreenshot { file, force_uuid }: CreateScreenshot,
    db: PgPool,
) -> Result<(), GISSTCliError> {
    let file = Path::new(&file);

    if !file.exists() || !file.is_file() {
        return Err(GISSTCliError::CreateScreenshot(format!(
            "File not found: {}",
            file.to_string_lossy()
        )));
    }

    let mut conn = db.acquire().await?;

    Screenshot::insert(
        &mut conn,
        Screenshot {
            screenshot_data: std::fs::read(file)?,
            screenshot_id: force_uuid.unwrap_or_else(Uuid::new_v4),
        },
    )
    .await
    .map_err(GISSTCliError::NewModel)?;

    info!("Wrote screenshot {} to database.", file.to_string_lossy());
    Ok(())
}

async fn create_state(
    CreateState {
        link,
        depth,
        force_uuid,
        file,
        state_name,
        state_description,
        screenshot_id,
        replay_id,
        creator_id,
        state_replay_index,
        state_derived_from,
        created_on,
    }: CreateState,
    db: PgPool,
    storage_path: String,
) -> Result<(), GISSTCliError> {
    let file = Path::new(&file);
    if !file.exists() || !file.is_file() {
        return Err(GISSTCliError::CreateState(format!(
            "File not found: {}",
            file.to_string_lossy()
        )));
    }

    let mut conn = db.acquire().await?;
    let mut source_path = PathBuf::from(file);
    source_path.pop();
    let created_on = created_on
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|e| GISSTCliError::CreateState(e.to_string()))
                .ok()
        })
        .map(chrono::DateTime::<chrono::Utc>::from)
        .unwrap_or(chrono::Utc::now());
    let file_size = std::fs::metadata(file)?.len() as i64;
    let mut file_record = gisst::models::File {
        file_id: Uuid::new_v4(),
        file_hash: StorageHandler::get_file_hash(file)?,
        file_filename: file
            .to_path_buf()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        file_source_path: source_path.to_string_lossy().to_string().replace("./", ""),
        file_dest_path: Default::default(),
        file_size,
        created_on,
    };
    let state = State {
        state_id: force_uuid.unwrap_or_else(Uuid::new_v4),
        instance_id: link,
        is_checkpoint: replay_id.is_some(),
        file_id: file_record.file_id,
        state_name: state_name.clone(),
        state_description: state_description.unwrap_or_else(|| state_name.clone()),
        screenshot_id,
        created_on,
        replay_id,
        creator_id,
        state_replay_index,
        state_derived_from,
    };
    if let Some(found_hash) = State::get_by_hash(&mut conn, &file_record.file_hash).await? {
        let found_file = gisst::models::File::get_by_id(&mut conn, found_hash.file_id)
            .await?
            .unwrap();
        return Err(GISSTCliError::CreateState(format!(
            "Found state {}:{} with matching hash value {} to state {}:{}, skipping...",
            found_hash.state_id,
            found_file.file_filename,
            found_file.file_hash,
            state.state_id,
            file_record.file_filename,
        )));
    }

    match StorageHandler::write_file_to_uuid_folder(
        &storage_path,
        depth,
        file_record.file_id,
        &file_record.file_filename,
        file,
    )
    .await
    {
        Ok(file_info) => {
            info!(
                "Wrote file {} to {}",
                file_info.dest_filename, file_info.dest_path
            );
            let file_uuid = file_record.file_id;
            file_record.file_dest_path = file_info.dest_path;
            gisst::models::File::insert(&mut conn, file_record).await?;
            if let Err(e) = State::insert(&mut conn, state).await {
                StorageHandler::delete_file_with_uuid(
                    &storage_path,
                    depth,
                    file_uuid,
                    &file_info.dest_filename,
                )
                .await?;
                return Err(GISSTCliError::NewModel(e));
            };
        }
        Err(e) => {
            error!("Error writing state file to database, aborting...\n{e}");
        }
    }
    Ok(())
}
async fn create_save(_c: CreateSave, _db: PgPool) -> Result<(), GISSTCliError> {
    Ok(())
}

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new().connect(&db_url).await
}
