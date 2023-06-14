use crate::GISSTError;
use tokio::fs::{
    create_dir_all,
    File,
    remove_file,
};

use std::path::{Component, Path, PathBuf};
use log::info;
use tokio::io::AsyncWriteExt;

use uuid::Uuid;

pub struct StorageHandler {
    root_storage_path: String,
    folder_depth: u8,
}

pub struct FileInformation {
    pub source_filename: String,
    pub dest_filename: String,
    pub dest_path: String,
    pub file_hash: String,
}

impl StorageHandler {
    pub fn init(root_storage_path:String, storage_folder_depth:u8) -> StorageHandler {

        if !Path::new(&root_storage_path).is_dir(){
            std::fs::create_dir_all(&root_storage_path).expect("Unable to create root storage directory");
        }

        StorageHandler{
            root_storage_path,
            folder_depth: storage_folder_depth,
        }
    }

    pub fn split_uuid_to_path_buf(uuid: Uuid, length: u8) -> PathBuf {
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
    pub fn get_dest_filename(hash: &str, filename:&str) -> String { format!("{}-{}", hash, filename).to_lowercase() }
    pub fn get_folder_depth_from_path(path: &Path, filename: Option<String>) -> u8 {
        let mut depth = 1;
        let mut path_buf = path.to_path_buf();
        if filename.is_some() && path_buf.ends_with(&filename.unwrap()){
            path_buf.pop();
        }
        for component in path_buf.components(){
            match component {
                Component::Normal(_) => depth += 1,
                _ => ()
            }
        }
        depth - 1
    }

    fn remove_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    pub async fn delete_file_with_uuid(&self, uuid: Uuid, dest_filename: &str) -> tokio::io::Result<()>{
        info!("Deleting file with filename: {}", dest_filename);
        let mut path = Path::new(&self.root_storage_path)
            .join(StorageHandler::split_uuid_to_path_buf(uuid, self.folder_depth).as_path());
        path.push(dest_filename);
        info!("Deleting file at path: {}", path.to_string_lossy());
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
        let save_filename = StorageHandler::get_dest_filename(&hash_string, &dest_filename);

        path.push(&save_filename);

        let mut file = File::create(path.to_path_buf()).await?;
        file.write_all(file_data).await?;

        path.pop();

        Ok(FileInformation{
            source_filename: filename.to_string(),
            dest_filename: save_filename,
            dest_path: path.strip_prefix(&self.root_storage_path)?.to_string_lossy().to_string(),
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
        assert_eq!(Path::new("/storage").join(path2), Path::new("/storage/0/0/0/0/"))
    }

    #[test]
    fn depth_from_dest_path(){
        let mut path = PathBuf::from("0/0/0/0/somefile.txt");
        path.pop();
        assert_eq!(Path::new("0/0/0/0"), path);

        let path = Path::new("0/0/0/0/");
        assert_eq!(4, StorageHandler::get_folder_depth_from_path(path, None));
        let path = Path::new("/0/0/0/0");
        assert_eq!(4, StorageHandler::get_folder_depth_from_path(path, None));
        let path = Path::new("0/0/0/0/somefile.txt");
        assert_eq!(5, StorageHandler::get_folder_depth_from_path(path, None));
        let path = Path::new("0/0/0/0/somefile.txt");
        assert_eq!(4, StorageHandler::get_folder_depth_from_path(path, Some("somefile.txt".to_string())));
        assert_eq!(0, StorageHandler::get_folder_depth_from_path(Path::new(""), None))
    }
}
