//! Media conversion utilities.
//!
//! This module provides async conversion functions for
//! audio and video files using external tools.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::media::audio::{AudioPreset, get_audio_process_cmd};
use crate::media::video::VideoPreset;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

/// Execute a shell command string cross-platform
async fn execute_shell_command(cmd_str: &str) -> Result<bool, std::io::Error> {
    let (shell, shell_arg) = if std::env::consts::OS == "windows" {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let status = Command::new(shell)
        .args([shell_arg, cmd_str])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await?;

    Ok(status.success())
}

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
    if execute_shell_command(&cmd_str).await? {
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
    let cmd_str = preset.get_video_process_cmd(input, output);
    info!("Running: {}", cmd_str);

    if execute_shell_command(&cmd_str).await? {
        Ok(())
    } else {
        Err(std::io::Error::other("Conversion failed"))
    }
}

/// Options for controlling audio transfer behavior.
#[allow(clippy::struct_excessive_bools)]
pub struct TransferOptions {
    /// Remove the original file after successful conversion.
    pub remove_origin_on_success: bool,
    /// Remove the original file when conversion fails.
    pub remove_origin_on_failed: bool,
    /// Remove existing target files before conversion.
    pub remove_existing_target_file: bool,
    /// Stop processing on the first error.
    pub stop_on_error: bool,
}

async fn collect_tasks(dir: &Path, input_exts: &[&str]) -> Vec<(PathBuf, usize)> {
    let mut tasks: Vec<(PathBuf, usize)> = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
                && input_exts
                    .iter()
                    .any(|e| e.to_lowercase() == ext.to_string_lossy().to_lowercase())
            {
                tasks.push((path, 0));
            }
        }
    }
    tasks
}

async fn remove_existing_target(output: &Path, remove: bool) {
    if remove
        && output.is_file()
        && let Err(e) = tokio::fs::remove_file(output).await
    {
        info!("Failed to remove existing target file {:?}: {}", output, e);
    }
}

/// Transfer audio files in directory using presets (with fallback)
///
/// This matches Python's `transfer_audio_by_format_in_dir` behavior:
/// - Supports preset fallback: when first preset fails, tries next one
/// - Handles `remove_origin_on_success` and `remove_origin_on_failed`
/// - Uses bounded concurrency based on disk type
pub async fn transfer_audio_by_format_in_dir(
    dir: &Path,
    input_exts: &[&str],
    presets: &[AudioPreset],
    options: &TransferOptions,
) -> Result<(), std::io::Error> {
    if presets.is_empty() {
        return Ok(());
    }

    let cpu_count = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);

    let tasks = collect_tasks(dir, input_exts).await;
    info!("Found {} files to convert in {:?}", tasks.len(), dir);

    if tasks.is_empty() {
        return Ok(());
    }

    let mut handles = Vec::new();
    let mut task_iter = tasks.into_iter();

    loop {
        while handles.len() < cpu_count {
            if let Some((input, preset_idx)) = task_iter.next() {
                let preset = presets[preset_idx].clone();
                let input_clone = input.clone();
                let stem = input.file_stem().unwrap_or_default().to_string_lossy();
                let output_ext = &preset.output_format;
                let output = input.parent().unwrap().join(format!("{stem}.{output_ext}"));
                let presets_vec = presets.to_vec();

                if output.is_file()
                    && let Ok(metadata) = std::fs::metadata(&output)
                    && metadata.len() > 0
                    && !options.remove_existing_target_file
                {
                    info!("File {:?} exists! Skipping...", output);
                    continue;
                }

                remove_existing_target(&output, options.remove_existing_target_file).await;

                let handle = tokio::spawn(async move {
                    let result = convert_audio(&input_clone, &output, &preset).await;
                    (input_clone, preset_idx, result, presets_vec)
                });
                handles.push(handle);
            } else {
                break;
            }
        }

        if handles.is_empty() {
            break;
        }

        let (result, _, remaining) = futures::future::select_all(handles).await;
        handles = remaining;

        if let Ok((input, preset_idx, result, presets_vec)) = result {
            match result {
                Ok(()) => {
                    if options.remove_origin_on_success
                        && input.is_file()
                        && let Err(e) = tokio::fs::remove_file(&input).await
                    {
                        info!("Failed to remove origin file {:?}: {}", input, e);
                    }
                }
                Err(e) => {
                    info!("Conversion failed for {:?}: {}", input, e);
                    let next_idx = preset_idx + 1;
                    if next_idx < presets_vec.len() {
                        info!("Falling back to preset {} for {:?}", next_idx, input);
                        let preset = presets_vec[next_idx].clone();
                        let input_clone = input.clone();
                        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
                        let output_ext = &preset.output_format;
                        let output = input.parent().unwrap().join(format!("{stem}.{output_ext}"));

                        if output.is_file()
                            && let Ok(metadata) = std::fs::metadata(&output)
                            && metadata.len() > 0
                            && !options.remove_existing_target_file
                        {
                            continue;
                        }

                        remove_existing_target(&output, options.remove_existing_target_file).await;

                        let presets_clone = presets_vec.clone();
                        let handle = tokio::spawn(async move {
                            let result = convert_audio(&input_clone, &output, &preset).await;
                            (input_clone, next_idx, result, presets_clone)
                        });
                        handles.push(handle);
                    } else {
                        if options.remove_origin_on_failed
                            && input.is_file()
                            && let Err(e) = tokio::fs::remove_file(&input).await
                        {
                            info!("Failed to remove failed origin file {:?}: {}", input, e);
                        }
                        if options.stop_on_error {
                            return Err(std::io::Error::other(format!(
                                "Conversion failed for {}: {e}",
                                input.display(),
                            )));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
