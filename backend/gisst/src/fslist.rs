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

pub fn recursive_listing(image_file: std::fs::File) -> Result<Vec<FSFileListing>, FSListError> {
    use tracing::info;
    let file_len = image_file.metadata()?.len();
    let mut image = fscommon::BufStream::new(image_file);
    let mut result = Vec::with_capacity(3);
    let partitions: Vec<(u32, u64, u64)> = mbrman::MBR::read_from(&mut image, 512)
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
    for (idx, start_lba, sz) in partitions {
        let root = recursive_listing_fat_partition(
            &mut image,
            start_lba,
            start_lba + sz,
            std::path::Path::new(&format!("part{idx}")),
        )?;
        result.push(root);
    }
    Ok(result)
}

fn recursive_listing_fat_partition<R: std::io::Read + std::io::Write + std::io::Seek>(
    image: R,
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

// fn recursive_listing_fat_directory<'a, T: fatfs::ReadWriteSeek + 'a>(
//     path: &std::path::Path,
//     entry: fatfs::DirEntry<'a, T>,
// ) -> Result<FSFileListing, FSListError> {
//     let path = path.join(entry.file_name());
//     let children = if entry.is_dir() {
//         let mut children = Vec::with_capacity(4);
//         let entry = entry.to_dir();
//         for child in entry.iter() {
//             children.push(recursive_listing_fat_directory(&path, child?)?)
//         }
//         children
//     } else {
//         vec![]
//     };
//     Ok(FSFileListing {
//         list_type: if entry.is_dir() {
//             FSFileListingType::Directory
//         } else if entry.is_file() {
//             FSFileListingType::File
//         } else {
//             return Err(FSListError::FATError(format!(
//                 "entry is neither file nor dir {:?}",
//                 path.join(entry.file_name())
//             )));
//         },
//         path,
//         name: entry.file_name(),
//         children,
//     })
// }

pub fn file_to_path(storage_root: &str, file: &crate::models::File) -> std::path::PathBuf {
    std::path::PathBuf::from(&format!(
        "{storage_root}/{}/{}-{}",
        file.file_dest_path, file.file_hash, file.file_filename
    ))
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
