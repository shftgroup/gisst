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

    #[clap(long = "dep")]
    pub deps: Vec<String>,

    #[clap(long = "dep-path")]
    pub dep_paths: Vec<String>,
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
    Storage(#[from] gisst::error::StorageError),
    #[error("new record")]
    NewRecord(#[from] gisst::error::RecordSQLError),
    #[error("directory traversal error")]
    Directory(#[from] walkdir::Error),
    #[error("rdb open error")]
    RDB(),
    #[error("file metadata error")]
    File(),
    #[error("file insertion error")]
    InsertFile(),
}

enum Duplicate {
    ReuseObject,
    ReuseData,
}

async fn insert_file_object(
    conn: &mut sqlx::PgConnection,
    storage_root: &str,
    path: &std::path::Path,
    object_description: Option<String>,
    file_source_path: String,
    duplicate: Duplicate,
) -> Result<Uuid, IngestError> {
    let created_on = chrono::Utc::now();
    let file_size = std::fs::metadata(path)?.len() as i64;
    let hash = StorageHandler::get_file_hash(path)?;
    if let Some(file_info) = GFile::get_by_hash(conn, &hash).await? {
        let object_id = match duplicate {
            Duplicate::ReuseData => {
                info!("adding duplicate file record for {path:?}");
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let file_id = Uuid::new_v4();
                let file_record = GFile {
                    file_id,
                    file_hash: file_info.file_hash,
                    file_filename: file_name,
                    file_source_path,
                    file_dest_path: file_info.file_dest_path,
                    file_size: file_info.file_size,
                    created_on,
                };
                GFile::insert(conn, file_record).await?;
                let object_id = Uuid::new_v4();
                let object = Object {
                    object_id,
                    file_id,
                    object_description,
                    created_on,
                };
                Object::insert(conn, object).await?;
                Some(object_id)
            }
            Duplicate::ReuseObject => {
                info!("skipping duplicate record for {path:?}, reusing object");
                Object::get_by_hash(conn, &file_info.file_hash)
                    .await?
                    .map(|o| o.object_id)
            }
        };
        object_id.ok_or(IngestError::InsertFile())
    } else {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let file_uuid = Uuid::new_v4();
        let file_info =
            StorageHandler::write_file_to_uuid_folder(storage_root, 4, file_uuid, &file_name, path)
                .await?;
        info!(
            "Wrote file {} to {}",
            file_info.dest_filename, file_info.dest_path
        );
        let file_record = GFile {
            file_id: file_uuid,
            file_hash: file_info.file_hash,
            file_filename: file_info.source_filename,
            file_source_path,
            file_dest_path: file_info.dest_path,
            file_size,
            created_on,
        };
        GFile::insert(conn, file_record).await?;
        let object_id = Uuid::new_v4();
        let object = Object {
            object_id,
            file_id: file_uuid,
            object_description,
            created_on,
        };
        Object::insert(conn, object).await?;
        Ok(object_id)
    }
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
        deps,
        dep_paths,
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
    let ra_cfg_object_id = insert_file_object(
        &mut conn,
        &storage_root,
        std::path::Path::new(&ra_cfg),
        Some("base retroarch config".to_string()),
        String::new(),
        Duplicate::ReuseObject,
    )
    .await?;
    let mut dep_ids = Vec::with_capacity(deps.len());
    for (i, dep) in deps.iter().enumerate() {
        let dep = std::path::Path::new(dep);
        let file_name = dep.file_name().unwrap().to_string_lossy().to_string();
        let dep_path = dep_paths.get(i).unwrap_or(&file_name);
        let dep_id = insert_file_object(
            &mut conn,
            &storage_root,
            dep,
            Some(dep_path.clone()),
            dep_path.clone(),
            Duplicate::ReuseObject,
        )
        .await?;
        dep_ids.push(dep_id);
    }
    let dep_ids = dep_ids;
    let db = RDB::open(std::path::Path::new(&rdb)).map_err(|_| IngestError::RDB())?;
    let roms = std::path::Path::new(&roms);

    for entry in walkdir::WalkDir::new(roms) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .map(std::ffi::OsStr::to_string_lossy)
            .unwrap_or(std::borrow::Cow::default());
        if ext == "chd" || ext == "cue" || ext == "bin" || ext == "iso" {
            continue;
        }
        if ext == "m3u" {
            for file in files_of_playlist(roms, entry.path())? {
                if let FindResult::InRDB(rval) = find_entry(&mut conn, &db, &file).await? {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let instance_id = create_metadata_records(
                        &mut conn,
                        &file_name,
                        &rval,
                        &platform,
                        &core_name,
                        &core_version,
                    )
                    .await?;
                    link_deps(&mut conn, ra_cfg_object_id, &dep_ids, instance_id).await?;
                    create_playlist_instance_objects(
                        &mut conn,
                        &storage_root,
                        roms,
                        instance_id,
                        entry.path(),
                        rval.map_get("description"),
                    )
                    .await?;
                    break;
                }
            }
        } else if let FindResult::InRDB(rval) = find_entry(&mut conn, &db, entry.path()).await? {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let instance_id = create_metadata_records(
                &mut conn,
                &file_name,
                &rval,
                &platform,
                &core_name,
                &core_version,
            )
            .await?;
            link_deps(&mut conn, ra_cfg_object_id, &dep_ids, instance_id).await?;
            create_single_file_instance_objects(
                &mut conn,
                &storage_root,
                roms,
                instance_id,
                entry.path(),
                rval.map_get("description"),
            )
            .await?;
        }
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
    let hash_str = format!("{:x}", hash);
    if GFile::get_by_hash(conn, &hash_str).await?.is_some() {
        info!("{:?}:{hash_str} already in DB, skip", path);
        return Ok(FindResult::AlreadyHave);
    }

    info!("{:?}: {}: {hash_str}", path, hash.len());

    if let Some(rval) = db.find_entry::<&str, &[u8]>("md5", &hash) {
        info!("metadata found\n{} for {:?}", rval, path);
        Ok(FindResult::InRDB(rval))
    } else {
        warn!("md5 not found");
        Ok(FindResult::NotInRDB)
    }
}

async fn create_metadata_records(
    conn: &mut sqlx::PgConnection,
    file_name: &str,
    rval: &RVal,
    platform: &str,
    core_name: &str,
    core_version: &str,
) -> Result<Uuid, IngestError> {
    // TODO: merge entries from result on libretrodb_query_compile(db, "{\"rom_name\":nom}", strlen query exp, error) cursor
    let created_on = chrono::Utc::now();
    let work = Work {
        work_id: Uuid::new_v4(),
        work_name: rval
            .map_get("name")
            .unwrap_or_else(|| file_name.to_string()),
        work_version: rval
            .map_get("rom_name")
            .unwrap_or_else(|| file_name.to_string()),
        work_platform: platform.to_string(),
        // TODO this should use the real cataloguing data
        created_on,
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
    Instance::insert(conn, instance).await?;
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
        path,
        desc,
        path.strip_prefix(&roms)
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
        path,
        desc.clone(),
        src_path.clone(),
        Duplicate::ReuseData,
    )
    .await?;
    Object::link_object_to_instance(conn, playlist_id, instance_id, ObjectRole::Content, 0).await?;
    let mut c_idx = 1;
    for file in files_of_playlist(roms, path)? {
        let file_id = insert_file_object(
            conn,
            storage_root,
            &file,
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
        Object::link_object_to_instance(conn, *dep, instance_id, ObjectRole::Dependency, i).await?;
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
        info!("Reading playlist line {file}");
        let file_path = roms.join(std::path::Path::new(file));
        out.push(file_path.clone());
        if file.ends_with(".cue") {
            for cue_line in std::fs::read_to_string(file_path)?.lines() {
                info!("Reading cue line {cue_line}");
                if let Some(captures) = cue_file_re.captures(cue_line) {
                    let track = &captures[1];
                    let track_path = roms.join(std::path::Path::new(track));
                    info!("track is {track}");
                    out.push(track_path.clone());
                }
            }
        } else {
            // it was e.g. a chd file with combined tracks
        }
    }
    Ok(out.into_iter())
}
