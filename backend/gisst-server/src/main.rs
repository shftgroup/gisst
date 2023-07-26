mod db;
mod error;
mod routes;
mod server;
mod serverconfig;
mod templates;
mod tus;
mod utils;

use anyhow::Result;
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    let config = serverconfig::ServerConfig::new()?;
    debug!(?config);
    server::launch(&config).await?;
    Ok(())
}
