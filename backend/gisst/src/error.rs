use std::fmt;
use std::fmt::Formatter;

use uuid::Uuid;

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
    File,
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
            ErrorTable::File => "file",
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
    #[error("File path not UTF-8")]
    UTF8(#[from] std::string::FromUtf8Error),
}

impl fmt::Display for RecordSQLError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} failed in table {}", self.table, self.action)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FSListError {
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("mbr error")]
    MBRError(#[from] mbrman::Error),
    #[error("fat filesystem IO error")]
    FATFSError(#[from] fatfs::Error<std::io::Error>),
    #[error("filesystem error")]
    FATError(String),
    #[error("filetype DB error")]
    FiletypeDBError,
    #[error("subobject path error")]
    PathError,
    #[error("partition id error")]
    PartitionIDError(#[from] std::num::ParseIntError),
    #[error("directory zip error")]
    ZIPError(#[from] zip::result::ZipError),
    #[error("file identifier error")]
    FileMIMEError(#[from] magic::cookie::Error),
    #[error("fs traversal error: depth limit exceeded")]
    TraversalDepth,
    #[error("fs traversal error: path {0} invalid or not found")]
    TraversalPath(String),
    #[error("fs traversal error: file {0} invalid or not found")]
    FileNotFound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum V86CloneError {
    #[error("instance {0} not found")]
    InstanceNotFound(Uuid),
    #[error("enviroment {0} not found")]
    EnvironmentNotFound(Uuid),
    #[error("not a v86 environment")]
    WrongEnvironmentType,
    #[error("state is for different instance")]
    WrongInstanceForState,
    #[error("v86 state {0} not found")]
    StateNotFound(Uuid),
    #[error("clone script not found")]
    NoCloneScript,
    #[error("storage error")]
    Storage(#[from] crate::error::StorageError),
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("record error")]
    Record(#[from] RecordSQLError),
    #[error("v86dump error: {0}")]
    V86DumpError(String),
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("incomplete clone for {0}")]
    IncompleteClone(Uuid),
}

#[derive(Debug, thiserror::Error)]
pub enum InsertFileError {
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("Invalid or missing duplicated object for file hash {0}")]
    ObjectMissing(String),
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("record error")]
    Record(#[from] RecordSQLError),
    #[error("storage error")]
    Storage(#[from] crate::error::StorageError),
}
