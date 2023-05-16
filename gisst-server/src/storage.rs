use std::fs::{create_dir_all, DirBuilder, File, metadata};
use std::path::{
    Path,
    PathBuf,
};

use uuid::Uuid;
use crate::server::GISSTError;

use anyhow::Result;

#[derive(Debug)]
pub struct StorageHandler {
    root_storage_path: String,
    folder_depth: i8,
}

impl StorageHandler {
    pub fn init(root_storage_path:String, storage_folder_depth:i8) -> StorageHandler {

        if !Path::new(&root_storage_path).is_dir(){
            create_dir_all(&root_storage_path).expect("Unable to create root storage directory");
        }

        StorageHandler{
            root_storage_path,
            folder_depth: storage_folder_depth,
        }
    }

    pub fn split_uuid_string_to_path_buf(uuid_string: &str, length: i8) -> &PathBuf {
        let mut p = PathBuf::with_capacity(length as usize);
        p = uuid_string.chars().collect().truncate(length);
        &p
    }

    pub fn write_bytes_to_uuid_folder(&self, uuid: Uuid, to_write_bytes: &[u8]) -> &Path {
        let mut path = Path::new(&self.root_storage_path).join(
            StorageHandler::split_uuid_string_to_path_buf(&uuid.to_string(), self.folder_depth)
        );

        if !path.as_path().is_dir(){
            create_dir_all(path.as_path())
        }

        path.as_path()
    }

}

