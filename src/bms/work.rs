//! Work name extraction utilities.
//!
//! This module provides functions for extracting work names
//! from BMS file titles and directory names.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::collections::HashMap;

/// Extract work name from multiple BMS titles (longest common prefix algorithm)
///
/// This replicates the Python `extract_work_name(titles: list[str])` behavior:
/// - Finds the longest common prefix among multiple titles
/// - Post-processes to remove unclosed brackets and trailing signs
///
/// # Arguments
/// * `titles` - A slice of title strings to extract common work name from
/// * `remove_unclosed_pair` - Whether to remove unclosed brackets and content after
/// * `remove_tailing_sign_list` - Additional trailing signs to remove
#[allow(dead_code)]
#[must_use]
pub fn extract_work_name(
    titles: &[String],
    remove_unclosed_pair: bool,
    remove_tailing_sign_list: &[&str],
) -> String {
    if titles.is_empty() {
        return String::new();
    }

    // Count prefix occurrences
    let mut prefix_counts: HashMap<String, usize> = HashMap::new();
    for title in titles {
        for i in 1..=title.len() {
            let prefix = title[..i].to_string();
            *prefix_counts.entry(prefix).or_insert(0) += 1;
        }
    }

    if prefix_counts.is_empty() {
        return String::new();
    }

    let max_count = *prefix_counts.values().max().unwrap_or(&0);

    // Find candidates with >= 67% of max count
    let mut candidates: Vec<(String, usize)> = prefix_counts
        .iter()
        .filter(|&(_, count)| *count >= (max_count as f64 * 0.67) as usize)
        .map(|(k, &v)| (k.clone(), v))
        .collect();

    // Sort: length desc, count desc, alphabetical asc
    candidates.sort_by(|a, b| {
        let len_cmp = b.0.len().cmp(&a.0.len());
        if len_cmp != std::cmp::Ordering::Equal {
            return len_cmp;
        }
        let count_cmp = b.1.cmp(&a.1);
        if count_cmp != std::cmp::Ordering::Equal {
            return count_cmp;
        }
        a.0.cmp(&b.0)
    });

    let best_candidate = candidates.first().map(|(s, _)| s.clone()).unwrap_or_default();

    // Post-processing
    post_process(
        &best_candidate,
        remove_unclosed_pair,
        remove_tailing_sign_list,
    )
}

/// Post-process extracted work name
/// Removes unclosed brackets and trailing signs
fn post_process(
    s: &str,
    remove_unclosed_pair: bool,
    remove_tailing_sign_list: &[&str],
) -> String {
    let mut result = s.trim().to_string();

    loop {
        let mut triggered = false;

        if remove_unclosed_pair {
            let mut stack: Vec<(char, usize)> = Vec::new();
            let pairs = [
                ('(', ')'),
                ('[', ']'),
                ('{', '}'),
                ('（', '）'),
                ('［', '］'),
                ('｛', '｝'),
                ('【', '】'),
            ];

            for (i, c) in result.char_indices() {
                for (open, close) in &pairs {
                    if c == *open {
                        stack.push((*open, i));
                    }
                    if c == *close
                        && let Some((top_open, _)) = stack.last()
                            && *top_open == *open {
                                stack.pop();
                            }
                }
            }

            // If unclosed brackets exist
            if let Some((_, pos)) = stack.last() {
                result = result[..*pos].trim_end().to_string();
                triggered = true;
            }
        }

        for sign in remove_tailing_sign_list {
            if result.ends_with(sign) {
                result = result[..result.len() - sign.len()].trim_end().to_string();
                triggered = true;
            }
        }

        if !triggered {
            break;
        }
    }

    result
}

/// Extract work name from a single title string (legacy single-title version)
///
/// This is a convenience wrapper for when you have a single title.
/// For the full algorithm that processes multiple titles, use `extract_work_name`.
#[allow(dead_code)]
#[must_use]
pub fn extract_work_name_single(title: &str) -> String {
    extract_work_name(&[title.to_string()], true, &[])
}

/// Extract work name from multiple titles (convenience wrapper)
/// Uses default post-processing: remove unclosed brackets, no extra trailing signs
#[allow(dead_code)]
#[must_use]
pub fn extract_work_name_default(titles: &[String]) -> String {
    extract_work_name(titles, true, &[])
}

/// Extract work name for artist extraction (with trailing sign removal)
/// Replicates Python's artist extraction with signs: /, :, :, -, obj, obj., Obj, Obj., OBJ, OBJ.
#[allow(dead_code)]
#[must_use]
pub fn extract_work_name_for_artist(titles: &[String]) -> String {
    extract_work_name(
        titles,
        true,
        &["/", ":", "：", "-", "obj", "obj.", "Obj", "Obj.", "OBJ", "OBJ."],
    )
}

/// Count prefix occurrences of a character (deprecated - kept for compatibility)
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use extract_work_name() instead")]
fn count_prefix(_s: &str, _c: char) -> usize {
    0 // Deprecated, kept for API compatibility
}

/// Extract work name from path using the original Python algorithm
/// Looks for patterns like "001 Artist - Title" and counts prefixes
#[allow(dead_code)]
#[must_use]
pub fn extract_work_name_from_path(path: &str) -> String {
    use regex::Regex;

    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);

    // Find the first space followed by a digit sequence followed by space and content
    // e.g., "001 Artist - Title" or "001_Artist - Title"
    let re = Regex::new(r"^[\d_]+(?:\s+)(.+)$").unwrap();
    if let Some(caps) = re.captures(filename)
        && let Some(matched) = caps.get(1) {
            return extract_work_name_single(matched.as_str());
        }

    extract_work_name_single(filename)
}

/// Parse work directory name into components
/// e.g., "001 Artist - Title" -> (Some("001"), "Artist - Title")
#[allow(dead_code)]
#[must_use]
pub fn parse_work_dir_name(name: &str) -> (Option<String>, String) {
    use regex::Regex;

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
    fn test_extract_work_name_single_title() {
        // Single title - longest common prefix is the whole title
        assert_eq!(
            extract_work_name_default(&["Artist - Title".to_string()]),
            "Artist - Title"
        );
        assert_eq!(
            extract_work_name_default(&["Single Title".to_string()]),
            "Single Title"
        );
    }

    #[test]
    fn test_extract_work_name_multiple() {
        // Multiple titles - common prefix
        let titles = vec![
            "Artist - Title [Insane]".to_string(),
            "Artist - Title [Hyper]".to_string(),
            "Artist - Title [Another]".to_string(),
        ];
        assert_eq!(extract_work_name_default(&titles), "Artist - Title");
    }

    #[test]
    fn test_extract_work_name_with_unclosed_bracket() {
        // Titles with unclosed bracket
        let titles_unclosed = vec![
            "Artist - Title [Unclosed".to_string(),
            "Artist - Title [Unclosed".to_string(),
        ];
        assert_eq!(extract_work_name_default(&titles_unclosed), "Artist - Title");
    }

    #[test]
    fn test_extract_work_name_artist_extraction() {
        // Artist with trailing obj-like suffix
        let titles_obj = vec![
            "Artist obj".to_string(),
            "Artist obj".to_string(),
        ];
        assert_eq!(
            extract_work_name_for_artist(&titles_obj),
            "Artist"
        );
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
