use std::fs::{create_dir_all, DirBuilder, metadata};
use std::path::Path;
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
}

