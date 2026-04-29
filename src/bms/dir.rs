//! Directory-level BMS operations.
//!
//! This module provides functions for scanning directories for BMS files
//! and extracting aggregated BMS information.

use std::path::Path;

use crate::bms::encoding::{get_bms_file_str, get_boftt_encoding};
use crate::bms::parse::{parse_bms_content, parse_bms_file, parse_bmson_file};
use crate::bms::types::{BMS_FILE_EXTS, BMSInfo, BMSON_FILE_EXTS};
use crate::bms::work::extract_work_name;

/// Get list of `BMSInfo` from all BMS files in a directory
///
/// This replicates Python's `get_dir_bms_list(dir_path)`:
/// - Scans first-level files in the directory
/// - For BOFTT packs, uses ID-specific encoding from directory name
/// - Parses BMS/BME/BML/PMS and BMSON files
#[must_use]
pub fn get_dir_bms_list(dir_path: &Path) -> Vec<BMSInfo> {
    let mut info_list: Vec<BMSInfo> = Vec::new();

    let dir_name = dir_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let id = if let Some(dot_pos) = dir_name.find('.') {
        Some(&dir_name[..dot_pos])
    } else {
        Some(dir_name)
    };
    let boftt_encoding = id.and_then(get_boftt_encoding);

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if !file_path.is_file() {
                continue;
            }

            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            let lower_name = file_name.to_lowercase();
            let is_bms_file = BMS_FILE_EXTS.iter().any(|ext| lower_name.ends_with(ext));
            let is_bmson_file = BMSON_FILE_EXTS.iter().any(|ext| lower_name.ends_with(ext));

            if is_bms_file {
                let info = parse_bms_file_with_encoding(&file_path, boftt_encoding);
                if info.title.is_empty() && info.artist.is_empty() && info.genre.is_empty() {
                    if let Ok(fallback_info) = parse_bms_file(&file_path)
                        && (!fallback_info.title.is_empty() || !fallback_info.artist.is_empty())
                    {
                        info_list.push(fallback_info);
                    }
                } else {
                    info_list.push(info);
                }
            } else if is_bmson_file && let Ok(info) = parse_bmson_file(&file_path) {
                info_list.push(info);
            }
        }
    }

    info_list
}

fn parse_bms_file_with_encoding(file_path: &Path, encoding: Option<&str>) -> BMSInfo {
    let Ok(bytes) = std::fs::read(file_path) else {
        return BMSInfo::default();
    };

    let content = get_bms_file_str(&bytes, encoding);

    parse_bms_content(&content)
}

/// Get aggregated `BMSInfo` for a directory
///
/// This replicates Python's `get_dir_bms_info(bms_dir_path)`:
/// - Gets list of all BMS files in directory
/// - Extracts common title/artist/genre using longest-common-prefix
/// - Returns `BMSInfo` with aggregated metadata
#[must_use]
pub fn get_dir_bms_info(bms_dir_path: &Path) -> Option<BMSInfo> {
    let bms_list = get_dir_bms_list(bms_dir_path);
    if bms_list.is_empty() {
        return None;
    }

    let titles: Vec<String> = bms_list.iter().map(|b| b.title.clone()).collect();
    let title = extract_work_name(&titles, true, &[]);

    let title =
        if title.ends_with('-') && !title.matches('-').count().is_multiple_of(2) && title.len() > 1
        {
            title[..title.len() - 1].trim().to_string()
        } else {
            title
        };

    let artist = extract_work_name_for_artist(&titles);

    let genres: Vec<String> = bms_list.iter().map(|b| b.genre.clone()).collect();
    let genre = extract_work_name(&genres, true, &[]);

    Some(BMSInfo::new(title, artist, genre))
}

pub use crate::bms::work::extract_work_name_for_artist;
