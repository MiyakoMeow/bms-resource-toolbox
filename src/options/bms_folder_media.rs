//! BMS folder media transfer utilities.
//!
//! This module provides interactive functions for transferring
//! audio and video files in BMS directories.

use std::path::Path;

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

    println!("Audio Transfer for: {root_dir:?}");

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
    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let bms_dir = entry.path();
        if !bms_dir.is_dir() {
            continue;
        }

        let bms_dir_name = bms_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        println!("Processing: {bms_dir_name}");

        transfer_audio_by_format_in_dir(
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

    println!("Video Transfer for: {root_dir:?}");

    // Video presets: (name, preset)
    let presets: [(&str, VideoPreset); 6] = [
        ("MP4 -> AVI 512x512", VIDEO_PRESET_AVI_512X512.clone()),
        ("MP4 -> WMV2 512x512", VIDEO_PRESET_WMV2_512X512.clone()),
        (
            "MP4 -> MPEG1VIDEO 512x512",
            VIDEO_PRESET_MPEG1VIDEO_512X512.clone(),
        ),
        ("MP4 -> AVI 480p", VIDEO_PRESET_AVI_480P.clone()),
        ("MP4 -> WMV2 480p", VIDEO_PRESET_WMV2_480P.clone()),
        (
            "MP4 -> MPEG1VIDEO 480p",
            VIDEO_PRESET_MPEG1VIDEO_480P.clone(),
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
    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let bms_dir = entry.path();
        if !bms_dir.is_dir() {
            continue;
        }

        let bms_dir_name = bms_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        println!("Processing: {bms_dir_name}");

        transfer_video_by_format_in_dir(
            &bms_dir,
            &["mp4"],
            std::slice::from_ref(&preset),
            true,
            true,
            false,
        )
        .await?;
    }

    Ok(())
}
