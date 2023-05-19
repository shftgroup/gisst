use std::time::Duration;

use crate::serverconfig::ServerConfig;

use secrecy::ExposeSecret;
use sqlx::{pool::PoolOptions, PgPool,};

pub(crate) async fn new_pool(config: &ServerConfig) -> sqlx::Result<PgPool> {
    PoolOptions::new()
        .acquire_timeout(Duration::from_secs(config.database.connect_timeout_seconds))
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .max_lifetime(Duration::from_secs(config.database.max_lifetime_seconds))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout_seconds))
        .connect(config.database.url.expose_secret())
        .await
}