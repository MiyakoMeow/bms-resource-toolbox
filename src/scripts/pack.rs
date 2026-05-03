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
use crate::options::bms_folder_bigpack::{get_remove_media_rule_oraja, remove_unneed_media_files};
use crate::options::rawpack::unzip_numeric_to_bms_folder;
use std::path::Path;

async fn bms_folder_transfer_audio(
    root_dir: &Path,
    input_exts: &[&str],
    presets: &[crate::media::audio::AudioPreset],
    options: &TransferOptions,
) -> Result<(), std::io::Error> {
    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let bms_dir_path = entry.path();
        if !bms_dir_path.is_dir() {
            continue;
        }
        if let Err(e) =
            transfer_audio_by_format_in_dir(&bms_dir_path, input_exts, presets, options).await
        {
            println!(" - Dir: {bms_dir_path:?} Error occured!");
            if options.stop_on_error {
                return Err(e);
            }
        }
    }

    Ok(())
}

async fn bms_folder_transfer_video(
    root_dir: &Path,
    input_exts: &[&str],
    presets: &[crate::media::video::VideoPreset],
    remove_origin_file: bool,
    remove_existing_target_file: bool,
    use_prefered: bool,
) -> Result<(), std::io::Error> {
    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let bms_dir_path = entry.path();
        if !bms_dir_path.is_dir() {
            continue;
        }
        if let Err(e) = transfer_video_by_format_in_dir(
            &bms_dir_path,
            input_exts,
            presets,
            remove_origin_file,
            remove_existing_target_file,
            use_prefered,
        )
        .await
        {
            println!("Error occured!");
            return Err(e);
        }
    }

    Ok(())
}

/// Pack raw BMS to HQ version (for beatoraja/Qwilight)
///
/// 1. Convert WAV -> FLAC
/// 2. Remove unnecessary media files
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn pack_raw_to_hq(root_dir: &Path) -> Result<(), std::io::Error> {
    println!("Pack RAW -> HQ for: {root_dir:?}");

    // Phase 1: Convert WAV to FLAC
    println!("Parsing Audio... Phase 1: WAV -> FLAC");
    let flac_preset = audio_preset_flac();
    let flac_ffmpeg_preset = audio_preset_flac_ffmpeg();
    bms_folder_transfer_audio(
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
    println!("Removing Unneed Files");
    remove_unneed_media_files(root_dir, Some(get_remove_media_rule_oraja()))?;

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
    println!("Pack HQ -> LQ for: {root_dir:?}");

    // Phase 1: Convert FLAC to OGG
    println!("Parsing Audio... Phase 1: FLAC -> OGG");
    let ogg_preset = audio_preset_ogg_q10();
    bms_folder_transfer_audio(
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
    println!("Parsing Video...");
    let presets = vec![
        video_preset_mpeg1video_512x512(),
        video_preset_wmv2_512x512(),
        video_preset_avi_512x512(),
    ];
    bms_folder_transfer_video(root_dir, &["mp4"], &presets, true, true, false).await?;

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
        println!("Pack dir is not vaild dir.");
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

/// Setup raw pack to HQ: extract -> rename -> convert -> clean
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn pack_setup_rawpack_to_hq(
    pack_dir: &Path,
    root_dir: &Path,
) -> Result<(), std::io::Error> {
    println!("Pack Setup RAW -> HQ: {pack_dir:?} -> {root_dir:?}");

    // Setup directories
    if root_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Directory {} already exists", root_dir.display()),
        ));
    }
    tokio::fs::create_dir_all(root_dir).await?;
    let cache_dir = root_dir.join("CacheDir");

    // Step 1: Unzip packs
    println!("Unzipping packs from {pack_dir:?} to {root_dir:?}");
    unzip_numeric_to_bms_folder(pack_dir, &cache_dir, root_dir, false)?;

    // Remove cache dir if empty
    if !is_dir_having_file(&cache_dir) {
        tokio::fs::remove_dir(&cache_dir).await?;
    }

    // Step 2: Set dir names from BMS files
    println!("Setting dir names from BMS Files");
    append_name_by_bms(root_dir).await?;

    // Step 3: Convert WAV -> FLAC
    println!("Parsing Audio... Phase 1: WAV -> FLAC");
    let flac_preset = audio_preset_flac();
    let flac_ffmpeg_preset = audio_preset_flac_ffmpeg();
    bms_folder_transfer_audio(
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

    // Step 4: Remove unnecessary media files
    println!("Removing Unneed Files");
    remove_unneed_media_files(root_dir, Some(get_remove_media_rule_oraja()))?;

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
    println!("Pack Update RAW -> HQ: {pack_dir:?} -> {root_dir:?} (sync from {sync_dir:?})");

    // Setup directories
    if root_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Directory {} already exists", root_dir.display()),
        ));
    }
    tokio::fs::create_dir_all(root_dir).await?;
    let cache_dir = root_dir.join("CacheDir");

    // Step 1: Unzip packs
    println!("Unzipping packs from {pack_dir:?} to {root_dir:?}");
    unzip_numeric_to_bms_folder(pack_dir, &cache_dir, root_dir, false)?;

    // Step 2: Sync dir names from sync_dir
    println!("Syncing dir name from {sync_dir:?} to {root_dir:?}");
    copy_numbered_workdir_names(sync_dir, root_dir)?;

    // Step 3: Convert WAV -> FLAC
    println!("Parsing Audio... Phase 1: WAV -> FLAC");
    let flac_preset = audio_preset_flac();
    let flac_ffmpeg_preset = audio_preset_flac_ffmpeg();
    bms_folder_transfer_audio(
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

    // Step 4: Remove unnecessary media files
    println!("Removing Unneed Files");
    remove_unneed_media_files(root_dir, Some(get_remove_media_rule_oraja()))?;

    // Step 5: Soft sync from sync_dir
    println!("Syncing dir files from {sync_dir:?} to {root_dir:?}");
    sync_folder(root_dir, sync_dir, &SYNC_PRESET_FOR_APPEND, 8).await?;

    // Step 6: Remove empty folders
    println!("Removing empty folder in {root_dir:?}");
    remove_empty_dirs(root_dir)?;

    Ok(())
}
