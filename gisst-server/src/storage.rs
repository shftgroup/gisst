use tokio::fs::{
    create_dir_all,
    File,
    remove_file,
};

use std::path::{
    Path,
    PathBuf,
};
use tokio::io::AsyncWriteExt;

use uuid::Uuid;
use crate::server::GISSTError;

pub struct ChunkStatus {
    total_chunks: u64,
    received_chunks: Vec<(Uuid, String)>,
    parent_id: Uuid,
    file_info: FileInformation,
}

pub struct StorageHandler {
    root_storage_path: String,
    folder_depth: i8,
    pending_uploads: Vec<ChunkStatus>,
}


pub struct FileInformation {
    pub source_filename: String,
    pub dest_filename: String,
    pub dest_path: String,
    pub file_hash: String,
}

impl StorageHandler {
    pub fn init(root_storage_path:String, storage_folder_depth:i8) -> StorageHandler {

        if !Path::new(&root_storage_path).is_dir(){
            std::fs::create_dir_all(&root_storage_path).expect("Unable to create root storage directory");
        }

        StorageHandler{
            root_storage_path,
            folder_depth: storage_folder_depth,
            pending_uploads: vec![],
        }
    }

    pub fn split_uuid_to_path_buf(uuid: Uuid, length: i8) -> PathBuf {
        let mut uuid_string = uuid.to_string();
        uuid_string.truncate(length as usize);
        let mut path = PathBuf::new();
        for val in uuid_string.chars(){
            path.push(val.to_string());
        }
        path
    }

    pub fn get_md5_hash(data: &[u8]) -> String {
        format!("{:x}", md5::compute(data))
    }

    fn remove_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    pub async fn delete_file_with_uuid(&self, uuid: Uuid, dest_filename: &str) -> tokio::io::Result<()>{
        let mut path = Path::new(&self.root_storage_path)
            .join(StorageHandler::split_uuid_to_path_buf(uuid, self.folder_depth).as_path());
        path.push(dest_filename);
        remove_file(path).await
    }

    pub async fn write_file_to_uuid_folder(&self, uuid: Uuid, filename: &str, file_data: &[u8]) -> Result<FileInformation, GISSTError>{
        let mut path = Path::new(&self.root_storage_path)
            .join(StorageHandler::split_uuid_to_path_buf(uuid, self.folder_depth).as_path());

        if !path.is_dir() {
            create_dir_all(path.as_path()).await.expect("Unable to create directory for uuid")
        }

        let hash_string = StorageHandler::get_md5_hash(file_data);
        let dest_filename = StorageHandler::remove_whitespace(filename).to_lowercase();
        let save_filename = format!("{}-{}", hash_string, dest_filename);

        path.push(&save_filename);

        let mut file = File::create(path.to_path_buf()).await?;
        file.write_all(file_data).await?;

        Ok(FileInformation{
            source_filename: filename.to_string(),
            dest_filename: save_filename,
            dest_path: path.to_string_lossy().into_owned(),
            file_hash: hash_string.to_string(),
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::uuid;

    #[test]
    fn uuid_path(){
        let path: PathBuf = StorageHandler::split_uuid_to_path_buf(uuid!("00000000-0000-0000-0000-000000000000"), 4);
        assert_eq!(Path::new("0/0/0/0/"), path.as_path());
        let path2: PathBuf = StorageHandler::split_uuid_to_path_buf(uuid!("00000000-0000-0000-0000-000000000000"), 4);
        assert_eq!(path2.join("/storage"), Path::new("/storage/0/0/0/0/"))

    }
}
