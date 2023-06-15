mod args;

use std::{fs::{
    read,
}, fs, io};
use std::path::{
    Path,
    PathBuf,
};
use log::{info, warn, error, trace, debug};
use clap::Parser;
use walkdir::WalkDir;
use sqlx::PgPool;
use sqlx::pool::PoolOptions;
use uuid::{Uuid,uuid};
use gisstlib::{
    models::{
        DBHashable,
        DBModel,
        Object,
        Image,
        Instance,
        Environment,
        Work,
    },
    storage::{
        StorageHandler,
}};
use args::{
    GISSTCli,
    GISSTCliError,
    RecordType,
    CreateObject,
    DeleteObject,
    CreateEnvironment,
    DeleteEnvironment,
    CreateImage,
    DeleteImage,
    UpdateObject,
    CreateInstance,
    DeleteInstance,
    CreateWork,
    DeleteWork,
    CreateSave,
    DeleteSave,
    CreateReplay,
    DeleteReplay,
    CreateState,
    DeleteState,
    BaseSubcommand,
};
use anyhow::{Result};
use env_logger;

#[tokio::main]
async fn main() -> Result<(), GISSTCliError> {
    let args = GISSTCli::parse();
    info!("Connecting to database: {}", args.gisst_cli_db_url.to_string());
    let db:PgPool = get_db_by_url(args.gisst_cli_db_url.to_string()).await?;
    info!("DB connection successful.");
    let storage_root = args.gisst_storage_root_path.to_string();
    info!("Storage root is set to: {}", args.gisst_storage_root_path.to_string());

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    match &args.record_type {
        RecordType::Object(object) => {
            match &object.command {
                BaseSubcommand::Create(create) => create_object(create, db, storage_root).await?,
                BaseSubcommand::Update(_update) => (),
                BaseSubcommand::Delete(delete) => delete_object(delete, db, storage_root).await?,
                BaseSubcommand::Locate(_locate) => (),
                BaseSubcommand::Export(_export) => (),
            }
        },
        RecordType::Creator(_creator) => {

        },
        RecordType::Environment(environment) => {
            match &environment.command {
                BaseSubcommand::Create(create) => create_environment(create, db).await?,
                BaseSubcommand::Update(_update) => (),
                BaseSubcommand::Delete(delete) => delete_environment(delete, db).await?,
                BaseSubcommand::Locate(_locate) => (),
                BaseSubcommand::Export(_export) => (),
            }
        },
        RecordType::Image(image) => {
            match &image.command {
                BaseSubcommand::Create(create) => create_image(create, db, storage_root).await?,
                BaseSubcommand::Update(_update) => (),
                BaseSubcommand::Delete(delete) => delete_image(delete, db, storage_root).await?,
                BaseSubcommand::Locate(_locate) => (),
                BaseSubcommand::Export(_export) => (),
            }

        },
        RecordType::Instance(instance) => {
            match &instance.command {
                BaseSubcommand::Create(create) => create_instance(create, db).await?,
                BaseSubcommand::Update(_update) => (),
                BaseSubcommand::Delete(delete) => delete_instance(delete, db).await?,
                BaseSubcommand::Locate(_locate) => (),
                BaseSubcommand::Export(_export) => (),
            }
        },
        RecordType::Work(work) => {
            match &work.command {
                BaseSubcommand::Create(create) => create_work(create, db).await?,
                BaseSubcommand::Update(_update) => (),
                BaseSubcommand::Delete(delete) => delete_work(delete, db).await?,
                BaseSubcommand::Locate(_locate) => (),
                BaseSubcommand::Export(_export) => (),
            }
        },
        RecordType::State(state) => {
            match &state.command {
                BaseSubcommand::Create(create) => create_state(create, db).await?,
                BaseSubcommand::Update(_update) => (),
                BaseSubcommand::Delete(delete) => delete_state(delete, db).await?,
                BaseSubcommand::Locate(_locate) => (),
                BaseSubcommand::Export(_export) => (),
            }
        },
        RecordType::State(state) => {
            match &state.command {
                BaseSubcommand::Create(create) => create_state(create, db).await?,
                BaseSubcommand::Update(_update) => (),
                BaseSubcommand::Delete(delete) => delete_state(delete, db).await?,
                BaseSubcommand::Locate(_locate) => (),
                BaseSubcommand::Export(_export) => (),
            }
        },
        RecordType::Save(save) => {
            match &save.command {
                BaseSubcommand::Create(create) => create_save(create, db).await?,
                BaseSubcommand::Update(_update) => (),
                BaseSubcommand::Delete(delete) => delete_save(delete, db).await?,
                BaseSubcommand::Locate(_locate) => (),
                BaseSubcommand::Export(_export) => (),
            }
        },
        RecordType::Replay(replay) => {
            match &replay.command {
                BaseSubcommand::Create(create) => create_replay(create, db).await?,
                BaseSubcommand::Update(_update) => (),
                BaseSubcommand::Delete(delete) => delete_replay(delete, db).await?,
                BaseSubcommand::Locate(_locate) => (),
                BaseSubcommand::Export(_export) => (),
            }
        },
    }

    println!("{:?}", &args);
    Ok(())
}

async fn create_object(c:&CreateObject, db: PgPool, storage_path:String) -> Result<(), GISSTCliError>{
    let (recursive, _extract, ignore, _skip, depth, link) = (
        c.recursive,
        c.extract,
        c.ignore_description,
        c.skip_yes,
        c.depth,
        c.link,
    );

    let mut valid_paths: Vec<PathBuf> = Vec::new();

    for path in &c.file {
        let p = Path::new(path);

        if p.exists() {
            if p.is_dir() {
                if recursive != true {
                    error!("Recursive flag must be set for directory ingest.");
                    return Err(GISSTCliError::CreateObjectError("Recursive flag must be set for directory ingest.".to_string()))
                }

                for entry in WalkDir::new(&p) {
                    let dir_entry = entry?;
                    if dir_entry.path().is_dir() == false {
                        valid_paths.push(dir_entry.path().to_path_buf());
                    }
                }
            } else {
                valid_paths.push(p.to_path_buf());
            }
        } else {
            error!("File not found: {}", &p.to_string_lossy());
            return Err(GISSTCliError::CreateObjectError(
                format!("File not found: {}", &p.to_string_lossy())))
        }
    }

    let mut conn = db.acquire().await?;

    for path in &valid_paths {
        let data = &read(&path)?;
        let mut object = Object {
            object_id: Uuid::new_v4(),
            object_hash: StorageHandler::get_md5_hash(data),
            object_filename: path.to_path_buf().file_name().unwrap().to_string_lossy().to_string(),
            object_source_path: path.to_path_buf().to_string_lossy().to_string(),
            object_dest_path: Default::default(),
            object_description: Some(path.to_path_buf().to_string_lossy().to_string()),
            created_on: None,
        };

        // DEBUG ONLY!! Need to find a more elegant solution
        if valid_paths.len() == 1 {
            object.object_id = c.force_uuid;
        }

        if let Some(found_hash) = Object::get_by_hash(&mut conn, &object.object_hash).await? {
            warn!("Found object {}:{} with matching hash value {} to object {}:{}, skipping...",
                found_hash.object_id,
                found_hash.object_filename,
                found_hash.object_hash,
                object.object_id,
                object.object_filename,
            );
            continue;
        }

        if ignore == false {
            let mut description = Default::default();
            println!("Please enter an object description for file: {}", &path.to_string_lossy());
            io::stdin()
                .read_line(&mut description)
                .ok();
            object.object_description = Some(description.trim().to_string());
        }

        let s_handler = StorageHandler::init(storage_path.to_string(), depth);

        match s_handler.write_file_to_uuid_folder(object.object_id, &object.object_filename, data).await {
            Ok(file_info) => {
                info!("Wrote file {} to {}", file_info.dest_filename, file_info.dest_path);
                let obj_uuid = object.object_id.clone();
                object.object_dest_path = file_info.dest_path;
                if let Ok(object) = Object::insert(&mut conn, object).await {
                    if link.is_some(){
                        Object::link_object_to_instance(&mut conn, object.object_id, link.unwrap()).await?;
                    }
                } else {
                    s_handler.delete_file_with_uuid(obj_uuid, &file_info.dest_filename).await?;
                }
            },
            Err(e) => {
                error!("Error writing object file to database, aborting...");
            }
        }
    }

    Ok(())
}

async fn delete_object(d:&DeleteObject, db:PgPool, storage_path:String) -> Result<(), GISSTCliError> {
    let mut conn = db.acquire().await?;
    if let Some(object) = Object::get_by_id(&mut conn, d.id).await? {
        Object::delete_object_instance_links_by_id(&mut conn, object.object_id).await?;

        info!("Deleting object record with uuid {}", d.id);

        Object::delete_by_id(&mut conn, object.object_id).await?;

        info!("Deleting object file at path:{}", Path::new(&object.object_dest_path).join(
            Path::new(&StorageHandler::get_dest_filename(&object.object_hash, &object.object_filename)
        )).to_string_lossy());
        debug!("Object path depth is set to {}", StorageHandler::get_folder_depth_from_path(Path::new(&object.object_dest_path), None));

        StorageHandler::init(storage_path,
                             StorageHandler::get_folder_depth_from_path(Path::new(&object.object_dest_path), None))
            .delete_file_with_uuid(d.id,
                                   &StorageHandler::get_dest_filename(&object.object_hash, &object.object_filename)).await?;
    } else {
        warn!("Object with id: {} not found in database", d.id);
    }

    Ok(())
}

async fn update_object(_u:&UpdateObject, _db:PgPool) -> Result<(), GISSTCliError> {

    Ok(())
}

async fn create_instance(c:&CreateInstance, db:PgPool) -> Result<(), GISSTCliError> {
    let instance_from_json: Option<Instance> = match (&c.json_file, &c.json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(|e| GISSTCliError::IoError(e))?;
            Some(serde_json::from_str(&json_data).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value.clone()).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (_, _) => None,
    };

    let instance_config_json: Option<serde_json::Value> = match(&c.instance_config_json_file, &c.instance_config_json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(|e| GISSTCliError::IoError(e))?;
            Some(serde_json::from_str(&json_data).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value.clone()).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (_, _) => None,
    };

    match (instance_from_json, instance_config_json) {
        (Some(instance), instance_config) => {
            let mut conn = db.acquire().await?;
            Instance::insert(&mut conn, Instance { instance_config, ..instance })
                .await
                .map_err(|e|GISSTCliError::NewModelError(e))?;
            Ok(())
        },
        _ => {
            Err(GISSTCliError::CreateInstanceError("Need to provide a JSON string or file to create instance record.".to_string()))
        }
    }
}

async fn delete_instance(d:&DeleteInstance, db:PgPool) -> Result<(), GISSTCliError> {
    let mut conn = db.acquire().await?;
    if let Some(instance) = Instance::get_by_id(&mut conn, d.id).await? {
        info!("Deleting unlinking images for instance record with uuid {}", d.id);
        Instance::delete_instance_object_links_by_id(&mut conn, instance.instance_id).await?;

        info!("Deleting instance record with uuid {}", d.id);

        Instance::delete_by_id(&mut conn, instance.instance_id).await?;
    } else {
        warn!("Instance with uuid {} not found in database.", d.id);
    }

    Ok(())
}

async fn create_environment(c:&CreateEnvironment, db:PgPool) -> Result<(), GISSTCliError> {

    let environment_from_json: Option<Environment> = match (&c.json_file, &c.json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(|e| GISSTCliError::IoError(e))?;
            Some(serde_json::from_str(&json_data).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value.clone()).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (_, _) => None,
    };

    let environment_config_json: Option<serde_json::Value> = match(&c.environment_config_json_file, &c.environment_config_json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(|e| GISSTCliError::IoError(e))?;
            Some(serde_json::from_str(&json_data).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value.clone()).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (_, _) => None,
    };

    match (environment_from_json, environment_config_json) {
        (Some(environment), environment_config) => {
            let mut conn = db.acquire().await?;
            Environment::insert(&mut conn, Environment { environment_config, ..environment })
                .await
                .map_err(|e|GISSTCliError::NewModelError(e))?;
            Ok(())
        },
        _ => {
            Err(GISSTCliError::CreateInstanceError("Need to provide a JSON string or file to create environment record.".to_string()))
        }
    }
}

async fn delete_environment(d:&DeleteEnvironment, db:PgPool) -> Result<(), GISSTCliError> {
    let mut conn = db.acquire().await?;
    if let Some(environment) = Environment::get_by_id(&mut conn, d.id).await? {
        info!("Deleting unlinking images for environment record with uuid {}", d.id);
        Environment::delete_environment_image_links_by_id(&mut conn, environment.environment_id).await?;

        info!("Deleting environment record with uuid {}", d.id);

        Environment::delete_by_id(&mut conn, environment.environment_id).await?;
    } else {
        warn!("Environment with uuid {} not found in database.", d.id);
    }
    Ok(())
}

async fn create_work(c:&CreateWork, db:PgPool) -> Result<(), GISSTCliError> {
    let work_from_json: Option<Work> = match (&c.json_file, &c.json_string) {
        (Some(file_path), None) => {
            let json_data = fs::read_to_string(file_path).map_err(|e| GISSTCliError::IoError(e))?;
            Some(serde_json::from_str(&json_data).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (None, Some(json_value)) => {
            Some(serde_json::from_value(json_value.clone()).map_err(|e| GISSTCliError::JsonParseError(e))?)
        },
        (_, _) => unreachable!(),
    };

    match work_from_json {
        Some(work) => {
            let mut conn = db.acquire().await?;
            Work::insert(&mut conn, work).await?;
            Ok(())
        },
        None => Err(GISSTCliError::CreateWorkError("Please provide JSON to parse for creating a work record.".to_string()))
    }
}

async fn delete_work(d:&DeleteWork, db:PgPool) -> Result<(), GISSTCliError> {
    let mut conn = db.acquire().await?;
    if let Some(work) = Work::get_by_id(&mut conn, d.id).await? {
        info!("Deleting work record with uuid {}", d.id);
        Work::delete_by_id(&mut conn, work.work_id).await?;
    } else {
        warn!("Work with uuid {} not found in database.", d.id);
    }
    Ok(())
}

async fn create_image(c:&CreateImage, db: PgPool, storage_path:String) -> Result<(), GISSTCliError> {
    let (ignore, _skip, depth, link) = (
        c.ignore_description,
        c.skip_yes,
        c.depth,
        c.link,
    );

    let mut valid_paths: Vec<PathBuf> = Vec::new();

    for path in &c.file {
        let p = Path::new(path);

        if p.exists() {
            valid_paths.push(p.to_path_buf());
        } else {
            error!("File not found: {}", &p.to_string_lossy());
            return Err(GISSTCliError::CreateImageError(
                format!("File not found: {}", &p.to_string_lossy())))
        }
    }

    let mut conn = db.acquire().await?;

    for path in &valid_paths {
        let data = &read(&path)?;
        let mut image = Image {
            image_id: Uuid::new_v4(),
            image_hash: StorageHandler::get_md5_hash(data),
            image_filename: path.to_path_buf().file_name().unwrap().to_string_lossy().to_string(),
            image_source_path: path.to_path_buf().to_string_lossy().to_string(),
            image_dest_path: Default::default(),
            image_description: Some(path.to_path_buf().to_string_lossy().to_string()),
            image_config: None,
            created_on: None,
        };

        // DEBUG ONLY!! Need to find a more elegant solution
        if valid_paths.len() == 1 {
            image.image_id = c.force_uuid;
        }

        if let Some(found_hash) = Image::get_by_hash(&mut conn, &image.image_hash).await? {
            warn!("Found image {}:{} with matching hash value {} to image {}:{}, skipping...",
                found_hash.image_id,
                found_hash.image_filename,
                found_hash.image_hash,
                image.image_id,
                image.image_filename,
            );
            continue;
        }

        if ignore == false {
            let mut description = Default::default();
            println!("Please enter an image description for file: {}", &path.to_string_lossy());
            io::stdin()
                .read_line(&mut description)
                .ok();
            image.image_description = Some(description.trim().to_string());
        }

        let s_handler = StorageHandler::init(storage_path.to_string(), depth);

        match s_handler.write_file_to_uuid_folder(image.image_id, &image.image_filename, data).await {
            Ok(file_info) => {
                info!("Wrote file {} to {}", file_info.dest_filename, file_info.dest_path);
                let image_uuid = image.image_id.clone();
                image.image_dest_path = file_info.dest_path;
                if let Ok(image) = Image::insert(&mut conn, image).await {
                    if link.is_some() {
                        Image::link_image_to_environment(&mut conn, image.image_id, link.unwrap()).await?;
                    }
                } else {
                    s_handler.delete_file_with_uuid(image_uuid, &file_info.dest_filename).await?;
                }
            },
            Err(e) => {
                error!("Error writing image file to database, aborting...");
            }
        }
    }
    Ok(())
}

async fn delete_image(d:&DeleteImage, db:PgPool, storage_path:String) -> Result<(), GISSTCliError> {
    let mut conn = db.acquire().await?;
    if let Some(image) = Image::get_by_id(&mut conn, d.id).await? {
        Image::delete_image_environment_links_by_id(&mut conn, image.image_id).await?;

        info!("Deleting image record with uuid {}", d.id);

        Image::delete_by_id(&mut conn, image.image_id).await?;

        info!("Deleting image file at path:{}", Path::new(&image.image_dest_path).join(
            Path::new(&StorageHandler::get_dest_filename(&image.image_hash, &image.image_filename)
        )).to_string_lossy());
        debug!("Image path depth is set to {}", StorageHandler::get_folder_depth_from_path(Path::new(&image.image_dest_path), None));

        StorageHandler::init(storage_path,
                             StorageHandler::get_folder_depth_from_path(Path::new(&image.image_dest_path), None))
            .delete_file_with_uuid(d.id,
                                   &StorageHandler::get_dest_filename(&image.image_hash, &image.image_filename)).await?;
    } else {
        warn!("Image with id: {} not found in database", d.id);
    }
    Ok(())
}

async fn create_replay(c:&CreateReplay, db: PgPool) -> Result<(), GISSTCliError> { Ok(())}
async fn create_state(c:&CreateState, db: PgPool) -> Result<(), GISSTCliError> { Ok(())}
async fn create_save(c:&CreateSave, db: PgPool) -> Result<(), GISSTCliError> { Ok(())}

async fn delete_replay(d:&DeleteReplay, db:PgPool) -> Result<(), GISSTCliError> { Ok(())}
async fn delete_save(d:&DeleteSave, db:PgPool) -> Result<(), GISSTCliError> { Ok(())}
async fn delete_state(d:&DeleteState, db:PgPool) -> Result<(), GISSTCliError> { Ok(())}

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new()
        .connect(&db_url)
        .await
}