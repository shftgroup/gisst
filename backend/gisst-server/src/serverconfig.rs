use std::net::Ipv4Addr;

use config::{Config, ConfigError, Environment, File};
use secrecy::Secret;
use serde::Deserialize;

//Configuration file setup taken from https://github.com/shanesveller/axum-rest-example/blob/develop/src/config.rs
#[derive(Debug, Default, Deserialize)]
pub struct ServerConfig {
    #[serde(default)]
    pub database: DatabaseConfig,

    #[serde(default)]
    pub http: HttpConfig,

    #[serde(default)]
    pub storage: StorageConfig,
}

impl ServerConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let env = std::env::var("GISST_ENV").unwrap_or_else(|_| "development".into());
        let builder = Config::builder()
            .add_source(File::with_name("config/default.toml"))
            .add_source(File::with_name(&format!("config/{}.toml", env)).required(false))
            .add_source(File::with_name("config/local.toml").required(false))
            .add_source(Environment::with_prefix("GISST").separator("__"));
        builder.build()?.try_deserialize()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    #[serde(default = "default_database_url")]
    pub database_url: Secret<String>,
}

fn default_database_url() -> Secret<String> {
    Secret::new("postgresql://postgres:postgres@localhost/gisstdb".to_owned())
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 128,
            min_connections: 8,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: 900,
            max_lifetime_seconds: 3600,
            database_url: default_database_url(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_root_folder_path")]
    pub root_folder_path: String,
    #[serde(default = "default_folder_depth")]
    pub folder_depth: u8,
    #[serde(default = "default_temp_folder_path")]
    pub temp_folder_path: String,
    #[serde(default = "default_upload_chunk_size")]
    pub chunk_size: usize,
}

fn default_root_folder_path() -> String {
    "./storage".to_string()
}

fn default_temp_folder_path() -> String {
    "./tmp".to_string()
}

fn default_folder_depth() -> u8 {
    4
}

fn default_upload_chunk_size() -> usize {
    10485760
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            root_folder_path: default_root_folder_path(),
            folder_depth: default_folder_depth(),
            temp_folder_path: default_temp_folder_path(),
            chunk_size: default_upload_chunk_size(),
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

// Not sure if all of this is redundant, it appears so
fn default_listen_address() -> Ipv4Addr {
    Ipv4Addr::new(0, 0, 0, 0)
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
