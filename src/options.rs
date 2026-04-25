//! CLI options and validation.
//!
//! This module provides command-line input handling
//! and external tool validation utilities.

pub mod input;
pub mod validator;

pub use validator::{check_ffmpeg, check_flac, check_oggenc};
