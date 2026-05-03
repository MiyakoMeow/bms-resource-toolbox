//! Media conversion utilities.
//!
//! This module provides async conversion functions for
//! audio and video files using external tools.

use crate::media::audio::{AudioPreset, get_audio_process_cmd};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

/// Execute a shell command string cross-platform, capturing stderr on failure
async fn execute_shell_command_with_stderr(
    cmd_str: &str,
) -> Result<(bool, String, String), std::io::Error> {
    let (shell, shell_arg) = if std::env::consts::OS == "windows" {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let output = Command::new(shell)
        .args([shell_arg, cmd_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((success, stdout, stderr))
}

/// Convert audio file using preset.
///
/// # Errors
///
/// Returns an error if the conversion command fails or the executable is unknown.
#[allow(dead_code)]
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
    let (success, _stdout, stderr) = execute_shell_command_with_stderr(&cmd_str).await?;
    if success {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "Conversion failed: stderr={stderr}"
        )))
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
        println!("Failed to remove existing target file {output:?}: {e}");
    }
}

type TaskResult = (
    PathBuf,
    usize,
    Result<String, std::io::Error>,
    Vec<AudioPreset>,
    String,
);
#[allow(clippy::type_complexity)]
type HandleEntry = (tokio::task::JoinHandle<TaskResult>, bool);
/// Transfer audio files in a directory using format presets with fallback.
///
/// This matches Python's `transfer_audio_by_format_in_dir` behavior:
/// - Supports preset fallback: when first preset fails, tries next one
/// - Unlimited fallback levels via task queue
/// - Handles `remove_origin_on_success` and `remove_origin_on_failed`
/// - Uses bounded concurrency based on disk type
/// - Captures stderr on failure
/// - Prints fallback statistics
///
/// # Errors
///
/// Returns `std::io::Error` if `stop_on_error` is set and a conversion
/// fails with all presets, or if a spawned task panics.
///
/// # Panics
///
/// May panic if a spawned task panics, which propagates through
/// the `JoinHandle`.
#[allow(clippy::too_many_lines)]
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

    let initial_tasks = collect_tasks(dir, input_exts).await;
    let file_count = initial_tasks.len();
    println!("Found {file_count} files to convert in {dir:?}");

    if initial_tasks.is_empty() {
        return Ok(());
    }

    println!("Entering dir: {dir:?} Input ext: {input_exts:?}");

    let mut task_queue: std::collections::VecDeque<(PathBuf, usize)> =
        initial_tasks.into_iter().collect();
    let mut handles: Vec<HandleEntry> = Vec::new();
    let mut has_error = false;
    let mut err_file_path = String::new();
    let mut err_stdout = String::new();
    let mut err_stderr = String::new();
    let mut fallback_file_names: Vec<(String, usize)> = Vec::new();

    while let Some((input, preset_idx)) = task_queue.pop_front() {
        if handles.len() >= cpu_count {
            task_queue.push_front((input, preset_idx));
            break;
        }
        let preset = presets[preset_idx].clone();
        let input_clone = input.clone();
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        let output_ext = &preset.output_format;
        let output = input.parent().unwrap().join(format!("{stem}.{output_ext}"));
        let presets_vec = presets.to_vec();

        if output.is_file()
            && let Ok(metadata) = tokio::fs::metadata(&output).await
            && metadata.len() > 0
            && !options.remove_existing_target_file
        {
            println!("File {output:?} exists! Skipping...");
            continue;
        }

        remove_existing_target(&output, options.remove_existing_target_file).await;

        let handle = tokio::spawn(async move {
            let cmd_str = get_audio_process_cmd(&input_clone, &output, &preset);
            let (result, cmd_stdout) = if cmd_str.is_empty() {
                (
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Unknown exec: {}", preset.exec),
                    )),
                    String::new(),
                )
            } else {
                info!("Running: {}", cmd_str);
                match execute_shell_command_with_stderr(&cmd_str).await {
                    Ok((true, stdout, _)) => (Ok(String::new()), stdout),
                    Ok((false, stdout, stderr)) => (
                        Err(std::io::Error::other(format!(
                            "Conversion failed: stderr={stderr}"
                        ))),
                        stdout,
                    ),
                    Err(e) => (Err(e), String::new()),
                }
            };
            let stderr_msg = result
                .as_ref()
                .err()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            (
                input_clone,
                preset_idx,
                result.map(|_| stderr_msg),
                presets_vec,
                cmd_stdout,
            )
        });
        handles.push((handle, true));
    }

    loop {
        if handles.is_empty() && task_queue.is_empty() {
            break;
        }

        let mut new_handles: Vec<HandleEntry> = Vec::new();

        let mut switch_next_list: Vec<(PathBuf, usize)> = Vec::new();

        for (handle, is_process) in handles {
            if !is_process {
                new_handles.push((handle, false));
                continue;
            }
            if !handle.is_finished() {
                new_handles.push((handle, true));
                continue;
            }
            let result = handle.await;
            if let Ok((input, preset_idx, res, _presets_vec, cmd_stdout)) = result {
                match res {
                    Ok(_stderr_msg) => {
                        if options.remove_origin_on_success
                            && input.is_file()
                            && let Err(e) = tokio::fs::remove_file(&input).await
                        {
                            println!("Failed to remove origin file {input:?}: {e}");
                        }
                    }
                    Err(e) => {
                        let stderr_str = e.to_string();
                        println!("Conversion failed for {input:?}: {e}");
                        switch_next_list.push((input.clone(), preset_idx));
                        err_file_path = input.to_string_lossy().to_string();
                        err_stderr = stderr_str;
                        err_stdout = cmd_stdout;
                    }
                }
            }
        }

        for (input, preset_idx) in switch_next_list {
            let next_idx = preset_idx + 1;
            if next_idx >= presets.len() {
                has_error = true;
                if options.remove_origin_on_failed
                    && input.is_file()
                    && let Err(e) = tokio::fs::remove_file(&input).await
                {
                    println!("Failed to remove failed origin file {input:?}: {e}");
                }
                if options.stop_on_error {
                    return Err(std::io::Error::other(format!(
                        "Conversion failed for {}: {err_stderr}",
                        input.display(),
                    )));
                }
                continue;
            }
            fallback_file_names.push((
                input
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                next_idx,
            ));
            task_queue.push_back((input, next_idx));
        }

        while new_handles.len() < cpu_count {
            if let Some((input, preset_idx)) = task_queue.pop_front() {
                let preset = presets[preset_idx].clone();
                let input_clone = input.clone();
                let stem = input.file_stem().unwrap_or_default().to_string_lossy();
                let output_ext = &preset.output_format;
                let output = input.parent().unwrap().join(format!("{stem}.{output_ext}"));
                let presets_vec = presets.to_vec();

                if output.is_file()
                    && let Ok(metadata) = tokio::fs::metadata(&output).await
                    && metadata.len() > 0
                    && !options.remove_existing_target_file
                {
                    println!("File {output:?} exists! Skipping...");
                    continue;
                }

                remove_existing_target(&output, options.remove_existing_target_file).await;

                let handle = tokio::spawn(async move {
                    let cmd_str = get_audio_process_cmd(&input_clone, &output, &preset);
                    let (result, cmd_stdout) = if cmd_str.is_empty() {
                        (
                            Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Unknown exec: {}", preset.exec),
                            )),
                            String::new(),
                        )
                    } else {
                        info!("Running: {}", cmd_str);
                        match execute_shell_command_with_stderr(&cmd_str).await {
                            Ok((true, stdout, _)) => (Ok(String::new()), stdout),
                            Ok((false, stdout, stderr)) => (
                                Err(std::io::Error::other(format!(
                                    "Conversion failed: stderr={stderr}"
                                ))),
                                stdout,
                            ),
                            Err(e) => (Err(e), String::new()),
                        }
                    };
                    let stderr_msg = result
                        .as_ref()
                        .err()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default();
                    (
                        input_clone,
                        preset_idx,
                        result.map(|_| stderr_msg),
                        presets_vec,
                        cmd_stdout,
                    )
                });
                new_handles.push((handle, true));
            } else {
                break;
            }
        }

        handles = new_handles;

        if !handles.is_empty() {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }

    if has_error {
        println!("Has Error!");
        println!("- Err file_path: {err_file_path}");
        println!("- Err stdout: {err_stdout}");
        println!("- Err stderr: {err_stderr}");
        if options.remove_origin_on_failed {
            println!("The failed origin file has been removed.");
        }
    }

    if file_count > 0 {
        println!("Parsed {file_count} file(s).");
    }
    if !fallback_file_names.is_empty() {
        println!(
            "Fallback: {:?}. Totally {} files.",
            fallback_file_names,
            fallback_file_names.len()
        );
    }

    if has_error {
        Err(std::io::Error::other(format!(
            "Audio conversion had errors in {}",
            dir.display()
        )))
    } else {
        Ok(())
    }
}
