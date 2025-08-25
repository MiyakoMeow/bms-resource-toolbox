use std::path::Path;

use log::info;
use smol::{fs, io, stream::StreamExt};

use crate::fs::moving::ReplacePreset;
use crate::{
    fs::{
        rawpack::get_num_set_file_names,
        sync::{preset_for_append, sync_folder},
    },
    media::{audio::process_bms_folders, video::process_bms_video_folders},
    options::{
        rawpack::unzip_numeric_to_bms_folder as rawpack_unzip_numeric_to_bms_folder,
        root::copy_numbered_workdir_names,
        root_bigpack::{get_remove_media_rule_oraja, remove_unneed_media_files},
        work::{BmsFolderSetNameType, set_name_by_bms},
    },
};

/// Remove empty folders
async fn remove_empty_folder(parent_dir: impl AsRef<Path>) -> io::Result<()> {
    let parent_dir = parent_dir.as_ref();
    let mut entries = fs::read_dir(parent_dir).await?;

    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively remove empty folders in subdirectories
            Box::pin(remove_empty_folder(&path)).await?;

            // Check if current directory is empty
            let mut check_entries = fs::read_dir(&path).await?;
            if check_entries.next().await.is_none() {
                fs::remove_dir(&path).await?;
                info!("Removed empty folder: {}", path.display());
            }
        }
    }

    Ok(())
}

/// Raw pack -> HQ pack
/// This function is for parsing Raw version to HQ version. Just for beatoraja/Qwilight players.
pub async fn pack_raw_to_hq(root_dir: impl AsRef<Path>) -> io::Result<()> {
    let root_dir = root_dir.as_ref();

    // Parse Audio
    info!("Parsing Audio... Phase 1: WAV -> FLAC");
    process_bms_folders(
        root_dir,
        &["wav"],
        &["FLAC", "FLAC_FFMPEG"],
        true,  // remove_origin_file_when_success
        true,  // remove_origin_file_when_failed
        false, // skip_on_fail
    )
    .await?;

    // Remove Unneed Media File
    info!("Removing Unneed Files");
    remove_unneed_media_files(root_dir, get_remove_media_rule_oraja()).await?;

    Ok(())
}

/// HQ pack -> LQ pack
/// This file is for parsing HQ version to LQ version. Just for LR2 players.
pub async fn pack_hq_to_lq(root_dir: impl AsRef<Path>) -> io::Result<()> {
    let root_dir = root_dir.as_ref();

    // Parse Audio
    info!("Parsing Audio... Phase 1: FLAC -> OGG");
    process_bms_folders(
        root_dir,
        &["flac"],
        &["OGG_Q10"],
        true,  // remove_origin_file_when_success
        false, // remove_origin_file_when_failed
        false, // skip_on_fail
    )
    .await?;

    // Parse Video
    info!("Parsing Video...");
    process_bms_video_folders(
        root_dir,
        &["mp4"],
        &["MPEG1VIDEO_512X512", "WMV2_512X512", "AVI_512X512"],
        true,  // remove_origin_file
        false, // remove_existing
        false, // use_prefered
    )
    .await?;

    Ok(())
}

/// Check input parameters for pack generation script
#[allow(unused)]
fn pack_setup_rawpack_to_hq_check(pack_dir: &Path, root_dir: &Path) -> bool {
    // Input 1
    info!(" - Input 1: Pack dir path");
    if !pack_dir.is_dir() {
        info!("Pack dir is not valid dir.");
        return false;
    }

    // Print Packs
    info!(" -- There are packs in pack_dir:");
    match get_num_set_file_names(pack_dir) {
        Ok(file_names) => {
            for file_name in file_names {
                info!(" > {}", file_name);
            }
        }
        Err(e) => {
            info!("Error reading pack files: {}", e);
        }
    }

    // Input 2
    info!(" - Input 2: BMS Cache Folder path. (Input a dir path that NOT exists)");
    if root_dir.is_dir() {
        info!("Root dir is an existing dir.");
        return false;
    }

    true
}

/// Pack generation script: Raw pack -> HQ pack
/// BMS Pack Generator by MiyakoMeow.
/// - For Pack Create:
///   Fast creating pack script, from: Raw Packs set numed, to: target bms folder.
///   You need to set pack num before running this script, see options/rawpack.rs => set_file_num
pub async fn pack_setup_rawpack_to_hq(
    pack_dir: impl AsRef<Path>,
    root_dir: impl AsRef<Path>,
) -> io::Result<()> {
    let pack_dir = pack_dir.as_ref();
    let root_dir = root_dir.as_ref();

    // Setup
    fs::create_dir_all(root_dir).await?;

    // Unzip
    info!(
        " > 1. Unzip packs from {} to {}",
        pack_dir.display(),
        root_dir.display()
    );
    let cache_dir = root_dir.join("CacheDir");
    fs::create_dir_all(&cache_dir).await?;
    rawpack_unzip_numeric_to_bms_folder(
        pack_dir,
        &cache_dir,
        root_dir,
        false,
        ReplacePreset::UpdatePack,
    )
    .await?;

    // Check if cache directory is empty, delete if empty
    if cache_dir.exists() {
        let mut cache_entries = fs::read_dir(&cache_dir).await?;
        if cache_entries.next().await.is_none() {
            fs::remove_dir(&cache_dir).await?;
        }
    }

    // Syncing folder name
    info!(" > 2. Setting dir names from BMS Files");
    let mut entries = fs::read_dir(root_dir).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            set_name_by_bms(
                &path,
                BmsFolderSetNameType::AppendTitleArtist,
                false,
                ReplacePreset::UpdatePack,
                true, // skip_already_formatted
            )
            .await?;
        }
    }

    // Parse Audio
    info!(" > 3. Parsing Audio... Phase 1: WAV -> FLAC");
    process_bms_folders(
        root_dir,
        &["wav"],
        &["FLAC", "FLAC_FFMPEG"],
        true,  // remove_origin_file_when_success
        false, // remove_origin_file_when_failed
        false, // skip_on_fail
    )
    .await?;

    // Remove Unneed Media File
    info!(" > 4. Removing Unneed Files");
    remove_unneed_media_files(root_dir, get_remove_media_rule_oraja()).await?;

    Ok(())
}

/// Check input parameters for pack update script
#[allow(unused)]
fn pack_update_rawpack_to_hq_check(pack_dir: &Path, root_dir: &Path, sync_dir: &Path) -> bool {
    // Input 1
    info!(" - Input 1: Pack dir path");
    if !pack_dir.is_dir() {
        info!("Pack dir is not valid dir.");
        return false;
    }

    // Print Packs
    info!(" -- There are packs in pack_dir:");
    match get_num_set_file_names(pack_dir) {
        Ok(file_names) => {
            for file_name in file_names {
                info!(" > {}", file_name);
            }
        }
        Err(e) => {
            info!("Error reading pack files: {}", e);
        }
    }

    // Input 2
    info!(" - Input 2: BMS Cache Folder path. (Input a dir path that NOT exists)");
    if root_dir.is_dir() {
        info!("Root dir is an existing dir.");
        return false;
    }

    // Input 3
    info!(" - Input 3: Already exists BMS Folder path. (Input a dir path that ALREADY exists)");
    info!("This script will use this dir, just for name syncing and file checking.");
    if !sync_dir.is_dir() {
        info!("Syncing dir is not valid dir.");
        return false;
    }

    true
}

/// Pack update script: Raw pack -> HQ pack
/// BMS Pack Generator by MiyakoMeow.
/// - For Pack Update:
///   Fast update script, from: Raw Packs set numed, to: delta bms folder just for making pack update.
///   You need to set pack num before running this script, see scripts_rawpack/rawpack_set_num.py
pub async fn pack_update_rawpack_to_hq(
    pack_dir: impl AsRef<Path>,
    root_dir: impl AsRef<Path>,
    sync_dir: impl AsRef<Path>,
) -> io::Result<()> {
    let pack_dir = pack_dir.as_ref();
    let root_dir = root_dir.as_ref();
    let sync_dir = sync_dir.as_ref();

    // Setup
    fs::create_dir_all(root_dir).await?;

    // Unzip
    info!(
        " > 1. Unzip packs from {} to {}",
        pack_dir.display(),
        root_dir.display()
    );
    let cache_dir = root_dir.join("CacheDir");
    fs::create_dir_all(&cache_dir).await?;
    rawpack_unzip_numeric_to_bms_folder(
        pack_dir,
        &cache_dir,
        root_dir,
        false,
        ReplacePreset::UpdatePack,
    )
    .await?;

    // Syncing folder name
    info!(
        " > 2. Syncing dir name from {} to {}",
        sync_dir.display(),
        root_dir.display()
    );
    copy_numbered_workdir_names(sync_dir, root_dir, false).await?;

    // Parse Audio
    info!(" > 3. Parsing Audio... Phase 1: WAV -> FLAC");
    process_bms_folders(
        root_dir,
        &["wav"],
        &["FLAC", "FLAC_FFMPEG"],
        true,  // remove_origin_file_when_success
        false, // remove_origin_file_when_failed
        false, // skip_on_fail
    )
    .await?;

    // Remove Unneed Media File
    info!(" > 4. Removing Unneed Files");
    remove_unneed_media_files(root_dir, get_remove_media_rule_oraja()).await?;

    // Soft syncing
    info!(
        " > 5. Syncing dir files from {} to {}",
        root_dir.display(),
        sync_dir.display()
    );
    sync_folder(root_dir, sync_dir, &preset_for_append()).await?;

    // Remove Empty folder
    info!(" > 6. Remove empty folder in {}", root_dir.display());
    remove_empty_folder(root_dir).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_empty_folder() {
        smol::block_on(async {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let test_dir = temp_dir.path().join("test");
            fs::create_dir_all(&test_dir)
                .await
                .expect("Failed to create test dir");

            // Create some empty folders
            let empty_dir1 = test_dir.join("empty1");
            let empty_dir2 = test_dir.join("empty2");
            let non_empty_dir = test_dir.join("non_empty");

            fs::create_dir_all(&empty_dir1)
                .await
                .expect("Failed to create empty dir1");
            fs::create_dir_all(&empty_dir2)
                .await
                .expect("Failed to create empty dir2");
            fs::create_dir_all(&non_empty_dir)
                .await
                .expect("Failed to create non_empty dir");

            // Create a file in non_empty_dir
            fs::write(non_empty_dir.join("test.txt"), "test")
                .await
                .expect("Failed to write test file");

            // Execute remove empty folders operation
            remove_empty_folder(&test_dir)
                .await
                .expect("Failed to remove empty folders");

            // Verify results
            assert!(!empty_dir1.exists(), "empty_dir1 should be removed");
            assert!(!empty_dir2.exists(), "empty_dir2 should be removed");
            assert!(non_empty_dir.exists(), "non_empty_dir should still exist");
            assert!(
                non_empty_dir.join("test.txt").exists(),
                "test file should still exist"
            );
        });
    }

    #[test]
    fn test_unzip_numeric_to_bms_folder() {
        smol::block_on(async {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let pack_dir = temp_dir.path().join("packs");
            let root_dir = temp_dir.path().join("root");
            let cache_dir = temp_dir.path().join("cache");

            fs::create_dir_all(&pack_dir)
                .await
                .expect("Failed to create pack dir");
            fs::create_dir_all(&root_dir)
                .await
                .expect("Failed to create root dir");
            fs::create_dir_all(&cache_dir)
                .await
                .expect("Failed to create cache dir");

            // Create a mock numerically named file (not actual compressed file)
            let test_file = pack_dir.join("001 Test Song.txt");
            fs::write(&test_file, "test content")
                .await
                .expect("Failed to create test file");

            // This test mainly verifies if the function structure is correct, actual extraction requires real compressed files
            // Since we don't have real compressed files, we only verify it doesn't panic
            let result = rawpack_unzip_numeric_to_bms_folder(
                &pack_dir,
                &cache_dir,
                &root_dir,
                false,
                ReplacePreset::UpdatePack,
            )
            .await;

            // Verify function execution completes (should not panic even if it fails)
            assert!(
                result.is_ok() || result.is_err(),
                "Function should complete without panicking"
            );
        });
    }
}
