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

/// Video conversion preset.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VideoPreset {
    /// Name of the preset
    pub name: String,
    /// Executable name (ffmpeg)
    pub exec: String,
    /// Output format (avi, mpg, wmv)
    pub output_format: String,
    /// Target width
    pub width: u32,
    /// Target height
    pub height: u32,
    /// Codec arguments
    pub codec_args: String,
}

impl VideoPreset {
    /// Create a new video preset.
    #[must_use]
    pub fn new(
        name: &str,
        exec: &str,
        output_format: &str,
        width: u32,
        height: u32,
        codec_args: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            exec: exec.to_string(),
            output_format: output_format.to_string(),
            width,
            height,
            codec_args: codec_args.to_string(),
        }
    }

    /// Get output file path by replacing extension
    #[allow(dead_code)]
    #[must_use]
    pub fn get_output_file_path(&self, input_file_path: &Path) -> PathBuf {
        let stem = input_file_path.file_stem().unwrap_or_default();
        input_file_path.parent().unwrap_or(Path::new(".")).join(format!(
            "{}.{}",
            stem.to_string_lossy(),
            self.output_format
        ))
    }

    /// Get ffmpeg filter complex for resizing (Python-style with boxblur overlay)
    #[must_use]
    pub fn filter_complex(&self) -> String {
        if self.width == 512 && self.height == 512 {
            FLITER_512X512.to_string()
        } else {
            FLITER_480P.to_string()
        }
    }

    /// Get ffmpeg command string
    #[allow(dead_code)]
    #[must_use]
    pub fn get_video_process_cmd(&self, input_file_path: &Path, output_file_path: &Path) -> String {
        let input = input_file_path.to_string_lossy();
        let output = output_file_path.to_string_lossy();
        let filter = self.filter_complex();

        format!(
            "ffmpeg -hide_banner -i \"{}\" {} -map_metadata 0 -c:v {} {} \"{}\"",
            input,
            filter,
            self.codec_args.split_whitespace().next().unwrap_or("mpeg4"),
            self.codec_args,
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
    VideoPreset::new("AVI_512X512", "ffmpeg", "avi", 512, 512, "-c:v mpeg4 -q:v 4")
}

/// Video preset for MPEG1 encoding at 512x512.
#[must_use]
pub fn video_preset_mpeg1video_512x512() -> VideoPreset {
    VideoPreset::new("MPEG1VIDEO_512X512", "ffmpeg", "mpg", 512, 512, "-c:v mpeg1video -b:v 2000k")
}

/// Video preset for WMV2 encoding at 512x512.
#[must_use]
pub fn video_preset_wmv2_512x512() -> VideoPreset {
    VideoPreset::new("WMV2_512X512", "ffmpeg", "wmv", 512, 512, "-c:v wmv2 -b:v 2000k")
}

/// Video preset for AVI encoding at 480p.
#[must_use]
pub fn video_preset_avi_480p() -> VideoPreset {
    VideoPreset::new("AVI_480P", "ffmpeg", "avi", 640, 480, "-c:v mpeg4 -q:v 4")
}

/// Video preset for WMV2 encoding at 480p.
#[must_use]
pub fn video_preset_wmv2_480p() -> VideoPreset {
    VideoPreset::new("WMV2_480P", "ffmpeg", "wmv", 640, 480, "-c:v wmv2 -b:v 2000k")
}

/// Video preset for MPEG1 encoding at 480p.
#[must_use]
pub fn video_preset_mpeg1video_480p() -> VideoPreset {
    VideoPreset::new("MPEG1VIDEO_480P", "ffmpeg", "mpg", 640, 480, "-c:v mpeg1video -b:v 1500k")
}

/// Lazy static for AVI 512x512 video preset.
pub static VIDEO_PRESET_AVI_512X512: LazyLock<VideoPreset> = LazyLock::new(video_preset_avi_512x512);
/// Lazy static for MPEG1 512x512 video preset.
pub static VIDEO_PRESET_MPEG1VIDEO_512X512: LazyLock<VideoPreset> = LazyLock::new(video_preset_mpeg1video_512x512);
/// Lazy static for WMV2 512x512 video preset.
pub static VIDEO_PRESET_WMV2_512X512: LazyLock<VideoPreset> = LazyLock::new(video_preset_wmv2_512x512);
/// Lazy static for AVI 480p video preset.
pub static VIDEO_PRESET_AVI_480P: LazyLock<VideoPreset> = LazyLock::new(video_preset_avi_480p);
/// Lazy static for WMV2 480p video preset.
pub static VIDEO_PRESET_WMV2_480P: LazyLock<VideoPreset> = LazyLock::new(video_preset_wmv2_480p);
/// Lazy static for MPEG1 480p video preset.
pub static VIDEO_PRESET_MPEG1VIDEO_480P: LazyLock<VideoPreset> = LazyLock::new(video_preset_mpeg1video_480p);

/// Get video info from file using ffprobe
#[allow(dead_code)]
#[must_use]
pub fn get_video_info(file_path: &Path) -> Option<VideoInfo> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-show_format",
            "-show_streams",
            "-print_format", "json",
            "-v", "quiet",
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
            return Some(VideoInfo { width, height, bit_rate });
        }
    }

    None
}

/// Get video size (width, height) from file using ffprobe
#[allow(dead_code)]
#[must_use]
pub fn get_video_size(file_path: &Path) -> Option<(u32, u32)> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-show_format",
            "-show_streams",
            "-print_format", "json",
            "-v", "quiet",
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
#[allow(dead_code)]
#[must_use]
pub fn get_prefered_preset_list(file_path: &Path) -> Vec<VideoPreset> {
    let Some((width, height)) = get_video_size(file_path) else {
        return Vec::new();
    };

    // If width/height > 640/480, use 480p presets
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
#[allow(dead_code)]
#[must_use]
pub fn video_presets() -> [VideoPreset; 3] {
    [
        video_preset_mpeg1video_512x512(),
        video_preset_wmv2_512x512(),
        video_preset_avi_512x512(),
    ]
}

/// Get video process command.
#[must_use] 
pub fn get_video_process_cmd(
    file_path: &Path,
    output_file_path: &Path,
    preset: &VideoPreset,
) -> String {
    let input = file_path.to_string_lossy();
    let output = output_file_path.to_string_lossy();
    let filter = preset.filter_complex();

    format!(
        "ffmpeg -hide_banner -loglevel panic -i \"{}\" -vf \"{}\" {} -c:a copy \"{}\"",
        input, filter, preset.codec_args, output
    )
}

/// Convert video file using preset
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
        Err(std::io::Error::other(
            "Conversion failed",
        ))
    }
}

/// Transfer video files in directory using presets (with fallback)
pub async fn transfer_video_by_format_in_dir(
    dir: &Path,
    input_exts: &[&str],
    presets: &[VideoPreset],
    _remove_origin_on_success: bool,
    _remove_origin_on_failed: bool,
) -> Result<(), std::io::Error> {
    let hdd = !dir.to_string_lossy().contains(":\\C\\");

    let max_workers = if hdd { 4 } else { 8 };

    // Find files matching input extensions
    let mut tasks: Vec<(PathBuf, PathBuf, usize)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension() {
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

    info!("Found {} video files to convert in {:?}", tasks.len(), dir);

    // Process with bounded concurrency
    let mut handles = Vec::new();
    for (input, output, preset_idx) in tasks {
        if handles.len() >= max_workers
            && let Some(res) = handles.pop() {
                let _ = res.await;
            }

        let preset = &presets[preset_idx];
        let input_clone = input.clone();
        let output_clone = output.clone();
        let preset_clone = preset.clone();

        let handle = tokio::spawn(async move {
            let result = convert_video(&input_clone, &output_clone, &preset_clone).await;
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

/// Check video tool availability (ffprobe)
#[allow(dead_code)]
pub async fn check_ffprobe() -> bool {
    let output = Command::new("ffprobe")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
    output.map(|s| s.success()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_preset_filter() {
        let preset = video_preset_mpeg1video_512x512();
        let filter = preset.filter_complex();
        assert!(filter.contains("512"));
        assert!(filter.contains("scale"));
    }

    #[test]
    fn test_get_video_process_cmd() {
        let input = Path::new("/path/to/input.mp4");
        let output = Path::new("/path/to/output.avi");
        let preset = video_preset_avi_512x512();
        let cmd = get_video_process_cmd(input, output, &preset);
        assert!(cmd.contains("ffmpeg"));
        assert!(cmd.contains("input.mp4"));
        assert!(cmd.contains("output.avi"));
    }
}
