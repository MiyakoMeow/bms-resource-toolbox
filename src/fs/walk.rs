//! Directory traversal utilities.
//!
//! This module provides functions for walking BMS directories
//! and checking for chart files.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::bms::CHART_FILE_EXTS;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Walk through BMS directories (directories containing BMS files)
#[must_use]
#[expect(dead_code)]
pub(crate) fn walk_bms_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    if !root.is_dir() {
        return dirs;
    }

    for entry in WalkDir::new(root)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.is_dir() && has_chart_file(path) {
            dirs.push(path.to_path_buf());
        }
    }

    dirs
}

/// Check if a directory contains a BMS chart file
#[must_use]
pub(crate) fn has_chart_file(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }

    std::fs::read_dir(dir)
        .map(|mut entries| {
            entries.any(|e| {
                e.map(|entry| {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    CHART_FILE_EXTS
                        .iter()
                        .any(|ext| name_str.ends_with(&format!(".{ext}")))
                })
                .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Remove empty directories recursively
pub fn remove_empty_dirs(dir: &Path) -> Result<(), std::io::Error> {
    if !dir.is_dir() {
        return Ok(());
    }

    // First, recurse into subdirectories
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                remove_empty_dirs(&path)?;
            }
        }
    }

    // Then, if this directory is empty, remove it
    if is_dir_empty(dir) {
        std::fs::remove_dir(dir)?;
    }

    Ok(())
}

/// Check if a directory is empty
#[must_use]
pub fn is_dir_empty(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_chart_file() {
        // This test requires actual BMS files
        let temp_dir = std::env::temp_dir();
        assert!(!has_chart_file(&temp_dir));
    }
}
