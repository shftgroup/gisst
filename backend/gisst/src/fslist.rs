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
                        part.starting_lba as u64 * mbr.sector_size as u64,
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
    use tracing::info;
    let partitions = get_partitions(&image_file)?;
    let mut result = Vec::with_capacity(partitions.len());
    for (idx, start_byte, sz) in partitions {
        info!("Slicing buffer for partition {idx}: {start_byte} <> {sz}");
        let root = recursive_listing_fat_partition(
            &mut image_file,
            start_byte,
            start_byte + sz,
            std::path::Path::new(&format!("part{idx}")),
        )?;
        result.push(root);
    }
    Ok(result)
}

fn recursive_listing_fat_partition(
    image: &mut std::fs::File,
    start_byte: u64,
    sz: u64,
    path: &std::path::Path,
) -> Result<FSFileListing, FSListError> {
    use fatfs::*;
    use fscommon::{BufStream, StreamSlice};
    use tracing::debug;
    debug!(
        "Loading image {:?} with bounds {}..{} as FAT filesystem",
        image, start_byte, sz
    );
    let image = BufStream::new(StreamSlice::new(image, start_byte, start_byte + sz)?);
    debug!("Stream slice initialized",);
    let fs = FileSystem::new(image, FsOptions::new())?;
    debug!("FS initialized",);
    let mut stack = vec![(fs.root_dir(), vec![], path.to_owned())];
    let mut root_lst = FSFileListing {
        name: path.to_string_lossy().to_string(),
        path: path.to_owned(),
        children: Vec::with_capacity(16),
        list_type: FSFileListingType::Partition,
    };
    while let Some((dir, idx, path)) = stack.pop() {
        if idx.len() > DEPTH_LIMIT {
            return Err(FSListError::TraversalDepth);
        }
        // dbg!(&path, &idx);
        let lst = get_lst_mut(&mut root_lst, &idx).ok_or(FSListError::TraversalPath(
            path.to_string_lossy().into_owned(),
        ))?;
        for (eidx, entry) in dir
            .iter()
            .filter(|entry| {
                let Ok(entry) = entry.as_ref() else {
                    return false;
                };
                let filename = entry.file_name();
                filename != "." && filename != ".."
            })
            .enumerate()
        {
            let entry = entry?;
            let filename = entry.file_name();
            // dbg!(&filename, &idx, eidx, &path);
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
        // dbg!(&lst.children, &lst.children.get(idx_path[0]));
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
    image_file: std::fs::File,
    path: &std::path::Path,
) -> Result<(String, Vec<u8>), FSListError> {
    let partitions = get_partitions(&image_file)?;
    let mut components = path.components();
    let std::path::Component::Normal(partid) = components.next().ok_or(
        FSListError::TraversalPath(path.to_string_lossy().into_owned()),
    )?
    else {
        return Err(FSListError::TraversalPath(
            path.to_string_lossy().into_owned(),
        ));
    };
    let partid = partid
        .to_string_lossy()
        .strip_prefix("part")
        .ok_or(FSListError::PathError)?
        .parse::<u32>()?;
    for (idx, start_byte, sz) in partitions {
        if idx != partid {
            continue;
        }
        use fatfs::*;
        use fscommon::{BufStream, StreamSlice};
        let image = BufStream::new(StreamSlice::new(image_file, start_byte, start_byte + sz)?);
        let fs = FileSystem::new(image, FsOptions::new())?;
        let subpath = components.as_path();
        if file_at_path_is_dir_fat(&fs, subpath) {
            let dir_zipped = get_dir_at_path_fat(&fs, subpath)?;
            return Ok(("application/zip".to_string(), dir_zipped));
        } else {
            let file = get_file_at_path_fat(&fs, subpath)?;
            let cookie = magic::Cookie::open(magic::cookie::Flags::MIME)
                .map_err(|_| FSListError::FiletypeDBError)?;
            let db = Default::default();
            let cookie = cookie.load(&db).map_err(|_| FSListError::FiletypeDBError)?;
            let mime = cookie.buffer(&file)?;
            return Ok((mime, file));
        }
    }
    Err(FSListError::FileNotFound(
        path.to_string_lossy().into_owned(),
    ))
}

type FATStorage = fatfs::StdIoWrapper<fscommon::BufStream<fscommon::StreamSlice<std::fs::File>>>;

fn file_at_path_is_dir_fat(fs: &fatfs::FileSystem<FATStorage>, path: &std::path::Path) -> bool {
    let root = fs.root_dir();
    path.parent().is_none() || root.open_dir(&path.to_string_lossy()).is_ok()
}

fn get_file_at_path_fat(
    fs: &fatfs::FileSystem<FATStorage>,
    path: &std::path::Path,
) -> Result<Vec<u8>, FSListError> {
    use fatfs::*;
    let mut file = fs.root_dir().open_file(&path.to_string_lossy()).unwrap();
    let file_size = file
        .extents()
        .try_fold(0, |sz, e| e.map(|ext| sz + ext.size))?;
    tracing::info!("download single file {path:?}, size {file_size:?}");
    let mut bytes = vec![0; file_size as usize];
    loop {
        match file.read(&mut bytes) {
            Ok(0) => {
                return Ok(bytes);
            }
            Ok(_n) => {
                continue;
            }
            Err(e) => return Err(FSListError::from(e)),
        }
    }
}

fn get_dir_at_path_fat(
    fs: &fatfs::FileSystem<FATStorage>,
    path: &std::path::Path,
) -> Result<Vec<u8>, FSListError> {
    use zip::{write::SimpleFileOptions, ZipWriter};
    let mut out_bytes: Vec<u8> = Vec::with_capacity(16 * 1024);
    let directory = if path.parent().is_none() {
        fs.root_dir()
    } else {
        fs.root_dir().open_dir(&path.to_string_lossy())?
    };
    let mut writer = ZipWriter::new(std::io::Cursor::new(&mut out_bytes));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut stack = Vec::with_capacity(1024);
    for entry in directory.iter() {
        let entry = entry?;
        let path = path.join(entry.file_name());
        stack.push((entry, path));
    }
    while let Some((entry, path)) = stack.pop() {
        let file_name = entry.file_name();
        if file_name == "." || file_name == ".." {
            continue;
        }
        let filepath = path.to_string_lossy().to_string();
        if entry.is_file() {
            writer.start_file(filepath, options)?;
            std::io::copy(&mut entry.to_file(), &mut writer)?;
        } else if entry.is_dir() {
            writer.add_directory(filepath, options)?;
            for subentry in entry.to_dir().iter() {
                let subentry = subentry?;
                let subpath = path.join(subentry.file_name());
                stack.push((subentry, subpath));
            }
        }
    }
    writer.finish()?;
    Ok(out_bytes)
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
