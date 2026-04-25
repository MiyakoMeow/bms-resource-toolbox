//! CLI options and validation.
//!
//! This module provides command-line input handling
//! and external tool validation utilities.

pub mod input;
pub mod validator;
pub mod bms_folder;
pub mod bms_folder_bigpack;
pub mod bms_events;
pub mod bms_folder_media;
pub mod bms_folder_event;
pub mod rawpack;

pub use validator::{check_ffmpeg, check_flac, check_oggenc};
pub use bms_folder::{append_name_by_bms, copy_numbered_workdir_names};
