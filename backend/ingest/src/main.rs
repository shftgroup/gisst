use std::{
    ffi::CString,
    fs::read,
};

use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use gisst::{
    model_enums::Framework,
    models::{DBHashable, DBModel, Environment, File as GFile, Instance, Object, ObjectRole, Work},
    storage::StorageHandler,
};
use log::{error, info, warn};
use rdb_sys::*;
use sqlx::pool::PoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(flatten)]
    pub verbose: Verbosity,

    /// GISST_CLI_DB_URL environment variable must be set to PostgreSQL path
    #[clap(env)]
    pub gisst_cli_db_url: String,

    /// GISST_STORAGE_ROOT_PATH environment variable must be set
    #[clap(env)]
    pub gisst_storage_root_path: String,

    #[clap(long)]
    pub rdb: String,

    #[clap(long)]
    pub dir: String,

    #[clap(long)]
    pub platform: String,

    #[clap(long)]
    pub ra_cfg: String,

    #[clap(long)]
    pub core_name: String,

    #[clap(long)]
    pub core_version: String,
}

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("file read error")]
    Io(#[from] std::io::Error),
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("nul error")]
    Nul(#[from] std::ffi::NulError),
    #[error("storage error")]
    Storage(#[from] gisst::storage::StorageError),
    #[error("new record")]
    NewRecord(#[from] gisst::models::NewRecordError),
    #[error("directory traversal error")]
    Directory(#[from] walkdir::Error),
    #[error("rdb open error")]
    RDB(),
    #[error("file metadata error")]
    File(),
}

#[tokio::main]
async fn main() -> Result<(), IngestError> {
    let Args {
        rdb,
        dir: roms,
        core_name,
        core_version,
        ra_cfg,
        gisst_cli_db_url,
        platform,
        gisst_storage_root_path,
        verbose,
    } = Args::parse();
    env_logger::Builder::new()
        .filter_level(verbose.log_level_filter())
        .init();
    info!("Connecting to database: {}", gisst_cli_db_url.to_string());
    let pool: PgPool = get_db_by_url(gisst_cli_db_url.to_string()).await?;
    let mut conn = pool.acquire().await?;
    info!("DB connection successful.");
    let storage_root = gisst_storage_root_path.to_string();
    info!("Storage root is set to: {}", &storage_root);
    let created_on = None;
    let ra_cfg_object_id = {
        let ra_cfg = std::path::Path::new(&ra_cfg);
        let ra = &read(ra_cfg)?;
        info!("RA: {}", ra.len());
        let hash = StorageHandler::get_md5_hash(ra);
        if let Some(obj) = Object::get_by_hash(&mut conn, &hash).await? {
            obj.object_id
        } else {
            let file_uuid = Uuid::new_v4();
            let file_info = StorageHandler::write_file_to_uuid_folder(
                &storage_root,
                4,
                file_uuid,
                "retroarch.cfg",
                ra,
            )
            .await?;
            info!(
                "Wrote file {} to {}",
                file_info.dest_filename, file_info.dest_path
            );
            let file_record = GFile {
                file_id: file_uuid,
                file_hash: file_info.file_hash,
                file_filename: file_info.source_filename,
                file_source_path: file_info.source_path,
                file_dest_path: file_info.dest_path,
                file_size: ra.len() as i64,
                created_on,
            };
            GFile::insert(&mut conn, file_record).await?;
            let object_id = Uuid::new_v4();
            let object = Object {
                object_id,
                file_id: file_uuid,
                object_description: Some("RetroArch Config".to_string()),
                created_on,
            };
            Object::insert(&mut conn, object).await?;
            object_id
        }
    };
    let rdb_path_c = CString::new(rdb)?;
    unsafe {
        let db: *mut RetroDB = libretrodb_new();
        if db.is_null() {
            return Err(IngestError::RDB());
        }
        let cursor: *mut RetroCursor = libretrodb_cursor_new();
        if cursor.is_null() {
            return Err(IngestError::RDB());
        }
        let mut rval: RVal = RVal {
            tag: RType::Null,
            value: RValInner { int_: 0 },
        };
        info!("opening DB");
        if libretrodb_open(rdb_path_c.as_ptr(), db) != 0 {
            error!("Not opened {rdb_path_c:?}");
            return Err(IngestError::RDB());
        }
        info!("Opened DB");
        let md5_idx = CString::new("md5")?;
        if libretrodb_create_index(db, md5_idx.as_ptr(), md5_idx.as_ptr()) == -1 {
            error!("Couldn't create md5 index");
            return Err(IngestError::RDB());
        }
        for entry in walkdir::WalkDir::new(&roms) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let data = &read(entry.path())?;
            let hash = md5::compute(data);
            if GFile::get_by_hash(&mut conn, &format!("{:x}", hash))
                .await?
                .is_some()
            {
                info!("{:?}:{:x} already in DB, skip", entry.path(), hash);
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            let key_bytes = hash.as_ptr();
            info!("{:?}: {}: {:x}", entry.path(), hash.len(), hash);
            if libretrodb_find_entry(db, md5_idx.as_ptr(), key_bytes, &mut rval) == 0 {
                info!("FOUND IT\n{}", rval);
                // TODO: merge entries from result on libretrodb_query_compile(db, "{\"rom_name\":nom}", strlen query exp, error) cursor
                let work = Work {
                    work_id: Uuid::new_v4(),
                    work_name: rval.map_get("name").unwrap_or_else(|| file_name.clone()),
                    work_version: rval
                        .map_get("rom_name")
                        .unwrap_or_else(|| file_name.clone()),
                    work_platform: platform.clone(),
                    created_on,
                };
                let env = Environment {
                    environment_id: Uuid::new_v4(),
                    environment_name: work.work_name.clone(),
                    environment_framework: Framework::RetroArch,
                    environment_core_name: core_name.clone(),
                    environment_core_version: core_version.clone(),
                    environment_derived_from: None,
                    environment_config: None,
                    created_on,
                };
                let instance_id = Uuid::new_v4();
                let instance = Instance {
                    instance_id,
                    work_id: work.work_id,
                    environment_id: env.environment_id,
                    instance_config: None,
                    created_on,
                };
                let file_uuid = Uuid::new_v4();
                let file_info = StorageHandler::write_file_to_uuid_folder(
                    &storage_root,
                    4,
                    file_uuid,
                    &file_name,
                    data,
                )
                .await?;
                info!(
                    "Wrote file {} to {}",
                    file_info.dest_filename, file_info.dest_path
                );
                let file_record = GFile {
                    file_id: file_uuid,
                    file_hash: format!("{:x}", hash),
                    file_filename: file_name,
                    file_source_path: entry
                        .path()
                        .strip_prefix(&roms)
                        .map_err(|_e| IngestError::File())?
                        .to_string_lossy()
                        .to_string(),
                    file_dest_path: file_info.dest_path,
                    file_size: data.len() as i64,
                    created_on,
                };
                let object_id = Uuid::new_v4();
                let object = Object {
                    object_id,
                    file_id: file_uuid,
                    object_description: rval.map_get("description"),
                    created_on: None,
                };
                Work::insert(&mut conn, work).await?;
                Environment::insert(&mut conn, env).await?;
                Instance::insert(&mut conn, instance).await?;
                GFile::insert(&mut conn, file_record).await?;
                Object::insert(&mut conn, object).await?;
                Object::link_object_to_instance(
                    &mut conn,
                    object_id,
                    instance_id,
                    ObjectRole::Content,
                )
                .await?;
                Object::link_object_to_instance(
                    &mut conn,
                    ra_cfg_object_id,
                    instance_id,
                    ObjectRole::Config,
                )
                .await?;

                rmsgpack_dom_value_free(&mut rval);
            } else {
                warn!("md5 not found");
                continue;
            }
        }
        libretrodb_cursor_free(cursor);
        libretrodb_close(db);
        libretrodb_free(db);
    }
    Ok(())
}

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new().connect(&db_url).await
}
#[allow(dead_code)]
fn char_to_num(c: u8) -> u8 {
    if c <= b'9' {
        c - b'0'
    } else {
        (c - b'A') + 10
    }
}
