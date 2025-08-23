use std::path::Path;

use futures::StreamExt;
use smol::{fs, io};
use strsim::jaro_winkler;

use super::work::BmsFolderSetNameType;

pub async fn set_name_by_bms(root_dir: &Path, set_type: BmsFolderSetNameType) -> io::Result<()> {
    let mut entries = fs::read_dir(root_dir).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            super::work::set_name_by_bms(&path, set_type).await?;
        }
    }

    Ok(())
}

/// This script is used for the following scenario:
/// There is already a folder A, whose subfolder names are in the form of "number + decimal point" like "1.1".
/// Now there is another folder B, whose subfolder names are only numbers.
/// Copy the subfolder names from A to the corresponding subfolders in B.
pub async fn copy_numbered_workdir_names(
    root_dir_from: impl AsRef<Path>,
    root_dir_to: impl AsRef<Path>,
) -> io::Result<()> {
    let root_from = root_dir_from.as_ref();
    let root_to = root_dir_to.as_ref();

    // Collect all directory names under root_from
    let mut src_names = Vec::new();
    let mut entries = fs::read_dir(root_from).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && let Some(name) = path.file_name()
        {
            src_names.push(name.to_string_lossy().into_owned());
        }
    }

    // Process directories under root_to
    let mut dst_entries = fs::read_dir(root_to).await?;
    while let Some(entry) = dst_entries.next().await {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();

        // Take the first segment of numbers (before space or dot)
        let dir_num = dir_name_str
            .split_whitespace()
            .next()
            .and_then(|s| s.split('.').next())
            .unwrap_or_default();

        if dir_num.chars().all(|c| c.is_ascii_digit()) {
            // Find directory starting with dir_num in src_names
            if let Some(src_name) = src_names.iter().find(|n| n.starts_with(dir_num)) {
                let target_path = root_to.join(src_name);
                println!("Rename {:?} -> {}", path.display(), src_name);
                fs::rename(&path, &target_path).await?;
            }
        }
    }

    Ok(())
}

/// Asynchronously scan subdirectories under `root_dir` and compare similarity between pairs in lexicographic order.
/// When similarity ≥ `similarity_trigger`, print this pair of directories.
///
/// # Example
/// ```
/// use be_music_cabinet::options::root::scan_folder_similar_folders;
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     smol::run(async {
///         scan_folder_similar_folders("./", 0.7).await?;
///         Ok(())
///     })
/// }
/// ```
pub async fn scan_folder_similar_folders(
    root_dir: impl AsRef<Path>,
    similarity_trigger: f64,
) -> io::Result<Vec<(String, String, f64)>> {
    // Read directory -> collect all subdirectory names (relative names)
    let mut entries = fs::read_dir(root_dir.as_ref()).await?;
    let mut dir_names = Vec::new();

    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let file_type = entry.file_type().await?;
        if file_type.is_dir() {
            dir_names.push(entry.file_name().into_string().unwrap());
        }
    }

    // Sort in lexicographic order
    dir_names.sort_unstable();

    // Scan adjacent items in order
    let print_tasks = dir_names
        .windows(2)
        .filter_map(|w| {
            let (former, current) = (&w[0], &w[1]);
            let similarity = jaro_winkler(former, current); // ← Change is here
            (similarity >= similarity_trigger).then_some((
                former.to_string(),
                current.to_string(),
                similarity,
            ))
        })
        .collect::<Vec<_>>();

    Ok(print_tasks)
}
