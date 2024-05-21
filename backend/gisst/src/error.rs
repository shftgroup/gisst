use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ErrorAction {
    Insert,
    Update,
    Select,
    Delete,
}

impl fmt::Display for ErrorAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ErrorAction::Insert => "insert",
            ErrorAction::Update => "update",
            ErrorAction::Select => "select",
            ErrorAction::Delete => "delete",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub enum ErrorTable {
    Creator,
    Environment,
    EnvironmentImage,
    File,
    Image,
    Instance,
    InstanceObject,
    Object,
    Replay,
    Save,
    Screenshot,
    State,
    Users,
    Work,
}

impl fmt::Display for ErrorTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ErrorTable::Creator => "creator",
            ErrorTable::Environment => "environment",
            ErrorTable::EnvironmentImage => "environment_image",
            ErrorTable::File => "file",
            ErrorTable::Image => "image",
            ErrorTable::Instance => "instance",
            ErrorTable::InstanceObject => "instance_object",
            ErrorTable::Object => "object",
            ErrorTable::Replay => "replay",
            ErrorTable::Save => "save",
            ErrorTable::Screenshot => "screenshot",
            ErrorTable::State => "state",
            ErrorTable::Users => "users",
            ErrorTable::Work => "work",
        };
        write!(f, "{}", s)
    }
}
#[derive(thiserror::Error, Debug)]
pub struct RecordSQLError {
    pub table: ErrorTable,
    pub action: ErrorAction,
    pub source: sqlx::Error,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("tokio task error")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("path prefix error")]
    PathPrefixError(#[from] std::path::StripPrefixError),
    #[error("Storage file not found")]
    FileNotFoundError,
}

impl fmt::Display for RecordSQLError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} failed in table {}", self.table, self.action)
    }
}