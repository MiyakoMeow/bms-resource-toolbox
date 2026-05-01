//! Directory traversal utilities.
//!
//! This module provides functions for walking BMS directories
//! and checking for chart files.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::bms::CHART_FILE_EXTS;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Walk through BMS directories (directories containing BMS files) - 异步版本
#[must_use]
#[expect(dead_code)]
pub(crate) async fn walk_bms_dirs(root: &Path, max_concurrent: usize) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    if !root.is_dir() {
        return dirs;
    }

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut handles = Vec::new();

    if let Ok(mut entries) = tokio::fs::read_dir(root).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let sem = semaphore.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                if path.is_dir() && has_chart_file(&path).await {
                    Some(path)
                } else {
                    None
                }
            });

            handles.push(handle);
        }
    }

    for handle in handles {
        if let Ok(Some(path)) = handle.await {
            dirs.push(path);
        }
    }

    dirs
}

/// Check if a directory contains a BMS chart file - 异步版本
#[must_use]
pub async fn has_chart_file(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }

    match tokio::fs::read_dir(dir).await {
        Ok(mut entries) => {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if CHART_FILE_EXTS.iter().any(|ext| name_str.ends_with(*ext)) {
                    return true;
                }
            }
            false
        }
        Err(_) => false,
    }
}

/// Remove empty child directories (non-recursive, matches Python's `remove_empty_folder`).
pub fn remove_empty_dirs(dir: &Path) -> Result<(), std::io::Error> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(());
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !super::is_dir_having_file(&path) {
            println!("Remove empty dir: {path:?}");
            match std::fs::remove_dir_all(&path) {
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    println!(" x PermissionError!");
                }
                Err(e) => return Err(e),
                Ok(()) => {}
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_has_chart_file() {
        let temp_dir = std::env::temp_dir();
        assert!(!has_chart_file(&temp_dir).await);
    }
}
