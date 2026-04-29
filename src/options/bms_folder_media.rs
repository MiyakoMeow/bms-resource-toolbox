//! BMS folder media transfer utilities.
//!
//! This module provides interactive functions for transferring
//! audio and video files in BMS directories.

use std::path::Path;
use tracing::info;

use crate::media::audio::{
    AudioPreset, audio_preset_flac, audio_preset_flac_ffmpeg, audio_preset_ogg_q10,
    audio_preset_wav_ffmpeg, audio_preset_wav_from_flac,
};
use crate::media::video::{
    VIDEO_PRESET_AVI_480P, VIDEO_PRESET_AVI_512X512, VIDEO_PRESET_MPEG1VIDEO_480P,
    VIDEO_PRESET_MPEG1VIDEO_512X512, VIDEO_PRESET_WMV2_480P, VIDEO_PRESET_WMV2_512X512,
    VideoPreset, transfer_video_by_format_in_dir,
};
use crate::media::{TransferOptions, transfer_audio_by_format_in_dir};

/// Transfer audio files in a BMS root directory
///
/// This is an interactive function that prompts the user for settings
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
///
/// # Panics
///
/// Panics if stdout flush fails.
pub async fn transfer_audio(root_dir: &Path) -> Result<(), std::io::Error> {
    use std::io::{self, Write};

    info!("Audio Transfer for: {:?}", root_dir);

    // Audio transfer modes: (name, input_exts, presets)
    let modes: [(&str, Vec<&str>, Vec<AudioPreset>); 4] = [
        (
            "Convert: WAV to FLAC",
            vec!["wav"],
            vec![audio_preset_flac(), audio_preset_flac_ffmpeg()],
        ),
        (
            "Compress: FLAC to OGG Q10",
            vec!["flac"],
            vec![audio_preset_ogg_q10()],
        ),
        (
            "Compress: WAV to OGG Q10",
            vec!["wav"],
            vec![audio_preset_ogg_q10()],
        ),
        (
            "Reverse: FLAC to WAV",
            vec!["flac"],
            vec![audio_preset_wav_from_flac(), audio_preset_wav_ffmpeg()],
        ),
    ];

    info!("Available audio modes:");
    for (i, (name, _, _)) in modes.iter().enumerate() {
        info!("  {}: {}", i, name);
    }

    print!("Select mode (number, or space-separated for multiple): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    let selections: Vec<usize> = input
        .split_whitespace()
        .filter_map(|s| s.parse::<usize>().ok())
        .filter(|i| *i < modes.len())
        .collect();

    if selections.is_empty() {
        info!("No valid selection, aborting.");
        return Ok(());
    }

    // Build combined presets list from selections
    let mut combined_exts: Vec<&str> = Vec::new();
    let mut combined_presets: Vec<AudioPreset> = Vec::new();
    for &idx in &selections {
        let (_, exts, presets) = &modes[idx];
        combined_exts.extend(exts.iter().copied());
        combined_presets.extend(presets.iter().cloned());
    }

    // Process each work directory
    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let bms_dir = entry.path();
        if !bms_dir.is_dir() {
            continue;
        }

        let bms_dir_name = bms_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        info!("Processing: {}", bms_dir_name);

        transfer_audio_by_format_in_dir(
            &bms_dir,
            &combined_exts,
            &combined_presets,
            &TransferOptions {
                remove_origin_on_success: true,
                remove_origin_on_failed: true,
                remove_existing_target_file: true,
                stop_on_error: false,
            },
        )
        .await?;
    }

    Ok(())
}

/// Transfer video files in a BMS root directory
///
/// This is an interactive function that prompts the user for settings
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
///
/// # Panics
///
/// Panics if stdout flush fails.
pub async fn transfer_video(root_dir: &Path) -> Result<(), std::io::Error> {
    use std::io::{self, Write};

    info!("Video Transfer for: {:?}", root_dir);

    // Video presets: (name, preset)
    let presets: [(&str, VideoPreset); 6] = [
        ("MP4 -> AVI 512x512", VIDEO_PRESET_AVI_512X512.clone()),
        ("MP4 -> AVI 480p", VIDEO_PRESET_AVI_480P.clone()),
        ("MP4 -> WMV2 512x512", VIDEO_PRESET_WMV2_512X512.clone()),
        ("MP4 -> WMV2 480p", VIDEO_PRESET_WMV2_480P.clone()),
        (
            "MP4 -> MPEG1VIDEO 512x512",
            VIDEO_PRESET_MPEG1VIDEO_512X512.clone(),
        ),
        (
            "MP4 -> MPEG1VIDEO 480p",
            VIDEO_PRESET_MPEG1VIDEO_480P.clone(),
        ),
    ];

    info!("Available video modes:");
    for (i, (name, _)) in presets.iter().enumerate() {
        info!("  {}: {}", i, name);
    }

    print!("Select modes (space-separated numbers): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    let selections: Vec<usize> = input
        .split_whitespace()
        .filter_map(|s| s.parse::<usize>().ok())
        .filter(|i| *i < presets.len())
        .collect();

    if selections.is_empty() {
        info!("No valid selection, aborting.");
        return Ok(());
    }

    // Build combined presets list
    let mut combined_presets: Vec<VideoPreset> = Vec::new();
    for &idx in &selections {
        let (_, preset) = &presets[idx];
        combined_presets.push(preset.clone());
    }

    // Process each work directory
    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let bms_dir = entry.path();
        if !bms_dir.is_dir() {
            continue;
        }

        let bms_dir_name = bms_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        info!("Processing: {}", bms_dir_name);

        transfer_video_by_format_in_dir(&bms_dir, &["mp4"], &combined_presets, true, true).await?;
    }

    Ok(())
}
