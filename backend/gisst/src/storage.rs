#![allow(clippy::missing_errors_doc)]

use tokio::{
    fs::{File, create_dir_all, remove_file},
    io::AsyncRead,
};

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

use std::path::{Component, Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::error::Storage;
use bytes::Bytes;
use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
pub struct StorageHandler;

#[derive(Debug)]
pub struct PendingUpload {
    pub file_information: FileInformation,
    pub length: usize,
    pub offset: usize,
}

#[derive(Clone, Debug)]
pub struct FileInformation {
    pub source_filename: String,
    pub source_path: String,
    pub dest_filename: String,
    pub dest_path: String,
    pub file_hash: String,
}

impl StorageHandler {
    pub fn init_storage(root_path: &str, temp_path: &str) -> Result<(), Storage> {
        if !Path::new(root_path).is_dir() {
            std::fs::create_dir_all(root_path)?;
        }

        if !Path::new(temp_path).is_dir() {
            std::fs::create_dir_all(temp_path)?;
        }

        Ok(())
    }

    #[must_use]
    pub fn split_uuid_to_path_buf(uuid: Uuid, length: u8) -> PathBuf {
        let mut uuid_string = uuid.to_string();
        uuid_string.truncate(length as usize);
        let mut path = PathBuf::new();
        for val in uuid_string.chars() {
            path.push(val.to_string());
        }
        path
    }

    // TODO fixme: this is synchronous but should probably be async.
    // But we don't have an async md5 hasher out there.
    pub fn get_file_hash(path: impl AsRef<Path>) -> Result<String, Storage> {
        use md5::Digest;

        let mut hasher = md5::Md5::new();
        let mut file = std::fs::File::open(path)?;
        std::io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(format!("{hash:x}"))
    }

    #[must_use]
    pub fn get_dest_filename(hash: &str, filename: &str) -> String {
        format!("{hash}-{filename}")
    }

    #[must_use]
    pub fn get_dest_file_path(root_path: &str, file_info: &FileInformation) -> PathBuf {
        let mut path = PathBuf::from(root_path);
        path.push(&file_info.dest_path);
        path
    }

    #[must_use]
    pub fn get_temp_file_path(temp_path: &str, file_info: &FileInformation) -> PathBuf {
        let mut path = PathBuf::from(temp_path);
        path.push(&file_info.dest_filename);
        path
    }

    #[must_use]
    pub fn get_folder_depth_from_path(path: &Path, filename: Option<String>) -> u8 {
        let mut depth = 1;
        let mut path_buf = path.to_path_buf();
        if filename.is_some_and(|filename| path_buf.ends_with(filename)) {
            path_buf.pop();
        }
        for component in path_buf.components() {
            if let Component::Normal(_) = component {
                depth += 1;
            }
        }
        depth - 1
    }

    pub async fn delete_file_with_uuid(
        root_path: &str,
        folder_depth: u8,
        uuid: Uuid,
        dest_filename: &str,
    ) -> tokio::io::Result<()> {
        let mut path =
            Path::new(root_path).join(Self::split_uuid_to_path_buf(uuid, folder_depth).as_path());
        path.push(dest_filename);
        info!("Deleting file at path: {}", path.to_string_lossy());
        // TODO also delete file.gz
        remove_file(path).await
    }

    pub async fn rename_file_from_temp_to_storage(
        root_path: &str,
        temp_path: &str,
        file_info: &FileInformation,
    ) -> Result<(), Storage> {
        let path = Self::get_dest_file_path(root_path, file_info);
        info!("In rename_file, dest_path is {}", path.to_string_lossy());
        let parent = path
            .parent()
            .ok_or_else(|| Storage::PathTooShallow(path.clone()))?;
        if !parent.is_dir() {
            create_dir_all(parent).await?;
        }

        tokio::fs::rename(Self::get_temp_file_path(temp_path, file_info), &path)
            .await
            .map_err(Storage::IO)?;

        let data = tokio::fs::File::open(&path).await?;
        Self::gzip_file(&path, data).await
    }

    pub async fn add_bytes_to_file(
        temp_path: &str,
        file_info: &FileInformation,
        mut bytes: Bytes,
    ) -> Result<(), Storage> {
        let temp_path = Self::get_temp_file_path(temp_path, file_info);

        // This should never trigger, but may as well check
        if !temp_path.as_path().exists() {
            return Err(Storage::FileNotFoundError);
        }

        tokio::task::spawn_blocking(move || {
            let file = OpenOptions::new()
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
    ) -> Result<(), Storage> {
        let temp_path = Self::get_temp_file_path(temp_path, file_info);
        tokio::task::spawn_blocking(move || {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .create_new(true)
                .open(temp_path)
                .map_err(Storage::IO)?;
            Ok(())
        })
        .await?
    }

    pub async fn write_file_to_uuid_folder(
        root_path: &str,
        folder_depth: u8,
        uuid: Uuid,
        filename: &str,
        file_path: impl AsRef<Path>,
    ) -> Result<FileInformation, Storage> {
        // TODO dedupe if the hash is present somewhere in here?
        let file_path = file_path.as_ref();
        let mut path =
            Path::new(root_path).join(Self::split_uuid_to_path_buf(uuid, folder_depth).as_path());

        if !path.is_dir() {
            create_dir_all(path.as_path()).await?;
        }

        let hash_string = Self::get_file_hash(file_path)?;
        let dest_filename = filename;
        let save_filename = Self::get_dest_filename(&hash_string, dest_filename);

        path.push(&save_filename);
        info!("copying from {file_path:?} to {path:?}");
        tokio::fs::copy(&file_path, &path).await?;

        Self::gzip_file(&path, File::open(file_path).await?).await?;

        path.pop();
        Ok(FileInformation {
            source_filename: filename.to_string(),
            source_path: filename.to_string(),
            dest_path: path
                .strip_prefix(root_path)?
                .join(&save_filename)
                .to_string_lossy()
                .to_string(),
            dest_filename: save_filename,
            file_hash: hash_string.to_string(),
        })
    }
    async fn gzip_file<R: AsyncRead + Unpin>(path: &Path, data: R) -> Result<(), Storage> {
        let ext: Option<&str> = path.extension().and_then(|ext| ext.to_str());
        // Skip compressing disk images
        if let Some("chd" | "img" | "iso" | "bin") = ext {
            return Ok(());
        }
        let gz_path = if let Some(e) = ext {
            let mut s = e.to_string();
            s.push_str(".gz");
            path.with_extension(s)
        } else {
            path.with_extension("gz")
        };
        let gz_file = File::create(gz_path).await?;
        let mut gz_enc = async_compression::tokio::write::GzipEncoder::with_quality(
            gz_file,
            async_compression::Level::Best,
        );
        let mut buf = tokio::io::BufReader::with_capacity(1024 * 1024, data);
        let _bytes_written = tokio::io::copy_buf(&mut buf, &mut gz_enc).await?;
        gz_enc.shutdown().await?;
        Ok(())
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
        );
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
        );
    }
}
