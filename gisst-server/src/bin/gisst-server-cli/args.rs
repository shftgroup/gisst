use clap::{
    Parser,
    Command,
    Args,
    Subcommand
};

use clap_verbosity_flag::Verbosity;
use uuid::Uuid;
use thiserror::Error;
use gisstlib::GISSTError;

#[derive(Debug, Error)]
pub enum GISSTCliError {
    #[error("create object error")]
    CreateObjectError(String),
    #[error("directory traversal error")]
    DirectoryError(#[from] walkdir::Error),
    #[error("file read error")]
    IoError(#[from] std::io::Error),
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
    #[error("gisst new model error")]
    NewModelError(#[from] gisstlib::models::NewRecordError)
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct GISSTCli {
    #[command(subcommand)]
    pub record_type: RecordType,

    #[command(flatten)]
    pub verbose: Verbosity,

    /// GISST_CLI_DB_URL environment variable must be set to PostgreSQL path
    #[clap(env)]
    pub gisst_cli_db_url: String,

    /// GISST_STORAGE_ROOT_PATH environment variable must be set
    #[clap(env)]
    pub gisst_storage_root_path: String,

}

#[derive(Debug, Subcommand)]
pub enum RecordType {
    Object(ObjectCommand),
    Image(ImageCommand),
    Work(WorkCommand),
    Creator(CreatorCommand),
}

#[derive(Debug, Args)]
pub struct ObjectCommand {
    #[command(subcommand)]
    pub command: ObjectSubcommand,
}

#[derive(Debug, Args)]
pub struct ImageCommand {

}

#[derive(Debug, Args)]
pub struct WorkCommand {

}
#[derive(Debug, Args)]
pub struct CreatorCommand {

}

#[derive(Debug, Subcommand)]
pub enum ObjectSubcommand {
    /// Create object(s)
    Create(CreateObject),

    /// Update an existing object
    Update(UpdateObject),

    /// Delete an existing object
    Delete(DeleteObject),

    /// Locate objects in CLI interface
    Locate(LocateObject),

    /// Export objects
    Export(ExportObject),
}

#[derive(Debug, Args)]
pub struct CreateObject {
    /// Create objects recursively if input file is directory
    #[arg(short, long)]
    pub recursive: bool,

    /// Extract any archive files and create individual object records for each extracted file, this is recursive
    #[arg(short, long)]
    pub extract: bool,

    /// Will skip requests for a description for an object and default to using the object's filename
    #[arg(short, long="ignore-description")]
    pub ignore_description: bool,

    /// Will answer yes "y" to all "y/n" prompts on record creation
    #[arg(short='y', long="skip-yes")]
    pub skip_yes: bool,

    /// Link to a specific instance based on UUID
    #[arg(short, long)]
    pub link: Option<Uuid>,

    /// Folder depth to use for input file to path based off of characters in assigned UUID
    #[arg(short, long, default_value_t = 4)]
    pub depth: u8,

    /// Paths of file(s) to create in the database, directories will be ignored unless -r/--recursive flag is enabled
    pub file: Vec<String>,
}

#[derive(Debug, Args)]
pub struct UpdateObject {

}

#[derive(Debug, Args)]
pub struct DeleteObject{
    /// Uuid to delete from the database, this will also disconnect the object from any associated instances
    pub id: Uuid,
}

#[derive(Debug, Args)]
pub struct LocateObject {

}

#[derive(Debug, Args)]
pub struct ExportObject {

}
