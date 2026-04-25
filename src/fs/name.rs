//! Filename sanitization utilities.
//!
//! This module provides functions for creating valid
//! filesystem names by replacing invalid characters.

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
