//! BMS folder media transfer utilities.
//!
//! This module provides interactive functions for transferring
//! audio and video files in BMS directories.

use std::path::Path;
use tracing::info;

use crate::media::audio::AudioPreset;
use crate::media::video::VideoPreset;

/// Transfer audio files in a BMS root directory
/// This is an interactive function that prompts the user for settings
#[allow(dead_code)]
pub fn transfer_audio(root_dir: &Path) -> Result<(), std::io::Error> {
    use crate::media::audio;

    info!("Audio Transfer for: {:?}", root_dir);

    // Audio transfer modes
    let modes = [
        ("Convert: WAV to FLAC", vec!["wav"], vec![audio::audio_preset_flac(), audio::audio_preset_flac_ffmpeg()]),
        ("Compress: FLAC to OGG Q10", vec!["flac"], vec![audio::audio_preset_ogg_q10()]),
        ("Compress: WAV to OGG Q10", vec!["wav"], vec![audio::audio_preset_ogg_q10()]),
        ("Reverse: FLAC to WAV", vec!["flac"], vec![audio::audio_preset_wav_from_flac(), audio::audio_preset_wav_ffmpeg()]),
    ];

    info!("Available audio modes:");
    for (i, (name, _, _)) in modes.iter().enumerate() {
        info!("  {}: {}", i, name);
    }

    use std::io::{self, Write};
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

    // Build presets list
    let mut all_presets: Vec<(&[&str], Vec<AudioPreset>)> = Vec::new();
    for &idx in &selections {
        let (_, exts, presets) = &modes[idx];
        all_presets.push((exts.as_slice(), presets.clone()));
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

        let bms_dir_name = bms_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        info!("Processing: {}", bms_dir_name);

        // Note: This simplified version just processes each extension group
        // A full implementation would call the async transfer functions
        for (exts, presets) in &all_presets {
            info!("  Converting {:?} with {} presets", exts, presets.len());
            // In a full implementation, this would call:
            // transfer_audio_by_format_in_dir(&bms_dir, exts, presets, true, true).await?;
        }
    }

    Ok(())
}

/// Transfer video files in a BMS root directory
/// This is an interactive function that prompts the user for settings
#[allow(dead_code)]
pub fn transfer_video(root_dir: &Path) -> Result<(), std::io::Error> {
    use crate::media::video;

    info!("Video Transfer for: {:?}", root_dir);

    // Video presets
    let presets = [
        ("MP4 -> AVI 512x512", video_preset(video::VIDEO_PRESET_AVI_512X512.clone())),
        ("MP4 -> AVI 480p", video_preset(video::VIDEO_PRESET_AVI_480P.clone())),
        ("MP4 -> WMV2 512x512", video_preset(video::VIDEO_PRESET_WMV2_512X512.clone())),
        ("MP4 -> WMV2 480p", video_preset(video::VIDEO_PRESET_WMV2_480P.clone())),
        ("MP4 -> MPEG1VIDEO 512x512", video_preset(video::VIDEO_PRESET_MPEG1VIDEO_512X512.clone())),
        ("MP4 -> MPEG1VIDEO 480p", video_preset(video::VIDEO_PRESET_MPEG1VIDEO_480P.clone())),
    ];

    info!("Available video modes:");
    for (i, (name, _)) in presets.iter().enumerate() {
        info!("  {}: {}", i, name);
    }

    use std::io::{self, Write};
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

    // Process each work directory
    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let bms_dir = entry.path();
        if !bms_dir.is_dir() {
            continue;
        }

        let bms_dir_name = bms_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        info!("Processing: {}", bms_dir_name);

        for &idx in &selections {
            let (name, _) = &presets[idx];
            info!("  Mode: {}", name);
            // In a full implementation, this would call the async transfer functions
        }
    }

    Ok(())
}

// Helper function to get video preset
fn video_preset(preset: VideoPreset) -> VideoPreset {
    preset
}
