mod db;
mod serverconfig;
mod server;
mod templates;
mod routes;

use anyhow::Result;
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    let config = serverconfig::ServerConfig::new()?;
    debug!(?config);
    server::launch(&config).await?;
    Ok(())
}