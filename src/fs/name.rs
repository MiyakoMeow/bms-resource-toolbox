//! Filename sanitization utilities.
//!
//! This module provides functions for creating valid
//! filesystem names by replacing invalid characters.

#[allow(dead_code)]
use crate::bms::types::BMSInfo;
#[allow(dead_code)]
use std::path::{Path, PathBuf};

/// Get a valid filesystem name by replacing invalid characters
#[allow(dead_code)]
#[must_use]
pub fn get_valid_fs_name(name: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut result = String::with_capacity(name.len());
    for c in name.chars() {
        if invalid_chars.contains(&c) {
            result.push('_');
        } else {
            result.push(c);
        }
    }
    result
}

/// Sanitize path for filesystem use
#[allow(dead_code)]
pub fn sanitize_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(get_valid_fs_name)
        .unwrap_or_default();
    path.with_file_name(name)
}

/// Generate work folder name in format "id. title [artist]"
///
/// This replicates Python's `get_work_folder_name(id, info)`:
/// ```python
/// def get_work_folder_name(id: str, info: BMSInfo) -> str:
///     return f"{id}. {get_valid_fs_name(info.title)} [{get_valid_fs_name(info.artist)}]"
/// ```
#[allow(dead_code)]
#[must_use]
pub fn get_work_folder_name(id: &str, info: &BMSInfo) -> String {
    format!(
        "{}. {} [{}]",
        id,
        get_valid_fs_name(&info.title),
        get_valid_fs_name(&info.artist)
    )
}

/// Calculate media filename similarity between two directories
///
/// This replicates Python's `bms_dir_similarity(dir_path_a, dir_path_b)`:
/// - Compares media files (ogg, wav, flac, mp4, wmv, avi, mpg, mpeg, bmp, jpg, png) between two directories
/// - Returns the ratio of intersecting media files to the minimum media file count
/// - Returns 0.0 if either directory has no files, no media files, or no non-media files
#[allow(dead_code)]
#[must_use]
pub fn bms_dir_similarity(dir_path_a: &Path, dir_path_b: &Path) -> f64 {
    use std::collections::HashSet;

    const MEDIA_EXTS: &[&str] = &[
        ".ogg", ".wav", ".flac", ".mp4", ".wmv", ".avi", ".mpg", ".mpeg", ".bmp", ".jpg", ".png",
    ];

    fn fetch_dir_elements(dir_path: &Path) -> (HashSet<String>, HashSet<String>, HashSet<String>) {
        let mut file_set = HashSet::new();
        let mut media_set = HashSet::new();
        let mut non_media_set = HashSet::new();

        let entries = match std::fs::read_dir(dir_path) {
            Ok(e) => e,
            Err(_) => return (file_set, media_set, non_media_set),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let ext = match path.extension().and_then(|e| e.to_str()) {
                Some(e) => e.to_lowercase(),
                None => continue,
            };
            let with_dot = format!(".{ext}");

            file_set.insert(name.clone());
            if MEDIA_EXTS.contains(&with_dot.as_str()) {
                // Use stem (filename without extension) for media comparison
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    media_set.insert(stem.to_string());
                }
            } else {
                non_media_set.insert(name);
            }
        }

        (file_set, media_set, non_media_set)
    }

    let (file_set_a, media_set_a, non_media_set_a) = fetch_dir_elements(dir_path_a);
    let (file_set_b, media_set_b, non_media_set_b) = fetch_dir_elements(dir_path_b);

    // Return 0.0 if either dir has no files, no media, or no non-media
    if file_set_a.is_empty() || file_set_b.is_empty()
        || media_set_a.is_empty() || media_set_b.is_empty()
        || non_media_set_a.is_empty() || non_media_set_b.is_empty() {
        return 0.0;
    }

    // Calculate media intersection ratio
    let intersection: HashSet<_> = media_set_a.intersection(&media_set_b).collect();
    let min_media_count = media_set_a.len().min(media_set_b.len());

    if min_media_count == 0 {
        return 0.0;
    }

    intersection.len() as f64 / min_media_count as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_valid_fs_name() {
        assert_eq!(get_valid_fs_name("Artist - Title"), "Artist - Title");
        assert_eq!(get_valid_fs_name("Artist: Title"), "Artist_ Title");
        assert_eq!(get_valid_fs_name("Test/File"), "Test_File");
    }
}
