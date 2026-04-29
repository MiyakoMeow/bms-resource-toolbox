//! Error types for BMS Resource Toolbox.

use thiserror::Error;

/// Application error types.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub(crate) enum AppError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Encoding error
    #[error("Encoding error: {0}")]
    Encoding(String),
    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),
    /// External tool error
    #[error("External tool error: {0}")]
    ExternalTool(String),
    /// Path error
    #[error("Path error: {0}")]
    Path(String),
    /// Invalid operation error
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    /// Archive error
    #[error("Archive error: {0}")]
    Archive(String),
    /// BMS error
    #[error("BMS error: {0}")]
    Bms(String),
}

/// Result type alias using `AppError`.
#[allow(dead_code)]
pub(crate) type Result<T> = std::result::Result<T, AppError>;

impl From<zip::result::ZipError> for AppError {
    fn from(e: zip::result::ZipError) -> Self {
        AppError::Archive(e.to_string())
    }
}

impl From<sevenz_rust::Error> for AppError {
    fn from(e: sevenz_rust::Error) -> Self {
        AppError::Archive(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Parse(e.to_string())
    }
}
