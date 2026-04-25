//! BMS folder operations.
//!
//! This module provides functions for renaming and managing BMS work directories.

use std::path::Path;
use tracing::info;

/// Append title and artist info to folder names based on BMS files
///
/// This replicates Python's `append_name_by_bms(root_dir)`:
/// - Iterates through subdirectories
/// - Renames folders that are purely numeric to "num. title [artist]" format
pub fn append_name_by_bms(root_dir: &Path) -> Result<(), std::io::Error> {
    if !root_dir.is_dir() {
        return Ok(());
    }

    let entries = std::fs::read_dir(root_dir)?;
    let mut to_rename: Vec<(PathBuf, PathBuf)> = Vec::new();

    for entry in entries.flatten() {
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        let dir_name = dir_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Skip if already renamed (has content after the number)
        if !dir_name.trim().is_empty() && dir_name.chars().all(|c| c.is_ascii_digit() || c.is_whitespace()) {
            // This is a numeric-only folder, try to rename
            if let Some(new_name) = rename_folder_by_bms(&dir_path)? {
                let new_path = dir_path.with_file_name(&new_name);
                to_rename.push((dir_path, new_path));
            }
        }
    }

    for (from, to) in to_rename {
        info!("Renaming {:?} -> {:?}", from.file_name(), to.file_name());
        std::fs::rename(&from, &to)?;
    }

    Ok(())
}

/// Rename a single folder based on its BMS info
/// Returns the new folder name if renamed, None if skipped
fn rename_folder_by_bms(work_dir: &Path) -> Result<Option<String>, std::io::Error> {
    use crate::bms::dir::get_dir_bms_info;
    use crate::fs::name::get_valid_fs_name;

    let dir_name = work_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Check if it's a purely numeric folder
    if !dir_name.trim().is_empty() && !dir_name.chars().all(|c| c.is_ascii_digit() || c.is_whitespace()) {
        return Ok(None);
    }

    let info = get_dir_bms_info(work_dir);
    if info.is_none() {
        return Ok(None);
    }

    let info = info.unwrap();
    if info.title.is_empty() && info.artist.is_empty() {
        return Ok(None);
    }

    let new_dir_name = format!(
        "{}. {} [{}]",
        dir_name.trim(),
        get_valid_fs_name(&info.title),
        get_valid_fs_name(&info.artist)
    );

    Ok(Some(new_dir_name))
}

/// Copy folder names from source to destination based on numeric prefix
///
/// This replicates Python's `copy_numbered_workdir_names(root_dir_from, root_dir_to)`:
/// - Source folders have format "num. title [artist]"
/// - Destination folders have format "num"
/// - Copies source names to destination based on numeric prefix match
pub fn copy_numbered_workdir_names(root_dir_from: &Path, root_dir_to: &Path) -> Result<(), std::io::Error> {
    if !root_dir_from.is_dir() || !root_dir_to.is_dir() {
        return Ok(());
    }

    // Get source directory names
    let src_names: Vec<(String, PathBuf)> = std::fs::read_dir(root_dir_from)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|n| (n.to_string(), e.path())))
        .collect();

    // Iterate destination directories
    for entry in std::fs::read_dir(root_dir_to)?.flatten() {
        let dst_path = entry.path();
        if !dst_path.is_dir() {
            continue;
        }

        let dst_name = dst_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Extract numeric prefix
        let num_prefix = dst_name.split_whitespace().next().unwrap_or("");
        let numeric_part = num_prefix.split('.').next().unwrap_or("");

        if !numeric_part.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        // Search for matching source folder
        for (src_name, _src_path) in &src_names {
            if src_name.starts_with(numeric_part) {
                let target_path = dst_path.with_file_name(src_name);
                if target_path != dst_path {
                    info!("Renaming {:?} -> {:?}", dst_path.file_name(), target_path.file_name());
                    std::fs::rename(&dst_path, &target_path)?;
                }
                break;
            }
        }
    }

    Ok(())
}

use std::path::PathBuf;
