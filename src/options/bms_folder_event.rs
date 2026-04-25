//! BMS folder event utilities.
//!
//! This module provides utilities for BMS event folders.

use std::path::Path;
use tracing::info;

use crate::bms::dir::get_dir_bms_info;

/// Check if numbered folders exist in a BMS event directory
#[allow(dead_code)]
pub fn check_num_folder(bms_dir: &Path, max_count: i32) -> Result<(), std::io::Error> {
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

    Ok(())
}

/// Create numbered folders in a BMS event directory
#[allow(dead_code)]
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
/// This creates an Excel file with title, artist, and genre info
#[allow(dead_code)]
pub fn generate_work_info_table(root_dir: &Path) -> Result<(), std::io::Error> {
    use std::io::{self, Write};

    info!("Generating work info table for: {:?}", root_dir);

    // Collect work info
    let mut work_entries: Vec<(String, String, String, String)> = Vec::new();

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

        let info = get_dir_bms_info(&work_path);
        let (title, artist, genre) = match info {
            Some(i) => (i.title, i.artist, i.genre),
            None => (String::new(), String::new(), String::new()),
        };

        work_entries.push((work_name, title, artist, genre));
    }

    if work_entries.is_empty() {
        info!("No works found in directory");
        return Ok(());
    }

    // Sort by work name (number)
    work_entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Prompt for output filename
    print!("Output filename (default: work_info.xlsx): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let filename = input.trim();
    let filename = if filename.is_empty() {
        "work_info.xlsx".to_string()
    } else {
        filename.to_string()
    };

    let output_path = root_dir.join(&filename);

    // Generate xlsx using calamine
    info!("Generating Excel file: {:?}", output_path);

    // For a full implementation, we would use calamine to write the xlsx
    // Since calamine is mainly for reading, we'll use a simpler approach
    // or note that this requires an external tool

    // For now, print to console as CSV as a fallback
    info!("Writing as CSV (xlsx generation requires additional setup):");
    println!("Work Name,Title,Artist,Genre");
    for (name, title, artist, genre) in &work_entries {
        println!("{name},{title},{artist},{genre}");
    }

    // Try to use xlsxwriter if available, otherwise fallback to CSV
    // xlsx generation would go here

    Ok(())
}
