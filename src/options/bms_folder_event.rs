//! BMS folder event utilities.
//!
//! This module provides utilities for BMS event folders.

use std::path::Path;
use tracing::info;

use crate::bms::dir::get_dir_bms_info;
use crate::bms::work::parse_work_dir_name;
use rust_xlsxwriter::{Format, Workbook};

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
    info!(
        "Creating {} numbered folders in {:?}",
        folder_count, root_dir
    );

    // Get existing elements to check for conflicts
    let existing_elements: Vec<String> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    for i in 1..=folder_count {
        let folder_name = format!("{i}");
        let folder_path = root_dir.join(&folder_name);

        // Check if folder exists or conflicts with similar names
        // Python logic: exact match OR starts with "{id}." OR starts with "{id} "
        // This prevents "1" from matching "10" while catching "1.txt" or "1 backup"
        let id_exists = existing_elements.iter().any(|element_name| {
            element_name == &folder_name
                || element_name.starts_with(&format!("{folder_name}."))
                || element_name.starts_with(&format!("{folder_name} "))
        });

        if id_exists {
            info!("  Folder {} conflicts with existing entry, skipping", i);
            continue;
        }

        std::fs::create_dir_all(&folder_path)?;
        info!("  Created folder {}", i);
    }

    Ok(())
}

/// Generate a work info table (Excel xlsx) for BMS works in a directory
///
/// This creates an Excel file with title, artist, and genre info
///
/// # Errors
///
/// Returns error if directory operations or xlsx generation fail.
///
/// # Panics
///
/// Panics if stdout flush fails.
pub async fn generate_work_info_table(root_dir: &Path) -> anyhow::Result<()> {
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

        let work_name = work_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let bms_info = get_dir_bms_info(&work_path).await;
        let (title, artist, genre) = match bms_info {
            Some(i) => (i.title, i.artist, i.genre),
            None => (String::new(), String::new(), String::new()),
        };

        let (num_str, _rest) = parse_work_dir_name(&work_name);
        if let Some(id_str) = num_str
            && let Ok(id) = id_str.parse::<u32>()
        {
            work_entries.push((id, work_name, title, artist, genre));
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

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new().set_bold();
    worksheet.write_string_with_format(0, 0, "ID", &header_format)?;
    worksheet.write_string_with_format(0, 1, "Title", &header_format)?;
    worksheet.write_string_with_format(0, 2, "Artist", &header_format)?;
    worksheet.write_string_with_format(0, 3, "Genre", &header_format)?;

    for (row_idx, (id, _name, title, artist, genre)) in work_entries.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let row = (row_idx + 1) as u32;
        worksheet.write_number(row, 0, f64::from(*id))?;
        worksheet.write_string(row, 1, title)?;
        worksheet.write_string(row, 2, artist)?;
        worksheet.write_string(row, 3, genre)?;
    }

    workbook.save(&output_path)?;

    println!("Saved table to {}", output_path.display());

    Ok(())
}
