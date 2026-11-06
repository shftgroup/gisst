use std::time::Duration;

use server_types::config::Config;

use secrecy::ExposeSecret;
use sqlx::{PgPool, pool::PoolOptions};

pub(crate) async fn new_pool(config: &Config) -> sqlx::Result<PgPool> {
    PoolOptions::new()
        .acquire_timeout(Duration::from_secs(config.database.connect_timeout_seconds))
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .max_lifetime(Duration::from_secs(config.database.max_lifetime_seconds))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout_seconds))
        .connect(config.database.database_url.expose_secret())
        .await
}
