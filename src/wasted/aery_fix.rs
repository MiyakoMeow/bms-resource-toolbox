//! Aery fix utility.
//!
//! This module provides a utility for fixing Aery-themed BMS directories.

use std::path::{Path, PathBuf};

use crate::fs::name::bms_dir_similarity;
use crate::fs::pack_move::{MoveOptions, REPLACE_OPTION_UPDATE_PACK, move_elements_across_dir};

/// Fix Aery folders by merging similar directories
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
///
/// # Panics
///
/// Panics if stdout flush fails.
pub fn aery_fix(src_dir: &Path) -> Result<(), std::io::Error> {
    use std::io::{self, Write};

    if !src_dir.is_dir() {
        println!("{}: not a dir.", src_dir.display());
        return Ok(());
    }

    // Get all directories
    let dirs: Vec<String> = std::fs::read_dir(src_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();

    // Find all Aery directories and their matches
    let mut aery_pairs: Vec<(PathBuf, PathBuf, f64)> = Vec::new();

    for dir in &dirs {
        if !dir.contains("Aery") && !dir.contains("AERY") {
            continue;
        }

        let dir_path = src_dir.join(dir);

        // Look for matching directories by prefix
        for i in 0..dir.len() {
            let prefix = &dir[..=i];
            let matching_dirs: Vec<_> = dirs
                .iter()
                .filter(|d| d.starts_with(prefix) && *d != dir)
                .collect();

            if matching_dirs.len() == 1 {
                let scan_dir = matching_dirs[0];
                let scan_dir_path = src_dir.join(scan_dir);

                let similarity = bms_dir_similarity(&dir_path, &scan_dir_path);
                aery_pairs.push((dir_path.clone(), scan_dir_path.clone(), similarity));
                break;
            }
        }
    }

    // Print pairs
    for p in &aery_pairs {
        println!(
            "{} => {}, similarity: {}",
            p.0.display(),
            p.1.display(),
            p.2
        );
    }

    let similarity_border = 0.95;

    print!("Confirm? (border: {similarity_border}) [y/N]");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input_stripped = input
        .strip_suffix('\n')
        .or_else(|| input.strip_suffix("\r\n"))
        .unwrap_or(&input);
    if !input_stripped.to_lowercase().starts_with('y') {
        return Ok(());
    }

    // Execute moves
    for (p_from, p_to, similarity) in aery_pairs {
        if similarity < similarity_border {
            continue;
        }
        println!(
            "Moving: {} => {}, similarity: {similarity}",
            p_from.display(),
            p_to.display()
        );
        move_elements_across_dir(
            &p_from,
            &p_to,
            MoveOptions::default(),
            &REPLACE_OPTION_UPDATE_PACK,
        )?;
    }

    Ok(())
}
