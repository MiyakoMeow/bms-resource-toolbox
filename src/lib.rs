#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::unused_async,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::match_same_arms,
    clippy::needless_pass_by_value
)]

pub mod bms;
pub mod cli;
pub mod commands;
pub mod fs;
pub mod media;
pub mod util;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Encoding error: {0}")]
    Encoding(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Process error: {command} failed with code {code}")]
    Process { command: String, code: i32 },
    #[error("Invalid argument: {0}")]
    InvalidArg(String),
    #[error("Cancelled by user")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, AppError>;
