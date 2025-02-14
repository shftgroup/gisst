mod auth;
mod db;
mod error;
mod routes;
mod selective_serve_dir;
mod server;
mod serverconfig;
mod tus;
mod utils;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();
    let path = std::env::current_dir()?;
    let config = serverconfig::ServerConfig::new()?;

    let default_tracing_directive = config
        .env
        .default_directive
        .clone()
        .parse()
        .expect("default tracing directive has to be valid");
    let filter = EnvFilter::builder()
        .with_default_directive(default_tracing_directive)
        .parse(config.env.rust_log.clone())?;
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tracing::info!("The current directory is {}", path.display());
    tracing::debug!("{:?}", &config);

    server::launch(&config).await?;
    Ok(())
}
