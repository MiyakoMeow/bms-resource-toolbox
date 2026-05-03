//! BMS folder event utilities.
//!
//! This module provides utilities for BMS event folders.

use std::path::Path;

use crate::bms::dir::get_dir_bms_info;
use rust_xlsxwriter::Workbook;

/// Check if numbered folders exist in a BMS event directory
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn check_num_folder(bms_dir: &Path, max_count: i32) {
    for i in 1..=max_count {
        let folder_path = bms_dir.join(format!("{i}"));

        if !folder_path.is_dir() {
            println!("{} is not exist!", folder_path.display());
        }
    }
}

/// Create numbered folders in a BMS event directory
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn create_num_folders(root_dir: &Path, folder_count: i32) -> Result<(), std::io::Error> {
    println!("Creating {folder_count} numbered folders in {root_dir:?}");

    // Get existing elements to check for conflicts
    let existing_elements: Vec<String> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().is_dir())
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
            println!("  Folder {i} conflicts with existing entry, skipping");
            continue;
        }

        std::fs::create_dir_all(&folder_path)?;
        println!("  Created folder {i}");
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
    println!("Generating work info table for: {root_dir:?}");

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("BMS List")?;

    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let work_path = entry.path();
        if !work_path.is_dir() {
            continue;
        }

        let work_name = work_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let Some(info) = get_dir_bms_info(&work_path).await else {
            continue;
        };

        let id_str = work_name.split('.').next().unwrap_or("");
        if id_str.is_empty() || !id_str.chars().all(|c| c.is_ascii_digit()) {
            println!("Warning: Skipping dir {work_name} - invalid id format: {id_str}");
            continue;
        }
        let id_num: u32 = id_str.parse().unwrap_or(0);
        if id_num == 0 {
            continue;
        }
        // rust_xlsxwriter is 0-indexed, Python openpyxl is 1-indexed
        let row = id_num - 1;

        worksheet.write_number(row, 0, f64::from(id_num))?;
        worksheet.write_string(row, 1, &info.title)?;
        worksheet.write_string(row, 2, &info.artist)?;
        worksheet.write_string(row, 3, &info.genre)?;
    }

    let table_path = root_dir.join("bms_list.xlsx");
    println!("Saving table to {}", table_path.display());
    workbook.save(&table_path)?;

    Ok(())
}
