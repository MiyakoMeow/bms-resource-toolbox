//! BMS folder media transfer utilities.
//!
//! This module provides interactive functions for transferring
//! audio and video files in BMS directories.

use std::path::Path;

use crate::media::audio::{
    AUDIO_PRESET_FLAC, AUDIO_PRESET_FLAC_FFMPEG, AUDIO_PRESET_OGG_Q10, AUDIO_PRESET_WAV_FFMPEG,
    AUDIO_PRESET_WAV_FROM_FLAC, AudioPreset,
};
use crate::media::video::{
    VIDEO_PRESET_AVI_512X512, VIDEO_PRESET_MPEG1VIDEO_512X512, VIDEO_PRESET_WMV2_512X512,
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

    println!("Audio Transfer for: {root_dir:?}");

    // Audio transfer modes: (name, input_exts, presets)
    let modes: [(&str, Vec<&str>, Vec<AudioPreset>); 4] = [
        (
            "Convert: WAV to FLAC",
            vec!["wav"],
            vec![AUDIO_PRESET_FLAC.clone(), AUDIO_PRESET_FLAC_FFMPEG.clone()],
        ),
        (
            "Compress: FLAC to OGG Q10",
            vec!["flac"],
            vec![AUDIO_PRESET_OGG_Q10.clone()],
        ),
        (
            "Compress: WAV to OGG Q10",
            vec!["wav"],
            vec![AUDIO_PRESET_OGG_Q10.clone()],
        ),
        (
            "Reverse: FLAC to WAV",
            vec!["flac"],
            vec![
                AUDIO_PRESET_WAV_FROM_FLAC.clone(),
                AUDIO_PRESET_WAV_FFMPEG.clone(),
            ],
        ),
    ];

    println!("Available audio modes:");
    for (i, (name, _, _)) in modes.iter().enumerate() {
        println!("  {i}: {name}");
    }

    let max_index = modes.len() - 1;
    print!("输入数字选择目标格式（0-{max_index}）：");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let selection = input.trim().parse::<usize>();

    let idx = match selection {
        Ok(i) if i < modes.len() => i,
        _ => {
            println!("No valid selection, aborting.");
            return Ok(());
        }
    };

    let (_, exts, presets) = &modes[idx];
    let combined_exts: Vec<&str> = exts.clone();
    let combined_presets: Vec<AudioPreset> = presets.clone();

    // Process each work directory
    let mut read_dir = tokio::fs::read_dir(root_dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let bms_dir = entry.path();
        if !bms_dir.is_dir() {
            continue;
        }

        let bms_dir_name = bms_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        println!("Processing: {bms_dir_name}");

        let _ = transfer_audio_by_format_in_dir(
            &bms_dir,
            &combined_exts,
            &combined_presets,
            &TransferOptions {
                remove_origin_on_success: true,
                remove_origin_on_failed: false,
                remove_existing_target_file: true,
                stop_on_error: true,
            },
        )
        .await;
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

    println!("Video Transfer for: {root_dir:?}");

    // Video presets: (name, preset)
    let presets: [(&str, VideoPreset); 3] = [
        ("MP4 -> AVI 512x512", VIDEO_PRESET_AVI_512X512.clone()),
        ("MP4 -> WMV2 512x512", VIDEO_PRESET_WMV2_512X512.clone()),
        (
            "MP4 -> MPEG1VIDEO 512x512",
            VIDEO_PRESET_MPEG1VIDEO_512X512.clone(),
        ),
    ];

    println!("Available video modes:");
    for (i, (name, _)) in presets.iter().enumerate() {
        println!("  {i}: {name}");
    }

    let max_index = presets.len() - 1;
    print!("输入数字选择目标格式（0-{max_index}）：");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let selection = input.trim().parse::<usize>();

    let idx = match selection {
        Ok(i) if i < presets.len() => i,
        _ => {
            println!("No valid selection, aborting.");
            return Ok(());
        }
    };

    let preset = presets[idx].1.clone();

    // Process each work directory
    let mut read_dir = tokio::fs::read_dir(root_dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let bms_dir = entry.path();
        if !bms_dir.is_dir() {
            continue;
        }

        let bms_dir_name = bms_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        println!("Processing: {bms_dir_name}");

        transfer_video_by_format_in_dir(
            &bms_dir,
            &["mp4", "mkv", "avi", "wmv", "mpg", "mpeg"],
            std::slice::from_ref(&preset),
            true,
            true,
            false,
        )
        .await?;
    }

    Ok(())
}
