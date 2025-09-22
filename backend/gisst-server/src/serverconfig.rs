use std::net::Ipv4Addr;

use config::{Config, ConfigError, Environment, File};
use secrecy::SecretString;
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

    #[serde(default)]
    pub auth: AuthConfig,

    #[serde(default)]
    pub env: EnvConfig,

    #[serde(default)]
    pub search: SearchConfig,
}

impl ServerConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let env = std::env::var("GISST_ENV").unwrap_or_else(|_| "development".into());
        let builder = Config::builder()
            .add_source(File::with_name("config/default.toml"))
            .add_source(File::with_name(&format!("config/{env}.toml")).required(false))
            .add_source(File::with_name("config/local.toml").required(false))
            .add_source(Environment::with_prefix("GISST").separator("__"));
        builder.build()?.try_deserialize()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(clippy::struct_field_names)]
pub struct SearchConfig {
    pub meili_url: String,
    pub meili_external_url: String,
    pub meili_api_key: String,
    pub meili_search_key: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvConfig {
    // RUST_LOG env variable as parsed by EnvFilter in tracing_subscriber
    // May change this to an environment (dev / test / prod) variable with internal
    pub rust_log: String,
    pub trace_include_headers: bool,
    // Jaeger endpoint is the address and port to which the tracer will export traces
    pub jaeger_endpoint: String,
    pub prometheus_endpoint: String,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            rust_log: "warn,gisst_server=debug,gisst=debug".to_string(),
            trace_include_headers: false,
            jaeger_endpoint: String::new(),
            prometheus_endpoint: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub google_client_id: SecretString,
    pub google_client_secret: SecretString,
    pub user_whitelist: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            google_client_id: SecretString::new("PROVIDE GOOGLE CLIENT ID in local.toml".into()),
            google_client_secret: SecretString::new(
                "PROVIDE GOOGLE CLIENT SECRET in local.toml".into(),
            ),
            user_whitelist: vec![],
        }
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
    pub database_url: SecretString,
}

fn default_database_url() -> SecretString {
    SecretString::new("postgresql://postgres:postgres@localhost/gisstdb".into())
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
    10_485_760
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

#[derive(Clone, Debug, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_listen_address")]
    pub listen_address: Ipv4Addr,

    #[serde(default = "default_port")]
    pub listen_port: u16,

    #[serde(default = "default_base_url")]
    pub base_url: String,

    pub dev_cert: Option<String>,
    pub dev_key: Option<String>,

    #[serde(default)]
    pub dev_ssl: bool,
}

// Not sure if all of this is redundant, it appears so
fn default_listen_address() -> Ipv4Addr {
    Ipv4Addr::UNSPECIFIED
}

fn default_port() -> u16 {
    3000
}

fn default_base_url() -> String {
    String::from("http://localhost:3000")
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            listen_address: default_listen_address(),
            listen_port: default_port(),
            base_url: default_base_url(),
            dev_key: None,
            dev_cert: None,
            dev_ssl: false,
        }
    }
}
