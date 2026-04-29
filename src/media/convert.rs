//! Media conversion utilities.
//!
//! This module provides async conversion functions for
//! audio and video files using external tools.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::media::audio::{AudioPreset, get_audio_process_cmd};
use crate::media::video::{VideoPreset, get_video_process_cmd};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

/// Convert audio file using preset
pub async fn convert_audio(
    input: &Path,
    output: &Path,
    preset: &AudioPreset,
) -> Result<(), std::io::Error> {
    let cmd_str = get_audio_process_cmd(input, output, preset);
    if cmd_str.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unknown exec: {}", preset.exec),
        ));
    }

    info!("Running: {}", cmd_str);
    let output = Command::new("cmd")
        .args(["/C", &cmd_str])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await?;

    if output.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("Conversion failed"))
    }
}

/// Convert video file using preset
#[allow(dead_code)]
pub async fn convert_video(
    input: &Path,
    output: &Path,
    preset: &VideoPreset,
) -> Result<(), std::io::Error> {
    let cmd_str = get_video_process_cmd(input, output, preset);
    info!("Running: {}", cmd_str);

    let output = Command::new("cmd")
        .args(["/C", &cmd_str])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await?;

    if output.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("Conversion failed"))
    }
}

/// Transfer audio files in directory using presets (with fallback)
pub async fn transfer_audio_by_format_in_dir(
    dir: &Path,
    input_exts: &[&str],
    presets: &[AudioPreset],
    _remove_origin_on_success: bool,
    _remove_origin_on_failed: bool,
) -> Result<(), std::io::Error> {
    let hdd = !dir.to_string_lossy().starts_with("C:");

    let max_workers = if hdd { 4 } else { 8 };

    // Find files matching input extensions
    let mut tasks: Vec<(PathBuf, PathBuf, usize)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
            {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if input_exts.iter().any(|e| e.to_lowercase() == ext_str) {
                    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                    let output_ext = &presets[0].output_format;
                    let output = path.parent().unwrap().join(format!("{stem}.{output_ext}"));
                    tasks.push((path, output, 0));
                }
            }
        }
    }

    info!("Found {} files to convert in {:?}", tasks.len(), dir);

    // Process with bounded concurrency
    let mut handles = Vec::new();
    for (input, output, preset_idx) in tasks {
        if handles.len() >= max_workers {
            // Wait for one to complete
            if let Some(res) = handles.pop() {
                let _ = res.await;
            }
        }

        let preset = &presets[preset_idx];
        let input_clone = input.clone();
        let output_clone = output.clone();
        let preset_clone = preset.clone();
        let _presets_clone = presets.to_vec();

        let handle = tokio::spawn(async move {
            let result = convert_audio(&input_clone, &output_clone, &preset_clone).await;
            (input_clone, preset_idx, result)
        });
        handles.push(handle);
    }

    // Wait for remaining
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
