//! Directory-level BMS operations.
//!
//! This module provides functions for scanning directories for BMS files
//! and extracting aggregated BMS information.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::path::Path;

use crate::bms::encoding::get_boftt_encoding;
use crate::bms::parse::{parse_bms_content, parse_bms_file, parse_bmson_file};
use crate::bms::types::{BMSInfo, BMS_FILE_EXTS, BMSON_FILE_EXTS};
use crate::bms::work::extract_work_name;

/// Get list of BMSInfo from all BMS files in a directory
///
/// This replicates Python's `get_dir_bms_list(dir_path)`:
/// - Scans first-level files in the directory
/// - For BOFTT packs, uses ID-specific encoding from directory name
/// - Parses BMS/BME/BML/PMS and BMSON files
#[allow(dead_code)]
pub fn get_dir_bms_list(dir_path: &Path) -> Vec<BMSInfo> {
    let mut info_list: Vec<BMSInfo> = Vec::new();

    // For BOFTT: extract ID from directory name
    let dir_name = dir_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let id = if let Some(dot_pos) = dir_name.find('.') {
        Some(&dir_name[..dot_pos])
    } else {
        Some(dir_name)
    };
    let boftt_encoding = id.and_then(|s| get_boftt_encoding(s));

    // Scan directory
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if !file_path.is_file() {
                continue;
            }

            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Check BMS file extensions
            let lower_name = file_name.to_lowercase();
            let is_bms_file = BMS_FILE_EXTS.iter().any(|ext| lower_name.ends_with(ext));
            let is_bmson_file = BMSON_FILE_EXTS.iter().any(|ext| lower_name.ends_with(ext));

            if is_bms_file {
                // Parse BMS file with optional BOFTT encoding
                let info = parse_bms_file_with_encoding(&file_path, boftt_encoding);
                if info.title.is_empty() && info.artist.is_empty() && info.genre.is_empty() {
                    // Try without encoding override if empty
                    if let Ok(fallback_info) = parse_bms_file(&file_path) {
                        if !fallback_info.title.is_empty() || !fallback_info.artist.is_empty() {
                            info_list.push(fallback_info);
                        }
                    }
                } else {
                    info_list.push(info);
                }
            } else if is_bmson_file {
                if let Ok(info) = parse_bmson_file(&file_path) {
                    info_list.push(info);
                }
            }
        }
    }

    info_list
}

/// Parse BMS file with optional BOFTT-specific encoding
fn parse_bms_file_with_encoding(file_path: &Path, encoding: Option<&str>) -> BMSInfo {
    // Read file bytes
    let bytes = match std::fs::read(file_path) {
        Ok(b) => b,
        Err(_) => return BMSInfo::default(),
    };

    // Get content with encoding priority
    use crate::bms::encoding::get_bms_file_str;
    let content = get_bms_file_str(&bytes, encoding);

    parse_bms_content(&content)
}

/// Get aggregated BMSInfo for a directory
///
/// This replicates Python's `get_dir_bms_info(bms_dir_path)`:
/// - Gets list of all BMS files in directory
/// - Extracts common title/artist/genre using longest-common-prefix
/// - Returns BMSInfo with aggregated metadata
#[allow(dead_code)]
pub fn get_dir_bms_info(bms_dir_path: &Path) -> Option<BMSInfo> {
    let bms_list = get_dir_bms_list(bms_dir_path);
    if bms_list.is_empty() {
        return None;
    }

    // Extract titles and find common work name
    let titles: Vec<String> = bms_list.iter().map(|b| b.title.clone()).collect();
    let title = extract_work_name(&titles, true, &[]);

    // Post-process title: remove trailing "-" if odd number of dashes
    let title = if title.ends_with('-') && title.matches('-').count() % 2 != 0 && title.len() > 1 {
        title[..title.len() - 1].trim().to_string()
    } else {
        title
    };

    // Artist extraction with special trailing sign removal
    let artist = extract_work_name_for_artist(&titles);

    // Genre extraction
    let genres: Vec<String> = bms_list.iter().map(|b| b.genre.clone()).collect();
    let genre = extract_work_name(&genres, true, &[]);

    Some(BMSInfo::new(title, artist, genre))
}

// Re-export extract_work_name variants for use by get_dir_bms_info
pub use crate::bms::work::extract_work_name_for_artist;
