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
        let chars: Vec<char> = title.chars().collect();
        for i in 1..=chars.len() {
            let prefix: String = chars[..i].iter().collect();
            *prefix_counts.entry(prefix).or_insert(0) += 1;
        }
    }

    if prefix_counts.is_empty() {
        return String::new();
    }

    let max_count = *prefix_counts.values().max().unwrap_or(&0);

    // Find candidates with >= 67% of max count
    #[expect(clippy::cast_precision_loss)]
    let threshold = max_count as f64 * 0.67;
    let mut candidates: Vec<(String, usize)> = prefix_counts
        .iter()
        .filter(|&(_, count)| {
            #[expect(clippy::cast_precision_loss)]
            {
                (*count as f64) >= threshold
            }
        })
        .map(|(k, &v)| (k.clone(), v))
        .collect();

    // Sort: length desc, count desc, alphabetical asc
    candidates.sort_by(|a, b| {
        let len_cmp = b.0.chars().count().cmp(&a.0.chars().count());
        if len_cmp != std::cmp::Ordering::Equal {
            return len_cmp;
        }
        let count_cmp = b.1.cmp(&a.1);
        if count_cmp != std::cmp::Ordering::Equal {
            return count_cmp;
        }
        a.0.cmp(&b.0)
    });

    let best_candidate = candidates
        .first()
        .map(|(s, _)| s.clone())
        .unwrap_or_default();

    // Post-processing
    post_process(
        &best_candidate,
        remove_unclosed_pair,
        remove_tailing_sign_list,
    )
}

/// Post-process extracted work name
/// Removes unclosed brackets and trailing signs
fn post_process(s: &str, remove_unclosed_pair: bool, remove_tailing_sign_list: &[&str]) -> String {
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
                        && *top_open == *open
                    {
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
                let char_count = result.chars().count();
                let sign_char_count = sign.chars().count();
                result = result
                    .chars()
                    .take(char_count - sign_char_count)
                    .collect::<String>();
                result = result.trim_end().to_string();
                triggered = true;
            }
        }

        if !triggered {
            break;
        }
    }

    result
}

/// Extract work name for artist extraction (with trailing sign removal)
/// Replicates Python's artist extraction with signs: /, :, :, -, obj, obj., Obj, Obj., OBJ, OBJ.
#[must_use]
pub(crate) fn extract_work_name_for_artist(titles: &[String]) -> String {
    extract_work_name(
        titles,
        true,
        &[
            "/", ":", "：", "-", "obj", "obj.", "Obj", "Obj.", "OBJ", "OBJ.",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_work_name_artist_extraction() {
        // Artist with trailing obj-like suffix
        let titles_obj = vec!["Artist obj".to_string(), "Artist obj".to_string()];
        assert_eq!(extract_work_name_for_artist(&titles_obj), "Artist");
    }
}
