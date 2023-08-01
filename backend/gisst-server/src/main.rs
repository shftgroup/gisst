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
    let path = std::env::current_dir()?;
    println!("The current directory is {}", path.display());
    let config = serverconfig::ServerConfig::new()?;
    println!("{:?}", &config);
    debug!(?config);
    server::launch(&config).await?;
    Ok(())
}
