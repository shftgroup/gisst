use clap::{
    Parser,
    Args,
    Subcommand
};

use clap_verbosity_flag::Verbosity;
use uuid::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GISSTCliError {
    #[error("create object error")]
    CreateObjectError(String),
    #[error("create object error")]
    CreateImageError(String),
    #[error("create instance error")]
    CreateInstanceError(String),
    #[error("create instance error")]
    CreateWorkError(String),
    #[error("directory traversal error")]
    DirectoryError(#[from] walkdir::Error),
    #[error("file read error")]
    IoError(#[from] std::io::Error),
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
    #[error("gisst new model error")]
    NewModelError(#[from] gisstlib::models::NewRecordError),
    #[error("json parse error")]
    JsonParseError(#[from] serde_json::Error),
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
    Instance(InstanceCommand),
    Work(WorkCommand),
    Creator(CreatorCommand),
    Environment(EnvironmentCommand),
}

#[derive(Debug, Args)]
pub struct ObjectCommand {
    #[command(subcommand)]
    pub command: ObjectSubcommand,
}

#[derive(Debug, Args)]
pub struct EnvironmentCommand {
    #[command(subcommand)]
    pub command: EnvironmentSubcommand,
}

#[derive(Debug, Args)]
pub struct ImageCommand {
    #[command(subcommand)]
    pub command: ImageSubcommand,
}

#[derive(Debug, Args)]
pub struct InstanceCommand {
    #[command(subcommand)]
    pub command: InstanceSubcommand,
}

#[derive(Debug, Args)]
pub struct WorkCommand {
    #[command(subcommand)]
    pub command: WorkSubcommand,
}
#[derive(Debug, Args)]
pub struct CreatorCommand {
    #[command(subcommand)]
    pub command: CreatorSubcommand,
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

#[derive(Debug, Subcommand)]
pub enum ImageSubcommand {
    /// Create image(s)
    Create(CreateImage),

    /// Update an existing image
    Update(UpdateImage),

    /// Delete an existing image
    Delete(DeleteImage),

    /// Locate images in CLI interface
    Locate(LocateImage),

    /// Export images
    Export(ExportImage),
}

#[derive(Debug, Subcommand)]
pub enum EnvironmentSubcommand {
    /// Create environment(s)
    Create(CreateEnvironment),

    /// Update an existing environment
    Update(UpdateEnvironment),

    /// Delete an existing environment
    Delete(DeleteEnvironment),

    /// Locate environments in CLI interface
    Locate(LocateEnvironment),

    /// Export environments
    Export(ExportEnvironment),
}

#[derive(Debug, Subcommand)]
pub enum InstanceSubcommand {
    /// Create instance(s)
    Create(CreateInstance),

    /// Update an existing instance
    Update(UpdateInstance),

    /// Delete an existing instance
    Delete(DeleteInstance),

    /// Locate instances in CLI interface
    Locate(LocateInstance),

    /// Export instances
    Export(ExportInstance),
}

#[derive(Debug, Subcommand)]
pub enum WorkSubcommand {
    /// Create work(s)
    Create(CreateWork),

    /// Delete an existing work
    Delete(DeleteWork),
}

#[derive(Debug, Subcommand)]
pub enum CreatorSubcommand {

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

    /// (DEBUG) Force the use of specific UUID, only works with a single object create
    #[arg(long="force-uuid")]
    pub force_uuid: Uuid,

    /// Paths of file(s) to create in the database, directories will be ignored unless -r/--recursive flag is enabled
    pub file: Vec<String>,
}

#[derive(Debug, Args)]
pub struct UpdateObject {
    /// Uuid of object to update
    pub id: Uuid,

    /// Update object based on a provided JSON string
    #[arg(long="json-string", group="json_input")]
    pub json_string: serde_json::Value,

    /// Update object based on a provided JSON file
    #[arg(long="json-file", group="json_input")]
    pub json_file: String,
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

#[derive(Debug, Args)]
pub struct CreateInstance {

    /// Provide a JSON string to create instance
    #[arg(long="json-string", group="json_input")]
    pub json_string: Option<serde_json::Value>,

    /// Provide a JSON file to create instance
    #[arg(long="json-file", group="json_input")]
    pub json_file: Option<String>,

    /// Provide a JSON file for instance configuration
    #[arg(long="instance-config-file", group="config_input")]
    pub instance_config_json_file: Option<String>,

    /// Provide a JSON string for instance configuration
    #[arg(long="instance-config-string", group="config_input")]
    pub instance_config_json_string: Option<serde_json::Value>,

}

#[derive(Debug, Args)]
pub struct UpdateInstance {

}
#[derive(Debug, Args)]
pub struct DeleteInstance {
    /// Uuid to delete from the database, this will also disconnect the instance from any associated objects
    pub id: Uuid,

}
#[derive(Debug, Args)]
pub struct ExportInstance {

}
#[derive(Debug, Args)]
pub struct LocateInstance {

}
#[derive(Debug, Args)]
pub struct CreateImage {
    /// Will skip requests for a description for an image and default to using the image's filename
    #[arg(short, long="ignore-description")]
    pub ignore_description: bool,

    /// Will answer yes "y" to all "y/n" prompts on record creation
    #[arg(short='y', long="skip-yes")]
    pub skip_yes: bool,

    /// Link to a specific environment based on UUID
    #[arg(short, long)]
    pub link: Option<Uuid>,

    /// Folder depth to use for input file to path based off of characters in assigned UUID
    #[arg(short, long, default_value_t = 4)]
    pub depth: u8,

    /// (DEBUG) Force the use of specific UUID, only works with a single image create
    #[arg(long="force-uuid")]
    pub force_uuid: Uuid,

    /// Paths of image files
    pub file: Vec<String>,

}
#[derive(Debug, Args)]
pub struct UpdateImage {

}
#[derive(Debug, Args)]
pub struct DeleteImage {
    /// Uuid to delete from the database, this will also disconnect the image from any associated environments
    pub id: Uuid,

}
#[derive(Debug, Args)]
pub struct ExportImage {

}
#[derive(Debug, Args)]
pub struct LocateImage {

}
#[derive(Debug, Args)]
pub struct CreateEnvironment {

    /// Provide a JSON string to create instance
    #[arg(long="json-string", group="json_input")]
    pub json_string: Option<serde_json::Value>,

    /// Provide a JSON file to create instance
    #[arg(long="json-file", group="json_input")]
    pub json_file: Option<String>,

    /// Provide a JSON file for environment configuration
    #[arg(long="environment-config-file", group="config_input")]
    pub environment_config_json_file: Option<String>,

    /// Provide a JSON string for environment configuration
    #[arg(long="environment-config-string", group="config_input")]
    pub environment_config_json_string: Option<serde_json::Value>,

}
#[derive(Debug, Args)]
pub struct UpdateEnvironment {

}
#[derive(Debug, Args)]
pub struct DeleteEnvironment {
    /// Uuid to delete from the database, this will also disconnect the environment from any associated images
    pub id: Uuid,

}
#[derive(Debug, Args)]
pub struct ExportEnvironment {

}
#[derive(Debug, Args)]
pub struct LocateEnvironment {

}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct CreateWork {
    /// Provide a JSON string to create work
    #[arg(long="json-string")]
    pub json_string: Option<serde_json::Value>,

    /// Provide a JSON file to create work
    #[arg(long="json-file")]
    pub json_file: Option<String>,
}

#[derive(Debug, Args)]
pub struct DeleteWork {
    /// Uuid to delete from the database
    pub id: Uuid,

}