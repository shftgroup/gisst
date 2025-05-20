use std::fmt::{self, Formatter};

use uuid::Uuid;

#[derive(Debug)]
pub enum Action {
    Insert,
    Update,
    Select,
    Delete,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Action::Insert => "insert",
            Action::Update => "update",
            Action::Select => "select",
            Action::Delete => "delete",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug)]
pub enum Table {
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

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Table::Creator => "creator",
            Table::Environment => "environment",
            Table::File => "file",
            Table::Instance => "instance",
            Table::InstanceObject => "instance_object",
            Table::Object => "object",
            Table::Replay => "replay",
            Table::Save => "save",
            Table::Screenshot => "screenshot",
            Table::State => "state",
            Table::Users => "users",
            Table::Work => "work",
        };
        write!(f, "{s}")
    }
}
#[derive(thiserror::Error, Debug)]
pub struct RecordSQL {
    pub table: Table,
    pub action: Action,
    pub source: sqlx::Error,
}
#[derive(thiserror::Error, Debug)]
pub enum Insert {
    #[error("SQL error {0}")]
    Sql(#[from] RecordSQL),
    #[error("Indexing error {0}")]
    Idx(#[from] SearchIndex),
}

#[derive(Debug, thiserror::Error)]
pub enum Storage {
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("File size too big error")]
    FileSize(#[from] std::num::TryFromIntError),
    #[error("tokio task error")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("path prefix error")]
    PathPrefixError(#[from] std::path::StripPrefixError),
    #[error("Storage file not found")]
    FileNotFoundError,
    #[error("File path not UTF-8")]
    UTF8(#[from] std::string::FromUtf8Error),
    #[error("path missing parent")]
    PathTooShallow(std::path::PathBuf),
}

impl fmt::Display for RecordSQL {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} failed in table {}", self.table, self.action)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FSList {
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("mbr error")]
    MBR(#[from] mbrman::Error),
    #[error("fat filesystem IO error")]
    FATFS(#[from] fatfs::Error<std::io::Error>),
    #[error("filesystem error")]
    FATError(String),
    #[error("filetype DB error")]
    FiletypeDB,
    #[error("subobject path error")]
    Path,
    #[error("partition id error")]
    PartitionID(#[from] std::num::ParseIntError),
    #[error("directory zip error")]
    ZIP(#[from] zip::result::ZipError),
    #[error("fs traversal error: depth limit exceeded")]
    TraversalDepth,
    #[error("fs traversal error: path {0} invalid or not found")]
    TraversalPath(String),
    #[error("fs traversal error: file {0} invalid or not found")]
    FileNotFound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum V86Clone {
    #[error("instance {0} not found")]
    InstanceNotFound(Uuid),
    #[error("disk {0} not found")]
    DiskNotFound(String),
    #[error("enviroment {0} not found")]
    EnvironmentNotFound(Uuid),
    #[error("enviroment {0} does not have a proper config")]
    EnvironmentInvalid(Uuid),
    #[error("not a v86 environment")]
    WrongEnvironmentType,
    #[error("state is for different instance")]
    WrongInstanceForState,
    #[error("v86 state {0} not found")]
    StateNotFound(Uuid),
    #[error("file path not valid UTF-8 {0}")]
    PathNotUTF8(#[from] std::string::FromUtf8Error),
    #[error("clone script not found")]
    NoCloneScript,
    #[error("storage error")]
    Storage(#[from] crate::error::Storage),
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("record error")]
    Record(#[from] RecordSQL),
    #[error("couldn't insert new record {0}")]
    Insert(#[from] crate::error::Insert),
    #[error("v86dump error: {0}")]
    V86Dump(String),
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("incomplete clone for {0}")]
    IncompleteClone(Uuid),
    #[error("Disk too big to get metadata {0}")]
    DiskTooBig(std::num::TryFromIntError),
}

#[derive(Debug, thiserror::Error)]
pub enum InsertFile {
    #[error("Invalid path or no file at path")]
    Path(std::path::PathBuf),
    #[error("too big {0}")]
    TooBig(#[from] std::num::TryFromIntError),
    #[error("IO error {0}")]
    IO(#[from] std::io::Error),
    #[error("Invalid or missing duplicated object for file hash {0}")]
    ObjectMissing(String),
    #[error("database error")]
    Sql(#[from] sqlx::Error),
    #[error("insertion error")]
    Insert(#[from] Insert),
    #[error("record error")]
    Record(#[from] RecordSQL),
    #[error("storage error")]
    Storage(#[from] Storage),
}

#[derive(Debug, thiserror::Error)]
pub enum SearchIndex {
    #[error("SQL error {0}")]
    Sql(#[from] sqlx::Error),
    #[error("Meilisearch error {0}")]
    Meili(#[from] meilisearch_sdk::errors::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Search {
    #[error("Meilisearch error {0}")]
    Meili(#[from] meilisearch_sdk::errors::Error),
}
