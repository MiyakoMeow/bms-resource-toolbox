//! Archive extraction utilities.
//!
//! This module handles extraction of compressed archives
//! including zip, 7z, and rar formats.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::items_after_statements)]

use std::path::Path;
use tracing::info;
use regex::Regex;

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
    use zip::ZipArchive;

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

    for file_name in file_names {
        let pack_path = pack_dir.join(&file_name);
        info!("Extracting: {}", file_name);

        let file = std::fs::File::open(&pack_path)?;
        let mut archive = ZipArchive::new(file)?;

        // Extract to cache dir
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = cache_dir.join(file.mangled_name());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_num_set_file_names() {
        let temp_dir = std::env::temp_dir();
        let _names = get_num_set_file_names(&temp_dir);
    }
}
