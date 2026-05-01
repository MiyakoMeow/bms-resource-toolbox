//! Filename sanitization utilities.
//!
//! This module provides functions for creating valid
//! filesystem names by replacing invalid characters.

use std::fmt::Write as _;
use std::path::Path;

/// Get a valid filesystem name by replacing invalid characters with full-width equivalents.
///
/// This matches Python behavior: invalid characters are replaced with their
/// full-width Unicode counterparts rather than underscores.
#[must_use]
pub fn get_valid_fs_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            ':' => '：',
            '\\' => '＼',
            '/' => '／',
            '*' => '＊',
            '?' => '？',
            '!' => '！',
            '"' => '＂',
            '<' => '＜',
            '>' => '＞',
            '|' => '｜',
            _ => c,
        })
        .collect()
}

/// Get a valid folder name for a BMS work.
#[allow(dead_code)]
#[must_use]
pub fn get_work_folder_name(id: &str, title: &str, artist: &str) -> String {
    let raw = format!("{id}. {title}");
    let mut name = raw;
    if !artist.is_empty() {
        // Intentionally ignored: write to String never fails
        let _ = write!(name, " [{artist}]");
    }
    get_valid_fs_name(&name)
}

/// Calculate media filename similarity between two directories
///
/// This replicates Python's `bms_dir_similarity(dir_path_a, dir_path_b)`:
/// - Compares media files (ogg, wav, flac, mp4, wmv, avi, mpg, mpeg, bmp, jpg, png) between two directories
/// - Returns the ratio of intersecting media files to the minimum media file count
/// - Returns 0.0 if either directory has no files, no media files, or no non-media files
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

        let Ok(entries) = std::fs::read_dir(dir_path) else {
            return (file_set, media_set, non_media_set);
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            let ext = ext.to_lowercase();
            let with_dot = format!(".{ext}");

            file_set.insert(name.to_string());
            if MEDIA_EXTS.contains(&with_dot.as_str()) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    media_set.insert(stem.to_string());
                }
            } else {
                non_media_set.insert(name.to_string());
            }
        }

        (file_set, media_set, non_media_set)
    }

    let (file_set_a, media_set_a, non_media_set_a) = fetch_dir_elements(dir_path_a);
    let (file_set_b, media_set_b, non_media_set_b) = fetch_dir_elements(dir_path_b);

    if file_set_a.is_empty()
        || file_set_b.is_empty()
        || media_set_a.is_empty()
        || media_set_b.is_empty()
        || non_media_set_a.is_empty()
        || non_media_set_b.is_empty()
    {
        return 0.0;
    }

    // Calculate media intersection ratio
    let intersection: HashSet<_> = media_set_a.intersection(&media_set_b).collect();
    let min_media_count = media_set_a.len().min(media_set_b.len());

    if min_media_count == 0 {
        return 0.0;
    }

    #[expect(clippy::cast_precision_loss)]
    let intersection_ratio = intersection.len() as f64 / min_media_count as f64;
    intersection_ratio
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_valid_fs_name() {
        assert_eq!(get_valid_fs_name("Artist - Title"), "Artist - Title");
        assert_eq!(get_valid_fs_name("Artist: Title"), "Artist： Title");
        assert_eq!(get_valid_fs_name("Test/File"), "Test／File");
        assert_eq!(get_valid_fs_name("Test\\File"), "Test＼File");
        assert_eq!(get_valid_fs_name("Test*File"), "Test＊File");
        assert_eq!(get_valid_fs_name("Test?File"), "Test？File");
        assert_eq!(get_valid_fs_name("Test!File"), "Test！File");
        assert_eq!(get_valid_fs_name("Test\"File"), "Test＂File");
        assert_eq!(get_valid_fs_name("Test<File"), "Test＜File");
        assert_eq!(get_valid_fs_name("Test>File"), "Test＞File");
        assert_eq!(get_valid_fs_name("Test|File"), "Test｜File");
    }

    #[test]
    fn test_get_work_folder_name() {
        assert_eq!(
            get_work_folder_name("1", "Title", "Artist"),
            "1. Title [Artist]"
        );
        assert_eq!(get_work_folder_name("1", "Title", ""), "1. Title");
        assert_eq!(
            get_work_folder_name("1", "Title: Part 1", "Artist"),
            "1. Title： Part 1 [Artist]"
        );
    }
}
