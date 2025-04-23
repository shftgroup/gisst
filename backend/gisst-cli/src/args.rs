use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use gisst::models::ObjectRole;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GISSTCliError {
    #[error("create creator error")]
    CreateCreator(String),
    #[error("create instance error")]
    CreateInstance(String),
    #[error("create environment error")]
    CreateEnvironment(String),
    #[error("create instance error")]
    CreateWork(String),
    #[error("create state error")]
    CreateState(String),
    #[error("create save error")]
    CreateSave(String),
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
    NewModel(#[from] gisst::error::RecordSQL),
    #[error("json parse error")]
    JsonParse(#[from] serde_json::Error),
    #[error("storage error")]
    Storage(#[from] gisst::error::Storage),
    #[error("configuration error")]
    Config(#[from] config::ConfigError),
    #[error("record not found error")]
    RecordNotFound(Uuid),
    #[error("invalid link record type")]
    InvalidRecordType(String),
    #[error("v86 clone error")]
    V86CloneError(#[from] gisst::error::V86Clone),
    #[error("insert file error")]
    InsertFileError(#[from] gisst::error::InsertFile),
    #[error("invalid role index {0}")]
    InvalidRoleIndex(std::num::TryFromIntError),
    #[error("file size int conversion")]
    FileSize(std::num::TryFromIntError),
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct GISSTCli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    /// `GISST_CONFIG_PATH` environment variable must be set
    #[clap(env)]
    pub gisst_config_path: String,
}

#[derive(Debug, Args)]
pub struct GISSTCommand<T: clap::FromArgMatches + clap::Subcommand> {
    #[command(subcommand)]
    pub command: T,
}

#[derive(Debug, Subcommand)]
pub enum BaseSubcommand<C: clap::FromArgMatches + clap::Args> {
    /// Create record(s)
    Create(C),
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Recalculate file sizes and compressed sizes
    RecalcSizes,

    /// Link records together
    Link {
        /// Record type that is being linked to another record
        record_type: String,
        source_uuid: Uuid,
        target_uuid: Uuid,
        #[arg(long)]
        role: Option<ObjectRole>,
        #[arg(long)]
        role_index: Option<u16>,
    },

    /// Manage object records and files
    Object(GISSTCommand<BaseSubcommand<CreateObject>>),
    /// Manage instance records
    Instance(GISSTCommand<BaseSubcommand<CreateInstance>>),
    /// Manage work records
    Work(GISSTCommand<BaseSubcommand<CreateWork>>),
    /// Manage creator records
    Creator(GISSTCommand<BaseSubcommand<CreateCreator>>),
    /// Manage environment records
    Environment(GISSTCommand<BaseSubcommand<CreateEnvironment>>),
    /// Manage save records
    Save(GISSTCommand<BaseSubcommand<CreateSave>>),
    /// Manage state records
    State(GISSTCommand<BaseSubcommand<CreateState>>),
    /// Manage replay records
    Replay(GISSTCommand<BaseSubcommand<CreateReplay>>),
    /// Manage screenshot records
    Screenshot(GISSTCommand<BaseSubcommand<CreateScreenshot>>),

    /// Clone a v86 machine and state into a new instance
    CloneV86 {
        instance: Uuid,
        state: Uuid,
        #[arg(default_value_t = 4)]
        depth: u8,
    },

    /// Derives a new work and instance for a romhack or other patched instance, using the existing objects plus some patches
    AddPatch {
        /// The instance to clone and patch along with its work
        instance: Uuid,
        /// A JSON file containing a JSON string that parses as `PatchData` for the new work
        data: String,
        #[arg(default_value_t = 4)]
        depth: u8,
    },
}

#[derive(Debug, Args)]
pub struct CreateObject {
    /// Link to a specific instance based on UUID
    #[arg(short, long)]
    pub link: Option<Uuid>,

    /// Object role for instance link. Must be one of "content", "dependency", or "config".
    #[arg(long)]
    pub role: gisst::models::ObjectRole,

    /// Object role index for instance link. For retroarch, this is typically 0 for all content; for v86, 0=fda, 1=fdb, 2=hda, 3=hdb, 4=cdrom.
    #[arg(long)]
    pub role_index: u16,

    /// Folder depth to use for input file to path based off of characters in assigned UUID
    #[arg(short, long, default_value_t = 4)]
    pub depth: u8,

    /// (DEBUG) Force the use of specific UUID, only works with a single object create
    #[arg(long = "force-uuid")]
    pub force_uuid: Uuid,

    /// Paths of file(s) to create in the database, directories will be ignored unless -r/--recursive flag is enabled
    pub file: String,

    /// Search for files in this directory, useful if you don't want deeply nested `file_source_paths`.
    #[arg(long)]
    pub cwd: Option<String>,
}

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
pub struct CreateSave {
    /// Link to a specific instance based on UUID
    #[arg(short, long)]
    pub link: Uuid,

    /// Folder depth to use for input file to path based off of characters in assigned UUID
    #[arg(short, long, default_value_t = 4)]
    pub depth: u8,

    /// (DEBUG) Force the use of specific save UUID
    #[arg(long = "force-uuid")]
    pub force_uuid: Option<Uuid>,

    /// Paths of file to create in the database, must be a regular file
    #[arg(long)]
    pub file: String,

    #[arg(long = "name")]
    pub save_short_desc: String,
    #[arg(long = "description")]
    pub save_description: Option<String>,
    #[arg(long = "from-state")]
    pub state_derived_from: Option<Uuid>,
    #[arg(long = "from-save")]
    pub save_derived_from: Option<Uuid>,
    #[arg(long = "from-replay")]
    pub replay_derived_from: Option<Uuid>,
    #[arg(long = "creator-id")]
    pub creator_id: Uuid,
    #[arg(long = "created-on")]
    pub created_on: Option<String>,
    #[arg(long = "version")]
    pub version: Option<i64>,
}

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
pub struct CreateScreenshot {
    /// (DEBUG) Force the use of specific screenshot UUID
    #[arg(long = "force-uuid")]
    pub force_uuid: Option<Uuid>,

    /// Path to image file to create in the database, must be a PNG file
    pub file: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct PatchData {
    pub version: String,
    pub name: String,
    pub files: Vec<String>,
}
