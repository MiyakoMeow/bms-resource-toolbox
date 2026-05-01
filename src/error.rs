//! Error types for BMS Resource Toolbox.

use std::path::PathBuf;

use thiserror::Error;

/// Top-level error type for the toolbox.
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum ToolboxError {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A path is not a valid BMS root directory.
    #[error("not a BMS root directory: {0}")]
    NotRootDir(PathBuf),

    /// A media conversion step failed.
    #[error("media conversion error: {0}")]
    MediaConversion(String),

    /// A required external tool is missing.
    #[error("missing dependency: {0}")]
    MissingDependency(String),

    /// Input validation failed.
    #[error("validation error: {0}")]
    Validation(String),

    /// A generic error from an anyhow source.
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
