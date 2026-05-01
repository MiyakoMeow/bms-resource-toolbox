//! Raw pack utilities.
//!
//! This module provides utilities for extracting raw BMS packs
//! and setting file numbers.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::info;

use crate::fs::pack_move::is_dir_having_file;
use crate::fs::rawpack::{
    extract_archive, get_num_set_file_names, move_out_files_in_folder_in_cache_dir,
};
use crate::fs::utils::copy_dir_recursive;

/// Extract archives by original filename to BMS folder structure
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn unzip_with_name_to_bms_folder(
    pack_dir: &Path,
    cache_dir: &Path,
    root_dir: &Path,
    confirm: bool,
) -> Result<(), std::io::Error> {
    info!(
        "Unzip with name to BMS folder: {:?} -> {:?}",
        pack_dir, root_dir
    );

    create_directories(cache_dir, root_dir)?;
    let archive_names = get_archive_files(pack_dir);

    if archive_names.is_empty() {
        info!("No archive files found in {:?}", pack_dir);
        return Ok(());
    }

    let archive_names = sorted_archive_names(archive_names);

    if confirm {
        confirm_archive_processing(&archive_names)?;
    }

    for file_name in &archive_names {
        process_single_archive(pack_dir, cache_dir, root_dir, file_name)?;
    }

    Ok(())
}

fn create_directories(cache_dir: &Path, root_dir: &Path) -> Result<(), std::io::Error> {
    if !cache_dir.is_dir() {
        std::fs::create_dir_all(cache_dir)?;
    }
    if !root_dir.is_dir() {
        std::fs::create_dir_all(root_dir)?;
    }
    Ok(())
}

fn get_archive_files(pack_dir: &Path) -> Vec<String> {
    let mut archive_names = Vec::new();

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

    archive_names
}

fn sorted_archive_names(mut archive_names: Vec<String>) -> Vec<String> {
    archive_names.sort();
    archive_names
}

fn confirm_archive_processing(archive_names: &[String]) -> Result<(), std::io::Error> {
    for file_name in archive_names {
        println!(" --> {file_name}");
    }
    print!("-> Confirm [y/N]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if !input.trim().to_lowercase().starts_with('y') {
        println!("Aborted.");
        return Ok(());
    }
    Ok(())
}

fn process_single_archive(
    pack_dir: &Path,
    cache_dir: &Path,
    root_dir: &Path,
    file_name: &str,
) -> Result<(), std::io::Error> {
    let file_path = pack_dir.join(file_name);

    let file_stem = file_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim_end_matches('.')
        .to_string();

    let cache_dir_path = cache_dir.join(&file_stem);
    prepare_cache_directory(&cache_dir_path)?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(extract_archive(&file_path, &cache_dir_path))?;

    if !move_out_files_in_folder_in_cache_dir(&cache_dir_path) {
        info!("Failed to process cache dir: {:?}", cache_dir_path);
        return Ok(());
    }

    let target_dir_path = root_dir.join(&file_stem);
    move_cache_to_bms_dir(&cache_dir_path, &target_dir_path)?;
    let _ = std::fs::remove_dir(&cache_dir_path);
    // Intentionally ignored: directory may not be empty
    move_original_to_bofttpacks(&file_path, pack_dir, file_name);

    info!("Finished processing: {}", file_name);
    Ok(())
}

fn prepare_cache_directory(cache_dir_path: &Path) -> Result<(), std::io::Error> {
    if cache_dir_path.is_dir() {
        let has_files = std::fs::read_dir(cache_dir_path)
            .is_ok_and(|mut e| e.any(|r| r.is_ok_and(|e| e.path().is_file())));
        if has_files {
            info!("Removing existing cache dir: {:?}", cache_dir_path);
            std::fs::remove_dir_all(cache_dir_path)?;
        }
    }
    std::fs::create_dir_all(cache_dir_path)?;
    Ok(())
}

fn move_cache_to_bms_dir(
    cache_dir_path: &Path,
    target_dir_path: &Path,
) -> Result<(), std::io::Error> {
    info!(
        "Moving files from {:?} to {:?}",
        cache_dir_path, target_dir_path
    );

    let entries: Vec<_> = std::fs::read_dir(cache_dir_path)?
        .filter_map(std::result::Result::ok)
        .collect();

    std::fs::create_dir_all(target_dir_path)?;

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

    Ok(())
}

fn move_original_to_bofttpacks(file_path: &Path, pack_dir: &Path, file_name: &str) {
    let used_pack_dir = pack_dir.join("BOFTTPacks");
    if !used_pack_dir.is_dir() {
        // Intentionally ignored: directory may already exist
        let _ = std::fs::create_dir_all(&used_pack_dir);
    }
    let target_file_path = used_pack_dir.join(file_name);
    // Intentionally ignored: cross-device rename may fail, best-effort move
    let _ = std::fs::rename(file_path, &target_file_path);
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
#[allow(clippy::too_many_lines)]
pub fn unzip_numeric_to_bms_folder(
    pack_dir: &Path,
    cache_dir: &Path,
    root_dir: &Path,
    confirm: bool,
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

    if confirm {
        for file_name in &num_set_file_names {
            println!(" --> {file_name}");
        }
        print!("-> Confirm [y/N]: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Aborted.");
            return Ok(());
        }
    }

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

        // Intentionally ignored: directory may not be empty
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

    loop {
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

                // Skip if companion .part file exists (e.g., foo.zip has foo.zip.part)
                let part_file_path = path.with_file_name(format!("{name}.part"));
                if part_file_path.is_file() {
                    continue;
                }

                // Skip empty files
                if path.metadata().map_or(true, |m| m.len() == 0) {
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
            println!();
            continue;
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
            println!();
            continue;
        }

        let file_name = &file_names[file_idx];
        let file_path = dir.join(file_name);
        let new_file_name = format!("{num} {file_name}");
        let new_file_path = dir.join(&new_file_name);

        info!("Rename {} to {}", file_name, new_file_name);
        std::fs::rename(&file_path, &new_file_path)?;

        // Ask if continue
        print!("继续处理其他文件? [y/N]: ");
        io::stdout().flush().unwrap();

        let mut cont = String::new();
        io::stdin().read_line(&mut cont).unwrap();
        if !cont.trim().to_lowercase().starts_with('y') {
            break;
        }
    }

    Ok(())
}
