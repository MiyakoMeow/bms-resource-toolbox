//! Video conversion presets and processing.
//!
//! This module provides video conversion presets for formats
//! like AVI, WMV, and MPEG using ffmpeg.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;
use tokio::process::Command;

/// Video conversion preset matching Python's `VideoPreset` field model.
#[derive(Debug, Clone)]
pub struct VideoPreset {
    /// Name of the preset
    #[allow(dead_code)]
    pub name: String,
    /// Executable name (e.g. "ffmpeg")
    pub exec: String,
    /// Input argument (e.g. "`-hide_banner` -i")
    pub input_arg: String,
    /// Filter argument (e.g. `FLITER_512X512`)
    pub filter_arg: String,
    /// Output file extension (e.g. "avi")
    pub output_file_ext: String,
    /// Output codec (e.g. "mpeg4")
    pub output_codec: String,
    /// Additional arguments (e.g. "-an -q:v 8")
    pub arg: String,
}

impl VideoPreset {
    /// Create a new video preset.
    #[must_use]
    pub fn new(
        name: &str,
        exec: &str,
        input_arg: &str,
        filter_arg: &str,
        output_file_ext: &str,
        output_codec: &str,
        arg: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            exec: exec.to_string(),
            input_arg: input_arg.to_string(),
            filter_arg: filter_arg.to_string(),
            output_file_ext: output_file_ext.to_string(),
            output_codec: output_codec.to_string(),
            arg: arg.to_string(),
        }
    }

    /// Get output file path by replacing extension
    #[must_use]
    pub fn get_output_file_path(&self, input_file_path: &Path) -> PathBuf {
        let stem = input_file_path.file_stem().unwrap_or_default();
        input_file_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!(
                "{}.{}",
                stem.to_string_lossy(),
                self.output_file_ext
            ))
    }

    /// Get ffmpeg command string matching Python's `get_video_process_cmd` exactly.
    #[must_use]
    pub fn get_video_process_cmd(&self, input_file_path: &Path, output_file_path: &Path) -> String {
        let input = input_file_path.to_string_lossy();
        let output = output_file_path.to_string_lossy();
        let inner_arg = if self.exec == "ffmpeg" {
            "-map_metadata 0"
        } else {
            ""
        };
        format!(
            "{} {} \"{}\" {} {} -c:v {} {} \"{}\"",
            self.exec,
            self.input_arg,
            input,
            self.filter_arg,
            inner_arg,
            self.output_codec,
            self.arg,
            output
        )
    }
}

/// Filter complex for 512x512 with boxblur overlay
pub const FLITER_512X512: &str = "-filter_complex \"[0:v]scale=512:512:force_original_aspect_ratio=increase,crop=512:512:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=512:512:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]\" -map [vid]";

/// Video preset for AVI encoding at 512x512.
#[must_use]
pub fn video_preset_avi_512x512() -> VideoPreset {
    VideoPreset::new(
        "AVI_512X512",
        "ffmpeg",
        "-hide_banner -i",
        FLITER_512X512,
        "avi",
        "mpeg4",
        "-an -q:v 8",
    )
}

/// Video preset for MPEG1 encoding at 512x512.
#[must_use]
pub fn video_preset_mpeg1video_512x512() -> VideoPreset {
    VideoPreset::new(
        "MPEG1VIDEO_512X512",
        "ffmpeg",
        "-hide_banner -i",
        FLITER_512X512,
        "mpg",
        "mpeg1video",
        "-an -b:v 1500k",
    )
}

/// Video preset for WMV2 encoding at 512x512.
#[must_use]
pub fn video_preset_wmv2_512x512() -> VideoPreset {
    VideoPreset::new(
        "WMV2_512X512",
        "ffmpeg",
        "-hide_banner -i",
        FLITER_512X512,
        "wmv",
        "wmv2",
        "-an -q:v 8",
    )
}

/// Lazy static for AVI 512x512 video preset.
pub static VIDEO_PRESET_AVI_512X512: LazyLock<VideoPreset> =
    LazyLock::new(video_preset_avi_512x512);
/// Lazy static for MPEG1 512x512 video preset.
pub static VIDEO_PRESET_MPEG1VIDEO_512X512: LazyLock<VideoPreset> =
    LazyLock::new(video_preset_mpeg1video_512x512);
/// Lazy static for WMV2 512x512 video preset.
pub static VIDEO_PRESET_WMV2_512X512: LazyLock<VideoPreset> =
    LazyLock::new(video_preset_wmv2_512x512);

/// Transfer video files in directory using presets (with fallback).
/// Matches Python's `process_video_in_dir` behavior:
/// - For each file matching `input_exts`, try each preset in order
/// - If conversion succeeds: delete original (if `remove_origin_file`), break
/// - If conversion fails: delete failed output, try next preset
/// - Only report error when last preset fails
/// - Uses FIFO ordering (`VecDeque`) for handle completion
/// - Propagates errors from handles
///
/// # Errors
///
/// Returns `std::io::Error` if all presets fail for any file,
/// or if a spawned task panics.
///
/// # Panics
///
/// May panic if a spawned task panics, which propagates through
/// the `JoinHandle`.
#[allow(clippy::too_many_lines)]
pub async fn transfer_video_by_format_in_dir(
    dir: &Path,
    input_exts: &[&str],
    presets: &[VideoPreset],
    remove_origin_file: bool,
    remove_existing_target_file: bool,
    _use_prefered: bool,
) -> Result<(), std::io::Error> {
    let cpu_count = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);

    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
                && input_exts
                    .iter()
                    .any(|e| e.to_lowercase() == ext.to_string_lossy().to_lowercase())
            {
                files.push(path);
            }
        }
    }

    println!("Found {} video files to convert in {:?}", files.len(), dir);

    if files.is_empty() {
        return Ok(());
    }

    let mut handles: std::collections::VecDeque<
        tokio::task::JoinHandle<Result<(), std::io::Error>>,
    > = std::collections::VecDeque::new();

    for file_path in files {
        while handles.len() >= cpu_count {
            if let Some(handle) = handles.pop_front() {
                handle.await??;
            }
        }

        let presets_clone = presets.to_vec();
        let handle = tokio::spawn(async move {
            let mut last_error = false;
            let mut last_err_msg = String::new();

            let presets_for_file = presets_clone;

            for (i, preset) in presets_for_file.iter().enumerate() {
                let output = preset.get_output_file_path(&file_path);

                if file_path == output {
                    break;
                }

                if output.is_file() {
                    if remove_existing_target_file {
                        let _ = tokio::fs::remove_file(&output).await;
                    } else {
                        println!("File exists: {output:?}");
                        continue;
                    }
                }

                let cmd_str = preset.get_video_process_cmd(&file_path, &output);
                tracing::info!("Running: {}", cmd_str);

                let (shell, shell_arg) = if std::env::consts::OS == "windows" {
                    ("cmd", "/C")
                } else {
                    ("sh", "-c")
                };

                let result = Command::new(shell)
                    .args([shell_arg, &cmd_str])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await;

                match result {
                    Ok(output_result) if output_result.status.success() => {
                        if remove_origin_file && file_path.is_file() {
                            let _ = tokio::fs::remove_file(&file_path).await;
                        }
                        break;
                    }
                    Ok(output_result) => {
                        let stdout = String::from_utf8_lossy(&output_result.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output_result.stderr).to_string();
                        if output.is_file() {
                            let _ = tokio::fs::remove_file(&output).await;
                        }
                        if i == presets_for_file.len() - 1 {
                            last_error = true;
                            last_err_msg = format!(
                                "Conversion failed\nCmd: {cmd_str}\nStdout: {stdout}\nStderr: {stderr}"
                            );
                        }
                    }
                    Err(e) => {
                        if output.is_file() {
                            let _ = tokio::fs::remove_file(&output).await;
                        }
                        if i == presets_for_file.len() - 1 {
                            last_error = true;
                            last_err_msg = format!("Conversion failed: {e}");
                        }
                    }
                }
            }

            if last_error {
                println!("Has Error!");
                println!("{last_err_msg}");
                Err(std::io::Error::other(format!(
                    "All presets failed for {}",
                    file_path.display()
                )))
            } else {
                Ok(())
            }
        });
        handles.push_back(handle);
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_preset_cmd_matches_python() {
        let preset = video_preset_avi_512x512();
        let input = Path::new("/path/to/input.mp4");
        let output = Path::new("/path/to/output.avi");
        let cmd = preset.get_video_process_cmd(input, output);
        assert!(cmd.contains("ffmpeg -hide_banner -i"));
        assert!(cmd.contains("input.mp4"));
        assert!(cmd.contains("output.avi"));
        assert!(cmd.contains("-c:v mpeg4"));
        assert!(cmd.contains("-an -q:v 8"));
        assert!(cmd.contains("-map_metadata 0"));
        assert!(cmd.contains("-filter_complex"));
        assert!(!cmd.contains("-vf"));
    }

    #[test]
    fn test_video_preset_avi() {
        let preset = VIDEO_PRESET_AVI_512X512.clone();
        assert_eq!(preset.name, "AVI_512X512");
        assert_eq!(preset.output_file_ext, "avi");
        assert_eq!(preset.output_codec, "mpeg4");
    }

    #[test]
    fn test_output_file_path() {
        let preset = video_preset_avi_512x512();
        let input = Path::new("/some/dir/video.mp4");
        let output = preset.get_output_file_path(input);
        assert_eq!(output, PathBuf::from("/some/dir/video.avi"));
    }
}
