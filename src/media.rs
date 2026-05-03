//! Audio and video conversion processing.
//!
//! This module handles media file conversion using external tools
//! like ffmpeg, flac, and oggenc.

pub mod audio;
pub mod convert;
pub mod video;

/// Options for controlling audio transfer behavior.
#[allow(unused)]
pub use convert::{TransferOptions, transfer_audio_by_format_in_dir};
