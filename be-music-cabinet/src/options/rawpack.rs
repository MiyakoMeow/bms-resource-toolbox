use log::info;
use smol::{fs, io, stream::StreamExt};
use std::path::Path;

use crate::fs::{
    is_dir_having_file,
    moving::{MoveOptions, move_elements_across_dir},
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
        let move_result = move_out_files_in_folder_in_cache_dir(&cache_dir_path).await?;
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

        if !target_dir_path.exists() {
            fs::create_dir_all(&target_dir_path).await?;
        }

        // Move cache to bms dir
        info!(
            " > Moving files in {} to {}",
            cache_dir_path.display(),
            target_dir_path.display()
        );
        move_elements_across_dir(
            &cache_dir_path,
            &target_dir_path,
            MoveOptions::default(),
            Default::default(),
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
        let move_result = move_out_files_in_folder_in_cache_dir(&cache_dir_path).await?;
        if !move_result {
            continue;
        }

        let target_dir_path = root_dir.join(file_name_without_ext);

        // Create New Target dir
        if !target_dir_path.exists() {
            fs::create_dir_all(&target_dir_path).await?;
        }

        // Move cache to bms dir
        info!(
            " > Moving files in {} to {}",
            cache_dir_path.display(),
            target_dir_path.display()
        );
        move_elements_across_dir(
            &cache_dir_path,
            &target_dir_path,
            MoveOptions::default(),
            Default::default(),
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

/// Set file number (interactive)
pub async fn set_file_num(dir: impl AsRef<Path>) -> io::Result<()> {
    let dir = dir.as_ref();

    let mut file_names = Vec::new();
    let mut entries = fs::read_dir(dir).await?;

    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let file_path = entry.path();

        // Not File?
        if !entry.file_type().await?.is_file() {
            continue;
        }

        // Has been numbered?
        if file_name
            .split_whitespace()
            .next()
            .is_some_and(|s| s.chars().all(|c| c.is_ascii_digit()))
        {
            continue;
        }

        // Linux: Has Partial File?
        let part_file_path = format!("{}.part", file_path.display());
        if std::path::Path::new(&part_file_path).exists() {
            continue;
        }

        // Linux: Empty File?
        let metadata = fs::metadata(&file_path).await?;
        if metadata.len() == 0 {
            continue;
        }

        // Is Allowed?
        let file_ext = file_name.rsplit('.').next().unwrap_or("").to_lowercase();
        let allowed_exts = ["zip", "7z", "rar", "mp4", "bms", "bme", "bml", "pms"];
        let allowed = allowed_exts.contains(&file_ext.as_str());

        if !allowed {
            continue;
        }

        file_names.push(file_name);
    }

    // Print Selections
    info!("Here are files in {}:", dir.display());
    for (i, file_name) in file_names.iter().enumerate() {
        info!(" - {}: {}", i, file_name);
    }

    info!("Input a number: to set num [0] to the first selection.");
    info!("Input two numbers: to set num [1] to the selection in index [0].");
    info!("Input:");

    // TODO: Implement user input handling
    info!("Note: Interactive input not yet implemented");

    Ok(())
}
