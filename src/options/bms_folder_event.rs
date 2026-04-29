//! BMS folder event utilities.
//!
//! This module provides utilities for BMS event folders.

use std::path::Path;
use tracing::info;

use crate::bms::dir::get_dir_bms_info;

/// Check if numbered folders exist in a BMS event directory
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn check_num_folder(bms_dir: &Path, max_count: i32) {
    info!("Checking BMS event directory: {:?}", bms_dir);

    for i in 1..=max_count {
        let folder_name = format!("{i}");
        let folder_path = bms_dir.join(&folder_name);

        if folder_path.is_dir() {
            info!("  [OK] Folder {} exists", i);
        } else {
            info!("  [MISSING] Folder {} does not exist", i);
        }
    }
}

/// Create numbered folders in a BMS event directory
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn create_num_folders(root_dir: &Path, folder_count: i32) -> Result<(), std::io::Error> {
    info!("Creating {} numbered folders in {:?}", folder_count, root_dir);

    for i in 1..=folder_count {
        let folder_name = format!("{i}");
        let folder_path = root_dir.join(&folder_name);

        if folder_path.is_dir() {
            info!("  Folder {} already exists, skipping", i);
        } else {
            std::fs::create_dir_all(&folder_path)?;
            info!("  Created folder {}", i);
        }
    }

    Ok(())
}

/// Generate a work info table (Excel xlsx) for BMS works in a directory
///
/// This creates an Excel file with title, artist, and genre info
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
///
/// # Panics
///
/// Panics if stdout flush fails.
pub fn generate_work_info_table(root_dir: &Path) -> Result<(), std::io::Error> {
    use std::io::{self, Write};

    info!("Generating work info table for: {:?}", root_dir);

    let mut work_entries: Vec<(u32, String, String, String, String)> = Vec::new();

    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let work_path = entry.path();
        if !work_path.is_dir() {
            continue;
        }

        let work_name = work_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let bms_info = get_dir_bms_info(&work_path);
        let (title, artist, genre) = match bms_info {
            Some(i) => (i.title, i.artist, i.genre),
            None => (String::new(), String::new(), String::new()),
        };

        let id_str = work_name.split('.').next().unwrap_or("");
        if let Ok(id) = id_str.parse::<u32>() {
            work_entries.push((id, work_name, title, artist, genre));
        } else {
            println!("Warning: Skipping dir {work_name} - invalid id format: {id_str}");
        }
    }

    if work_entries.is_empty() {
        info!("No works found in directory");
        return Ok(());
    }

    work_entries.sort_by(|a, b| a.0.cmp(&b.0));

    print!("Output filename (default: bms_list.xlsx): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let filename = input.trim();
    let filename = if filename.is_empty() {
        "bms_list.xlsx".to_string()
    } else {
        filename.to_string()
    };

    let output_path = root_dir.join(&filename);

    info!("Writing to file: {:?}", output_path);

    let mut file = std::fs::File::create(&output_path)?;
    writeln!(file, "ID,Title,Artist,Genre")?;
    for (id, _name, title, artist, genre) in &work_entries {
        writeln!(file, "{id},{title},{artist},{genre}")?;
    }

    println!("Saved table to {}", output_path.display());

    Ok(())
}
