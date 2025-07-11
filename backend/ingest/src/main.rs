#![allow(clippy::unnecessary_debug_formatting)]
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use gisst::{
    model_enums::Framework,
    models::{
        Duplicate, Environment, File as GFile, Instance, Object, ObjectRole, Work,
        insert_file_object,
    },
};
use log::{error, info, warn};
use rdb_sys::{RDB, RVal};
use sqlx::PgPool;
use sqlx::pool::PoolOptions;
use uuid::Uuid;

const DEPTH: u8 = 4;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(flatten)]
    pub verbose: Verbosity,

    /// `GISST_CLI_DB_URL` environment variable must be set to database path
    #[clap(env)]
    pub gisst_cli_db_url: String,

    /// `GISST_STORAGE_ROOT_PATH` environment variable must be set
    #[clap(env)]
    pub gisst_storage_root_path: String,
    #[clap(env)]
    pub meili_url: String,
    #[clap(env)]
    pub meili_api_key: String,

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

    #[clap(long = "dep")]
    pub deps: Vec<String>,

    #[clap(long = "dep-path")]
    pub dep_paths: Vec<String>,

    #[clap(short = 'f', long = "force")]
    pub force: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("file read error")]
    Io(#[from] std::io::Error),
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("insert error {0}")]
    Insert(#[from] gisst::error::Insert),
    #[error("search index error {0}")]
    Index(#[from] gisst::error::SearchIndex),
    #[error("nul error")]
    Nul(#[from] std::ffi::NulError),
    #[error("storage error")]
    Storage(#[from] gisst::error::Storage),
    #[error("new record")]
    NewRecord(#[from] gisst::error::RecordSQL),
    #[error("directory traversal error")]
    Directory(#[from] walkdir::Error),
    #[error("rdb open error")]
    RDB(),
    #[error("file metadata error")]
    File(),
    #[error("file insertion error")]
    InsertFile(#[from] gisst::error::InsertFile),
    #[error("Role index too high, must be <= 65535")]
    RoleTooHigh(usize),
    #[error("CHD error")]
    Chd(#[from] chd::Error),
    #[error("CHD no checksum error")]
    ChdNoChecksum(),
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<(), IngestError> {
    use rayon::prelude::*;
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
        deps,
        dep_paths,
        force,
        meili_url,
        meili_api_key,
    } = Args::parse();
    env_logger::Builder::new()
        .filter_level(verbose.log_level_filter())
        .init();
    info!("Connecting to database: {gisst_cli_db_url}");
    let pool: Arc<PgPool> = Arc::new(get_db_by_url(gisst_cli_db_url.to_string()).await?);
    info!("DB connection successful.");
    let storage_root = gisst_storage_root_path.to_string();
    info!("Storage root is set to: {}", &storage_root);
    let mut base_conn = pool.begin().await?;
    let ra_cfg_object_id = insert_file_object(
        &mut base_conn,
        &storage_root,
        DEPTH,
        std::path::Path::new(&ra_cfg),
        Some("retroarch.cfg".to_string()),
        Some("base retroarch config".to_string()),
        String::new(),
        Duplicate::ReuseObject,
    )
    .await?;
    let indexer = gisst::search::MeiliIndexer::new(&meili_url, &meili_api_key)?;
    let mut dep_ids = Vec::with_capacity(deps.len());
    for (i, dep) in deps.iter().enumerate() {
        let dep = std::path::Path::new(dep);
        let file_name = dep.file_name().unwrap().to_string_lossy().to_string();
        let dep_path = dep_paths.get(i).unwrap_or(&file_name);
        info!("inserting dep {dep:?} @ {dep_path:?}");
        let dep_id = insert_file_object(
            &mut base_conn,
            &storage_root,
            DEPTH,
            dep,
            None,
            Some(dep_path.clone()),
            dep_path.clone(),
            Duplicate::ReuseObject,
        )
        .await?;
        dep_ids.push(dep_id);
    }

    base_conn.commit().await?;

    let db = Arc::new(RDB::open(std::path::Path::new(&rdb)).map_err(|_| IngestError::RDB())?);
    let roms = Arc::new(std::path::PathBuf::from(roms));
    let files: Vec<_> = walkdir::WalkDir::new(&*roms)
        .into_iter()
        .map(|e| e.unwrap())
        .collect();

    let handle = tokio::runtime::Handle::current();
    let result: Result<_, _> = files
        .par_iter()
        .map(|entry| {
            if !entry.file_type().is_file() {
                return Ok(());
            }
            let path = entry.path().to_owned();
            let file_name = entry.file_name().to_string_lossy().to_string();
            let ext = path
                .extension()
                .map(std::ffi::OsStr::to_string_lossy)
                .unwrap_or_default()
                .into_owned();
            let stem = path
                .file_stem()
                .map(std::ffi::OsStr::to_string_lossy)
                .unwrap_or_default()
                .into_owned();
            if matches!(
                ext.as_str(),
                "chd" | "cue" | "bin" | "iso" | "srm" | "7z" | "zip"
            ) {
                // Skip this one
                return Ok(());
            }

            let dep_ids = dep_ids.clone();
            let db = Arc::clone(&db);
            let roms = Arc::clone(&roms);
            let platform = platform.clone();
            let core_name = core_name.clone();
            let core_version = core_version.clone();
            let storage_root = storage_root.clone();
            let pool = Arc::clone(&pool);
            let indexer = indexer.clone();
            handle.block_on(async move {
                let mut tx = pool.begin().await?;

                // acquire a connection pool -- start transaction here
                // multi disc rom
                if ext.eq_ignore_ascii_case("m3u") {
                    let mut found = false;
                    info!("playlist file {path:?}");
                    for file in files_of_playlist(&roms, &path)? {
                        // match find_entry(&mut conn, &db, &file).await? {
                        match find_entry(&mut tx, &db, &file).await? {
                            FindResult::AlreadyHave => {
                                found = true;
                                break;
                            }
                            FindResult::NotInRDB => {}
                            FindResult::InRDB(rval) => {
                                let instance_id = create_metadata_records_from_rval(
                                    &mut tx,
                                    &file_name,
                                    &rval,
                                    &platform,
                                    &core_name,
                                    &core_version,
                                    &indexer,
                                )
                                .await?;
                                link_deps(&mut tx, ra_cfg_object_id, &dep_ids, instance_id).await?;
                                create_playlist_instance_objects(
                                    &mut tx,
                                    &storage_root,
                                    &roms,
                                    instance_id,
                                    &path,
                                    rval.map_get("description"),
                                )
                                .await?;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found && force {
                        let instance_id = create_metadata_records(
                            &mut tx,
                            &file_name,
                            &stem,
                            &file_name,
                            &platform,
                            &core_name,
                            &core_version,
                            &indexer,
                        )
                        .await?;
                        link_deps(&mut tx, ra_cfg_object_id, &dep_ids, instance_id).await?;
                        create_playlist_instance_objects(
                            &mut tx,
                            &storage_root,
                            &roms,
                            instance_id,
                            &path,
                            Some(stem.to_string()),
                        )
                        .await?;
                    }
                } else {
                    // normal rom
                    match find_entry(&mut tx, &db, &path).await? {
                        FindResult::InRDB(rval) => {
                            let instance_id = create_metadata_records_from_rval(
                                &mut tx,
                                &file_name,
                                &rval,
                                &platform,
                                &core_name,
                                &core_version,
                                &indexer,
                            )
                            .await?;
                            link_deps(&mut tx, ra_cfg_object_id, &dep_ids, instance_id).await?;
                            create_single_file_instance_objects(
                                &mut tx,
                                &storage_root,
                                &roms,
                                instance_id,
                                &path,
                                rval.map_get("description"),
                            )
                            .await
                            .expect("Create_single_file_instance_objects failed in RDB");
                        }
                        FindResult::NotInRDB if force => {
                            let instance_id = create_metadata_records(
                                &mut tx,
                                &file_name,
                                &stem,
                                &file_name,
                                &platform,
                                &core_name,
                                &core_version,
                                &indexer,
                            )
                            .await?;
                            link_deps(&mut tx, ra_cfg_object_id, &dep_ids, instance_id).await?;
                            create_single_file_instance_objects(
                                &mut tx,
                                &storage_root,
                                &roms,
                                instance_id,
                                &path,
                                Some(stem.to_string()),
                            )
                            .await
                            .expect("Create_single_file_instance_objects failed not in RDB");
                            // ?? unwraps Result<(), Ingest Error>
                        }
                        FindResult::AlreadyHave if force => {
                            let instance_id = create_metadata_records(
                                &mut tx,
                                &file_name,
                                &stem,
                                &file_name,
                                &platform,
                                &core_name,
                                &core_version,
                                &indexer,
                            )
                            .await?;
                            link_deps(&mut tx, ra_cfg_object_id, &dep_ids, instance_id).await?;
                            create_single_file_instance_objects(
                                &mut tx,
                                &storage_root,
                                &roms,
                                instance_id,
                                &path,
                                Some(stem.to_string()),
                            )
                            .await
                            .expect("Create_single_file_instance_objects failed not in RDB");
                            // ?? unwraps Result<(), Ingest Error>
                        }
                        // _ => Ok(()),
                        _ => {}
                    }
                }
                // commit transaction + ok
                tx.commit().await.map_err(IngestError::Sql)?;
                // TODO: index instance here
                Ok(())
            })
        })
        .collect();
    result
}

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new().connect(&db_url).await
}
#[allow(dead_code)]
fn char_to_num(c: u8) -> u8 {
    if c <= b'9' { c - b'0' } else { (c - b'A') + 10 }
}

enum FindResult {
    AlreadyHave,
    NotInRDB,
    InRDB(RVal),
}

async fn find_entry(
    conn: &mut sqlx::PgConnection,
    db: &RDB,
    path: &std::path::Path,
) -> Result<FindResult, IngestError> {
    let hash = {
        use md5::Digest;
        let mut hasher = md5::Md5::new();
        let mut file = std::fs::File::open(path)?;
        std::io::copy(&mut file, &mut hasher)?;
        hasher.finalize()
    };
    let hash_str = format!("{hash:x}");
    if GFile::get_by_hash(conn, &hash_str).await?.is_some() {
        info!("{path:?}:{hash_str} already in DB, skip");
        return Ok(FindResult::AlreadyHave);
    }
    let name_without_ext = path.file_stem();

    info!("{:?}: {}: {hash_str}", path, hash.len());

    if let Some(rval) = db.find_entry::<&str, &[u8]>("md5", &hash) {
        info!("metadata found\n{rval} for {path:?}");
        Ok(FindResult::InRDB(rval))
    } else if let Some(rval) = name_without_ext
        .and_then(|name| name.to_str())
        .and_then(|name| db.find_entry_by::<&str, &str>("name", |n| n == name))
    {
        info!("metadata found\n{rval} for {path:?} by name prefix");
        Ok(FindResult::InRDB(rval))
    } else {
        warn!("md5 or name not found");
        Ok(FindResult::NotInRDB)
    }
}

async fn create_metadata_records_from_rval(
    conn: &mut sqlx::PgConnection,
    file_name: &str,
    rval: &RVal,
    platform: &str,
    core_name: &str,
    core_version: &str,
    indexer: &gisst::search::MeiliIndexer,
) -> Result<Uuid, IngestError> {
    create_metadata_records(
        conn,
        file_name,
        &rval
            .map_get("name")
            .unwrap_or_else(|| file_name.to_string()),
        &rval
            .map_get("rom_name")
            .unwrap_or_else(|| file_name.to_string()),
        platform,
        core_name,
        core_version,
        indexer,
    )
    .await
}
#[allow(clippy::too_many_arguments)]
async fn create_metadata_records(
    conn: &mut sqlx::PgConnection,
    file_name: &str,
    work_name: &str,
    rom_name: &str,
    platform: &str,
    core_name: &str,
    core_version: &str,
    indexer: &gisst::search::MeiliIndexer,
) -> Result<Uuid, IngestError> {
    // TODO: merge entries from result on libretrodb_query_compile(db, "{\"rom_name\":nom}", strlen query exp, error) cursor
    let created_on = chrono::Utc::now();
    let work = Work {
        work_id: Uuid::new_v4(),
        work_name: work_name.to_string(),
        work_version: rom_name.to_string(),
        work_platform: platform.to_string(),
        // TODO this should use the real cataloguing data
        created_on,
        work_derived_from: None,
    };
    info!("creating work {} with file {file_name}", work.work_name);
    let env = Environment {
        environment_id: Uuid::new_v4(),
        environment_name: work.work_name.clone(),
        environment_framework: Framework::RetroArch,
        environment_core_name: core_name.to_string(),
        environment_core_version: core_version.to_string(),
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
        derived_from_instance: None,
        derived_from_state: None,
    };
    Work::insert(conn, work).await?;
    Environment::insert(conn, env).await?;
    Instance::insert(conn, instance, indexer).await?;
    Ok(instance_id)
}

async fn create_single_file_instance_objects(
    conn: &mut sqlx::PgConnection,
    storage_root: &str,
    roms: &std::path::Path,
    instance_id: Uuid,
    path: &std::path::Path,
    desc: Option<String>,
) -> Result<(), IngestError> {
    let object_id = insert_file_object(
        conn,
        storage_root,
        DEPTH,
        path,
        None,
        desc,
        path.strip_prefix(roms)
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        Duplicate::ReuseData,
    )
    .await?;
    Object::link_object_to_instance(conn, object_id, instance_id, ObjectRole::Content, 0).await?;
    Ok(())
}

async fn create_playlist_instance_objects(
    conn: &mut sqlx::PgConnection,
    storage_root: &str,
    roms: &std::path::Path,
    instance_id: Uuid,
    path: &std::path::Path,
    desc: Option<String>,
) -> Result<(), IngestError> {
    let src_path = path
        .strip_prefix(roms)
        .unwrap()
        .parent()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let playlist_id = insert_file_object(
        conn,
        storage_root,
        DEPTH,
        path,
        None,
        desc.clone(),
        src_path.clone(),
        Duplicate::ReuseData,
    )
    .await?;
    Object::link_object_to_instance(conn, playlist_id, instance_id, ObjectRole::Content, 0).await?;
    let mut c_idx = 1;
    info!("inserting playlist objects {path:?}");
    for file in files_of_playlist(roms, path)? {
        let file_id = insert_file_object(
            conn,
            storage_root,
            DEPTH,
            &file,
            None,
            desc.clone(),
            src_path.clone(),
            Duplicate::ReuseData,
        )
        .await?;
        info!("linking {file_id} with {instance_id}");
        Object::link_object_to_instance(conn, file_id, instance_id, ObjectRole::Content, c_idx)
            .await?;
        c_idx += 1;
    }
    Ok(())
}

async fn link_deps(
    conn: &mut sqlx::PgConnection,
    ra_cfg_object_id: Uuid,
    deps: &[Uuid],
    instance_id: Uuid,
) -> Result<(), IngestError> {
    Object::link_object_to_instance(conn, ra_cfg_object_id, instance_id, ObjectRole::Config, 0)
        .await?;
    for (i, dep) in deps.iter().enumerate() {
        Object::link_object_to_instance(
            conn,
            *dep,
            instance_id,
            ObjectRole::Dependency,
            u16::try_from(i).map_err(|_| IngestError::RoleTooHigh(i))?,
        )
        .await?;
    }
    Ok(())
}

fn files_of_playlist(
    roms: &std::path::Path,
    path: &std::path::Path,
) -> Result<impl IntoIterator<Item = std::path::PathBuf>, IngestError> {
    let mut out = Vec::with_capacity(8);
    let cue_file_re = regex::Regex::new("^FILE \"(.*)\"").unwrap();
    for file in std::fs::read_to_string(path)?.lines() {
        let file_path = roms.join(std::path::Path::new(file));
        out.push(file_path.clone());
        info!("read playlist line {file:?}");
        if file_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("cue"))
        {
            info!("read cue file {file_path:?}");
            for cue_line in std::fs::read_to_string(file_path)?.lines() {
                info!("read cue line {cue_line}");
                if let Some(captures) = cue_file_re.captures(cue_line) {
                    info!("cap: {captures:?}");
                    let track = &captures[1];
                    let track_path = roms.join(std::path::Path::new(track));
                    out.push(track_path.clone());
                }
            }
        } else {
            // it was e.g. a chd file with combined tracks
        }
    }
    Ok(out.into_iter())
}
