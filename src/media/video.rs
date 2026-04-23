use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::Result;

#[derive(Debug, Clone)]
pub struct VideoPreset {
    pub exec: String,
    pub input_arg: Option<String>,
    pub filter_arg: Option<String>,
    pub output_file_ext: String,
    pub output_codec: String,
    pub arg: Option<String>,
}

impl VideoPreset {
    #[must_use]
    pub fn get_output_file_path(&self, input_file_path: &Path) -> PathBuf {
        input_file_path.with_extension(&self.output_file_ext)
    }

    #[must_use]
    pub fn get_video_process_cmd(&self, input_file_path: &Path, output_file_path: &Path) -> String {
        let input_arg = self.input_arg.as_deref().unwrap_or("");
        let filter_arg = self.filter_arg.as_deref().unwrap_or("");
        let inner_arg = if self.exec == "ffmpeg" { "-map_metadata 0" } else { "" };
        let arg = self.arg.as_deref().unwrap_or("");
        let input = input_file_path.display();
        let output = output_file_path.display();
        format!(
            "{exec} {input_arg} \"{input}\" {filter_arg} {inner_arg} -c:v {codec} {arg} \"{output}\"",
            exec = self.exec,
            codec = self.output_codec
        )
    }
}

pub const FILTER_512X512: &str = "-filter_complex \"[0:v]scale=512:512:force_original_aspect_ratio=increase,crop=512:512:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=512:512:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]\" -map [vid]";
pub const FILTER_480P: &str = "-filter_complex \"[0:v]scale=640:480:force_original_aspect_ratio=increase,crop=640:480:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=640:480:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]\" -map [vid]";

pub static VIDEO_PRESET_AVI_512X512: LazyLock<VideoPreset> = LazyLock::new(|| VideoPreset {
    exec: "ffmpeg".into(),
    input_arg: Some("-hide_banner -i".into()),
    filter_arg: Some(FILTER_512X512.into()),
    output_file_ext: "avi".into(),
    output_codec: "mpeg4".into(),
    arg: Some("-an -q:v 8".into()),
});

pub static VIDEO_PRESET_WMV2_512X512: LazyLock<VideoPreset> = LazyLock::new(|| VideoPreset {
    exec: "ffmpeg".into(),
    input_arg: Some("-hide_banner -i".into()),
    filter_arg: Some(FILTER_512X512.into()),
    output_file_ext: "wmv".into(),
    output_codec: "wmv2".into(),
    arg: Some("-an -q:v 8".into()),
});

pub static VIDEO_PRESET_MPEG1VIDEO_512X512: LazyLock<VideoPreset> = LazyLock::new(|| VideoPreset {
    exec: "ffmpeg".into(),
    input_arg: Some("-hide_banner -i".into()),
    filter_arg: Some(FILTER_512X512.into()),
    output_file_ext: "mpg".into(),
    output_codec: "mpeg1video".into(),
    arg: Some("-an -b:v 1500k".into()),
});

pub static VIDEO_PRESET_AVI_480P: LazyLock<VideoPreset> = LazyLock::new(|| VideoPreset {
    exec: "ffmpeg".into(),
    input_arg: Some("-hide_banner -i".into()),
    filter_arg: Some(FILTER_480P.into()),
    output_file_ext: "avi".into(),
    output_codec: "mpeg4".into(),
    arg: Some("-an -q:v 8".into()),
});

pub static VIDEO_PRESET_WMV2_480P: LazyLock<VideoPreset> = LazyLock::new(|| VideoPreset {
    exec: "ffmpeg".into(),
    input_arg: Some("-hide_banner -i".into()),
    filter_arg: Some(FILTER_480P.into()),
    output_file_ext: "wmv".into(),
    output_codec: "wmv2".into(),
    arg: Some("-an -q:v 8".into()),
});

pub static VIDEO_PRESET_MPEG1VIDEO_480P: LazyLock<VideoPreset> = LazyLock::new(|| VideoPreset {
    exec: "ffmpeg".into(),
    input_arg: Some("-hide_banner -i".into()),
    filter_arg: Some(FILTER_480P.into()),
    output_file_ext: "mpg".into(),
    output_codec: "mpeg1video".into(),
    arg: Some("-an -b:v 1500k".into()),
});

#[must_use]
pub fn get_video_preset(name: &str) -> Option<VideoPreset> {
    match name {
        "avi-512" => Some(VIDEO_PRESET_AVI_512X512.clone()),
        "wmv-512" => Some(VIDEO_PRESET_WMV2_512X512.clone()),
        "mpg-512" => Some(VIDEO_PRESET_MPEG1VIDEO_512X512.clone()),
        "avi-480" => Some(VIDEO_PRESET_AVI_480P.clone()),
        "wmv-480" => Some(VIDEO_PRESET_WMV2_480P.clone()),
        "mpg-480" => Some(VIDEO_PRESET_MPEG1VIDEO_480P.clone()),
        _ => None,
    }
}

pub static VIDEO_PRESET_MAP: LazyLock<Vec<(&str, VideoPreset)>> = LazyLock::new(|| {
    vec![
        ("avi-512", VIDEO_PRESET_AVI_512X512.clone()),
        ("avi-480", VIDEO_PRESET_AVI_480P.clone()),
        ("wmv-512", VIDEO_PRESET_WMV2_512X512.clone()),
        ("wmv-480", VIDEO_PRESET_WMV2_480P.clone()),
        ("mpg-512", VIDEO_PRESET_MPEG1VIDEO_512X512.clone()),
        ("mpg-480", VIDEO_PRESET_MPEG1VIDEO_480P.clone()),
    ]
});

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub bit_rate: u64,
}

#[allow(clippy::missing_errors_doc)]
pub async fn get_media_file_probe(path: &Path) -> Result<serde_json::Value> {
    let output = tokio::process::Command::new("ffprobe")
        .args([
            "-show_format",
            "-show_streams",
            "-print_format",
            "json",
            "-v",
            "quiet",
        ])
        .arg(path)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| crate::AppError::Parse(format!("Failed to parse ffprobe output: {e}")))?;
    Ok(value)
}

pub async fn get_video_info(path: &Path) -> Option<VideoInfo> {
    let probe = get_media_file_probe(path).await.ok()?;
    let streams = probe.get("streams")?.as_array()?;
    for stream in streams {
        if stream.get("codec_type")?.as_str()? != "video" {
            continue;
        }
        return Some(VideoInfo {
            #[allow(clippy::cast_possible_truncation)]
            width: stream.get("width")?.as_u64()? as u32,
            #[allow(clippy::cast_possible_truncation)]
            height: stream.get("height")?.as_u64()? as u32,
            bit_rate: stream
                .get("bit_rate")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        });
    }
    None
}

pub async fn get_prefered_preset_list(path: &Path) -> Vec<VideoPreset> {
    let Some(info) = get_video_info(path).await else {
        return vec![];
    };

    let ratio = f64::from(info.width) / f64::from(info.height);
    let target_ratio = 640_f64 / 480_f64;

    if ratio > target_ratio {
        vec![
            VIDEO_PRESET_MPEG1VIDEO_480P.clone(),
            VIDEO_PRESET_WMV2_480P.clone(),
            VIDEO_PRESET_AVI_480P.clone(),
        ]
    } else {
        vec![
            VIDEO_PRESET_MPEG1VIDEO_512X512.clone(),
            VIDEO_PRESET_WMV2_512X512.clone(),
            VIDEO_PRESET_AVI_512X512.clone(),
        ]
    }
}

fn collect_video_files(dir: &Path, input_exts: &[String]) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let file_name = path.file_name()?.to_string_lossy();
            let lower = file_name.to_lowercase();
            if input_exts.iter().any(|ext| lower.ends_with(&format!(".{ext}"))) {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

#[allow(clippy::missing_errors_doc)]
pub async fn process_video_in_dir(
    dir: &Path,
    input_exts: &[String],
    presets: &[VideoPreset],
    auto_size: bool,
    remove_origin: bool,
) -> Result<bool> {
    let files = collect_video_files(dir, input_exts);
    if files.is_empty() {
        return Ok(true);
    }

    tracing::info!("Entering dir: {dir:?}");

    let mut has_error = false;

    for file_path in &files {
        let mut presets_for_file = presets.to_vec();
        if auto_size {
            let preferred = get_prefered_preset_list(file_path).await;
            let mut combined = preferred;
            combined.append(&mut presets_for_file);
            presets_for_file = combined;
        }

        let mut file_success = false;
        for preset in &presets_for_file {
            let output_path = preset.get_output_file_path(file_path);

            if *file_path == output_path {
                break;
            }

            if output_path.is_file() {
                tracing::info!("Remove existing file: {}", output_path.display());
                let _ = std::fs::remove_file(&output_path);
            }

            let cmd = preset.get_video_process_cmd(file_path, &output_path);
            tracing::info!("Processing Video: {:?} Preset: {preset:?}", file_path);

            let result = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .await;

            match result {
                Ok(output) if output.status.success() => {
                    if remove_origin && file_path.is_file() {
                        let _ = std::fs::remove_file(file_path);
                    }
                    file_success = true;
                    break;
                }
                Ok(output) => {
                    if output_path.is_file() {
                        let _ = std::fs::remove_file(&output_path);
                    }
                    tracing::warn!(
                        "Preset failed for {:?}: {}",
                        file_path,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to execute command: {e}");
                }
            }
        }

        if !file_success {
            has_error = true;
        }
    }

    Ok(!has_error)
}

#[allow(clippy::missing_errors_doc)]
pub async fn bms_folder_transfer_video(
    root_dir: &Path,
    input_exts: &[String],
    presets: &[VideoPreset],
    auto_size: bool,
    remove_origin: bool,
) -> Result<bool> {
    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return Ok(true);
    };

    let dirs: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    for bms_dir in dirs {
        let success = process_video_in_dir(&bms_dir, input_exts, presets, auto_size, remove_origin).await?;
        if !success {
            tracing::error!("Dir: {bms_dir:?} Error occurred!");
            return Ok(false);
        }
    }

    Ok(true)
}
