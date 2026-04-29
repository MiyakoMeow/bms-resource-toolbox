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
pub(crate) async fn walk_bms_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    if !root.is_dir() {
        return dirs;
    }

    let semaphore = Arc::new(Semaphore::new(8));
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
                if CHART_FILE_EXTS
                    .iter()
                    .any(|ext| name_str.ends_with(&format!(".{ext}")))
                {
                    return true;
                }
            }
            false
        }
        Err(_) => false,
    }
}

/// Remove empty directories recursively - 异步版本
pub async fn remove_empty_dirs(dir: &Path) -> Result<(), std::io::Error> {
    if !dir.is_dir() {
        return Ok(());
    }

    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                Box::pin(remove_empty_dirs(&path)).await?;
            }
        }
    }

    if is_dir_empty(dir).await {
        tokio::fs::remove_dir(dir).await?;
    }

    Ok(())
}

/// Check if a directory is empty - 异步版本
#[must_use]
pub async fn is_dir_empty(dir: &Path) -> bool {
    match tokio::fs::read_dir(dir).await {
        Ok(mut entries) => entries.next_entry().await.ok().flatten().is_none(),
        Err(_) => false,
    }
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
