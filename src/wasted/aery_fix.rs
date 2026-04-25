//! Aery fix utility.
//!
//! This module provides a utility for fixing Aery-themed BMS directories.

use std::path::{Path, PathBuf};
use tracing::info;

use crate::fs::name::bms_dir_similarity;
use crate::fs::pack_move::{move_elements_across_dir, MoveOptions, REPLACE_OPTION_UPDATE_PACK};

/// Fix Aery folders by merging similar directories
#[allow(dead_code)]
pub fn aery_fix(src_dir: &Path) -> Result<(), std::io::Error> {
    use std::io::{self, Write};

    info!("Aery fix for: {:?}", src_dir);

    if !src_dir.is_dir() {
        info!("{:?}: not a dir.", src_dir);
        return Ok(());
    }

    // Get all directories
    let dirs: Vec<String> = std::fs::read_dir(src_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    // Find all Aery directories and their matches
    let mut aery_pairs: Vec<(PathBuf, PathBuf, f64)> = Vec::new();

    for dir in &dirs {
        let dir_lc = dir.to_lowercase();
        if !dir_lc.contains("aery") {
            continue;
        }

        let dir_path = src_dir.join(dir);

        // Look for matching directories by prefix
        for i in 0..dir.len() {
            let prefix = &dir[..=i];
            let matching_dirs: Vec<_> = dirs.iter()
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
        info!("{:?} => {:?}, similarity: {}", p.0, p.1, p.2);
    }

    let similarity_border = 0.95;

    print!("Confirm? (border: {similarity_border}) [y/N]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if !input.trim().to_lowercase().starts_with('y') {
        info!("Aborted.");
        return Ok(());
    }

    // Execute moves
    for (p_from, p_to, similarity) in aery_pairs {
        if similarity < similarity_border {
            continue;
        }
        info!("Moving: {:?} => {:?}, similarity: {}", p_from, p_to, similarity);
        move_elements_across_dir(&p_from, &p_to, MoveOptions::default(), REPLACE_OPTION_UPDATE_PACK.clone())?;
    }

    Ok(())
}
