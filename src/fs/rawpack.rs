//! Archive extraction utilities.
//!
//! This module handles extraction of compressed archives
//! including zip, 7z, and rar formats.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::items_after_statements)]

use std::collections::HashMap;
use std::path::Path;
use tracing::info;
use regex::Regex;

use crate::bms::types::CHART_FILE_EXTS;
use crate::fs::pack_move::{move_elements_across_dir, DEFAULT_MOVE_OPTIONS, DEFAULT_REPLACE_OPTIONS};

/// Get numbered file names from a directory
/// Matches patterns like "001 filename.zip", "`001_filename.7z`"
#[allow(dead_code)]
#[must_use]
pub fn get_num_set_file_names(dir: &Path) -> Vec<String> {
    let re = Regex::new(r"^(\d+)[_\s]+(.+)$").unwrap();
    let mut names: Vec<String> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if re.is_match(&name) {
                names.push(name);
            }
        }
    }

    names.sort();
    names
}

/// Extract numeric-prefixed archives to BMS folder structure
#[allow(dead_code)]
pub fn extract_numeric_to_bms_folder(
    pack_dir: &Path,
    cache_dir: &Path,
    root_dir: &Path,
) -> Result<(), std::io::Error> {
    use tokio::runtime::Runtime;

    if !pack_dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Pack dir is not a directory",
        ));
    }

    std::fs::create_dir_all(cache_dir)?;
    std::fs::create_dir_all(root_dir)?;

    let file_names = get_num_set_file_names(pack_dir);
    info!("Found {} pack files", file_names.len());

    // Create runtime for async extraction
    let rt = Runtime::new()?;

    for file_name in file_names {
        let pack_path = pack_dir.join(&file_name);
        info!("Extracting: {}", file_name);

        // Determine archive type and extract
        let ext = pack_path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .unwrap_or_default();

        match ext.as_str() {
            "zip" => {
                extract_zip(&pack_path, cache_dir)?;
            }
            "7z" => {
                let pack_path_buf = pack_path.clone();
                let cache_dir_buf = cache_dir.to_path_buf();
                rt.block_on(async {
                    extract_7z(&pack_path_buf, &cache_dir_buf).await
                })?;
            }
            "rar" => {
                let pack_path_buf = pack_path.clone();
                let cache_dir_buf = cache_dir.to_path_buf();
                rt.block_on(async {
                    extract_rar(&pack_path_buf, &cache_dir_buf).await
                })?;
            }
            _ => {
                info!("Skipping non-archive file: {}", file_name);
            }
        }
    }

    Ok(())
}

/// Extract archive files (zip, 7z, rar)
#[allow(dead_code)]
pub async fn extract_archive(
    archive_path: &Path,
    output_dir: &Path,
) -> Result<(), std::io::Error> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    std::fs::create_dir_all(output_dir)?;

    match ext.as_str() {
        "zip" => extract_zip(archive_path, output_dir),
        "7z" => extract_7z(archive_path, output_dir).await,
        "rar" => extract_rar(archive_path, output_dir).await,
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported archive format: {ext}"),
        )),
    }
}

#[allow(dead_code)]
fn extract_zip(archive_path: &Path, output_dir: &Path) -> Result<(), std::io::Error> {
    use zip::ZipArchive;

    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = output_dir.join(file.mangled_name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn extract_7z(archive_path: &Path, output_dir: &Path) -> Result<(), std::io::Error> {
    let archive_path = archive_path.to_path_buf();
    let output_dir = output_dir.to_path_buf();
    let result = tokio::task::spawn_blocking(move || {
        sevenz_rust::decompress_file(&archive_path, &output_dir)
    }).await;
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(std::io::Error::other(format!("7z error: {e}"))),
        Err(e) => Err(std::io::Error::other(format!("Join error: {e}"))),
    }
}

#[allow(dead_code)]
async fn extract_rar(archive_path: &Path, output_dir: &Path) -> Result<(), std::io::Error> {
    // Use unrar crate or fall back to external tool
    use tokio::process::Command;

    let output = Command::new("unrar")
        .args(["x", "-o+", &archive_path.to_string_lossy(), &output_dir.to_string_lossy()])
        .output()
        .await?;

    if !output.status.success() {
        return Err(std::io::Error::other(
            "Failed to extract rar archive",
        ));
    }
    Ok(())
}

/// Move out files from nested folders in cache directory
///
/// This replicates Python's `move_out_files_in_folder_in_cache_dir(cache_dir_path)`:
/// - Iteratively unpacks nested folder structures
/// - Removes __MACOSX directories
/// - Handles single inner folder case (if >= 10 files, considered "done")
/// - If multiple inner folders, checks for BMS files to determine if state is acceptable
/// - Moves files out of inner directories to the cache root
///
/// Returns `true` on success, `false` on error or empty cache
#[allow(dead_code)]
pub fn move_out_files_in_folder_in_cache_dir(cache_dir_path: &Path) -> bool {
    let mut error = false;

    loop {
        let mut file_ext_count: HashMap<String, Vec<String>> = HashMap::new();
        let mut cache_folder_count: usize = 0;
        let mut cache_file_count: usize = 0;
        let mut inner_dir_name: Option<String> = None;

        // Scan cache directory
        let entries = match std::fs::read_dir(cache_dir_path) {
            Ok(e) => e,
            Err(_) => break,
        };

        for entry in entries.flatten() {
            let cache_path = entry.path();
            let cache_name = cache_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if cache_path.is_dir() {
                // Remove __MACOSX directory
                if cache_name == "__MACOSX" {
                    info!("Removing __MACOSX directory: {:?}", cache_path);
                    if let Err(e) = std::fs::remove_dir_all(&cache_path) {
                        info!("Failed to remove __MACOSX: {}", e);
                    }
                    continue;
                }
                // Normal directory
                cache_folder_count += 1;
                inner_dir_name = Some(cache_name.to_string());
            }

            if cache_path.is_file() {
                cache_file_count += 1;
                // Count extensions
                let ext = cache_path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                file_ext_count.entry(ext).or_default().push(cache_name.to_string());
            }
        }

        // Determine state
        let done;
        if cache_folder_count == 0 {
            done = true;
        } else if cache_folder_count == 1 && cache_file_count >= 10 {
            done = true;
        } else if cache_folder_count > 1 {
            // Check if there are BMS files anywhere in cache_dir
            let mut has_bms = false;
            for entry in walkdir::WalkDir::new(cache_dir_path).into_iter().flatten() {
                let path = entry.path();
                if path.is_file()
                    && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let lower_name = name.to_lowercase();
                        if CHART_FILE_EXTS.iter().any(|ext| lower_name.ends_with(ext)) {
                            has_bms = true;
                            break;
                        }
                    }
            }
            if has_bms {
                done = true;
            } else {
                info!(" !_! {}: has more than 1 folders, please do it manually.", cache_dir_path.display());
                error = true;
                done = false;
            }
        } else {
            done = false;
        }

        if done || error {
            break;
        }

        // Move files out of inner directory
        if let Some(ref inner_name) = inner_dir_name {
            let inner_dir_path = cache_dir_path.join(inner_name);
            // Avoid two floor same name: if inner_dir_path/inner_name exists, rename it
            let inner_inner_dir_path = inner_dir_path.join(inner_name);
            if inner_inner_dir_path.is_dir() {
                info!(" - Renaming inner inner dir name: {:?}", inner_inner_dir_path);
                let new_path = inner_dir_path.with_file_name(format!("{inner_name}-rep"));
                if let Err(e) = std::fs::rename(&inner_inner_dir_path, &new_path) {
                    info!("Failed to rename inner inner dir: {}", e);
                }
            }
            // Move files
            info!(" - Moving inner files in {:?} to {:?}", inner_dir_path, cache_dir_path);
            if let Err(e) = move_elements_across_dir(&inner_dir_path, cache_dir_path, DEFAULT_MOVE_OPTIONS, DEFAULT_REPLACE_OPTIONS.clone()) {
                info!("Failed to move elements: {}", e);
            }
            // Try to remove the now-empty inner directory
            let _ = std::fs::remove_dir(&inner_dir_path);
        }
    }

    // Final checks
    let (final_folder_count, final_file_count) = count_cache_contents(cache_dir_path);

    if error {
        return false;
    }

    if final_folder_count == 0 && final_file_count == 0 {
        info!(" !_! {}: Cache is Empty!", cache_dir_path.display());
        // Try to remove the empty cache directory
        let _ = std::fs::remove_dir(cache_dir_path);
        return false;
    }

    // Check for multiple mp4 files
    let mp4_files: usize = file_ext_count_at_path(cache_dir_path)
        .get("mp4")
        .map_or(0, std::vec::Vec::len);
    if mp4_files > 1 {
        info!(" - Tips: {} has more than 1 mp4 files!", cache_dir_path.display());
    }

    true
}

/// Count folders and files in a directory
fn count_cache_contents(dir: &Path) -> (usize, usize) {
    let mut folder_count = 0;
    let mut file_count = 0;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                folder_count += 1;
            } else if entry.path().is_file() {
                file_count += 1;
            }
        }
    }

    (folder_count, file_count)
}

/// Get file extension counts in a directory
fn file_ext_count_at_path(dir: &Path) -> HashMap<String, Vec<String>> {
    let mut ext_count: HashMap<String, Vec<String>> = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                let ext = entry.path().extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                ext_count.entry(ext).or_default().push(name);
            }
        }
    }

    ext_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_num_set_file_names() {
        let temp_dir = std::env::temp_dir();
        let _names = get_num_set_file_names(&temp_dir);
    }
}
