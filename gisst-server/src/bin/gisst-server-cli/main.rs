mod args;

use std::{
    fs::{
        File,
        read,
    },
    io};
use std::io::Read;
use std::path::{
    Path,
    PathBuf,
};
use log::{info, warn, error, trace, debug};
use clap::Parser;
use walkdir::WalkDir;
use serde_json::to_string;
use sqlx::PgPool;
use sqlx::pool::PoolOptions;
use uuid::Uuid;
use gisstlib::{
    models::{
        DBHashable,
        DBModel,
        Object,
    },
    storage::{
        StorageHandler,
        FileInformation,
    }
};
use args::{
    GISSTCli,
    GISSTCliError,
    RecordType,
    ObjectSubcommand,
    CreateObject,
    DeleteObject,
};
use anyhow::{Result};
use sqlx::postgres::PgQueryResult;
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
                ObjectSubcommand::Create(create) => create_object(create, db, storage_root).await?,
                ObjectSubcommand::Update(update) => (),
                ObjectSubcommand::Delete(delete) => delete_object(delete, db, storage_root).await?,
                ObjectSubcommand::Locate(locate) => (),
                ObjectSubcommand::Export(export) => (),
            }
        },
        RecordType::Creator(creator) => {

        },
        RecordType::Image(image) => {

        },
        RecordType::Work(work) => {

        }
    }

    println!("{:?}", &args);
    Ok(())
}

async fn create_object(c:&CreateObject, db: PgPool, storage_path:String) -> Result<(), GISSTCliError>{
    let (recursive, extract, ignore, skip, depth, link) = (
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

    for path in valid_paths {
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
            object.object_description = Some(description.to_string());
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

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new()
        .connect(&db_url)
        .await
}