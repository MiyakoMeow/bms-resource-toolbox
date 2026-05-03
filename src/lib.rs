//! BMS Resource Toolbox Library
//!
//! A Rust library for BMS (Beatmania) chart resource management.
//!
//! # Modules
//!
//! - [`bms`] - BMS file parsing and encoding detection
//! - [`fs`] - File system operations
//! - [`media`] - Audio and video conversion
//! - [`pack`] - Pack generation and archive handling
//! - [`folder`] - BMS folder operations
//! - [`event`] - BMS event operations

// Pre-existing clippy lint — Debug formatting intentional for logging paths.
#![allow(clippy::unnecessary_debug_formatting)]

pub mod bms;
pub mod cli;
pub mod event;
pub mod folder;
pub mod fs;
pub mod media;
pub mod pack;
