//! Common file system utilities.
//!
//! This module provides shared utilities for file operations
//! used across the codebase.

use std::path::Path;

/// Get file extension using Python's `rsplit(".")[-1]` semantics.
///
/// Returns the part after the last dot, or the full filename if no dot exists.
#[must_use]
pub fn get_ext(path: &Path) -> &str {
    path.file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.rsplit('.').next())
        .unwrap_or("")
}

/// Copy directory recursively.
///
/// # Errors
///
/// Returns [`std::io::Error`] if:
/// - `source` is not a directory
/// - directory creation or copy operations fail
pub fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), std::io::Error> {
    if !source.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Source is not a directory",
        ));
    }

    std::fs::create_dir_all(target)?;

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            std::fs::copy(&source_path, &target_path)?;
        }
    }

    Ok(())
}
