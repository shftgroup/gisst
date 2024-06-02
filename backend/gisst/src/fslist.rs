use crate::error::FSListError;

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

// TODO don't use fatfs dir, use a bufread and get the partition table etc
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
                    dbg!((
                        idx as u32,
                        part.starting_lba as u64,
                        part.sectors as u64 * mbr.sector_size as u64
                    ))
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
    let root = fs.root_dir();
    dbg!(root
        .iter()
        .map(|e| Ok(e?.file_name()))
        .collect::<Result<Vec<_>, FSListError>>()?);
    // let mut idx_path = [];
    // let mut path: PathBuf = path.to_owned();
    // let mut root_elt = FSFileListing {
    //     name: path.to_string_lossy().to_string(),
    //     path: path.to_owned(),
    //     children: Vec::with_capacity(16),
    //     list_type: FSFileListingType::Partition,
    // };
    // let mut cur_elt = &mut root_elt;
    // for entry in root.iter() {
    //     let entry = entry?;
    //     let name = entry.file_name();

    //     stack.push((
    //         path.join(&name),
    //         if entry.is_dir() {
    //             Vec::with_capacity(16)
    //         } else {
    //             vec![]
    //         },
    //         if entry.is_dir() {
    //             FSFileListingType::Directory
    //         } else if entry.is_file() {
    //             FSFileListingType::File
    //         } else {
    //             return Err(FSListError::FATError(path.join(entry.file_name())));
    //         },
    //         entry,
    //         0,
    //     ));
    // }
    // //
    // while let Some((path, children, ftype, entry, parent)) = stack.pop() {}
    Ok(FSFileListing {
        name: path.to_string_lossy().to_string(),
        path: path.to_owned(),
        children: vec![],
        list_type: FSFileListingType::Partition,
    })
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
