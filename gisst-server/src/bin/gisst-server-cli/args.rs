use clap::{Args, Parser, Subcommand};

use clap_verbosity_flag::Verbosity;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GISSTCliError {
    #[error("create object error")]
    CreateObject(String),
    #[error("create object error")]
    CreateImage(String),
    #[error("create instance error")]
    CreateInstance(String),
    #[error("create instance error")]
    CreateWork(String),
    #[error("directory traversal error")]
    Directory(#[from] walkdir::Error),
    #[error("file read error")]
    Io(#[from] std::io::Error),
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("gisst new model error")]
    NewModel(#[from] gisstlib::models::NewRecordError),
    #[error("json parse error")]
    JsonParse(#[from] serde_json::Error),
    #[error("record not found error")]
    RecordNotFound(Uuid),
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

#[derive(Debug, Args)]
pub struct GISSTCommand<T: clap::FromArgMatches + clap::Subcommand> {
    #[command(subcommand)]
    pub command: T,
}

#[derive(Debug, Subcommand)]
pub enum BaseSubcommand<
    C: clap::FromArgMatches + clap::Args,
    U: clap::FromArgMatches + clap::Args,
    D: clap::FromArgMatches + clap::Args,
    L: clap::FromArgMatches + clap::Args,
    E: clap::FromArgMatches + clap::Args,
> {
    /// Create record(s)
    Create(C),

    /// Update an existing record
    Update(U),

    /// Delete an existing record
    Delete(D),

    /// Locate records in CLI interface
    Locate(L),

    /// Export records
    Export(E),
}

#[derive(Debug, Subcommand)]
pub enum RecordType {
    /// Manage object records and files
    Object(
        GISSTCommand<
            BaseSubcommand<CreateObject, UpdateObject, DeleteRecord, LocateObject, ExportObject>,
        >,
    ),
    /// Manage image records and files
    Image(
        GISSTCommand<
            BaseSubcommand<CreateImage, UpdateImage, DeleteRecord, LocateImage, ExportImage>,
        >,
    ),
    /// Manage instance records
    Instance(
        GISSTCommand<
            BaseSubcommand<
                CreateInstance,
                UpdateInstance,
                DeleteInstance,
                LocateInstance,
                ExportInstance,
            >,
        >,
    ),
    /// Manage work records
    Work(GISSTCommand<BaseSubcommand<CreateWork, UpdateWork, DeleteWork, LocateWork, ExportWork>>),
    /// Manage creator records
    Creator(
        GISSTCommand<
            BaseSubcommand<
                CreateCreator,
                UpdateCreator,
                DeleteCreator,
                LocateCreator,
                ExportCreator,
            >,
        >,
    ),
    /// Manage environment records
    Environment(
        GISSTCommand<
            BaseSubcommand<
                CreateEnvironment,
                UpdateEnvironment,
                DeleteEnvironment,
                LocateEnvironment,
                ExportEnvironment,
            >,
        >,
    ),
    /// Manage save records
    Save(GISSTCommand<BaseSubcommand<CreateSave, UpdateSave, DeleteSave, LocateSave, ExportSave>>),
    /// Manage state records
    State(
        GISSTCommand<
            BaseSubcommand<CreateState, UpdateState, DeleteState, LocateState, ExportState>,
        >,
    ),
    /// Manage replay records
    Replay(
        GISSTCommand<
            BaseSubcommand<CreateReplay, UpdateReplay, DeleteReplay, LocateReplay, ExportReplay>,
        >,
    ),
}

#[derive(Debug, Args)]
pub struct DeleteRecord {
    /// Uuid of record to delete from database
    pub id: Uuid,
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
    #[arg(short, long = "ignore-description")]
    pub ignore_description: bool,

    /// Will answer yes "y" to all "y/n" prompts on record creation
    #[arg(short = 'y', long = "skip-yes")]
    pub skip_yes: bool,

    /// Link to a specific instance based on UUID
    #[arg(short, long)]
    pub link: Uuid,

    /// Object role for instance link. Must be one of "content", "dependency", or "config".
    #[arg(long)]
    pub role: gisstlib::models::ObjectRole,

    /// Folder depth to use for input file to path based off of characters in assigned UUID
    #[arg(short, long, default_value_t = 4)]
    pub depth: u8,

    /// (DEBUG) Force the use of specific UUID, only works with a single object create
    #[arg(long = "force-uuid")]
    pub force_uuid: Uuid,

    /// Paths of file(s) to create in the database, directories will be ignored unless -r/--recursive flag is enabled
    pub file: Vec<String>,
}

#[derive(Debug, Args)]
pub struct UpdateObject {
    /// Uuid of object to update
    pub id: Uuid,

    /// Update object based on a provided JSON string
    #[arg(long = "json-string", group = "json_input")]
    pub json_string: serde_json::Value,

    /// Update object based on a provided JSON file
    #[arg(long = "json-file", group = "json_input")]
    pub json_file: String,
}

#[derive(Debug, Args)]
pub struct DeleteObject {
    /// Uuid to delete from the database, this will also disconnect the object from any associated instances
    pub id: Uuid,
}

#[derive(Debug, Args)]
pub struct LocateObject {}

#[derive(Debug, Args)]
pub struct ExportObject {}

#[derive(Debug, Args)]
pub struct CreateInstance {
    /// Provide a JSON string to create instance
    #[arg(long = "json-string", group = "json_input")]
    pub json_string: Option<serde_json::Value>,

    /// Provide a JSON file to create instance
    #[arg(long = "json-file", group = "json_input")]
    pub json_file: Option<String>,

    /// Provide a JSON file for instance configuration
    #[arg(long = "instance-config-file", group = "config_input")]
    pub instance_config_json_file: Option<String>,

    /// Provide a JSON string for instance configuration
    #[arg(long = "instance-config-string", group = "config_input")]
    pub instance_config_json_string: Option<serde_json::Value>,
}

#[derive(Debug, Args)]
pub struct UpdateInstance {}
#[derive(Debug, Args)]
pub struct DeleteInstance {
    /// Uuid to delete from the database, this will also disconnect the instance from any associated objects
    pub id: Uuid,
}
#[derive(Debug, Args)]
pub struct ExportInstance {}
#[derive(Debug, Args)]
pub struct LocateInstance {}
#[derive(Debug, Args)]
pub struct CreateImage {
    /// Will skip requests for a description for an image and default to using the image's filename
    #[arg(short, long = "ignore-description")]
    pub ignore_description: bool,

    /// Will answer yes "y" to all "y/n" prompts on record creation
    #[arg(short = 'y', long = "skip-yes")]
    pub skip_yes: bool,

    /// Link to a specific environment based on UUID
    #[arg(short, long)]
    pub link: Option<Uuid>,

    /// Folder depth to use for input file to path based off of characters in assigned UUID
    #[arg(short, long, default_value_t = 4)]
    pub depth: u8,

    /// (DEBUG) Force the use of specific UUID, only works with a single image create
    #[arg(long = "force-uuid")]
    pub force_uuid: Uuid,

    /// Paths of image files
    pub file: Vec<String>,
}
#[derive(Debug, Args)]
pub struct UpdateImage {}
#[derive(Debug, Args)]
pub struct DeleteImage {
    /// Uuid to delete from the database, this will also disconnect the image from any associated environments
    pub id: Uuid,
}
#[derive(Debug, Args)]
pub struct ExportImage {}
#[derive(Debug, Args)]
pub struct LocateImage {}

#[derive(Debug, Args)]
pub struct CreateEnvironment {
    /// Provide a JSON string to create instance
    #[arg(long = "json-string", group = "json_input")]
    pub json_string: Option<serde_json::Value>,

    /// Provide a JSON file to create instance
    #[arg(long = "json-file", group = "json_input")]
    pub json_file: Option<String>,

    /// Provide a JSON file for environment configuration
    #[arg(long = "environment-config-file", group = "config_input")]
    pub environment_config_json_file: Option<String>,

    /// Provide a JSON string for environment configuration
    #[arg(long = "environment-config-string", group = "config_input")]
    pub environment_config_json_string: Option<serde_json::Value>,
}
#[derive(Debug, Args)]
pub struct UpdateEnvironment {}
#[derive(Debug, Args)]
pub struct DeleteEnvironment {
    /// Uuid to delete from the database, this will also disconnect the environment from any associated images
    pub id: Uuid,
}
#[derive(Debug, Args)]
pub struct ExportEnvironment {}
#[derive(Debug, Args)]
pub struct LocateEnvironment {}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct CreateWork {
    /// Provide a JSON string to create work
    #[arg(long = "json-string")]
    pub json_string: Option<serde_json::Value>,

    /// Provide a JSON file to create work
    #[arg(long = "json-file")]
    pub json_file: Option<String>,
}

#[derive(Debug, Args)]
pub struct DeleteWork {
    /// Uuid to delete from the database
    pub id: Uuid,
}

#[derive(Debug, Args)]
pub struct UpdateWork {}
#[derive(Debug, Args)]
pub struct LocateWork {}
#[derive(Debug, Args)]
pub struct ExportWork {}
#[derive(Debug, Args)]
pub struct CreateSave {}
#[derive(Debug, Args)]
pub struct UpdateSave {}
#[derive(Debug, Args)]
pub struct DeleteSave {}
#[derive(Debug, Args)]
pub struct LocateSave {}
#[derive(Debug, Args)]
pub struct ExportSave {}
#[derive(Debug, Args)]
pub struct CreateState {}
#[derive(Debug, Args)]
pub struct UpdateState {}
#[derive(Debug, Args)]
pub struct DeleteState {}
#[derive(Debug, Args)]
pub struct LocateState {}
#[derive(Debug, Args)]
pub struct ExportState {}
#[derive(Debug, Args)]
pub struct CreateReplay {}
#[derive(Debug, Args)]
pub struct UpdateReplay {}
#[derive(Debug, Args)]
pub struct DeleteReplay {}
#[derive(Debug, Args)]
pub struct LocateReplay {}
#[derive(Debug, Args)]
pub struct ExportReplay {}
#[derive(Debug, Args)]
pub struct CreateCreator {}
#[derive(Debug, Args)]
pub struct UpdateCreator {}
#[derive(Debug, Args)]
pub struct DeleteCreator {}
#[derive(Debug, Args)]
pub struct LocateCreator {}
#[derive(Debug, Args)]
pub struct ExportCreator {}
