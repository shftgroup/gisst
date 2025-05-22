#![allow(unknown_lints, clippy::unnecessary_debug_formatting)]

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
        Creator, Environment, Instance, Object, ObjectRole, Replay, Save, Screenshot, State, Work,
        insert_file_object,
    },
    storage::StorageHandler,
};
use log::info;
use sqlx::PgPool;
use sqlx::pool::PoolOptions;
use sqlx::types::chrono;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::{Uuid, uuid};

#[tokio::main]
async fn main() -> Result<(), GISSTCliError> {
    let args = GISSTCli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    info!("Found config file at path: {}", args.gisst_config_path);
    let cli_config: CLIConfig = CLIConfig::new(&args.gisst_config_path)?;

    info!(
        "Connecting to database: {}",
        cli_config.database.database_url
    );
    let db: PgPool = get_db_by_url(cli_config.database.database_url.to_string()).await?;
    info!("DB connection successful.");
    let storage_root = cli_config.storage.root_folder_path.to_string();
    info!(
        "Storage root is set to: {}",
        cli_config.storage.root_folder_path
    );
    let indexer = gisst::search::MeiliIndexer::new(&args.meili_url, &args.meili_api_key)?;

    match dbg!(args).command {
        Commands::Reindex => reindex(db, &indexer).await?,
        Commands::RecalcSizes => recalc_sizes(db, &storage_root).await?,
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
            BaseSubcommand::Create(create) => create_creator(create, db, &indexer).await?,
        },
        Commands::Environment(environment) => match environment.command {
            BaseSubcommand::Create(create) => create_environment(create, db).await?,
        },
        Commands::Instance(instance) => match instance.command {
            BaseSubcommand::Create(create) => create_instance(create, db, &indexer).await?,
        },
        Commands::Work(work) => match work.command {
            BaseSubcommand::Create(create) => create_work(create, db).await?,
        },
        Commands::State(state) => match state.command {
            BaseSubcommand::Create(create) => {
                create_state(create, db, storage_root, &indexer).await?;
            }
        },
        Commands::Save(save) => match save.command {
            BaseSubcommand::Create(create) => {
                create_save(create, db, storage_root, &indexer).await?;
            }
        },
        Commands::Replay(replay) => match replay.command {
            BaseSubcommand::Create(create) => {
                create_replay(create, db, storage_root, &indexer).await?;
            }
        },
        Commands::Screenshot(screenshot) => match screenshot.command {
            BaseSubcommand::Create(create) => create_screenshot(create, db).await?,
        },
        Commands::CloneV86 {
            instance,
            state,
            depth,
        } => {
            clone_v86_machine(db, instance, state, storage_root, depth, &indexer).await?;
        }
        Commands::AddPatch {
            instance,
            data,
            depth,
        } => {
            add_patched_instance(db, instance, data, storage_root, depth, &indexer).await?;
        }
    }
    Ok(())
}

async fn reindex(
    db: PgPool,
    indexer: &impl gisst::search::SearchIndexer,
) -> Result<(), GISSTCliError> {
    let mut conn = db.acquire().await?;
    let status = indexer.reindex(&mut conn).await;
    log::info!("{status:?}");
    Ok(())
}

async fn recalc_sizes(db: PgPool, storage_root: &str) -> Result<(), GISSTCliError> {
    let count: i64 = {
        let mut conn = db.acquire().await?;
        sqlx::query_scalar!("SELECT count(*) FROM file")
            .fetch_one(conn.as_mut())
            .await?
            .unwrap_or(0)
    };
    info!("count: {count}");
    for i in 0..=(count / 1024) {
        let this_count = (count - i * 1024).min(1024);
        if this_count <= 0 {
            break;
        }
        let mut tx = db.begin().await?;
        // for every file in the file table, recompute file size and file compressed size (if any)
        let files = sqlx::query_as!(
            gisst::models::File,
            "SELECT * FROM file ORDER BY file_id OFFSET $1 LIMIT $2",
            i * 1024,
            this_count
        )
        .fetch_all(tx.as_mut())
        .await?;
        info!("batch {i} : {}", files.len());
        let root = Path::new(storage_root);
        for f in files {
            let path = root.join(Path::new(&f.file_dest_path));
            info!("p {path:?}, fs {}", f.file_dest_path);
            let gz_path = if let Some(e) = path.extension().and_then(|e| e.to_str()) {
                let mut s = e.to_string();
                s.push_str(".gz");
                path.with_extension(s)
            } else {
                path.with_extension("gz")
            };
            let file_size =
                i64::try_from(std::fs::metadata(&path)?.len()).map_err(GISSTCliError::FileSize)?;
            let file_compressed_size = std::fs::metadata(&gz_path)
                .ok()
                .and_then(|md| i64::try_from(md.len()).ok());
            info!(
                "compute {file_size}, {file_compressed_size:?} for {}, {path:?}, {gz_path:?}",
                f.file_id
            );
            // TODO do this a bunch of files at a time
            sqlx::query!(
                r#"UPDATE file SET file_size=$1, file_compressed_size=$2 WHERE file_id=$3 "#,
                file_size,
                file_compressed_size,
                f.file_id,
            )
            .execute(tx.as_mut())
            .await?;
        }
        tx.commit().await?;
    }
    Ok(())
}

async fn clone_v86_machine(
    db: PgPool,
    instance_id: Uuid,
    state_id: Uuid,
    storage_root: String,
    depth: u8,
    indexer: &gisst::search::MeiliIndexer,
) -> Result<Uuid, GISSTCliError> {
    let mut conn = db.acquire().await?;
    // TODO: use transaction
    let uuid = gisst::v86clone::clone_v86_machine(
        &mut conn,
        instance_id,
        state_id,
        &storage_root,
        depth,
        indexer,
    )
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
    indexer: &gisst::search::MeiliIndexer,
) -> Result<(Uuid, Uuid), GISSTCliError> {
    let mut tx = db.begin().await?;
    let inst = Instance::get_by_id(&mut tx, instance_id)
        .await?
        .ok_or(GISSTCliError::RecordNotFound(instance_id))?;
    let work = Work::get_by_id(&mut tx, inst.work_id)
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
    Work::insert(&mut tx, new_work).await?;
    Instance::insert(&mut tx, new_inst, indexer).await?;
    let patch_root = Path::new(&patch_file).parent().unwrap_or(Path::new(""));
    for link in gisst::models::ObjectLink::get_all_for_instance_id(&mut tx, instance_id).await? {
        let role_index =
            u16::try_from(link.object_role_index).map_err(GISSTCliError::InvalidRoleIndex)?;
        if link.object_role == ObjectRole::Content
            && !data
                .files
                .get(role_index as usize)
                .map_or("", String::as_str)
                .is_empty()
        {
            let patch = Path::new(&data.files[role_index as usize]);
            info!("Patching file {patch:?} for index {role_index} @ {link:?}");
            let object_id = insert_file_object(
                &mut tx,
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
                &mut tx,
                object_id,
                derived_inst_id,
                ObjectRole::Content,
                role_index,
            )
            .await?;
        } else {
            Object::link_object_to_instance(
                &mut tx,
                link.object_id,
                derived_inst_id,
                link.object_role,
                role_index,
            )
            .await?;
        }
    }
    tx.commit().await.map_err(GISSTCliError::Sql)?;
    // TODO: index instance here
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
        depth,
        link,
        role,
        role_index,
        file,
        force_uuid,
        cwd,
    }: CreateObject,
    db: PgPool,
    storage_path: String,
) -> Result<(), GISSTCliError> {
    let cwd = cwd.as_deref().map_or(Path::new(""), Path::new);
    let mut conn = db.acquire().await?;
    let path = cwd.join(Path::new(&file));
    let mut source_path = PathBuf::from(path.strip_prefix(cwd).unwrap_or(&path));
    source_path.pop();
    let file_name = path
        .clone()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let object_id = gisst::models::insert_file_object(
        &mut conn,
        &storage_path,
        depth,
        &path,
        Some(file_name.clone()),
        Some(file_name.clone()),
        source_path.to_string_lossy().to_string().replace("./", ""),
        gisst::models::Duplicate::ForceUuid(force_uuid),
    )
    .await?;
    if let Some(inst) = link {
        Object::link_object_to_instance(&mut conn, object_id, inst, role, role_index).await?;
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
    indexer: &gisst::search::MeiliIndexer,
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
                indexer,
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
            ));
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
        (_, _) => environment.environment_config.clone(),
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
    indexer: &impl gisst::search::SearchIndexer,
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
            Creator::insert(&mut conn, creator, indexer).await?;
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
    indexer: &gisst::search::MeiliIndexer,
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
            file_size: file_info.file_size,
            file_compressed_size: file_info.file_compressed_size,
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
    Replay::insert(&mut conn, replay, indexer)
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
    indexer: &gisst::search::MeiliIndexer,
) -> Result<(), GISSTCliError> {
    let file = Path::new(&file);
    if !file.exists() || !file.is_file() {
        return Err(GISSTCliError::CreateState(format!(
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
    let mut source_path = PathBuf::from(file);
    source_path.pop();
    let hash = StorageHandler::get_file_hash(file)?;
    if let Some(_file) = gisst::models::File::get_by_hash(&mut conn, &hash).await? {
        return Err(GISSTCliError::CreateState(
            "File exists in DB already".to_string(),
        ));
    }

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
        file_size: file_info.file_size,
        file_compressed_size: file_info.file_compressed_size,
        created_on,
    };
    gisst::models::File::insert(&mut conn, file_record).await?;

    let state = State {
        state_id: force_uuid.unwrap_or_else(Uuid::new_v4),
        instance_id: link,
        is_checkpoint: replay_id.is_some(),
        file_id: uuid,
        state_name: state_name.clone(),
        state_description: state_description.unwrap_or_else(|| state_name.clone()),
        screenshot_id,
        created_on,
        replay_id,
        creator_id,
        state_replay_index,
        state_derived_from,
        save_derived_from: None,
    };
    State::insert(&mut conn, state, indexer).await?;
    Ok(())
}

async fn create_save(
    CreateSave {
        link,
        depth,
        force_uuid,
        file,
        save_short_desc,
        save_description,
        state_derived_from,
        save_derived_from,
        replay_derived_from,
        creator_id,
        created_on,
    }: CreateSave,
    db: PgPool,
    storage_path: String,
    indexer: &gisst::search::MeiliIndexer,
) -> Result<(), GISSTCliError> {
    let file = Path::new(&file);
    if !file.exists() || !file.is_file() {
        return Err(GISSTCliError::CreateSave(format!(
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
    let mut source_path = PathBuf::from(file);
    source_path.pop();
    let hash = StorageHandler::get_file_hash(file)?;

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
        file_size: file_info.file_size,
        file_compressed_size: file_info.file_compressed_size,
        created_on,
    };
    gisst::models::File::insert(&mut conn, file_record).await?;

    let save = Save {
        save_id: force_uuid.unwrap_or_else(Uuid::new_v4),
        instance_id: link,
        file_id: uuid,
        save_short_desc: save_short_desc.clone(),
        save_description: save_description.unwrap_or_else(|| save_short_desc.clone()),
        created_on,
        creator_id,
        state_derived_from,
        save_derived_from,
        replay_derived_from,
    };
    Save::insert(&mut conn, save, indexer).await?;
    Ok(())
}

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new().connect(&db_url).await
}
