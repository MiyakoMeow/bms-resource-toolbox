//! External tool validation utilities.
//!
//! This module provides functions for checking if external
//! tools like ffmpeg, flac, and oggenc are available.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::any::Any;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Check if ffmpeg is available
#[expect(dead_code)]
pub(crate) async fn check_ffmpeg() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if flac is available
#[expect(dead_code)]
pub(crate) async fn check_flac() -> bool {
    Command::new("flac")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if oggenc is available
#[expect(dead_code)]
pub(crate) async fn check_oggenc() -> bool {
    Command::new("oggenc")
        .arg("-v")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Validate path exists and is a directory
#[expect(dead_code)]
pub(crate) fn validate_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }
    Ok(())
}

/// Validate path exists and is a file
#[expect(dead_code)]
pub(crate) fn validate_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }
    Ok(())
}

/// Check if path is a BMS work directory (contains BMS files).
#[expect(dead_code)]
pub(crate) fn is_work_dir(args: &[Box<dyn Any>]) -> bool {
    let path = args
        .first()
        .and_then(|p| p.downcast_ref::<PathBuf>())
        .unwrap();
    if !path.is_dir() {
        return false;
    }

    let bms_exts = [".bms", ".bme", ".bml", ".pms", ".bmson"];
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file()
                && let Some(name) = p.file_name().and_then(|n| n.to_str())
            {
                let lower = name.to_lowercase();
                if bms_exts.iter().any(|ext| lower.ends_with(ext)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if path is not a directory.
#[expect(dead_code)]
pub(crate) fn is_not_a_dir(args: &[Box<dyn Any>]) -> bool {
    let path = args
        .first()
        .and_then(|p| p.downcast_ref::<PathBuf>())
        .unwrap();
    !path.is_dir()
}
