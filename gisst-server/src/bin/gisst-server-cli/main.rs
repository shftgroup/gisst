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
    models,
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
};
use gisstlib::models::{DBModel, Object};
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
                ObjectSubcommand::Delete(delete) => (),
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
            object_path: path.to_path_buf().to_string_lossy().to_string(),
            object_description: Some(path.to_path_buf().to_string_lossy().to_string()),
            created_on: None,
        };

        if ignore == false {
            let mut description = Default::default();
            println!("Please enter an object description for file: {}", &path.to_string_lossy());
            io::stdin()
                .read_line(&mut description)
                .ok();
            object.object_description = Some(description.to_string());
        }

        let s_handler = StorageHandler::init(storage_path.to_string(), depth as i8);
        if let Ok(object) = Object::insert(&mut conn, object).await {
            if link.is_some(){
                Object::link_object_to_instance(&mut conn, object.object_id, link.unwrap()).await?;
            }

            match s_handler.write_file_to_uuid_folder(object.object_id, &object.object_filename, data).await {
                Ok(file_info) => (),
                Err(_) => {
                    Object::delete_by_id(&mut conn, object.object_id).await?;
                }
            }
        }
    }

    Ok(())
}
async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new()
        .connect(&db_url)
        .await
}