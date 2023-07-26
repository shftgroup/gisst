use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use log::{debug, error, info, warn};
use sqlx::pool::PoolOptions;
use sqlx::PgPool;
use walkdir::WalkDir;

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
    #[error("directory traversal error")]
    Directory(#[from] walkdir::Error),
    #[error("rdb open error")]
    RDB(),
}

#[tokio::main]
async fn main() -> Result<(), IngestError> {
    let Args {
        rdb,
        dir: _dir,
        core_name: _core_name,
        core_version: _core_version,
        gisst_cli_db_url,
        gisst_storage_root_path,
        verbose,
    } = Args::parse();
    info!("Connecting to database: {}", gisst_cli_db_url.to_string());
    let _db: PgPool = get_db_by_url(gisst_cli_db_url.to_string()).await?;
    info!("DB connection successful.");
    let storage_root = gisst_storage_root_path.to_string();
    info!("Storage root is set to: {}", &storage_root);
    env_logger::Builder::new()
        .filter_level(verbose.log_level_filter())
        .init();
    let rdb_path_c = std::ffi::CString::new(rdb)?;
    unsafe {
        let db: *mut RetroDB = std::ptr::null_mut();
        let cursor: *mut RetroCursor = std::ptr::null_mut();
        let rval: *mut RVal = std::ptr::null_mut();
        info!("opening DB");
        if libretrodb_open(rdb_path_c.as_ptr(), db) != 0 {
            error!("Not opened {rdb_path_c:?}");
            return Err(IngestError::RDB());
        }
        info!("Opened DB");
        if libretrodb_cursor_open(db, cursor, std::ptr::null()) != 0 {
            error!("No cursor");
            return Err(IngestError::RDB());
        }
        info!("Got cursor");
        while libretrodb_cursor_read_item(cursor, rval) == 0 {
            info!("Read item");
            rmsgpack_dom_value_print(rval);
            rmsgpack_dom_value_free(rval);
        }
        libretrodb_cursor_close(cursor);
        libretrodb_cursor_free(cursor);
        libretrodb_close(db);
        libretrodb_free(db);
    }
    Ok(())
}

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new().connect(&db_url).await
}
