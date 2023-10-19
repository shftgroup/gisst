use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use thiserror::Error;
use uuid::Uuid;
use gisst::models::ObjectRole;

#[derive(Debug, Error)]
pub enum GISSTCliError {
    #[error("create creator error")]
    CreateCreator(String),
    #[error("create object error")]
    CreateObject(String),
    #[error("create object error")]
    CreateImage(String),
    #[error("create instance error")]
    CreateInstance(String),
    #[error("create instance error")]
    CreateWork(String),
    #[error("create state error")]
    CreateState(String),
    #[error("create replay error")]
    CreateReplay(String),
    #[error("create screenshot error")]
    CreateScreenshot(String),
    #[error("directory traversal error")]
    Directory(#[from] walkdir::Error),
    #[error("file read error")]
    Io(#[from] std::io::Error),
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("gisst new model error")]
    NewModel(#[from] gisst::models::NewRecordError),
    #[error("json parse error")]
    JsonParse(#[from] serde_json::Error),
    #[error("storage error")]
    Storage(#[from] gisst::storage::StorageError),
    #[error("configuration error")]
    Config(#[from] config::ConfigError),
    #[error("record not found error")]
    RecordNotFound(Uuid),
    #[error("invalid link record type")]
    InvalidRecordType(String)
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct GISSTCli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    /// GISST_CONFIG_PATH environment variable must be set
    #[clap(env)]
    pub gisst_config_path: String,
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
    E: clap::FromArgMatches + clap::Args,
> {
    /// Create record(s)
    Create(C),

    /// Update an existing record
    Update(U),

    /// Delete an existing record
    Delete(D),

    /// Export records
    Export(E),
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Link records together
    Link {
        /// Record type that is being linked to another record
        record_type: String,
        source_uuid: Uuid,
        target_uuid: Uuid,
        #[arg(long)]
        role: Option<ObjectRole>
    },

    /// Manage object records and files
    Object(GISSTCommand<BaseSubcommand<CreateObject, UpdateObject, DeleteRecord, ExportObject>>),
    /// Manage image records and files
    Image(GISSTCommand<BaseSubcommand<CreateImage, UpdateImage, DeleteRecord, ExportImage>>),
    /// Manage instance records
    Instance(
        GISSTCommand<BaseSubcommand<CreateInstance, UpdateInstance, DeleteRecord, ExportInstance>>,
    ),
    /// Manage work records
    Work(GISSTCommand<BaseSubcommand<CreateWork, UpdateWork, DeleteRecord, ExportWork>>),
    /// Manage creator records
    Creator(
        GISSTCommand<BaseSubcommand<CreateCreator, UpdateCreator, DeleteRecord, ExportCreator>>,
    ),
    /// Manage environment records
    Environment(
        GISSTCommand<
            BaseSubcommand<CreateEnvironment, UpdateEnvironment, DeleteRecord, ExportEnvironment>,
        >,
    ),
    /// Manage save records
    Save(GISSTCommand<BaseSubcommand<CreateSave, UpdateSave, DeleteRecord, ExportSave>>),
    /// Manage state records
    State(GISSTCommand<BaseSubcommand<CreateState, UpdateState, DeleteRecord, ExportState>>),
    /// Manage replay records
    Replay(GISSTCommand<BaseSubcommand<CreateReplay, UpdateReplay, DeleteRecord, ExportReplay>>),
    /// Manage screenshot records
    Screenshot(GISSTCommand<BaseSubcommand<CreateScreenshot, UpdateScreenshot, DeleteRecord, ExportScreenshot>>),
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
    pub link: Option<Uuid>,

    /// Object role for instance link. Must be one of "content", "dependency", or "config".
    #[arg(long)]
    pub role: gisst::models::ObjectRole,

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
    pub json_string: String,

    /// Update object based on a provided JSON file
    #[arg(long = "json-file", group = "json_input")]
    pub json_file: String,
}

#[derive(Debug, Args)]
pub struct ExportObject {}

#[derive(Debug, Args)]
pub struct CreateInstance {
    /// Provide a JSON string to create instance
    #[arg(long = "json-string", group = "json_input")]
    pub json_string: Option<String>,

    /// Provide a JSON file to create instance
    #[arg(long = "json-file", group = "json_input")]
    pub json_file: Option<String>,

    /// Provide a JSON file for instance configuration
    #[arg(long = "instance-config-file", group = "config_input")]
    pub instance_config_json_file: Option<String>,

    /// Provide a JSON string for instance configuration
    #[arg(long = "instance-config-string", group = "config_input")]
    pub instance_config_json_string: Option<String>,
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
pub struct CreateEnvironment {
    /// Provide a JSON string to create instance
    #[arg(long = "json-string", group = "json_input")]
    pub json_string: Option<String>,

    /// Provide a JSON file to create instance
    #[arg(long = "json-file", group = "json_input")]
    pub json_file: Option<String>,

    /// Provide a JSON file for environment configuration
    #[arg(long = "environment-config-file", group = "config_input")]
    pub environment_config_json_file: Option<String>,

    /// Provide a JSON string for environment configuration
    #[arg(long = "environment-config-string", group = "config_input")]
    pub environment_config_json_string: Option<String>,
}
#[derive(Debug, Args)]
pub struct UpdateEnvironment {}
#[derive(Debug, Args)]
pub struct ExportEnvironment {}
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct CreateWork {
    /// Provide a JSON string to create work
    #[arg(long = "json-string")]
    pub json_string: Option<String>,

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
pub struct ExportWork {}
#[derive(Debug, Args)]
pub struct CreateSave {}
#[derive(Debug, Args)]
pub struct UpdateSave {}
#[derive(Debug, Args)]
pub struct DeleteSave {}
#[derive(Debug, Args)]
pub struct ExportSave {}
#[derive(Debug, Args)]
pub struct CreateState {
    /// Link to a specific instance based on UUID
    #[arg(short, long)]
    pub link: Uuid,

    /// Folder depth to use for input file to path based off of characters in assigned UUID
    #[arg(short, long, default_value_t = 4)]
    pub depth: u8,

    /// (DEBUG) Force the use of specific state UUID
    #[arg(long = "force-uuid")]
    pub force_uuid: Option<Uuid>,

    /// Paths of file to create in the database, must be a regular file
    #[arg(long)]
    pub file: String,

    #[arg(long = "name")]
    pub state_name: String,
    #[arg(long = "description")]
    pub state_description: Option<String>,
    #[arg(long = "screenshot-id")]
    pub screenshot_id: Uuid,
    #[arg(long = "replay-id")]
    pub replay_id: Option<Uuid>,
    #[arg(long = "creator-id")]
    pub creator_id: Uuid,
    #[arg(long = "replay-index")]
    pub state_replay_index: Option<i32>,
    #[arg(long = "derived-from")]
    pub state_derived_from: Option<Uuid>,
    #[arg(long = "created-on")]
    pub created_on: Option<String>,
}
#[derive(Debug, Args)]
pub struct UpdateState {}
#[derive(Debug, Args)]
pub struct ExportState {}
#[derive(Debug, Args)]
pub struct CreateReplay {
    /// Link to a specific instance based on UUID
    #[arg(short, long)]
    pub link: Uuid,

    /// Folder depth to use for input file to path based off of characters in assigned UUID
    #[arg(short, long, default_value_t = 4)]
    pub depth: u8,

    /// (DEBUG) Force the use of specific state UUID
    #[arg(long = "force-uuid")]
    pub force_uuid: Option<Uuid>,

    /// Paths of file to create in the database, must be a regular file
    #[arg(long)]
    pub file: String,

    #[arg(long = "name")]
    pub replay_name: Option<String>,

    #[arg(long = "description")]
    pub replay_description: Option<String>,

    #[arg(long = "creator-id")]
    pub creator_id: Option<Uuid>,

    #[arg(long = "replay-forked-from")]
    pub replay_forked_from: Option<Uuid>,

    #[arg(long = "created-on")]
    pub created_on: Option<String>,
}
#[derive(Debug, Args)]
pub struct UpdateReplay {}
#[derive(Debug, Args)]
pub struct ExportReplay {}
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct CreateCreator {
    /// Provide a JSON string to create work
    #[arg(long = "json-string")]
    pub json_string: Option<String>,

    /// Provide a JSON file to create work
    #[arg(long = "json-file")]
    pub json_file: Option<String>,
}
#[derive(Debug, Args)]
pub struct UpdateCreator {}
#[derive(Debug, Args)]
pub struct ExportCreator {}


#[derive(Debug, Args)]
pub struct CreateScreenshot {
    /// (DEBUG) Force the use of specific screenshot UUID
    #[arg(long = "force-uuid")]
    pub force_uuid: Option<Uuid>,

    /// Path to image file to create in the database, must be a PNG file
    #[arg(long)]
    pub file: String,
}