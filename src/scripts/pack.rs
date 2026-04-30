//! Pack generation functions.
//!
//! This module provides high-level functions for generating
//! BMS packs including RAW to HQ and HQ to LQ conversion.

use crate::fs::{SYNC_PRESET_FOR_APPEND, is_dir_having_file, remove_empty_dirs, sync_folder};
use crate::media::video::{
    transfer_video_by_format_in_dir, video_preset_avi_512x512, video_preset_mpeg1video_512x512,
    video_preset_wmv2_512x512,
};
use crate::media::{
    TransferOptions,
    audio::{audio_preset_flac, audio_preset_flac_ffmpeg, audio_preset_ogg_q10},
    transfer_audio_by_format_in_dir,
};
use crate::options::bms_folder::{append_name_by_bms, copy_numbered_workdir_names};
use crate::options::rawpack::unzip_numeric_to_bms_folder;
use std::path::Path;
use tracing::info;

/// Remove unneeded media files from all works in a directory.
///
/// Processes each subdirectory in `root_dir` and removes media files
/// according to the provided rule.
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub fn remove_unneed_media_files(
    root_dir: &Path,
    rule: &[(&[&str], &[&str])],
) -> Result<(), std::io::Error> {
    for entry in walkdir::WalkDir::new(root_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let work_dir = entry.path();
        if !work_dir.is_dir() {
            continue;
        }

        workdir_remove_unneed_media_files(work_dir, rule)?;
    }

    Ok(())
}

/// Remove unnecessary media files in a single work directory
///
/// This matches Python's `_workdir_remove_unneed_media_files` behavior.
fn workdir_remove_unneed_media_files(
    work_dir: &Path,
    rules: &[(&[&str], &[&str])],
) -> Result<(), std::io::Error> {
    use std::collections::HashSet;

    let mut remove_pairs: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();
    let mut removed_files: HashSet<std::path::PathBuf> = HashSet::new();

    // Collect files in work directory
    let entries: Vec<_> = std::fs::read_dir(work_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    // Find files to remove based on rules
    for entry in &entries {
        let check_file_path = entry.path();
        if !check_file_path.is_file() {
            continue;
        }

        let file_ext = check_file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        for rule in rules {
            let (upper_exts, lower_exts) = *rule;
            if !upper_exts.contains(&file_ext.as_str()) {
                continue;
            }

            // Check if file is empty (skip if empty)
            if let Ok(metadata) = check_file_path.metadata()
                && metadata.len() == 0
            {
                info!("Skipping empty file: {:?}", check_file_path);
                continue;
            }

            // Search for lower quality files to remove
            for lower_ext in lower_exts {
                let replacing_file_path = check_file_path.with_extension(*lower_ext);
                if replacing_file_path.is_file() && !removed_files.contains(&replacing_file_path) {
                    remove_pairs.push((check_file_path.clone(), replacing_file_path.clone()));
                    removed_files.insert(replacing_file_path);
                }
            }
        }
    }

    if !remove_pairs.is_empty() {
        info!("Entering: {:?}", work_dir);
    }

    // Remove files
    for (check_file_path, replacing_file_path) in &remove_pairs {
        info!(
            "Remove file {:?}, because {:?} exists.",
            replacing_file_path.file_name().unwrap_or_default(),
            check_file_path.file_name().unwrap_or_default()
        );
        std::fs::remove_file(replacing_file_path)?;
    }

    // Count extensions for diagnostics
    let mut ext_count: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for entry in &entries {
        let count_file_path = entry.path();
        if !count_file_path.is_file() {
            continue;
        }

        let file_ext = count_file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let file_name = count_file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        ext_count
            .entry(file_ext.clone())
            .or_default()
            .push(file_name);
    }

    // Check for multiple mp4 files
    if let Some(mp4_files) = ext_count.get("mp4")
        && mp4_files.len() > 1
    {
        info!(
            "Tips: {:?} has more than 1 mp4 files! {:?}",
            work_dir, mp4_files
        );
    }

    Ok(())
}

/// Media file removal rule for ORAJA standard.
///
/// Removes redundant video files preferring specific formats:
/// - mp4 > avi/wmv/mpg/mpeg
/// - avi > wmv/mpg/mpeg
/// - flac/wav > ogg
/// - flac > wav
/// - mpg > wmv
pub const REMOVE_MEDIA_RULE_ORAJA: &[(&[&str], &[&str])] = &[
    (&["mp4"], &["avi", "wmv", "mpg", "mpeg"]),
    (&["avi"], &["wmv", "mpg", "mpeg"]),
    (&["flac", "wav"], &["ogg"]),
    (&["flac"], &["wav"]),
    (&["mpg"], &["wmv"]),
];

/// Pack raw BMS to HQ version (for beatoraja/Qwilight)
///
/// 1. Convert WAV -> FLAC
/// 2. Remove unnecessary media files
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn pack_raw_to_hq(root_dir: &Path) -> Result<(), std::io::Error> {
    info!("Pack RAW -> HQ for: {:?}", root_dir);

    // Phase 1: Convert WAV to FLAC
    info!("Parsing Audio... Phase 1: WAV -> FLAC");
    let flac_preset = audio_preset_flac();
    let flac_ffmpeg_preset = audio_preset_flac_ffmpeg();
    transfer_audio_by_format_in_dir(
        root_dir,
        &["wav"],
        &[flac_preset, flac_ffmpeg_preset],
        &TransferOptions {
            remove_origin_on_success: true,
            remove_origin_on_failed: true,
            remove_existing_target_file: true,
            stop_on_error: false,
        },
    )
    .await?;

    // Phase 2: Remove unnecessary media files
    info!("Removing Unneed Files");
    remove_unneed_media_files(root_dir, REMOVE_MEDIA_RULE_ORAJA)?;

    Ok(())
}

/// Pack HQ BMS to LQ version (for LR2)
///
/// 1. Convert FLAC -> OGG
/// 2. Convert MP4 -> AVI/WMV/MPEG (512x512)
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn pack_hq_to_lq(root_dir: &Path) -> Result<(), std::io::Error> {
    info!("Pack HQ -> LQ for: {:?}", root_dir);

    // Phase 1: Convert FLAC to OGG
    info!("Parsing Audio... Phase 1: FLAC -> OGG");
    let ogg_preset = audio_preset_ogg_q10();
    transfer_audio_by_format_in_dir(
        root_dir,
        &["flac"],
        &[ogg_preset],
        &TransferOptions {
            remove_origin_on_success: true,
            remove_origin_on_failed: false,
            remove_existing_target_file: true,
            stop_on_error: false,
        },
    )
    .await?;

    // Phase 2: Convert video
    info!("Parsing Video...");
    let presets = vec![
        video_preset_mpeg1video_512x512(),
        video_preset_wmv2_512x512(),
        video_preset_avi_512x512(),
    ];
    transfer_video_by_format_in_dir(root_dir, &["mp4"], &presets, true, false).await?;

    Ok(())
}

/// Validate inputs for `pack_setup_rawpack_to_hq`
///
/// This replicates Python's `_pack_setup_rawpack_to_hq_check`:
/// - Checks `pack_dir` exists
/// - Prints numbered pack files
/// - Checks `root_dir` does NOT exist
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
#[expect(dead_code)]
pub(crate) fn pack_setup_rawpack_to_hq_check(
    pack_dir: &Path,
    root_dir: &Path,
) -> Result<(), std::io::Error> {
    use crate::fs::rawpack::get_num_set_file_names;

    println!(" - Input 1: Pack dir path");
    if !pack_dir.is_dir() {
        println!("Pack dir is not valid dir.");
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Pack dir is not a valid directory",
        ));
    }

    // Print packs
    let file_names = get_num_set_file_names(pack_dir);
    println!(" -- There are packs in pack_dir:");
    for file_name in &file_names {
        println!(" > {file_name}");
    }

    println!(" - Input 2: BMS Cache Folder path. (Input a dir path that NOT exists)");
    if root_dir.is_dir() {
        println!("Root dir is an existing dir.");
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Root dir already exists",
        ));
    }

    Ok(())
}

/// Validate inputs for `pack_update_rawpack_to_hq`
///
/// This replicates Python's `_pack_update_rawpack_to_hq_check`:
/// - Checks `pack_dir` exists
/// - Prints numbered pack files
/// - Checks `root_dir` does NOT exist
/// - Checks `sync_dir` EXISTS
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
#[expect(dead_code)]
pub(crate) fn pack_update_rawpack_to_hq_check(
    pack_dir: &Path,
    root_dir: &Path,
    sync_dir: &Path,
) -> Result<(), std::io::Error> {
    use crate::fs::rawpack::get_num_set_file_names;

    println!(" - Input 1: Pack dir path");
    if !pack_dir.is_dir() {
        println!("Pack dir is not valid dir.");
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Pack dir is not a valid directory",
        ));
    }

    // Print packs
    let file_names = get_num_set_file_names(pack_dir);
    println!(" -- There are packs in pack_dir:");
    for file_name in &file_names {
        println!(" > {file_name}");
    }

    println!(" - Input 2: BMS Cache Folder path. (Input a dir path that NOT exists)");
    if root_dir.is_dir() {
        println!("Root dir is an existing dir.");
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Root dir already exists",
        ));
    }

    println!(" - Input 3: Already exists BMS Folder path. (Input a dir path that ALREADY exists)");
    println!("This script will use this dir, just for name syncing and file checking.");
    if !sync_dir.is_dir() {
        println!("Syncing dir is not valid dir.");
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Sync dir is not a valid directory",
        ));
    }

    Ok(())
}

/// Setup raw pack to HQ: extract -> rename -> convert -> clean
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn pack_setup_rawpack_to_hq(
    pack_dir: &Path,
    root_dir: &Path,
) -> Result<(), std::io::Error> {
    info!("Pack Setup RAW -> HQ: {:?} -> {:?}", pack_dir, root_dir);

    // Setup directories
    tokio::fs::create_dir_all(root_dir).await?;
    let cache_dir = root_dir.join("CacheDir");

    // Step 1: Unzip packs
    info!("Unzipping packs from {:?} to {:?}", pack_dir, root_dir);
    unzip_numeric_to_bms_folder(pack_dir, &cache_dir, root_dir, false)?;

    // Remove cache dir if empty
    if !is_dir_having_file(&cache_dir) {
        tokio::fs::remove_dir(&cache_dir).await?;
    }

    // Step 2: Set dir names from BMS files
    info!("Setting dir names from BMS Files");
    append_name_by_bms(root_dir).await?;

    // Step 3: Convert WAV -> FLAC
    info!("Parsing Audio... Phase 1: WAV -> FLAC");
    let flac_preset = audio_preset_flac();
    let flac_ffmpeg_preset = audio_preset_flac_ffmpeg();
    transfer_audio_by_format_in_dir(
        root_dir,
        &["wav"],
        &[flac_preset, flac_ffmpeg_preset],
        &TransferOptions {
            remove_origin_on_success: true,
            remove_origin_on_failed: false,
            remove_existing_target_file: true,
            stop_on_error: false,
        },
    )
    .await?;

    // Step 4: Remove unnecessary media files
    info!("Removing Unneed Files");
    remove_unneed_media_files(root_dir, REMOVE_MEDIA_RULE_ORAJA)?;

    Ok(())
}

/// Update raw pack to HQ with sync from existing directory
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn pack_update_rawpack_to_hq(
    pack_dir: &Path,
    root_dir: &Path,
    sync_dir: &Path,
) -> Result<(), std::io::Error> {
    info!(
        "Pack Update RAW -> HQ: {:?} -> {:?} (sync from {:?})",
        pack_dir, root_dir, sync_dir
    );

    // Setup directories
    tokio::fs::create_dir_all(root_dir).await?;
    let cache_dir = root_dir.join("CacheDir");

    // Step 1: Unzip packs
    info!("Unzipping packs from {:?} to {:?}", pack_dir, root_dir);
    unzip_numeric_to_bms_folder(pack_dir, &cache_dir, root_dir, false)?;

    // Step 2: Sync dir names from sync_dir
    info!("Syncing dir name from {:?} to {:?}", sync_dir, root_dir);
    copy_numbered_workdir_names(sync_dir, root_dir)?;

    // Step 3: Convert WAV -> FLAC
    info!("Parsing Audio... Phase 1: WAV -> FLAC");
    let flac_preset = audio_preset_flac();
    let flac_ffmpeg_preset = audio_preset_flac_ffmpeg();
    transfer_audio_by_format_in_dir(
        root_dir,
        &["wav"],
        &[flac_preset, flac_ffmpeg_preset],
        &TransferOptions {
            remove_origin_on_success: true,
            remove_origin_on_failed: false,
            remove_existing_target_file: true,
            stop_on_error: false,
        },
    )
    .await?;

    // Step 4: Remove unnecessary media files
    info!("Removing Unneed Files");
    remove_unneed_media_files(root_dir, REMOVE_MEDIA_RULE_ORAJA)?;

    // Step 5: Soft sync from sync_dir
    info!("Syncing dir files from {:?} to {:?}", sync_dir, root_dir);
    sync_folder(root_dir, sync_dir, &SYNC_PRESET_FOR_APPEND, 8).await?;

    // Step 6: Remove empty folders
    info!("Removing empty folder in {:?}", root_dir);
    remove_empty_dirs(root_dir).await?;

    Ok(())
}
