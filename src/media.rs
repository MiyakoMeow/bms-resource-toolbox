//! Audio and video conversion processing.
//!
//! This module handles media file conversion using external tools
//! like ffmpeg, flac, and oggenc.

pub mod audio;
pub mod convert;
pub mod video;

pub use convert::transfer_audio_by_format_in_dir;
