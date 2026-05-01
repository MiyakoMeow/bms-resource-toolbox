//! BMS folder operations.
//!
//! This module provides functions for renaming and managing BMS work directories.

use std::path::Path;
use tracing::info;

use crate::bms::types::MEDIA_FILE_EXTS;

/// Append title and artist info to folder names based on BMS files.
///
/// This replicates Python's `append_name_by_bms(root_dir)`:
/// - Iterates through subdirectories
/// - Renames folders that are purely numeric to "num. title \[artist\]" format
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn append_name_by_bms(root_dir: &Path) -> Result<(), std::io::Error> {
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

        let dir_name = dir_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip if already renamed (has content after the number)
        if !dir_name.trim().is_empty()
            && dir_name
                .chars()
                .all(|c| c.is_ascii_digit() || ('\u{FF10}'..='\u{FF19}').contains(&c))
        {
            // This is a numeric-only folder, try to rename
            if let Some(new_name) = rename_folder_by_bms(&dir_path).await {
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
///
/// Returns the new folder name if renamed, `None` if skipped.
#[must_use]
async fn rename_folder_by_bms(work_dir: &Path) -> Option<String> {
    use crate::bms::dir::get_dir_bms_info;
    use crate::fs::name::get_valid_fs_name;

    let dir_name = work_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Check if it's a purely numeric folder
    if !dir_name.trim().is_empty()
        && !dir_name
            .chars()
            .all(|c| c.is_ascii_digit() || ('\u{FF10}'..='\u{FF19}').contains(&c))
    {
        return None;
    }

    let info = get_dir_bms_info(work_dir).await?;
    if info.title.is_empty() && info.artist.is_empty() {
        return None;
    }

    let new_dir_name = format!(
        "{}. {} [{}]",
        dir_name.trim(),
        get_valid_fs_name(&info.title),
        get_valid_fs_name(&info.artist)
    );

    Some(new_dir_name)
}

/// Copy folder names from source to destination based on numeric prefix
///
/// This replicates Python's `copy_numbered_workdir_names(root_dir_from, root_dir_to)`:
/// - Source folders have format "num. title \[artist\]"
/// - Destination folders have format "num"
/// - Copies source names to destination based on numeric prefix match
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn copy_numbered_workdir_names(
    root_dir_from: &Path,
    root_dir_to: &Path,
) -> Result<(), std::io::Error> {
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

        let dst_name = dst_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

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
                    info!(
                        "Renaming {:?} -> {:?}",
                        dst_path.file_name(),
                        target_path.file_name()
                    );
                    std::fs::rename(&dst_path, &target_path)?;
                }
                break;
            }
        }
    }

    Ok(())
}

/// Append artist name to folder names based on BMS files
///
/// This replicates Python's `append_artist_name_by_bms(root_dir)`:
/// - Adds " \[artist\]" suffix to folders not already ending with "\]"
/// - Shows confirmation before renaming
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
///
/// # Panics
///
/// Panics if stdout flush or stdin read fails.
pub async fn append_artist_name_by_bms(root_dir: &Path) -> Result<(), std::io::Error> {
    use std::io::{self, Write};

    use crate::bms::dir::get_dir_bms_info;
    use crate::fs::name::get_valid_fs_name;

    if !root_dir.is_dir() {
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();

    for entry in entries {
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        let dir_name = dir_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip if already has artist suffix
        if dir_name.ends_with(']') {
            continue;
        }

        let info = get_dir_bms_info(&dir_path).await;
        if info.is_none() {
            println!("Dir {} has no bms files!", dir_path.display());
            continue;
        }

        let info = info.unwrap();
        let new_dir_name = format!("{dir_name} [{}]", get_valid_fs_name(&info.artist));
        println!("- Ready to rename: {dir_name} -> {new_dir_name}");
        pairs.push((dir_path, root_dir.join(&new_dir_name)));
    }

    if pairs.is_empty() {
        println!("No folders to rename");
        return Ok(());
    }

    print!("Do transferring? [y/N]: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().to_lowercase().starts_with('y') {
        println!("Aborted.");
        return Ok(());
    }

    for (from, to) in pairs {
        std::fs::rename(&from, &to)?;
    }

    Ok(())
}

/// Set folder names based on BMS info (title \[artist\] format)
///
/// This replicates Python's `set_name_by_bms(root_dir)`:
/// - Renames folders to "title \[artist\]" format
/// - Handles merging if target already exists (with similarity check)
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn set_name_by_bms(root_dir: &Path) -> Result<(), std::io::Error> {
    if !root_dir.is_dir() {
        return Ok(());
    }

    let mut fail_list: Vec<String> = Vec::new();

    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        let dir_name = dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if !set_single_folder_name_by_bms(&dir_path).await? {
            fail_list.push(dir_name);
        }
    }

    if !fail_list.is_empty() {
        println!("Fail Count: {}", fail_list.len());
        for name in &fail_list {
            println!("  {name}");
        }
    }

    Ok(())
}

/// Set a single folder's name based on its BMS info
/// Returns true if successful, false if skipped or failed
async fn set_single_folder_name_by_bms(work_dir: &Path) -> Result<bool, std::io::Error> {
    use crate::bms::dir::get_dir_bms_info;
    use crate::fs::name::{bms_dir_similarity, get_valid_fs_name};
    use crate::fs::pack_move::{
        MoveOptions, REPLACE_OPTION_UPDATE_PACK, ReplaceOptions, move_elements_across_dir,
    };

    let mut info = get_dir_bms_info(work_dir).await;

    // If no BMS info found, try to move out nested contents and retry
    while info.is_none() {
        println!(
            "{} has no bms/bmson files! Trying to move out.",
            work_dir.display()
        );

        let elements: Vec<_> = std::fs::read_dir(work_dir)?
            .filter_map(std::result::Result::ok)
            .collect();

        if elements.is_empty() {
            println!(" - Empty dir! Deleting...");
            match std::fs::remove_dir(work_dir) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    println!(" x PermissionError: {e}");
                }
                Err(e) => return Err(e),
            }
            return Ok(false);
        }

        if elements.len() != 1 {
            println!(" - Element count: {}", elements.len());
            return Ok(false);
        }

        let inner_path = &elements[0].path();
        if !inner_path.is_dir() {
            println!(" - Folder has only a file: {:?}", inner_path.file_name());
            return Ok(false);
        }

        println!(" - Moving out files...");
        move_elements_across_dir(
            inner_path,
            work_dir,
            MoveOptions::default(),
            &ReplaceOptions::default(),
        )?;
        info = get_dir_bms_info(work_dir).await;
    }

    let info = info.unwrap();
    let parent_dir = work_dir.parent().unwrap_or(work_dir);

    if info.title.is_empty() && info.artist.is_empty() {
        println!("{}: Info title and artist is EMPTY!", work_dir.display());
        return Ok(false);
    }

    let new_dir_path = parent_dir.join(format!(
        "{} [{}]",
        get_valid_fs_name(&info.title),
        get_valid_fs_name(&info.artist)
    ));

    // Same directory?
    if work_dir == new_dir_path {
        return Ok(true);
    }

    println!(
        "{}: Rename! Title: {}; Artist: {}",
        work_dir.display(),
        info.title,
        info.artist
    );

    if !new_dir_path.is_dir() {
        std::fs::rename(work_dir, &new_dir_path)?;
        return Ok(true);
    }

    // Directory already exists - check similarity
    let similarity = bms_dir_similarity(work_dir, &new_dir_path);
    println!(
        " - Directory {} exists! Similarity: {similarity}",
        new_dir_path.display()
    );

    if similarity < 0.8 {
        println!(" - Merge canceled.");
        return Ok(false);
    }

    println!(" - Merge start!");
    move_elements_across_dir(
        work_dir,
        &new_dir_path,
        MoveOptions::default(),
        &REPLACE_OPTION_UPDATE_PACK,
    )?;
    Ok(true)
}

/// Scan for similar folder names
///
/// This replicates Python's `scan_folder_similar_folders(root_dir, similarity_trigger)`:
/// - Uses difflib.SequenceMatcher to find similar folder names
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn scan_folder_similar_folders(
    root_dir: &Path,
    similarity_trigger: f64,
) -> Result<(), std::io::Error> {
    if !root_dir.is_dir() {
        return Ok(());
    }

    let dir_names: Vec<String> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    println!("当前目录下有{}个文件夹。", dir_names.len());

    let mut sorted_names = dir_names.clone();
    sorted_names.sort();

    for i in 1..sorted_names.len() {
        let former = &sorted_names[i - 1];
        let current = &sorted_names[i];

        let similarity = similar_ratio(former, current);
        if similarity < similarity_trigger {
            continue;
        }
        println!("发现相似项：{former} <=> {current}");
    }

    Ok(())
}

/// Calculate similarity ratio between two strings.
/// Returns a value between 0.0 and 1.0.
fn similar_ratio(a: &str, b: &str) -> f64 {
    sequence_matcher_ratio(a, b)
}

fn sequence_matcher_ratio(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    if a_chars.is_empty() && b_chars.is_empty() {
        return 1.0;
    }
    if a_chars.is_empty() || b_chars.is_empty() {
        return 0.0;
    }
    let total = a_chars.len() + b_chars.len();
    let matches = find_longest_match(&a_chars, &b_chars);
    #[expect(clippy::cast_precision_loss)]
    {
        (2 * matches) as f64 / total as f64
    }
}

#[allow(clippy::similar_names)]
fn find_longest_match(a: &[char], b: &[char]) -> usize {
    let mut best_len = 0;
    let mut best_ai = 0;
    let mut best_bi = 0;

    for ai in 0..a.len() {
        for bi in 0..b.len() {
            let mut k = 0;
            while ai + k < a.len() && bi + k < b.len() && a[ai + k] == b[bi + k] {
                k += 1;
            }
            if k > best_len {
                best_len = k;
                best_ai = ai;
                best_bi = bi;
            }
        }
    }

    if best_len == 0 {
        return 0;
    }

    let left_matches = find_longest_match(&a[..best_ai], &b[..best_bi]);
    let right_matches = find_longest_match(&a[best_ai + best_len..], &b[best_bi + best_len..]);

    best_len + left_matches + right_matches
}

/// Undo `set_name` by removing " \[artist\]" suffix
///
/// This replicates Python's `undo_set_name(root_dir)`:
/// - Removes " \[artist\]" part from folder names
/// - Restores the original numeric prefix
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn undo_set_name(root_dir: &Path) -> Result<(), std::io::Error> {
    if !root_dir.is_dir() {
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        let dir_name = dir_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let parts: Vec<&str> = dir_name.splitn(2, ' ').collect();
        let new_dir_name = parts[0];

        if dir_name == new_dir_name {
            continue;
        }

        let new_dir_path = root_dir.join(new_dir_name);

        if new_dir_path.is_dir() {
            println!(
                "Warning: Target {} already exists! Skipping {dir_name}",
                new_dir_path.display()
            );
            continue;
        }

        println!("Rename {dir_name} to {new_dir_name}");
        std::fs::rename(&dir_path, &new_dir_path)?;
    }

    Ok(())
}

/// Remove zero-sized media files and temp files
///
/// This replicates Python's `remove_zero_sized_media_files(current_dir, print_dir)`:
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn remove_zero_sized_media_files(
    current_dir: &Path,
    print_dir: bool,
) -> Result<(), std::io::Error> {
    if print_dir {
        println!("Entering dir: {}", current_dir.display());
    }

    if !current_dir.is_dir() {
        println!("Not a vaild dir! Aborting...");
        return Ok(());
    }

    let mut next_dirs: Vec<PathBuf> = Vec::new();

    let entries: Vec<_> = std::fs::read_dir(current_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in &entries {
        let element_path = entry.path();
        let element_name = element_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if element_path.is_file() {
            let is_temp_file = element_name.to_lowercase() == "desktop.ini"
                || element_name.to_lowercase() == "thumbs.db"
                || element_name.to_lowercase() == ".ds_store"
                || element_name.starts_with(".trash-")
                || element_name.starts_with("._");

            if is_temp_file {
                match std::fs::remove_file(&element_path) {
                    Ok(()) => println!(" - Remove temp file: {}", element_path.display()),
                    Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                        println!(" x PermissionError!");
                    }
                    Err(_) => {}
                }
                continue;
            }

            if !MEDIA_FILE_EXTS
                .iter()
                .any(|ext| element_name.ends_with(*ext))
            {
                continue;
            }

            match element_path.metadata() {
                Ok(metadata) if metadata.len() == 0 => match std::fs::remove_file(&element_path) {
                    Ok(()) => {
                        println!(" - Remove empty file: {}", element_path.display());
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                        println!(" x PermissionError!");
                    }
                    Err(_) => {}
                },
                _ => {}
            }
        } else if element_path.is_dir() {
            next_dirs.push(element_path);
        }
    }

    for next_dir in next_dirs {
        remove_zero_sized_media_files(&next_dir, print_dir)?;
    }

    Ok(())
}

use std::path::PathBuf;
