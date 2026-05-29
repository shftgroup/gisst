#![allow(clippy::unnecessary_debug_formatting)]
use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use gisst::models::RDBWork;
use log::info;
use rdb_sys::RDB;
use sqlx::PgPool;
use sqlx::pool::PoolOptions;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(flatten)]
    pub verbose: Verbosity,

    /// `DATABASE_URL` environment variable must be set to database path
    #[clap(env, long = "database-url")]
    pub database_url: String,

    #[clap(long = "platform")]
    pub platform: String,

    #[clap()]
    pub rdb: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MirrorRDBError {
    #[error("file read error")]
    Io(#[from] std::io::Error),
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("insert error {0}")]
    Insert(#[from] gisst::error::Insert),
    #[error("nul error")]
    Nul(#[from] std::ffi::NulError),
    #[error("new record")]
    NewRecord(#[from] gisst::error::RecordSQL),
    #[error("rdb open error")]
    RDB(),
    #[error("file metadata error")]
    File(),
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<(), MirrorRDBError> {
    let Args {
        platform,
        rdb,
        database_url,
        verbose,
    } = Args::parse();
    env_logger::Builder::new()
        .filter_level(verbose.log_level_filter())
        .init();
    info!("Connecting to database: {database_url}");
    let pool: PgPool = get_db_by_url(database_url.clone()).await?;
    info!("DB connection successful.");
    let rdb = RDB::open(std::path::Path::new(&rdb)).map_err(|_| MirrorRDBError::RDB())?;
    let cursor = rdb.open_cursor().ok_or(MirrorRDBError::RDB())?;
    let mut tx = pool.begin().await?;
    for rval in cursor {
        use rdb_sys::RType;
        assert_eq!(rval.tag, RType::Map);
        let Some(name) = rval.map_get("name") else {
            println!("{rval}");
            println!("No name specified for rval");
            continue;
        };
        let work = RDBWork {
            platform: platform.clone(),
            name,
            serial: rval.map_get("serial"),
            sha1: rval.map_get("sha1"),
            crc: rval.map_get("crc"),
            md5: rval.map_get("md5"),
            size: rval.map_get("size"),
            description: rval.map_get("description"),
            releaseyear: rval.map_get("releaseyear"),
            releasemonth: rval.map_get("releasemonth"),
            releaseday: rval.map_get("releaseday"),
            genre: rval.map_get("genre"),
            analog: rval.map_get("analog"),
            famitsu_rating: rval.map_get("famitsu_rating"),
            franchise: rval.map_get("franchise"),
            publisher: rval.map_get("publisher"),
            rom_name: rval.map_get("rom_name"),
            users: rval.map_get("users"),
            esrb_rating: rval.map_get("esrb_rating"),
            edge_issue: rval.map_get("edge_issue"),
            rumble: rval.map_get("rumble"),
            origin: rval.map_get("origin"),
            enhancement_hw: rval.map_get("enhancement_hw"),
            elspa_rating: rval.map_get("elspa_rating"),
            edge_rating: rval.map_get("edge_rating"),
            region: rval.map_get("region"),
            developer: rval.map_get("developer"),
        };
        RDBWork::insert(&mut tx, work).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new().connect(&db_url).await
}
