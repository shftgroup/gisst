mod args;

use anyhow::Result;
use args::{
    BaseSubcommand, CreateEnvironment, CreateImage, CreateInstance, CreateObject, CreateReplay,
    CreateSave, CreateState, CreateWork, CreateCreator, DeleteRecord,
    GISSTCli, GISSTCliError, RecordType,
};
use clap::Parser;
use gisstlib::{
    models::{DBHashable, DBModel, Environment, Image, Instance, Object, State, Save, Work, Replay, Creator},
    storage::StorageHandler,
};
use log::{debug, error, info, warn};
use sqlx::pool::PoolOptions;
use sqlx::PgPool;
use std::path::{Path, PathBuf};
use std::{fs, fs::read, io};
use uuid::{Uuid,uuid};
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<(), GISSTCliError> {
    let args = GISSTCli::parse();
    info!(
        "Connecting to database: {}",
        args.gisst_cli_db_url.to_string()
    );
    let db: PgPool = get_db_by_url(args.gisst_cli_db_url.to_string()).await?;
    info!("DB connection successful.");
    let storage_root = args.gisst_storage_root_path.to_string();
    info!(
        "Storage root is set to: {}",
        args.gisst_storage_root_path.to_string()
    );

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    match dbg!(args).record_type {
        RecordType::Object(object) => match object.command {
            BaseSubcommand::Create(create) => create_object(create, db, storage_root).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_file_record::<Object>(delete, db, storage_root).await?,
            BaseSubcommand::Export(_export) => (),
        },
        RecordType::Creator(creator) => match creator.command {
            BaseSubcommand::Create(create) => create_creator(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_record::<Creator>(delete, db).await?,
            BaseSubcommand::Export(_export) => (),
        },
        RecordType::Environment(environment) => match environment.command {
            BaseSubcommand::Create(create) => create_environment(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_record::<Environment>(delete, db).await?,
            BaseSubcommand::Export(_export) => (),
        },
        RecordType::Image(image) => match image.command {
            BaseSubcommand::Create(create) => create_image(create, db, storage_root).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_file_record::<Image>(delete, db, storage_root).await?,
            BaseSubcommand::Export(_export) => (),
        },
        RecordType::Instance(instance) => match instance.command {
            BaseSubcommand::Create(create) => create_instance(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_record::<Instance>(delete, db).await?,
            BaseSubcommand::Export(_export) => (),
        },
        RecordType::Work(work) => match work.command {
            BaseSubcommand::Create(create) => create_work(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_record::<Work>(delete, db).await?,
            BaseSubcommand::Export(_export) => (),
        },
        RecordType::State(state) => match state.command {
            BaseSubcommand::Create(create) => create_state(create, db, storage_root).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_file_record::<State>(delete, db, storage_root).await?,
            BaseSubcommand::Export(_export) => (),
        },
        RecordType::Save(save) => match save.command {
            BaseSubcommand::Create(create) => create_save(create, db).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_file_record::<Save>(delete, db, storage_root).await?,
            BaseSubcommand::Export(_export) => (),
        },
        RecordType::Replay(replay) => match replay.command {
            BaseSubcommand::Create(create) => create_replay(create, db, storage_root).await?,
            BaseSubcommand::Update(_update) => (),
            BaseSubcommand::Delete(delete) => delete_file_record::<Replay>(delete, db, storage_root).await?,
            BaseSubcommand::Export(_export) => (),
        },
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
        let data = &read(path)?;
        let mut object = Object {
            object_id: Uuid::new_v4(),
            object_hash: StorageHandler::get_md5_hash(data),
            object_filename: path
                .to_path_buf()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            object_source_path: path.to_path_buf().to_string_lossy().to_string(),
            object_dest_path: Default::default(),
            object_description: Some(path.to_path_buf().to_string_lossy().to_string()),
            created_on: None,
        };

        // DEBUG ONLY!! Need to find a more elegant solution
        if valid_paths.len() == 1 {
            object.object_id = force_uuid;
        }

        if let Some(found_hash) = Object::get_by_hash(&mut conn, &object.object_hash).await? {
            warn!(
                "Found object {}:{} with matching hash value {} to object {}:{}, skipping...",
                found_hash.object_id,
                found_hash.object_filename,
                found_hash.object_hash,
                object.object_id,
                object.object_filename,
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

        let s_handler = StorageHandler::init(storage_path.to_string(), depth);

        match s_handler
            .write_file_to_uuid_folder(object.object_id, &object.object_filename, data)
            .await
        {
            Ok(file_info) => {
                info!(
                    "Wrote file {} to {}",
                    file_info.dest_filename, file_info.dest_path
                );
                let obj_uuid = object.object_id;
                object.object_dest_path = file_info.dest_path;
                if let Ok(object) = Object::insert(&mut conn, object).await {
                    Object::link_object_to_instance(&mut conn, object.object_id, link, role)
                        .await?;
                } else {
                    s_handler
                        .delete_file_with_uuid(obj_uuid, &file_info.dest_filename)
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
async fn delete_record<T: DBModel>(d: DeleteRecord, db: PgPool) -> Result<(), GISSTCliError>
where
    T: DBModel,
{
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

    T::delete_by_id(&mut conn, d.id)
        .await
        .map_err(GISSTCliError::Sql)?;
    info!("Deleted record with uuid {}", d.id);

    debug!(
        "File path depth is set to {}",
        StorageHandler::get_folder_depth_from_path(Path::new(model.dest_file_path()), None)
    );

    StorageHandler::init(
        storage_path,
        StorageHandler::get_folder_depth_from_path(Path::new(model.dest_file_path()), None),
    )
    .delete_file_with_uuid(
        d.id,
        &StorageHandler::get_dest_filename(model.hash(), model.filename()),
    )
    .await
    .map_err(GISSTCliError::Io)?;

    info!(
        "Deleted object file at path:{}",
        Path::new(model.dest_file_path())
            .join(Path::new(&StorageHandler::get_dest_filename(
                model.hash(),
                model.filename()
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
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value).map_err(GISSTCliError::JsonParse)?)
        }
        (_, _) => None,
    };

    let instance_config_json: Option<serde_json::Value> =
        match (&instance_config_json_file, &instance_config_json_string) {
            (Some(file_path), None) => {
                let json_data = fs::read_to_string(file_path).map_err(GISSTCliError::Io)?;
                Some(serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?)
            }
            (None, Some(json_value)) => {
                Some(serde_json::from_value(json_value.clone()).map_err(GISSTCliError::JsonParse)?)
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
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value).map_err(GISSTCliError::JsonParse)?)
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
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value.clone()).map_err(GISSTCliError::JsonParse)?)
        }
        (_, _) => None,
    };

    match (environment_from_json, environment_config_json) {
        (Some(environment), environment_config) => {
            let mut conn = db.acquire().await?;
            println!("{:?}", environment);
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
    let creator_from_json: Option<Creator> = match (&json_file, &json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(GISSTCliError::Io)?;
            Some(serde_json::from_str(&json_data).map_err(GISSTCliError::JsonParse)?)
        }
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value.clone()).map_err(GISSTCliError::JsonParse)?)
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
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value.clone()).map_err(GISSTCliError::JsonParse)?)
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
        let data = &read(path)?;
        let mut image = Image {
            image_id: Uuid::new_v4(),
            image_hash: StorageHandler::get_md5_hash(data),
            image_filename: path
                .to_path_buf()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            image_source_path: path.to_path_buf().to_string_lossy().to_string(),
            image_dest_path: Default::default(),
            image_description: Some(path.to_path_buf().to_string_lossy().to_string()),
            image_config: None,
            created_on: None,
        };

        // DEBUG ONLY!! Need to find a more elegant solution
        if valid_paths.len() == 1 {
            image.image_id = force_uuid;
        }

        if let Some(found_hash) = Image::get_by_hash(&mut conn, &image.image_hash).await? {
            warn!(
                "Found image {}:{} with matching hash value {} to image {}:{}, skipping...",
                found_hash.image_id,
                found_hash.image_filename,
                found_hash.image_hash,
                image.image_id,
                image.image_filename,
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

        let s_handler = StorageHandler::init(storage_path.to_string(), depth);

        match s_handler
            .write_file_to_uuid_folder(image.image_id, &image.image_filename, data)
            .await
        {
            Ok(file_info) => {
                info!(
                    "Wrote file {} to {}",
                    file_info.dest_filename, file_info.dest_path
                );
                let image_uuid = image.image_id;
                image.image_dest_path = file_info.dest_path;
                if let Ok(image) = Image::insert(&mut conn, image).await {
                    if let Some(link) = link {
                        Image::link_image_to_environment(&mut conn, image.image_id, link).await?;
                    }
                } else {
                    s_handler
                        .delete_file_with_uuid(image_uuid, &file_info.dest_filename)
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
    CreateReplay{
        link,
        depth,
        force_uuid,
        file,
        creator_id,
        replay_forked_from,
        created_on
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

    let mut conn = db.acquire().await?;
    let data = &read(file)?;
    let mut replay = Replay{
        replay_id: force_uuid.unwrap_or_else(|| Uuid::new_v4()),
        instance_id: link,
        creator_id: creator_id.unwrap_or_else(|| uuid!("00000000-0000-0000-0000-000000000000")),
        replay_forked_from,
        replay_path: Default::default(),
        replay_hash: StorageHandler::get_md5_hash(data),
        replay_filename: file.file_name().unwrap().to_string_lossy().to_string(),
        created_on: created_on.and_then(|s| {
            time::OffsetDateTime::parse(
                &s,
                &time::format_description::well_known::iso8601::Iso8601::DEFAULT,
            )
                .map_err(|e| GISSTCliError::CreateReplay(e.to_string()))
                .ok()
        }),
    };
    if let Some(found_hash) = Replay::get_by_hash(&mut conn, &replay.replay_hash).await? {
        return Err(GISSTCliError::CreateReplay(format!(
            "Found replay {}:{} with matching hash value {} to replay {}:{}, skipping...",
            found_hash.replay_id,
            found_hash.replay_filename,
            found_hash.replay_hash,
            replay.replay_id,
            replay.replay_filename,
        )));
    }

    let s_handler = StorageHandler::init(storage_path.to_string(), depth);

    match s_handler
        .write_file_to_uuid_folder(replay.replay_id, &replay.replay_filename, data)
        .await
    {
        Ok(file_info) => {
            info!(
                "Wrote file {} to {}",
                file_info.dest_filename, file_info.dest_path
            );
            let replay_uuid = replay.replay_id;
            replay.replay_path = file_info.dest_path;
            if let Err(e) = Replay::insert(&mut conn, replay).await {
                s_handler
                    .delete_file_with_uuid(replay_uuid, &file_info.dest_filename)
                    .await?;
                return Err(GISSTCliError::NewModel(e));
            };
        }
        Err(e) => {
            error!("Error writing replay file to database, aborting...\n{e}");
        }
    }
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
    let data = &read(file)?;
    let mut state = State {
        state_id: force_uuid.unwrap_or_else(|| Uuid::new_v4()),
        instance_id: link,
        is_checkpoint: replay_id.is_some(),
        state_path: Default::default(),
        state_name: state_name.clone(),
        state_hash: StorageHandler::get_md5_hash(data),
        state_filename: file.file_name().unwrap().to_string_lossy().to_string(),
        state_description: state_description.unwrap_or_else(|| state_name.clone()),
        screenshot_id,
        created_on: created_on.and_then(|s| {
            time::OffsetDateTime::parse(
                &s,
                &time::format_description::well_known::iso8601::Iso8601::DEFAULT,
            )
            .map_err(|e| GISSTCliError::CreateState(e.to_string()))
            .ok()
        }),
        replay_id,
        creator_id,
        state_replay_index,
        state_derived_from,
    };
    if let Some(found_hash) = State::get_by_hash(&mut conn, &state.state_hash).await? {
        return Err(GISSTCliError::CreateState(format!(
            "Found state {}:{} with matching hash value {} to state {}:{}, skipping...",
            found_hash.state_id,
            found_hash.state_filename,
            found_hash.state_hash,
            state.state_id,
            state.state_filename,
        )));
    }

    let s_handler = StorageHandler::init(storage_path.to_string(), depth);

    match s_handler
        .write_file_to_uuid_folder(state.state_id, &state.state_filename, data)
        .await
    {
        Ok(file_info) => {
            info!(
                "Wrote file {} to {}",
                file_info.dest_filename, file_info.dest_path
            );
            let state_uuid = state.state_id;
            state.state_path = file_info.dest_path;
            if let Err(e) = State::insert(&mut conn, state).await {
                s_handler
                    .delete_file_with_uuid(state_uuid, &file_info.dest_filename)
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