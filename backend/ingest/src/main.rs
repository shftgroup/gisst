use std::ffi::CString;

use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use log::{error, info, warn};
use rdb_sys::*;
use sqlx::pool::PoolOptions;
use sqlx::PgPool;

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
        // if libretrodb_cursor_open(db, cursor, std::ptr::null()) != 0 {
        //     error!("No cursor");
        //     return Err(IngestError::RDB());
        // }
        // info!("Got cursor");
        // while libretrodb_cursor_read_item(cursor, &mut rval) == 0 {
        //     info!("Read item");
        //     rmsgpack_dom_value_print(&rval);
        //     rmsgpack_dom_value_free(&mut rval);
        // }
        info!("lookup md5");
        let input_md5_key = "3369347F7663B133CE445C15200A5AFA";
        if input_md5_key.len() != 32 {
            error!("Invalid md5 key length {}", input_md5_key.len());
            return Err(IngestError::RDB());
        }
        assert_eq!(input_md5_key.len(), 32);
        let md5_key = input_md5_key
            .as_bytes()
            .chunks_exact(2)
            .map(|arr| {
                if let [hi, lo] = arr {
                    char_to_num(*hi) * 16 + char_to_num(*lo)
                } else {
                    unreachable!("Invalid md5 key length")
                }
            })
            .collect::<Vec<u8>>();
        info!("key len {}", md5_key.len());
        let key_bytes = md5_key.as_ptr();
        let md5_idx = CString::new("md5")?;
        if libretrodb_create_index(db, md5_idx.as_ptr(), md5_idx.as_ptr()) == -1 {
            error!("Couldn't create index");
            return Err(IngestError::RDB());
        }
        if libretrodb_find_entry(db, md5_idx.as_ptr(), key_bytes, &mut rval) == 0 {
            info!("FOUND IT");
            rmsgpack_dom_value_print(&rval);
            println!();
            rmsgpack_dom_value_free(&mut rval);
        } else {
            warn!("md5 not found");
            return Err(IngestError::RDB());
        }
        // libretrodb_cursor_close(cursor);
        libretrodb_cursor_free(cursor);
        libretrodb_close(db);
        libretrodb_free(db);
    }
    Ok(())
}

async fn get_db_by_url(db_url: String) -> sqlx::Result<PgPool> {
    PoolOptions::new().connect(&db_url).await
}
fn char_to_num(c: u8) -> u8 {
    if c <= b'9' {
        c - b'0'
    } else {
        (c - b'A') + 10
    }
}
