//! Work name extraction utilities.
//!
//! This module provides functions for extracting work names
//! from BMS file titles and directory names.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use regex::Regex;

/// Extract work name from a title string
/// Handles patterns like "Artist - Title [Suffix]" or "Artist - Title"
#[allow(dead_code)]
#[must_use]
pub fn extract_work_name(title: &str) -> String {
    // Pattern: "Artist - Title [Suffix]" -> "Artist - Title"
    // Pattern: "Artist - Title" -> "Artist - Title"
    if let Some(dash_pos) = title.find(" - ") {
        let artist_part = &title[..dash_pos];
        let rest = &title[dash_pos + 3..];

        // Remove bracket suffix from title part
        let title_part = if let Some(bracket_pos) = rest.find(" [") {
            &rest[..bracket_pos]
        } else {
            rest
        };

        return format!("{} - {}", artist_part.trim(), title_part.trim());
    }

    title.trim().to_string()
}

/// Count prefix occurrences of a character
#[allow(dead_code)]
fn count_prefix(s: &str, c: char) -> usize {
    s.chars().take_while(|&ch| ch == c).count()
}

/// Extract work name from path using the original Python algorithm
/// Looks for patterns like "001 Artist - Title" and counts prefixes
#[allow(dead_code)]
#[must_use]
pub fn extract_work_name_from_path(path: &str) -> String {
    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);

    // Find the first space followed by a digit sequence followed by space and content
    // e.g., "001 Artist - Title" or "001_Artist - Title"
    let re = Regex::new(r"^[\d_]+(?:\s+)(.+)$").unwrap();
    if let Some(caps) = re.captures(filename)
        && let Some(matched) = caps.get(1) {
            return extract_work_name(matched.as_str());
        }

    extract_work_name(filename)
}

/// Parse work directory name into components
/// e.g., "001 Artist - Title" -> (Some("001"), "Artist - Title")
#[allow(dead_code)]
#[must_use]
pub fn parse_work_dir_name(name: &str) -> (Option<String>, String) {
    let re = Regex::new(r"^(\d+)[_\s]+(.+)$").unwrap();
    if let Some(caps) = re.captures(name) {
        let num = caps.get(1).map(|m| m.as_str().to_string());
        let rest = caps.get(2).map_or(name, |m| m.as_str());
        return (num, rest.to_string());
    }
    (None, name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_work_name() {
        assert_eq!(
            extract_work_name("Artist - Title [Suffix]"),
            "Artist - Title"
        );
        assert_eq!(
            extract_work_name("Artist - Title"),
            "Artist - Title"
        );
        assert_eq!(extract_work_name("Single Title"), "Single Title");
    }

    #[test]
    fn test_parse_work_dir_name() {
        let (num, rest) = parse_work_dir_name("001 Artist - Title");
        assert_eq!(num, Some("001".to_string()));
        assert_eq!(rest, "Artist - Title");

        let (num, rest) = parse_work_dir_name("Artist - Title");
        assert_eq!(num, None);
        assert_eq!(rest, "Artist - Title");
    }
}
