//! Directory move and merge operations.
//!
//! This module provides utilities for moving files and directories
//! between locations with conflict resolution.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::items_after_statements)]

use std::path::Path;
use tracing::info;

/// Check if a directory contains any files (non-recursive)
#[must_use] 
pub fn is_dir_having_file(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    std::fs::read_dir(dir)
        .map(|mut entries| entries.any(|e| e.map(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false)).unwrap_or(false)))
        .unwrap_or(false)
}

/// Move elements (files and directories) from source to destination
/// If conflict exists, appends a suffix
#[allow(dead_code)]
pub fn move_elements_across_dir(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if !src.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Source is not a directory",
        ));
    }

    if !dst.is_dir() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let filename = entry.file_name();
        let mut final_dst_path = dst.join(&filename);

        // Handle naming conflicts
        if final_dst_path.exists() {
            let stem = final_dst_path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = final_dst_path.extension().map(|e| e.to_string_lossy().to_string());
            let mut counter = 1;
            loop {
                let new_name = match ext.as_ref() {
                    Some(e) => format!("{} ({}).{}", stem, counter, &e),
                    None => format!("{stem} ({counter})"),
                };
                let new_path = dst.join(&new_name);
                if !new_path.exists() {
                    final_dst_path = new_path;
                    break;
                }
                counter += 1;
            }
        }

        // Move the file or directory
        info!("Moving {:?} -> {:?}", src_path, final_dst_path);
        std::fs::rename(&src_path, &final_dst_path).or_else(|_| {
            // If rename fails (cross-device), try copy + delete
            if src_path.is_dir() {
                copy_dir_recursive(&src_path, &final_dst_path)?;
                std::fs::remove_dir_all(&src_path)
            } else {
                std::fs::copy(&src_path, &final_dst_path)?;
                std::fs::remove_file(&src_path)
            }
        })?;
    }

    Ok(())
}

/// Copy directory recursively
#[allow(dead_code)]
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if !src.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Source is not a directory",
        ));
    }

    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_dir_having_file() {
        let temp_dir = std::env::temp_dir();
        assert!(is_dir_having_file(&temp_dir));
        assert!(!is_dir_having_file(&PathBuf::from("/nonexistent")));
    }
}
