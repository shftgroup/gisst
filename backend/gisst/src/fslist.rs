use crate::error::FSListError;

const DEPTH_LIMIT: usize = 1024;

#[derive(serde::Serialize, Clone, Debug)]
pub struct FSFileListing {
    pub name: String,
    pub path: std::path::PathBuf,
    pub children: Vec<FSFileListing>,
    pub list_type: FSFileListingType,
}
#[derive(serde::Serialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum FSFileListingType {
    Partition,
    Directory,
    File,
}

fn get_partitions(image_file: &std::fs::File) -> Result<Vec<(u32, u64, u64)>, FSListError> {
    use tracing::info;
    let file_len = image_file.metadata()?.len();
    let mut image = fscommon::BufStream::new(image_file);
    let parts: Vec<_> = mbrman::MBR::read_from(&mut image, 512)
        .map(|mbr| {
            mbr.iter()
                .filter(|(_, part)| part.is_used())
                .map(|(idx, part)| {
                    (
                        idx as u32,
                        part.starting_lba as u64,
                        part.sectors as u64 * mbr.sector_size as u64,
                    )
                })
                .collect()
        })
        .or_else(|_err| {
            info!("disk image has bad or missing MBR, treating as raw filesystem");
            Ok::<Vec<(u32, u64, u64)>, mbrman::Error>(vec![(0, 0, file_len)])
        })?;
    Ok(parts)
}

pub fn recursive_listing(mut image_file: std::fs::File) -> Result<Vec<FSFileListing>, FSListError> {
    let partitions = get_partitions(&image_file)?;
    let mut result = Vec::with_capacity(partitions.len());
    for (idx, start_lba, sz) in partitions {
        let root = recursive_listing_fat_partition(
            &mut image_file,
            start_lba,
            start_lba + sz,
            std::path::Path::new(&format!("part{idx}")),
        )?;
        result.push(root);
    }
    Ok(result)
}

fn recursive_listing_fat_partition(
    image: &mut std::fs::File,
    start_lba: u64,
    sz: u64,
    path: &std::path::Path,
) -> Result<FSFileListing, FSListError> {
    use fatfs::*;
    use fscommon::{BufStream, StreamSlice};
    let image = BufStream::new(StreamSlice::new(image, start_lba, start_lba + sz)?);
    let fs = FileSystem::new(image, FsOptions::new())?;
    let mut stack = vec![(fs.root_dir(), vec![], path.to_owned())];
    let mut root_lst = FSFileListing {
        name: path.to_string_lossy().to_string(),
        path: path.to_owned(),
        children: Vec::with_capacity(16),
        list_type: FSFileListingType::Partition,
    };
    while let Some((dir, idx, path)) = stack.pop() {
        if idx.len() > DEPTH_LIMIT {
            return Err(FSListError::Traversal);
        }
        let lst = get_lst_mut(&mut root_lst, &idx).ok_or(FSListError::Traversal)?;
        for (eidx, entry) in dir.iter().enumerate() {
            let entry = entry?;
            let filename = entry.file_name();
            if filename == "." || filename == ".." {
                continue;
            }
            //dbg!(&filename, &idx, &path);
            // add a file entry or dir entry
            lst.children.push(FSFileListing {
                path: path.join(&filename),
                children: if entry.is_dir() {
                    Vec::with_capacity(16)
                } else {
                    vec![]
                },
                list_type: if entry.is_dir() {
                    FSFileListingType::Directory
                } else if entry.is_file() {
                    FSFileListingType::File
                } else {
                    return Err(FSListError::FATError(format!(
                        "fat file entry is neither dir nor file {:?}",
                        path.join(filename),
                    )));
                },
                name: filename,
            });
            if entry.is_dir() {
                let mut idx = idx.clone();
                idx.push(eidx);
                stack.push((entry.to_dir(), idx, path.join(entry.file_name())))
            }
        }
    }
    Ok(root_lst)
}
fn get_lst_mut<'a>(
    lst: &'a mut FSFileListing,
    idx_path: &[usize],
) -> Option<&'a mut FSFileListing> {
    if idx_path.is_empty() {
        Some(lst)
    } else {
        get_lst_mut(lst.children.get_mut(idx_path[0])?, &idx_path[1..])
    }
}

pub fn file_to_path(storage_root: &str, file: &crate::models::File) -> std::path::PathBuf {
    std::path::PathBuf::from(&format!(
        "{storage_root}/{}/{}-{}",
        file.file_dest_path, file.file_hash, file.file_filename
    ))
}

pub fn get_file_at_path(
    mut image_file: std::fs::File,
    path: &std::path::Path,
) -> Result<(String, Vec<u8>), FSListError> {
    let partitions = get_partitions(&image_file)?;
    let mut components = path.components();
    let std::path::Component::Normal(partid) = components.next().ok_or(FSListError::Traversal)?
    else {
        return Err(FSListError::Traversal);
    };
    let partid = partid
        .to_string_lossy()
        .strip_prefix("part")
        .ok_or(FSListError::PathError)?
        .parse::<u32>()?;
    for (idx, start_lba, sz) in partitions {
        if idx != partid {
            continue;
        }
        let file = get_file_at_path_fat(
            &mut image_file,
            start_lba,
            start_lba + sz,
            components.as_path(),
        )?;
        let cookie = magic::Cookie::open(magic::cookie::Flags::MIME)
            .map_err(|_| FSListError::FiletypeDBError)?;
        let db = Default::default();
        let cookie = cookie.load(&db).map_err(|_| FSListError::FiletypeDBError)?;
        let mime = cookie.buffer(&file)?;
        return Ok((mime, file));
    }
    Err(FSListError::Traversal)
}

fn get_file_at_path_fat(
    image: &mut std::fs::File,
    start_lba: u64,
    end_lba: u64,
    path: &std::path::Path,
) -> Result<Vec<u8>, FSListError> {
    use fatfs::*;
    use fscommon::{BufStream, StreamSlice};
    use std::io::Read;
    let image = BufStream::new(StreamSlice::new(image, start_lba, end_lba)?);
    let fs = FileSystem::new(image, FsOptions::new())?;
    let mut file = fs.root_dir().open_file(&path.to_string_lossy())?;
    let mut bytes = Vec::with_capacity(4096);
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn is_disk_image(file: &std::path::Path) -> bool {
    let Ok(cookie) = magic::Cookie::open(magic::cookie::Flags::ERROR) else {
        return false;
    };
    let db = Default::default();
    match cookie.load(&db) {
        Ok(cookie) => cookie
            .file(file)
            .map(|desc| desc.contains("DOS/MBR boot sector"))
            .unwrap_or(false),
        // TODO error reporting
        _ => false,
    }
}