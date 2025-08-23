use std::cmp::Ordering;
use std::collections::HashMap;

/// Extract common work name from multiple titles (improved version)
///
/// # Parameters
/// - `titles`: slice containing multiple titles
/// - `remove_unclosed_pair`: whether to remove unclosed bracket pairs (default: true)
/// - `remove_tailing_sign_list`: list of trailing symbols to remove (default: empty)
///
/// # Returns
/// Extracted common work name (post-processed)
pub fn extract_work_name(
    titles: &[&str],
    remove_unclosed_pair: bool,
    remove_tailing_sign_list: &[&str],
) -> String {
    // If title list is empty, return empty string directly
    if titles.is_empty() {
        return "[!!! EMPTY !!!]".to_string();
    }

    // Use HashMap to count occurrences of all possible prefixes
    let mut prefix_counts: HashMap<String, usize> = HashMap::new();

    // Iterate through each title
    for &title in titles {
        // Generate all prefixes of the title (by character length)
        for i in 1..=title.chars().count() {
            // Get prefix composed of first i characters
            let prefix: String = title.chars().take(i).collect();
            // Update prefix count
            *prefix_counts.entry(prefix).or_insert(0) += 1;
        }
    }

    // Find maximum occurrence count
    let max_count = *prefix_counts.values().max().unwrap_or(&0);

    // Filter candidate prefixes: occurrence count exceeds 2/3 of maximum count
    let mut candidates: Vec<(String, usize)> = prefix_counts
        .into_iter()
        .filter(|(_, count)| *count as f32 >= max_count as f32 * 0.67)
        .collect();

    // Sort candidate prefixes (priority: length descending, then count descending, finally lexicographic ascending)
    candidates.sort_by(|a, b| {
        // 1. Sort by length descending
        let len_cmp = b.0.len().cmp(&a.0.len());
        if len_cmp != Ordering::Equal {
            return len_cmp;
        }

        // 2. Sort by occurrence count descending
        let count_cmp = b.1.cmp(&a.1);
        if count_cmp != Ordering::Equal {
            return count_cmp;
        }

        // 3. Sort by lexicographic ascending
        a.0.cmp(&b.0)
    });

    // Extract optimal candidate (take first if exists, otherwise empty string)
    let best_candidate = candidates
        .first()
        .map(|(s, _)| s.clone())
        .unwrap_or_default();

    // Post-process the optimal candidate
    extract_work_name_post_process(
        &best_candidate,
        remove_unclosed_pair,
        remove_tailing_sign_list,
    )
}

/// Work name post-processing function: remove unclosed brackets and trailing symbols
///
/// # Parameters
/// - `s`: original string
/// - `remove_unclosed_pair`: whether to process unclosed brackets
/// - `remove_tailing_sign_list`: list of trailing symbols to remove
///
/// # Returns
/// Processed string
fn extract_work_name_post_process(
    s: &str,
    remove_unclosed_pair: bool,
    remove_tailing_sign_list: &[&str],
) -> String {
    // First remove leading and trailing whitespace
    let mut result = s.trim().to_string();

    // Define supported bracket pairs (including full-width and half-width)
    const PAIRS: [(char, char); 7] = [
        ('(', ')'),
        ('[', ']'),
        ('{', '}'),
        ('（', '）'),
        ('［', '］'),
        ('｛', '｝'),
        ('【', '】'),
    ];

    // Loop until no changes
    loop {
        let mut changed = false;

        // Process unclosed brackets
        if remove_unclosed_pair {
            // Store bracket state (bracket character + byte position)
            let mut stack: Vec<(char, usize)> = Vec::new();

            // Iterate through each character of the string (with byte index)
            for (byte_idx, c) in result.char_indices() {
                // Check if it's an opening bracket
                if PAIRS.iter().any(|&(open, _)| c == open) {
                    stack.push((c, byte_idx));
                }
                // Check if it's a closing bracket
                else if let Some(&(last_open, _)) = stack.last() {
                    // Find matching closing bracket
                    if let Some(&(_, close)) = PAIRS.iter().find(|&&(open, _)| open == last_open)
                        && c == close
                    {
                        stack.pop();
                    }
                }
            }

            // If there are unclosed brackets
            if let Some(&(_, unmatched_pos)) = stack.last() {
                // Truncate to the position of the first unmatched bracket
                result.truncate(unmatched_pos);
                // Remove trailing whitespace after truncation
                result = result.trim_end().to_string();
                changed = true;
            }
        }

        // Process trailing symbols
        for &sign in remove_tailing_sign_list {
            if result.ends_with(sign) {
                // Remove matching trailing symbol
                result.truncate(result.len() - sign.len());
                // Remove trailing whitespace
                result = result.trim_end().to_string();
                changed = true;
                // Only remove one symbol at a time, then recheck
                break;
            }
        }

        // End processing if no changes
        if !changed {
            break;
        }
    }

    result
}
