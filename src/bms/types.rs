//! BMS type definitions.
//!
//! This module provides core data structures for BMS files
//! including `BMSInfo`, `BMSDifficulty`, and related constants.

use serde::{Deserialize, Serialize};

/// BMS file extensions
pub const BMS_FILE_EXTS: [&str; 4] = [".bms", ".bme", ".bml", ".pms"];
/// BMSON file extensions
pub const BMSON_FILE_EXTS: [&str; 1] = [".bmson"];
/// Chart file extensions (BMS + BMSON)
pub const CHART_FILE_EXTS: [&str; 5] = [".bms", ".bme", ".bml", ".pms", ".bmson"];

/// BMS difficulty levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[derive(Default)]
pub enum BMSDifficulty {
    /// Unspecified or unknown difficulty
    #[default]
    Unknown = 0,
    /// Beginner difficulty (grade 1)
    Beginner = 1,
    /// Normal difficulty (grade 2)
    Normal = 2,
    /// Hyper difficulty (grade 3)
    Hyper = 3,
    /// Another difficulty (grade 4)
    Another = 4,
    /// Insane difficulty (grade 5)
    Insane = 5,
}

impl From<i32> for BMSDifficulty {
    fn from(value: i32) -> Self {
        match value {
            1 => BMSDifficulty::Beginner,
            2 => BMSDifficulty::Normal,
            3 => BMSDifficulty::Hyper,
            4 => BMSDifficulty::Another,
            5 => BMSDifficulty::Insane,
            _ => BMSDifficulty::Unknown,
        }
    }
}

/// BMS file information extracted from headers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BMSInfo {
    /// Title of the BMS track
    pub title: String,
    /// Artist of the BMS track
    pub artist: String,
    /// Genre of the BMS track
    pub genre: String,
    /// Difficulty level
    pub difficulty: BMSDifficulty,
    /// Play level (internal difficulty value)
    pub playlevel: i32,
    /// Supported BMP formats
    pub bmp_formats: Vec<String>,
    /// Total length in seconds
    pub total: Option<f64>,
    /// Stage file (background image)
    pub stage_file: Option<String>,
}

impl Default for BMSInfo {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            genre: String::new(),
            difficulty: BMSDifficulty::Unknown,
            playlevel: 0,
            bmp_formats: Vec::new(),
            total: None,
            stage_file: None,
        }
    }
}

impl BMSInfo {
    /// Create a new `BMSInfo` with basic info.
    #[must_use]
    pub fn new(title: String, artist: String, genre: String) -> Self {
        Self {
            title,
            artist,
            genre,
            ..Default::default()
        }
    }
}

/// Audio file extensions - matches Python `AUDIO_FILE_EXTS`
/// Reserved for API compatibility with Python version.
#[allow(dead_code)]
pub const AUDIO_FILE_EXTS: &[&str] = &[".flac", ".ogg", ".wav"];
/// Video file extensions - matches Python `VIDEO_FILE_EXTS`
/// Reserved for API compatibility with Python version.
#[allow(dead_code)]
pub const VIDEO_FILE_EXTS: &[&str] = &[".mp4", ".mkv", ".avi", ".wmv", ".mpg", ".mpeg"];
/// Image file extensions - matches Python `IMAGE_FILE_EXTS`
/// Reserved for API compatibility with Python version.
#[allow(dead_code)]
pub const IMAGE_FILE_EXTS: &[&str] = &[".jpg", ".png", ".bmp", ".svg"];
/// All media file extensions (audio + video + image) - matches Python `MEDIA_FILE_EXTS`
pub const MEDIA_FILE_EXTS: &[&str] = &[
    ".wav", ".flac", ".ogg", // audio
    ".mp4", ".mkv", ".avi", ".wmv", ".mpg", ".mpeg", // video
    ".jpg", ".png", ".bmp", ".svg", // image (no .gif, no .jpeg)
];
