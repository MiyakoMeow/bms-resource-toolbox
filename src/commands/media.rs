use std::path::Path;

use crate::cli::MediaCommand;
use crate::media::audio::{self, AudioTransferOpts};
use crate::media::video;

pub async fn handle(cmd: MediaCommand) -> crate::Result<()> {
    match cmd {
        MediaCommand::Audio { root_dir, mode } => transfer_audio(&root_dir, &mode).await,
        MediaCommand::Video {
            root_dir,
            preset,
            auto_size,
        } => transfer_video(&root_dir, &preset, auto_size).await,
    }
}

async fn transfer_audio(root_dir: &Path, mode: &str) -> crate::Result<()> {
    let Some(audio_mode) = audio::get_audio_mode(mode) else {
        return Err(crate::AppError::InvalidArg(format!("Unknown audio mode: {mode}")));
    };

    check_audio_tools(&audio_mode.presets)?;

    let opts = AudioTransferOpts {
        remove_origin_on_success: true,
        remove_origin_on_failure: false,
        remove_existing_target_file: true,
        stop_on_error: true,
    };

    audio::bms_folder_transfer_audio(root_dir, &audio_mode.input_exts, &audio_mode.presets, &opts)
        .await?;
    Ok(())
}

fn check_audio_tools(presets: &[audio::AudioPreset]) -> crate::Result<()> {
    let mut needed: Vec<String> = presets.iter().map(|p| p.exec.clone()).collect();
    needed.sort();
    needed.dedup();
    for tool in &needed {
        check_tool_available(tool)?;
    }
    Ok(())
}

async fn transfer_video(root_dir: &Path, preset: &str, auto_size: bool) -> crate::Result<()> {
    let Some(video_preset) = video::get_video_preset(preset) else {
        return Err(crate::AppError::InvalidArg(format!(
            "Unknown video preset: {preset}"
        )));
    };

    check_tool_available("ffmpeg")?;

    video::bms_folder_transfer_video(
        root_dir,
        &[String::from("mp4")],
        &[video_preset],
        auto_size,
        true,
    )
    .await?;
    Ok(())
}

fn check_tool_available(name: &str) -> crate::Result<()> {
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("which {name} 2>/dev/null"))
        .status()?;
    if !status.success() {
        return Err(crate::AppError::InvalidArg(format!(
            "{name} is not available"
        )));
    }
    Ok(())
}
