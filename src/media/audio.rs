use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::Result;

#[derive(Debug, Clone)]
pub struct AudioPreset {
    pub exec: String,
    pub output_format: String,
    pub arg: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct AudioTransferOpts {
    pub remove_origin_on_success: bool,
    pub remove_origin_on_failure: bool,
    pub remove_existing_target_file: bool,
    pub stop_on_error: bool,
}

pub static AUDIO_PRESET_FLAC: LazyLock<AudioPreset> = LazyLock::new(|| AudioPreset {
    exec: "flac".into(),
    output_format: "flac".into(),
    arg: Some("--keep-foreign-metadata-if-present --best -f".into()),
});

pub static AUDIO_PRESET_FLAC_FFMPEG: LazyLock<AudioPreset> = LazyLock::new(|| AudioPreset {
    exec: "ffmpeg".into(),
    output_format: "flac".into(),
    arg: Some(String::new()),
});

pub static AUDIO_PRESET_OGG_Q10: LazyLock<AudioPreset> = LazyLock::new(|| AudioPreset {
    exec: "oggenc".into(),
    output_format: "ogg".into(),
    arg: Some("-q10".into()),
});

pub static AUDIO_PRESET_WAV_FROM_FLAC: LazyLock<AudioPreset> = LazyLock::new(|| AudioPreset {
    exec: "flac".into(),
    output_format: "wav".into(),
    arg: Some("-d --keep-foreign-metadata-if-present -f".into()),
});

pub static AUDIO_PRESET_WAV_FFMPEG: LazyLock<AudioPreset> = LazyLock::new(|| AudioPreset {
    exec: "ffmpeg".into(),
    output_format: "wav".into(),
    arg: Some(String::new()),
});

pub static AUDIO_PRESET_OGG_FFMPEG: LazyLock<AudioPreset> = LazyLock::new(|| AudioPreset {
    exec: "ffmpeg".into(),
    output_format: "ogg".into(),
    arg: Some(String::new()),
});

#[derive(Debug, Clone)]
pub struct AudioMode {
    pub input_exts: Vec<String>,
    pub presets: Vec<AudioPreset>,
}

#[must_use]
pub fn get_audio_mode(mode: &str) -> Option<AudioMode> {
    match mode {
        "wav-to-flac" => Some(AudioMode {
            input_exts: vec!["wav".into()],
            presets: vec![AUDIO_PRESET_FLAC.clone(), AUDIO_PRESET_FLAC_FFMPEG.clone()],
        }),
        "flac-to-ogg" => Some(AudioMode {
            input_exts: vec!["flac".into()],
            presets: vec![AUDIO_PRESET_OGG_Q10.clone()],
        }),
        "wav-to-ogg" => Some(AudioMode {
            input_exts: vec!["wav".into()],
            presets: vec![AUDIO_PRESET_OGG_Q10.clone()],
        }),
        "flac-to-wav" => Some(AudioMode {
            input_exts: vec!["flac".into()],
            presets: vec![AUDIO_PRESET_WAV_FROM_FLAC.clone(), AUDIO_PRESET_WAV_FFMPEG.clone()],
        }),
        _ => None,
    }
}

pub static AUDIO_MODES: LazyLock<Vec<(&str, AudioMode)>> = LazyLock::new(|| {
    vec![
        (
            "wav-to-flac",
            get_audio_mode("wav-to-flac").unwrap(),
        ),
        (
            "flac-to-ogg",
            get_audio_mode("flac-to-ogg").unwrap(),
        ),
        (
            "wav-to-ogg",
            get_audio_mode("wav-to-ogg").unwrap(),
        ),
        (
            "flac-to-wav",
            get_audio_mode("flac-to-wav").unwrap(),
        ),
    ]
});

#[must_use]
pub fn get_audio_process_cmd(file_path: &Path, output_path: &Path, preset: &AudioPreset) -> String {
    let arg = preset.arg.as_deref().unwrap_or("");
    let file = file_path.display();
    let output = output_path.display();
    match preset.exec.as_str() {
        "ffmpeg" => {
            format!(
                "ffmpeg -hide_banner -loglevel panic -i \"{file}\" -f {fmt} -map_metadata 0 {arg} \"{output}\"",
                fmt = preset.output_format
            )
        }
        "oggenc" => {
            format!("oggenc {arg} \"{file}\" -o \"{output}\"")
        }
        "flac" => {
            format!("flac {arg} \"{file}\" -o \"{output}\"")
        }
        _ => String::new(),
    }
}

fn collect_audio_files(dir: &Path, input_exts: &[String]) -> Vec<PathBuf> {
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
            let ext = path.extension()?.to_str()?.to_lowercase();
            if input_exts.iter().any(|e| e == &ext) {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

#[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
pub async fn transfer_audio_in_dir(
    dir: &Path,
    exts: &[String],
    presets: &[AudioPreset],
    opts: &AudioTransferOpts,
) -> Result<bool> {
    let files = collect_audio_files(dir, exts);
    if files.is_empty() {
        return Ok(true);
    }

    tracing::info!("Entering dir: {dir:?} Input ext: {exts:?}");

    let cpu_count = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4);
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(cpu_count));

    let mut handles = Vec::new();

    for file_path in files {
        let presets = presets.to_vec();
        let opts = opts.clone();
        let semaphore = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            transfer_single_audio_file(&file_path, &presets, &opts).await
        });
        handles.push(handle);
    }

    let mut has_error = false;
    for handle in handles {
        match handle.await {
            Ok(Ok(success)) => {
                if !success {
                    has_error = true;
                }
            }
            Ok(Err(e)) => {
                tracing::error!("Task error: {e}");
                has_error = true;
            }
            Err(e) => {
                tracing::error!("Join error: {e}");
                has_error = true;
            }
        }
    }

    Ok(!has_error)
}

async fn transfer_single_audio_file(
    file_path: &Path,
    presets: &[AudioPreset],
    opts: &AudioTransferOpts,
) -> Result<bool> {
    for (preset_index, preset) in presets.iter().enumerate() {
        let output_path = file_path.with_extension(&preset.output_format);

        if output_path.is_file() {
            if output_path.metadata().map(|m| m.len()).unwrap_or(0) > 0
                && !opts.remove_existing_target_file
            {
                tracing::info!("File {} exists! Skipping...", output_path.display());
                return Ok(true);
            }
            tracing::info!("Remove existing file: {}", output_path.display());
            let _ = std::fs::remove_file(&output_path);
        }

        let cmd = get_audio_process_cmd(file_path, &output_path, preset);
        if cmd.is_empty() {
            continue;
        }

        tracing::info!("Processing: {} Preset: {preset:?}", file_path.display());

        let result = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .await;

        match result {
            Ok(output) if output.status.success() => {
                if opts.remove_origin_on_success
                    && file_path.is_file()
                    && let Err(e) = std::fs::remove_file(file_path)
                {
                    tracing::warn!("PermissionError When Deleting: {file_path:?}: {e}");
                }
                return Ok(true);
            }
            Ok(output) => {
                tracing::warn!(
                    "Preset {preset_index} failed for {:?}: {}",
                    file_path,
                    String::from_utf8_lossy(&output.stderr)
                );
                if preset_index == presets.len() - 1 {
                    if opts.remove_origin_on_failure && file_path.is_file() {
                        let _ = std::fs::remove_file(file_path);
                    }
                    return Ok(false);
                }
            }
            Err(e) => {
                tracing::error!("Failed to execute command: {e}");
                if preset_index == presets.len() - 1 {
                    return Ok(false);
                }
            }
        }
    }

    Ok(false)
}

#[allow(clippy::missing_errors_doc)]
pub async fn bms_folder_transfer_audio(
    root_dir: &Path,
    input_ext: &[String],
    transfer_mode: &[AudioPreset],
    opts: &AudioTransferOpts,
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
        let success = transfer_audio_in_dir(&bms_dir, input_ext, transfer_mode, opts).await?;
        if !success {
            tracing::error!("Dir: {bms_dir:?} Error occurred!");
            if opts.stop_on_error {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
