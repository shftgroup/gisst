use std::net::Ipv4Addr;
use axum::extract::rejection::InvalidUtf8InPathParam;
use axum::Server;

use config::{Config, ConfigError, Environment, File};
use secrecy::Secret;
use serde::Deserialize;
use sqlx::Encode;
use tracing_subscriber::registry::Data;

//Configuration file setup taken from https://github.com/shanesveller/axum-rest-example/blob/develop/src/config.rs
#[derive(Debug,Default,Deserialize)]
pub struct ServerConfig {
    #[serde(default)]
    pub database: DatabaseConfig,

    #[serde(default)]
    pub http: HttpConfig,
}

impl ServerConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let env = std::env::var("GISST_ENV").unwrap_or_else(|_| "development".into());

        let builder = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("GISST").separator("__"));

        builder.build()?.try_deserialize()
    }
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    #[serde(default = "default_database_url")]
    pub url: Secret<String>,
}

fn default_database_url() -> Secret<String> {
    Secret::new("postgresql://postgres:postgres@localhost/gisst_db".to_owned())
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 128,
            min_connections: 8,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: 900,
            max_lifetime_seconds: 3600,
            url: default_database_url(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_listen_address")]
    pub listen_address: Ipv4Addr,

    #[serde(default = "default_port")]
    pub listen_port: u16,
}

fn default_listen_address() -> Ipv4Addr {
    Ipv4Addr::new(0,0,0,0)
}

fn default_port() -> u16 {
    3000
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0".parse().unwrap(),
            listen_port: 3000,
        }
    }
}