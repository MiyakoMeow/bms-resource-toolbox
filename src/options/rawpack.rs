//! Raw pack utilities.
//!
//! This module provides utilities for extracting raw BMS packs
//! and setting file numbers.

use std::path::{Path, PathBuf};
use tracing::info;

use crate::fs::pack_move::is_dir_having_file;
use crate::fs::rawpack::{
    extract_archive, get_num_set_file_names, move_out_files_in_folder_in_cache_dir,
};

/// Extract archives by original filename to BMS folder structure
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn unzip_with_name_to_bms_folder(
    pack_dir: &Path,
    cache_dir: &Path,
    root_dir: &Path,
) -> Result<(), std::io::Error> {
    info!(
        "Unzip with name to BMS folder: {:?} -> {:?}",
        pack_dir, root_dir
    );

    // Create directories
    if !cache_dir.is_dir() {
        std::fs::create_dir_all(cache_dir)?;
    }
    if !root_dir.is_dir() {
        std::fs::create_dir_all(root_dir)?;
    }

    // Get archive files
    let mut archive_names: Vec<String> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(pack_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            let name_lower = name.to_lowercase();
            #[expect(clippy::case_sensitive_file_extension_comparisons)]
            if name_lower.ends_with(".zip")
                || name_lower.ends_with(".7z")
                || name_lower.ends_with(".rar")
            {
                archive_names.push(name.to_string());
            }
        }
    }

    if archive_names.is_empty() {
        info!("No archive files found in {:?}", pack_dir);
        return Ok(());
    }

    // Sort for consistent order
    archive_names.sort();

    for file_name in &archive_names {
        let file_path = pack_dir.join(file_name);

        // Get filename without extension
        let file_stem = file_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .trim_end_matches('.')
            .to_string();

        // Prepare cache directory
        let cache_dir_path = cache_dir.join(&file_stem);
        if cache_dir_path.is_dir() {
            // Check if has files
            let has_files = std::fs::read_dir(&cache_dir_path)
                .map(|mut e| e.any(|r| r.map(|e| e.path().is_file()).unwrap_or(false)))
                .unwrap_or(false);
            if has_files {
                info!("Removing existing cache dir: {:?}", cache_dir_path);
                std::fs::remove_dir_all(&cache_dir_path)?;
            }
        }
        std::fs::create_dir_all(&cache_dir_path)?;

        // Extract archive
        info!("Extracting {:?} to {:?}", file_path, cache_dir_path);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(extract_archive(&file_path, &cache_dir_path))?;

        // Move files out of nested folders
        if !move_out_files_in_folder_in_cache_dir(&cache_dir_path) {
            info!("Failed to process cache dir: {:?}", cache_dir_path);
            continue;
        }

        // Target directory
        let target_dir_path = root_dir.join(&file_stem);

        // Move cache to BMS dir
        info!(
            "Moving files from {:?} to {:?}",
            cache_dir_path, target_dir_path
        );

        // Get entries in cache dir
        let entries: Vec<_> = std::fs::read_dir(&cache_dir_path)?
            .filter_map(std::result::Result::ok)
            .collect();

        // Create target directory
        std::fs::create_dir_all(&target_dir_path)?;

        // Move each entry
        for entry in entries {
            let src_path = entry.path();
            let dst_path = target_dir_path.join(src_path.file_name().unwrap_or_default());
            std::fs::rename(&src_path, &dst_path).or_else(|_| {
                if src_path.is_dir() {
                    copy_dir_recursive(&src_path, &dst_path)?;
                    std::fs::remove_dir_all(&src_path)
                } else {
                    std::fs::copy(&src_path, &dst_path)?;
                    std::fs::remove_file(&src_path)
                }
            })?;
        }

        // Try to remove empty cache dir
        let _ = std::fs::remove_dir(&cache_dir_path);

        // Move original file to BOFTTPacks subdirectory
        info!("Finished processing: {}", file_name);
        let used_pack_dir = pack_dir.join("BOFTTPacks");
        if !used_pack_dir.is_dir() {
            std::fs::create_dir_all(&used_pack_dir)?;
        }
        let target_file_path = used_pack_dir.join(file_name);
        std::fs::rename(&file_path, &target_file_path).ok();
    }

    Ok(())
}

/// Extract numeric-prefixed archives to BMS folder structure
///
/// This replicates Python's `unzip_numeric_to_bms_folder`:
/// - Gets numbered file list (e.g., "001 filename.zip")
/// - Extracts to `cache_dir/{id}` for each file
/// - Finds or creates target directory in `root_dir` with exact numeric match
/// - Moves extracted files to target directory
/// - Moves original archive to `BOFTTPacks/`
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn unzip_numeric_to_bms_folder(
    pack_dir: &Path,
    cache_dir: &Path,
    root_dir: &Path,
) -> Result<(), std::io::Error> {
    info!(
        "Unzip numeric to BMS folder: {:?} -> {:?}",
        pack_dir, root_dir
    );

    // Create directories
    if !cache_dir.is_dir() {
        std::fs::create_dir_all(cache_dir)?;
    }
    if !root_dir.is_dir() {
        std::fs::create_dir_all(root_dir)?;
    }

    // Get numbered file names
    let num_set_file_names = get_num_set_file_names(pack_dir);
    info!("Found {} numbered pack files", num_set_file_names.len());

    for file_name in &num_set_file_names {
        let file_path = pack_dir.join(file_name);
        if !file_path.is_file() {
            continue;
        }

        // Parse id from filename (e.g., "001 filename.zip" -> "001")
        let id_str = file_name.split_whitespace().next().unwrap_or("");
        if id_str.is_empty() {
            continue;
        }

        // Prepare cache directory
        let cache_dir_path = cache_dir.join(id_str);

        if cache_dir_path.is_dir() && is_dir_having_file(&cache_dir_path) {
            std::fs::remove_dir_all(&cache_dir_path)?;
        }
        if !cache_dir_path.is_dir() {
            std::fs::create_dir_all(&cache_dir_path)?;
        }

        // Extract archive
        info!("Extracting {:?} to {:?}", file_path, cache_dir_path);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(extract_archive(&file_path, &cache_dir_path))?;

        // Move files out of nested folders
        if !move_out_files_in_folder_in_cache_dir(&cache_dir_path) {
            info!("Failed to process cache dir: {:?}", cache_dir_path);
            continue;
        }

        // Find existing target directory in root_dir with exact numeric match
        // Avoid matching "1" to "10", "11", etc.
        let mut target_dir_path: Option<PathBuf> = None;

        if let Ok(entries) = std::fs::read_dir(root_dir) {
            for entry in entries.flatten() {
                let dir_path = entry.path();
                if !dir_path.is_dir() {
                    continue;
                }
                let dir_name = dir_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Check if dir_name starts with id_str
                if !dir_name.starts_with(id_str) {
                    continue;
                }
                // Check remaining: must be empty or start with "." or " "
                let remaining = &dir_name[id_str.len()..];
                if !remaining.is_empty()
                    && !remaining.starts_with('.')
                    && !remaining.starts_with(' ')
                {
                    continue;
                }
                // Found match
                target_dir_path = Some(dir_path);
                break;
            }
        }

        // Create new target dir if not found
        let target_dir_path = target_dir_path.unwrap_or_else(|| root_dir.join(id_str));

        // Move cache to BMS dir
        info!(
            "Moving files from {:?} to {:?}",
            cache_dir_path, target_dir_path
        );

        // Get entries in cache dir
        let entries: Vec<_> = std::fs::read_dir(&cache_dir_path)?
            .filter_map(std::result::Result::ok)
            .collect();

        // Create target directory
        std::fs::create_dir_all(&target_dir_path)?;

        // Move each entry
        for entry in entries {
            let src_path = entry.path();
            let dst_path = target_dir_path.join(src_path.file_name().unwrap_or_default());
            std::fs::rename(&src_path, &dst_path).or_else(|_| {
                if src_path.is_dir() {
                    copy_dir_recursive(&src_path, &dst_path)?;
                    std::fs::remove_dir_all(&src_path)
                } else {
                    std::fs::copy(&src_path, &dst_path)?;
                    std::fs::remove_file(&src_path)
                }
            })?;
        }

        // Try to remove empty cache dir
        let _ = std::fs::remove_dir(&cache_dir_path);

        // Move original file to BOFTTPacks subdirectory
        info!("Finished processing: {}", file_name);
        let used_pack_dir = pack_dir.join("BOFTTPacks");
        if !used_pack_dir.is_dir() {
            std::fs::create_dir_all(&used_pack_dir)?;
        }
        let target_file_path = used_pack_dir.join(file_name);
        std::fs::rename(&file_path, &target_file_path).ok();
    }

    Ok(())
}

/// Copy directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;

    let entries = std::fs::read_dir(src)?;
    for entry in entries {
        let src_path = entry?.path();
        let dst_path = dst.join(src_path.file_name().unwrap_or_default());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Set file numbers for files in a directory
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
///
/// # Panics
///
/// Panics if stdout flush or stdin read fails.
pub fn set_file_num(dir: &Path) -> Result<(), std::io::Error> {
    use std::io::{self, Write};
    const ALLOWED_EXTS: &[&str] = &["zip", "7z", "rar", "mp4", "bms", "bme", "bml", "pms"];

    info!("Setting file numbers in: {:?}", dir);

    // Get files to number
    let mut file_names: Vec<String> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip if already numbered
            if name
                .split_whitespace()
                .next()
                .is_some_and(|s| s.chars().all(|c| c.is_ascii_digit()))
            {
                continue;
            }

            // Skip part files
            if name.to_lowercase().ends_with(".part") {
                continue;
            }

            // Skip empty files
            if path.metadata().map(|m| m.len() == 0).unwrap_or(true) {
                continue;
            }

            // Check extension
            let ext = name.rsplit('.').next().unwrap_or("");
            if !ALLOWED_EXTS.contains(&ext.to_lowercase().as_str()) {
                continue;
            }

            file_names.push(name.to_string());
        }
    }

    if file_names.is_empty() {
        info!("No files to number");
        return Ok(());
    }

    // Sort files
    file_names.sort();

    // Print selections
    println!("Here are files in {}:", dir.display());
    for (i, name) in file_names.iter().enumerate() {
        println!("  - {i}: {name}");
    }

    // Prompt for input
    println!("Input a number: to set num [0] to the first selection.");
    println!("Input two numbers: to set num [1] to the selection in index [0].");
    print!("Input: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        info!("Invalid input");
        return Ok(());
    }

    let (file_idx, num) = if parts.len() == 1 {
        (0, parts[0].parse::<i32>().unwrap_or(0))
    } else {
        (
            parts[0].parse::<usize>().unwrap_or(0),
            parts[1].parse::<i32>().unwrap_or(0),
        )
    };

    if file_idx >= file_names.len() {
        info!("Invalid file index");
        return Ok(());
    }

    let file_name = &file_names[file_idx];
    let file_path = dir.join(file_name);
    let new_file_name = format!("{num} {file_name}");
    let new_file_path = dir.join(&new_file_name);

    info!("Rename {} to {}", file_name, new_file_name);
    std::fs::rename(&file_path, &new_file_path)?;

    // Ask if continue
    print!("Continue processing other files? [y/N]: ");
    io::stdout().flush().unwrap();

    let mut cont = String::new();
    io::stdin().read_line(&mut cont).unwrap();
    if cont.trim().to_lowercase().starts_with('y') {
        set_file_num(dir)?;
    }

    Ok(())
}
