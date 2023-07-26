use tokio::fs::{create_dir_all, remove_file, File};

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

use log::info;
use std::path::{Component, Path, PathBuf};
use tokio::io::AsyncWriteExt;

use bytes::Bytes;
use uuid::Uuid;

use thiserror::Error;
#[derive(Debug, Error)]
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

pub struct StorageHandler;

pub struct PendingUpload {
    pub file_information: FileInformation,
    pub length: usize,
    pub offset: usize,
}

#[derive(Clone)]
pub struct FileInformation {
    pub source_filename: String,
    pub source_path: String,
    pub dest_filename: String,
    pub dest_path: String,
    pub file_hash: String,
}

impl StorageHandler {
    pub fn init_storage(root_path: &str, temp_path: &str) -> Result<(), StorageError> {
        if !Path::new(root_path).is_dir() {
            std::fs::create_dir_all(root_path)?;
        }

        if !Path::new(temp_path).is_dir() {
            std::fs::create_dir_all(temp_path)?;
        }

        Ok(())
    }

    pub fn split_uuid_to_path_buf(uuid: Uuid, length: u8) -> PathBuf {
        let mut uuid_string = uuid.to_string();
        uuid_string.truncate(length as usize);
        let mut path = PathBuf::new();
        for val in uuid_string.chars() {
            path.push(val.to_string());
        }
        path
    }

    pub fn get_md5_hash(data: &[u8]) -> String {
        format!("{:x}", md5::compute(data))
    }
    pub fn get_dest_filename(hash: &str, filename: &str) -> String {
        format!("{}-{}", hash, filename).to_lowercase()
    }

    pub fn get_dest_file_path(root_path: &str, file_info: &FileInformation) -> PathBuf {
        let mut path = PathBuf::from(root_path);
        path.push(&file_info.dest_path);
        path.push(&file_info.dest_filename);
        path
    }

    pub fn get_temp_file_path(temp_path: &str, file_info: &FileInformation) -> PathBuf {
        let mut path = PathBuf::from(temp_path);
        path.push(&file_info.dest_filename);
        path
    }

    pub fn get_folder_depth_from_path(path: &Path, filename: Option<String>) -> u8 {
        let mut depth = 1;
        let mut path_buf = path.to_path_buf();
        if filename.is_some() && path_buf.ends_with(&filename.unwrap()) {
            path_buf.pop();
        }
        for component in path_buf.components() {
            if let Component::Normal(_) = component {
                depth += 1;
            }
        }
        depth - 1
    }

    fn remove_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    pub async fn delete_file_with_uuid(
        root_path: &str,
        folder_depth: u8,
        uuid: Uuid,
        dest_filename: &str,
    ) -> tokio::io::Result<()> {
        info!("Deleting file with filename: {}", dest_filename);
        let mut path =
            Path::new(root_path).join(Self::split_uuid_to_path_buf(uuid, folder_depth).as_path());
        path.push(dest_filename);
        info!("Deleting file at path: {}", path.to_string_lossy());
        remove_file(path).await
    }

    pub async fn rename_file_from_temp_to_storage(
        root_path: &str,
        temp_path: &str,
        file_info: &FileInformation,
    ) -> Result<(), StorageError> {
        let mut path = Self::get_dest_file_path(&root_path, &file_info);
        println!(
            "{}",
            format!(
                "In rename_file, dest_path is {}",
                path.to_string_lossy().to_string()
            )
        );

        path.pop();
        println!(
            "{}",
            format!(
                "In rename_file, dest_path is {}",
                path.to_string_lossy().to_string()
            )
        );

        if !path.is_dir() {
            create_dir_all(path.as_path()).await?
        }

        tokio::fs::rename(
            Self::get_temp_file_path(temp_path, file_info),
            Self::get_dest_file_path(root_path, file_info),
        )
        .await
        .map_err(StorageError::IO)
    }

    pub async fn add_bytes_to_file(
        temp_path: &str,
        file_info: &FileInformation,
        mut bytes: Bytes,
    ) -> Result<(), StorageError> {
        let temp_path = Self::get_temp_file_path(temp_path, file_info);

        // This should never trigger, but may as well check
        if !temp_path.as_path().exists() {
            return Err(StorageError::FileNotFoundError);
        }

        tokio::task::spawn_blocking(move || {
            let file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(false)
                .read(false)
                .truncate(false)
                .open(temp_path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(bytes.as_ref())?;
            writer.flush()?;
            bytes.clear();
            Ok(())
        })
        .await?
    }

    pub async fn create_temp_file(
        temp_path: &str,
        file_info: &FileInformation,
    ) -> Result<(), StorageError> {
        let temp_path = Self::get_temp_file_path(temp_path, file_info);
        tokio::task::spawn_blocking(move || {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .create_new(true)
                .open(temp_path)
                .map_err(StorageError::IO)?;
            Ok(())
        })
        .await?
    }

    pub async fn write_file_to_uuid_folder(
        root_path: &str,
        folder_depth: u8,
        uuid: Uuid,
        filename: &str,
        file_data: &[u8],
    ) -> Result<FileInformation, StorageError> {
        let mut path =
            Path::new(root_path).join(Self::split_uuid_to_path_buf(uuid, folder_depth).as_path());

        if !path.is_dir() {
            create_dir_all(path.as_path())
                .await
                .expect("Unable to create directory for uuid")
        }

        let hash_string = Self::get_md5_hash(file_data);
        let dest_filename = Self::remove_whitespace(filename).to_lowercase();
        let save_filename = Self::get_dest_filename(&hash_string, &dest_filename);

        path.push(&save_filename);

        let mut file = File::create(path.to_path_buf()).await?;
        file.write_all(file_data).await?;

        path.pop();

        Ok(FileInformation {
            source_filename: filename.to_string(),
            // TODO: take in a path and not just a filename, and do away with filename?
            source_path: filename.to_string(),
            dest_filename: save_filename,
            dest_path: path.strip_prefix(root_path)?.to_string_lossy().to_string(),
            file_hash: hash_string.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::uuid;

    #[test]
    fn uuid_path() {
        let path: PathBuf = StorageHandler::split_uuid_to_path_buf(
            uuid!("00000000-0000-0000-0000-000000000000"),
            4,
        );
        assert_eq!(Path::new("0/0/0/0/"), path.as_path());
        let path2: PathBuf = StorageHandler::split_uuid_to_path_buf(
            uuid!("00000000-0000-0000-0000-000000000000"),
            4,
        );
        assert_eq!(
            Path::new("/storage").join(path2),
            Path::new("/storage/0/0/0/0/")
        )
    }

    #[test]
    fn depth_from_dest_path() {
        let mut path = PathBuf::from("0/0/0/0/some_file.txt");
        path.pop();
        assert_eq!(Path::new("0/0/0/0"), path);

        let path = Path::new("0/0/0/0/");
        assert_eq!(4, StorageHandler::get_folder_depth_from_path(path, None));
        let path = Path::new("/0/0/0/0");
        assert_eq!(4, StorageHandler::get_folder_depth_from_path(path, None));
        let path = Path::new("0/0/0/0/some_file.txt");
        assert_eq!(5, StorageHandler::get_folder_depth_from_path(path, None));
        let path = Path::new("0/0/0/0/some_file.txt");
        assert_eq!(
            4,
            StorageHandler::get_folder_depth_from_path(path, Some("some_file.txt".to_string()))
        );
        assert_eq!(
            0,
            StorageHandler::get_folder_depth_from_path(Path::new(""), None)
        )
    }
}
