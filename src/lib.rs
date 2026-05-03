//! BMS Resource Toolbox Library
//!
//! A Rust library for BMS (Beatmania) chart resource management.
//!
//! # Modules
//!
//! - [`bms`] - BMS file parsing and encoding detection
//! - [`fs`] - File system operations
//! - [`media`] - Audio and video conversion
//! - [`options`] - CLI options and validation
//! - [`scripts`] - Pack generation scripts

// Pre-existing clippy lint — Debug formatting intentional for logging paths.
#![allow(clippy::unnecessary_debug_formatting)]
// Many items lack docs; suppress warnings for now.
#![allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod bms;
pub mod error;
pub mod fs;
pub mod media;
pub mod options;
pub mod scripts;
