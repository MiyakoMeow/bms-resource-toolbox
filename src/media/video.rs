//! Video conversion presets and processing.
//!
//! This module provides video conversion presets for formats
//! like AVI, WMV, and MPEG using ffmpeg.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;
use tokio::process::Command;
use tracing::info;

/// Video information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VideoInfo {
    /// Video width in pixels.
    pub width: u32,
    /// Video height in pixels.
    pub height: u32,
    /// Video bit rate in bits per second.
    pub bit_rate: u64,
}

/// Video conversion preset matching Python's `VideoPreset` field model.
#[derive(Debug, Clone)]
pub struct VideoPreset {
    /// Name of the preset
    #[allow(dead_code)]
    pub name: String,
    /// Executable name (e.g. "ffmpeg")
    #[allow(dead_code)]
    pub exec: String,
    /// Input argument (e.g. "`-hide_banner` -i")
    pub input_arg: String,
    /// Filter argument (e.g. `FLITER_512X512`)
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

/// Filter complex for 480p with boxblur overlay
pub const FLITER_480P: &str = "-filter_complex \"[0:v]scale=640:480:force_original_aspect_ratio=increase,crop=640:480:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=640:480:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]\" -map [vid]";

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

/// Video preset for AVI encoding at 480p.
#[must_use]
pub fn video_preset_avi_480p() -> VideoPreset {
    VideoPreset::new(
        "AVI_480P",
        "ffmpeg",
        "-hide_banner -i",
        FLITER_480P,
        "avi",
        "mpeg4",
        "-an -q:v 8",
    )
}

/// Video preset for WMV2 encoding at 480p.
#[must_use]
pub fn video_preset_wmv2_480p() -> VideoPreset {
    VideoPreset::new(
        "WMV2_480P",
        "ffmpeg",
        "-hide_banner -i",
        FLITER_480P,
        "wmv",
        "wmv2",
        "-an -q:v 8",
    )
}

/// Video preset for MPEG1 encoding at 480p.
#[must_use]
pub fn video_preset_mpeg1video_480p() -> VideoPreset {
    VideoPreset::new(
        "MPEG1VIDEO_480P",
        "ffmpeg",
        "-hide_banner -i",
        FLITER_480P,
        "mpg",
        "mpeg1video",
        "-an -b:v 1500k",
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
/// Lazy static for AVI 480p video preset.
pub static VIDEO_PRESET_AVI_480P: LazyLock<VideoPreset> = LazyLock::new(video_preset_avi_480p);
/// Lazy static for WMV2 480p video preset.
pub static VIDEO_PRESET_WMV2_480P: LazyLock<VideoPreset> = LazyLock::new(video_preset_wmv2_480p);
/// Lazy static for MPEG1 480p video preset.
pub static VIDEO_PRESET_MPEG1VIDEO_480P: LazyLock<VideoPreset> =
    LazyLock::new(video_preset_mpeg1video_480p);

/// Get video info from file using ffprobe
#[must_use]
#[allow(dead_code)]
pub fn get_video_info(file_path: &Path) -> Option<VideoInfo> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-show_format",
            "-show_streams",
            "-print_format",
            "json",
            "-v",
            "quiet",
            &file_path.to_string_lossy(),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    for stream in json["streams"].as_array()? {
        if stream["codec_type"] == "video" {
            let width = u32::try_from(stream["width"].as_u64()?).ok()?;
            let height = u32::try_from(stream["height"].as_u64()?).ok()?;
            let bit_rate = stream["bit_rate"].as_str()?.parse().unwrap_or(0);
            return Some(VideoInfo {
                width,
                height,
                bit_rate,
            });
        }
    }

    None
}

/// Get video size (width, height) from file using ffprobe
#[must_use]
#[allow(dead_code)]
pub fn get_video_size(file_path: &Path) -> Option<(u32, u32)> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-show_format",
            "-show_streams",
            "-print_format",
            "json",
            "-v",
            "quiet",
            &file_path.to_string_lossy(),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    for stream in json["streams"].as_array()? {
        if stream["codec_type"] == "video" {
            let width = u32::try_from(stream["width"].as_u64()?).ok()?;
            let height = u32::try_from(stream["height"].as_u64()?).ok()?;
            return Some((width, height));
        }
    }

    None
}

/// Get preferred preset list based on video aspect ratio
#[must_use]
#[allow(dead_code)]
pub fn get_prefered_preset_list(file_path: &Path) -> Vec<VideoPreset> {
    let Some((width, height)) = get_video_size(file_path) else {
        return Vec::new();
    };

    if f64::from(width) / f64::from(height) > 640.0 / 480.0 {
        vec![
            video_preset_mpeg1video_480p(),
            video_preset_wmv2_480p(),
            video_preset_avi_480p(),
        ]
    } else {
        vec![
            video_preset_mpeg1video_512x512(),
            video_preset_wmv2_512x512(),
            video_preset_avi_512x512(),
        ]
    }
}

/// Get video presets array.
#[must_use]
#[allow(dead_code)]
pub fn video_presets() -> [VideoPreset; 3] {
    [
        video_preset_mpeg1video_512x512(),
        video_preset_wmv2_512x512(),
        video_preset_avi_512x512(),
    ]
}

/// Convert video file using preset
#[allow(dead_code)]
pub async fn convert_video(
    input: &Path,
    output: &Path,
    preset: &VideoPreset,
) -> Result<String, std::io::Error> {
    let cmd_str = preset.get_video_process_cmd(input, output);
    info!("Running: {}", cmd_str);

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
        .await?;

    if result.status.success() {
        Ok(cmd_str)
    } else {
        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        let stderr = String::from_utf8_lossy(&result.stderr).to_string();
        Err(std::io::Error::other(format!(
            "Conversion failed\nCmd: {cmd_str}\nStdout: {stdout}\nStderr: {stderr}"
        )))
    }
}

/// Transfer video files in directory using presets (with fallback).
/// Matches Python's `process_video_in_dir` behavior:
/// - For each file matching `input_exts`, try each preset in order
/// - If conversion succeeds: delete original (if `remove_origin_file`), break
/// - If conversion fails: delete failed output, try next preset
/// - Only report error when last preset fails
/// - When `use_prefered` is true, adds aspect-ratio-based presets before user presets
/// - Uses FIFO ordering (`VecDeque`) for handle completion
/// - Propagates errors from handles
pub async fn transfer_video_by_format_in_dir(
    dir: &Path,
    input_exts: &[&str],
    presets: &[VideoPreset],
    remove_origin_file: bool,
    remove_existing_target_file: bool,
    use_prefered: bool,
) -> Result<(), std::io::Error> {
    let cpu_count = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);

    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
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

    info!("Found {} video files to convert in {:?}", files.len(), dir);

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
        let use_prefered_clone = use_prefered;
        let handle = tokio::spawn(async move {
            let mut last_error = false;
            let mut last_err_msg = String::new();

            let mut presets_for_file = presets_clone;
            if use_prefered_clone {
                let mut prefered = get_prefered_preset_list(&file_path);
                prefered.extend(presets_for_file);
                presets_for_file = prefered;
            }

            for (i, preset) in presets_for_file.iter().enumerate() {
                let output = preset.get_output_file_path(&file_path);

                if file_path == output {
                    break;
                }

                if output.is_file() {
                    if remove_existing_target_file {
                        // Intentionally ignored: target removal before re-conversion
                        let _ = std::fs::remove_file(&output);
                    } else {
                        info!("File exists: {:?}", output);
                        continue;
                    }
                }

                let result = convert_video(&file_path, &output, preset).await;

                if result.is_ok() {
                    if remove_origin_file && file_path.is_file() {
                        // Intentionally ignored: origin removal after successful conversion
                        let _ = std::fs::remove_file(&file_path);
                    }
                    break;
                }
                if output.is_file() {
                    // Intentionally ignored: cleanup of failed conversion output
                    let _ = std::fs::remove_file(&output);
                }
                if i == presets_for_file.len() - 1 {
                    last_error = true;
                    last_err_msg = result.unwrap_err().to_string();
                }
            }

            if last_error {
                info!("Has Error!");
                info!("{}", last_err_msg);
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

/// Check video tool availability (ffprobe)
#[allow(dead_code)]
pub async fn check_ffprobe() -> bool {
    let output = Command::new("ffprobe")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
    output.is_ok_and(|s| s.success())
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
    fn test_video_preset_wmv() {
        let preset = VIDEO_PRESET_WMV2_480P.clone();
        assert_eq!(preset.name, "WMV2_480P");
        assert_eq!(preset.output_file_ext, "wmv");
        assert_eq!(preset.output_codec, "wmv2");
    }

    #[test]
    fn test_output_file_path() {
        let preset = video_preset_avi_512x512();
        let input = Path::new("/some/dir/video.mp4");
        let output = preset.get_output_file_path(input);
        assert_eq!(output, PathBuf::from("/some/dir/video.avi"));
    }
}
