use crate::args::GISSTCliError;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Default, Deserialize)]
pub struct CLIConfig {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,
}
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StorageConfig {
    #[serde(default = "default_root_folder_path")]
    pub root_folder_path: String,
    #[serde(default = "default_folder_depth")]
    pub folder_depth: u8,
}

impl CLIConfig {
    pub fn new(config_path: &str) -> Result<Self, GISSTCliError> {
        let env = std::env::var("GISST_ENV").unwrap_or_else(|_| "development".into());
        let binding = std::fs::canonicalize(config_path)?;
        let canonical_path = binding.as_path().to_str().unwrap();

        let mut default_path = PathBuf::from_str(canonical_path).unwrap();
        default_path.push("default.toml");
        dbg!(default_path.to_str());

        let mut local_path = PathBuf::from_str(canonical_path).unwrap();
        local_path.push("local.toml");
        dbg!(local_path.to_str());

        let mut env_path = PathBuf::from_str(canonical_path).unwrap();
        env_path.push(&env);

        let builder = Config::builder()
            .add_source(File::with_name(default_path.to_str().unwrap()))
            .add_source(
                File::with_name(&format!("{}.toml", env_path.to_str().unwrap())).required(false),
            )
            .add_source(File::with_name(local_path.to_str().unwrap()).required(false))
            .add_source(Environment::with_prefix("GISST").separator("__"));
        Ok(builder.build()?.try_deserialize()?)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: default_database_url(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            root_folder_path: default_root_folder_path(),
            folder_depth: default_folder_depth(),
        }
    }
}

fn default_root_folder_path() -> String {
    "./storage".to_string()
}

fn default_folder_depth() -> u8 {
    4
}

fn default_database_url() -> String {
    "postgresql://postgres:postgres@localhost/gisstdb".to_string()
}
