//! External tool validation utilities.
//!
//! This module provides functions for checking if external
//! tools like ffmpeg, flac, and oggenc are available.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

#[allow(dead_code)]
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Check if ffmpeg is available
#[allow(dead_code)]
pub async fn check_ffmpeg() -> bool {
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
#[allow(dead_code)]
pub async fn check_flac() -> bool {
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
#[allow(dead_code)]
pub async fn check_oggenc() -> bool {
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
#[allow(clippy::unnecessary_debug_formatting, dead_code)]
pub fn validate_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {path:?}"));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {path:?}"));
    }
    Ok(())
}

/// Validate path exists and is a file
#[allow(clippy::unnecessary_debug_formatting, dead_code)]
pub fn validate_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {path:?}"));
    }
    if !path.is_file() {
        return Err(format!("Path is not a file: {path:?}"));
    }
    Ok(())
}
