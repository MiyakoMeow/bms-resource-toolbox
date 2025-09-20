use log::info;
use smol::{fs, io, stream::StreamExt};
use std::path::Path;

use crate::fs::{
    is_dir_having_file,
    moving::{ReplacePreset, move_elements_across_dir, replace_options_from_preset},
    rawpack::{
        get_num_set_file_names, move_out_files_in_folder_in_cache_dir, unzip_file_to_cache_dir,
    },
};

/// Extract numerically named pack files to BMS folders
pub async fn unzip_numeric_to_bms_folder(
    pack_dir: impl AsRef<Path>,
    cache_dir: impl AsRef<Path>,
    root_dir: impl AsRef<Path>,
    confirm: bool,
    replace_preset: ReplacePreset,
) -> io::Result<()> {
    let pack_dir = pack_dir.as_ref();
    let cache_dir = cache_dir.as_ref();
    let root_dir = root_dir.as_ref();

    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir).await?;
    }
    if !root_dir.exists() {
        fs::create_dir_all(root_dir).await?;
    }

    let num_set_file_names = get_num_set_file_names(pack_dir)?;

    if confirm {
        for file_name in &num_set_file_names {
            info!(" --> {}", file_name);
        }
        info!("-> Confirm [y/N]:");
        // TODO: Implement user input confirmation
        return Ok(());
    }

    for file_name in num_set_file_names {
        let file_path = pack_dir.join(&file_name);
        let id_str = file_name.split(' ').next().unwrap_or("");

        // Prepare an empty cache dir
        let cache_dir_path = cache_dir.join(id_str);

        if cache_dir_path.exists() && is_dir_having_file(&cache_dir_path).await? {
            fs::remove_dir_all(&cache_dir_path).await?;
        }

        if !cache_dir_path.exists() {
            fs::create_dir_all(&cache_dir_path).await?;
        }

        // Unpack & Copy
        unzip_file_to_cache_dir(&file_path, &cache_dir_path).await?;

        // Move files in dir
        let move_result =
            move_out_files_in_folder_in_cache_dir(&cache_dir_path, replace_preset).await?;
        if !move_result {
            continue;
        }

        // Find Existing Target dir
        let mut target_dir_path = None;
        let mut entries = fs::read_dir(root_dir).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let dir_name = entry.file_name().to_string_lossy().into_owned();
            let dir_path = entry.path();

            if !entry.file_type().await?.is_dir() {
                continue;
            }

            if !(dir_name.starts_with(id_str)
                && (dir_name.len() == id_str.len() || dir_name[id_str.len()..].starts_with('.')))
            {
                continue;
            }
            target_dir_path = Some(dir_path);
        }

        // Create New Target dir
        let target_dir_path = if let Some(path) = target_dir_path {
            path
        } else {
            root_dir.join(id_str)
        };

        // Move cache to bms dir
        info!(
            " > Moving files in {} to {}",
            cache_dir_path.display(),
            target_dir_path.display()
        );
        move_elements_across_dir(
            &cache_dir_path,
            &target_dir_path,
            replace_options_from_preset(replace_preset),
        )
        .await?;

        // Try to remove empty cache directory
        fs::remove_dir(&cache_dir_path).await.ok();

        // Move File to Another dir
        info!(" > Finish dealing with file: {}", file_name);
        let used_pack_dir = pack_dir.join("BOFTTPacks");
        if !used_pack_dir.exists() {
            fs::create_dir_all(&used_pack_dir).await?;
        }
        fs::rename(&file_path, used_pack_dir.join(&file_name)).await?;
    }

    Ok(())
}

/// Extract files with names to BMS folders
pub async fn unzip_with_name_to_bms_folder(
    pack_dir: impl AsRef<Path>,
    cache_dir: impl AsRef<Path>,
    root_dir: impl AsRef<Path>,
    confirm: bool,
    replace_preset: ReplacePreset,
) -> io::Result<()> {
    let pack_dir = pack_dir.as_ref();
    let cache_dir = cache_dir.as_ref();
    let root_dir = root_dir.as_ref();

    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir).await?;
    }
    if !root_dir.exists() {
        fs::create_dir_all(root_dir).await?;
    }

    let mut num_set_file_names = Vec::new();
    let mut entries = fs::read_dir(pack_dir).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if !entry.file_type().await?.is_file() {
            continue;
        }

        if file_name.ends_with(".zip") || file_name.ends_with(".7z") || file_name.ends_with(".rar")
        {
            num_set_file_names.push(file_name);
        }
    }

    if confirm {
        for file_name in &num_set_file_names {
            info!(" --> {}", file_name);
        }
        info!("-> Confirm [y/N]:");
        // TODO: Implement user input confirmation
        return Ok(());
    }

    for file_name in num_set_file_names {
        let file_path = pack_dir.join(&file_name);
        let file_name_without_ext = if let Some(dot_pos) = file_name.rfind('.') {
            &file_name[..dot_pos]
        } else {
            &file_name
        };

        let file_name_without_ext = file_name_without_ext.trim_end_matches('.');

        // Prepare an empty cache dir
        let cache_dir_path = cache_dir.join(file_name_without_ext);

        if cache_dir_path.exists() && is_dir_having_file(&cache_dir_path).await? {
            fs::remove_dir_all(&cache_dir_path).await?;
        }

        if !cache_dir_path.exists() {
            fs::create_dir_all(&cache_dir_path).await?;
        }

        // Unpack & Copy
        unzip_file_to_cache_dir(&file_path, &cache_dir_path).await?;

        // Move files in dir
        let move_result =
            move_out_files_in_folder_in_cache_dir(&cache_dir_path, replace_preset).await?;
        if !move_result {
            continue;
        }

        let target_dir_path = root_dir.join(file_name_without_ext);

        // Move cache to bms dir
        info!(
            " > Moving files in {} to {}",
            cache_dir_path.display(),
            target_dir_path.display()
        );
        move_elements_across_dir(
            &cache_dir_path,
            &target_dir_path,
            replace_options_from_preset(replace_preset),
        )
        .await?;

        // Try to remove empty cache directory
        fs::remove_dir(&cache_dir_path).await.ok();

        // Move File to Another dir
        info!(" > Finish dealing with file: {}", file_name);
        let used_pack_dir = pack_dir.join("BOFTTPacks");
        if !used_pack_dir.exists() {
            fs::create_dir_all(&used_pack_dir).await?;
        }
        fs::rename(&file_path, used_pack_dir.join(&file_name)).await?;
    }

    Ok(())
}

/// Rename file with number
async fn _rename_file_with_num(
    dir: impl AsRef<Path>,
    file_name: &str,
    input_num: i32,
) -> io::Result<()> {
    let dir = dir.as_ref();
    let file_path = dir.join(file_name);
    let new_file_name = format!("{} {}", input_num, file_name);
    let new_file_path = dir.join(&new_file_name);

    fs::rename(&file_path, &new_file_path).await?;
    info!("Rename {} to {}.", file_name, new_file_name);
    info!("");

    Ok(())
}

/// Set file number (interactive loop)
pub async fn set_file_num(dir: impl AsRef<Path>, allowed_exts: &[&str]) -> io::Result<()> {
    let dir = dir.as_ref();

    loop {
        let mut file_names = Vec::new();
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().into_owned();
            let file_path = entry.path();

            // Not File?
            let Some(file_type) = entry.file_type().await.ok() else {
                continue;
            };
            if !file_type.is_file() {
                continue;
            }

            // Has been numbered?
            let Some(first_part) = file_name.split_whitespace().next() else {
                continue;
            };
            if first_part.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            // Linux: Has Partial File?
            let part_file_path = format!("{}.part", file_path.display());
            if std::path::Path::new(&part_file_path).exists() {
                continue;
            }

            // Linux: Empty File?
            let Ok(metadata) = fs::metadata(&file_path).await else {
                continue;
            };
            if metadata.len() == 0 {
                continue;
            }

            // Is Allowed?
            let Some(file_ext) = file_name.rsplit('.').next() else {
                continue;
            };
            let file_ext = file_ext.to_lowercase();
            if !allowed_exts.contains(&file_ext.as_str()) {
                continue;
            }

            file_names.push(file_name);
        }

        if file_names.is_empty() {
            info!("No files found to number in {}", dir.display());
            return Ok(());
        }

        // Print Selections
        info!("Here are files in {}:", dir.display());
        for (i, file_name) in file_names.iter().enumerate() {
            info!(" - {}: {}", i, file_name);
        }

        info!("Input a number: to set num [0] to the first selection.");
        info!("Input two numbers: to set num [1] to the selection in index [0].");
        info!("Input 'q' or 'quit' to exit.");
        info!("Input:");

        // Simple user input handling
        let mut input = String::new();
        let Ok(_) = std::io::stdin().read_line(&mut input) else {
            info!("Failed to read input, exiting.");
            return Ok(());
        };
        let input = input.trim();

        if input.is_empty() {
            info!("No input provided, exiting.");
            return Ok(());
        }

        // Check for exit commands
        if input == "q" || input == "quit" {
            info!("Exiting file numbering.");
            return Ok(());
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.len() {
            1 => {
                // Single number: set num [0] to the first selection
                let Ok(num) = parts[0].parse::<i32>() else {
                    info!("Invalid number: {}", parts[0]);
                    continue;
                };
                if num >= 0 && num < file_names.len() as i32 {
                    _rename_file_with_num(dir, &file_names[num as usize], 0).await?;
                } else {
                    info!("Invalid file index: {}", num);
                }
            }
            2 => {
                // Two numbers: set num [1] to the selection in index [0]
                let Ok(target_num) = parts[0].parse::<i32>() else {
                    info!("Invalid target number: {}", parts[0]);
                    continue;
                };
                let Ok(file_index) = parts[1].parse::<i32>() else {
                    info!("Invalid file index: {}", parts[1]);
                    continue;
                };
                if file_index >= 0 && file_index < file_names.len() as i32 {
                    _rename_file_with_num(dir, &file_names[file_index as usize], target_num)
                        .await?;
                } else {
                    info!("Invalid file index: {}", file_index);
                }
            }
            _ => {
                info!("Invalid input format. Expected: <number> or <target_number> <file_index>");
            }
        }

        info!(""); // Add blank line for better readability
    }
}
