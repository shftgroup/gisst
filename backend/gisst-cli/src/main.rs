mod args;
mod cliconfig;

use crate::args::CreateScreenshot;
use crate::cliconfig::CLIConfig;
use anyhow::Result;
use args::{
    BaseSubcommand, Commands, CreateCreator, CreateEnvironment, CreateInstance, CreateObject,
    CreateReplay, CreateSave, CreateState, CreateWork, GISSTCli, GISSTCliError, PatchData,
};
use clap::Parser;
use gisst::{
    models::{
        insert_file_object, Creator, Environment, Instance, Object, ObjectRole, Replay, Screenshot,
        State, Work,
    },
    storage::StorageHandler,
};
use log::{error, info, warn};
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
        },
        Commands::Creator(creator) => match creator.command {
            BaseSubcommand::Create(create) => create_creator(create, db).await?,
        },
        Commands::Environment(environment) => match environment.command {
            BaseSubcommand::Create(create) => create_environment(create, db).await?,
        },
        Commands::Instance(instance) => match instance.command {
            BaseSubcommand::Create(create) => create_instance(create, db).await?,
        },
        Commands::Work(work) => match work.command {
            BaseSubcommand::Create(create) => create_work(create, db).await?,
        },
        Commands::State(state) => match state.command {
            BaseSubcommand::Create(create) => create_state(create, db, storage_root).await?,
        },
        Commands::Save(save) => match save.command {
            BaseSubcommand::Create(create) => create_save(create, db).await?,
        },
        Commands::Replay(replay) => match replay.command {
            BaseSubcommand::Create(create) => create_replay(create, db, storage_root).await?,
        },
        Commands::Screenshot(screenshot) => match screenshot.command {
            BaseSubcommand::Create(create) => create_screenshot(create, db).await?,
        },
        Commands::CloneV86 {
            instance,
            state,
            depth,
        } => {
            clone_v86_machine(db, instance, state, storage_root, depth).await?;
        }
        Commands::AddPatch {
            instance,
            data,
            depth,
        } => {
            add_patched_instance(db, instance, data, storage_root, depth).await?;
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
    let uuid =
        gisst::v86clone::clone_v86_machine(&mut conn, instance_id, state_id, &storage_root, depth)
            .await?;
    Ok(uuid)
}

/// Returns the new work and instance created for this hack
async fn add_patched_instance(
    db: PgPool,
    instance_id: Uuid,
    patch_file: String,
    storage_root: String,
    depth: u8,
) -> Result<(Uuid, Uuid), GISSTCliError> {
    let mut conn = db.acquire().await?;
    let inst = Instance::get_by_id(&mut conn, instance_id)
        .await?
        .ok_or(GISSTCliError::RecordNotFound(instance_id))?;
    let work = Work::get_by_id(&mut conn, inst.work_id)
        .await?
        .ok_or(GISSTCliError::RecordNotFound(inst.work_id))?;
    let derived_inst_id = Uuid::new_v4();
    let derived_work_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let data: PatchData = {
        let json_data = fs::read_to_string(&patch_file).map_err(GISSTCliError::Io)?;
        serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?
    };
    let new_inst = Instance {
        instance_id: derived_inst_id,
        work_id: derived_work_id,
        derived_from_instance: Some(instance_id),
        created_on: now,
        ..inst
    };
    let new_work = Work {
        work_id: derived_work_id,
        work_derived_from: Some(inst.work_id),
        created_on: now,
        work_version: data.version,
        work_name: data.name,
        work_platform: work.work_platform,
    };
    Work::insert(&mut conn, new_work).await?;
    Instance::insert(&mut conn, new_inst).await?;
    let patch_root = Path::new(&patch_file).parent().unwrap_or(Path::new(""));
    for link in gisst::models::ObjectLink::get_all_for_instance_id(&mut conn, instance_id).await? {
        let role_index =
            u16::try_from(link.object_role_index).map_err(GISSTCliError::InvalidRoleIndex)?;
        if link.object_role == ObjectRole::Content
            && data
                .files
                .get(role_index as usize)
                .map_or("", String::as_str)
                .is_empty()
        {
            let patch = Path::new(&data.files[role_index as usize]);
            let object_id = insert_file_object(
                &mut conn,
                &storage_root,
                depth,
                &patch_root.join(patch),
                Some(link.file_filename),
                None,
                link.file_source_path,
                gisst::models::Duplicate::ReuseData,
            )
            .await?;
            Object::link_object_to_instance(
                &mut conn,
                object_id,
                derived_inst_id,
                ObjectRole::Content,
                role_index,
            )
            .await?;
        } else {
            Object::link_object_to_instance(
                &mut conn,
                link.object_id,
                derived_inst_id,
                link.object_role,
                role_index,
            )
            .await?;
        }
    }
    Ok((derived_work_id, derived_inst_id))
}

async fn link_record(
    record_type: &str,
    source_id: Uuid,
    target_id: Uuid,
    db: PgPool,
    role: Option<ObjectRole>,
    role_index: Option<u16>,
) -> Result<(), GISSTCliError> {
    match (record_type, role, role_index.or(Some(0))) {
        ("object", Some(role), Some(role_index)) => {
            let mut conn = db.acquire().await?;
            Object::link_object_to_instance(&mut conn, source_id, target_id, role, role_index)
                .await?;
            Ok(())
        }
        ("object", _, _) => Err(GISSTCliError::InvalidRecordType(
            "record type object needs a role index for link".to_string(),
        )),
        _ => Err(GISSTCliError::InvalidRecordType(format!(
            "{record_type} is not a valid record type",
        ))),
    }
}

#[allow(clippy::too_many_lines)]
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
        cwd,
        ..
    }: CreateObject,
    db: PgPool,
    storage_path: String,
) -> Result<(), GISSTCliError> {
    let mut valid_paths: Vec<PathBuf> = Vec::new();
    let cwd = cwd.unwrap_or(String::new());
    let cwd = Path::new(&cwd);
    for path in file {
        let p = cwd.join(Path::new(&path));
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
                valid_paths.push(p.clone());
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
        let mut source_path = PathBuf::from(path.strip_prefix(cwd).unwrap_or(path));
        source_path.pop();
        let file_size = i64::try_from(std::fs::metadata(path)?.len())
            .map_err(|_| GISSTCliError::CreateObject("File too big".to_string()))?;
        let mut file_record = gisst::models::File {
            file_id: Uuid::new_v4(),
            file_hash: StorageHandler::get_file_hash(path)?,
            file_filename: path
                .clone()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            file_source_path: source_path.to_string_lossy().to_string().replace("./", ""),
            file_dest_path: String::new(),
            file_size,
            created_on: chrono::Utc::now(),
        };

        let mut object = Object {
            object_id: Uuid::new_v4(),
            file_id: file_record.file_id,
            object_description: Some(
                path.clone()
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
                "Found object {}:{} with matching hash value {} to object {}:{}",
                found_hash.object_id,
                found_file.file_filename,
                found_file.file_hash,
                object.object_id,
                file_record.file_filename,
            );
        }

        if !ignore {
            let mut description = String::new();
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
    let environment: Environment = match (json_file, json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(GISSTCliError::Io)?;
            serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?
        }
        (None, Some(json_str)) => {
            serde_json::from_str(&json_str).map_err(GISSTCliError::JsonParse)?
        }
        (_, _) => {
            return Err(GISSTCliError::CreateEnvironment(
                "Need to provide a JSON string or file to create environment record.".to_string(),
            ))
        }
    };

    let environment_config: Option<serde_json::Value> = match (
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
        .map_or(chrono::Utc::now(), chrono::DateTime::<chrono::Utc>::from);
    let mut conn = db.acquire().await?;
    let file_size = i64::try_from(std::fs::metadata(file)?.len())
        .map_err(|_| GISSTCliError::CreateReplay("file too big".to_string()))?;
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
        .map_or(chrono::Utc::now(), chrono::DateTime::<chrono::Utc>::from);
    let file_size = i64::try_from(std::fs::metadata(file)?.len())
        .map_err(|_| GISSTCliError::CreateState("File too big".to_string()))?;
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
        file_dest_path: String::new(),
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
            "Found state {}:{} with matching hash value {} to state {}:{}",
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

async fn create_save(_c: CreateSave, db: PgPool) -> Result<(), GISSTCliError> {
    let _ = db.acquire().await?;
    todo!();
}

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new().connect(&db_url).await
}
