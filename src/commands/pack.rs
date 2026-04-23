use std::path::Path;

use crate::cli::PackCommand;
use crate::commands::bigpack;
use crate::commands::folder;
use crate::commands::rawpack;
use crate::fs;
use crate::fs::sync;
use crate::media::audio::{self, AudioTransferOpts, AUDIO_PRESET_FLAC, AUDIO_PRESET_FLAC_FFMPEG, AUDIO_PRESET_OGG_Q10};
use crate::media::video::{self, VIDEO_PRESET_AVI_512X512, VIDEO_PRESET_MPEG1VIDEO_512X512, VIDEO_PRESET_WMV2_512X512};

pub async fn handle(cmd: PackCommand) -> crate::Result<()> {
    match cmd {
        PackCommand::RawToHq { root_dir } => pack_raw_to_hq(&root_dir).await,
        PackCommand::HqToLq { root_dir } => pack_hq_to_lq(&root_dir).await,
        PackCommand::SetupRawpackToHq { pack_dir, root_dir } => {
            pack_setup_rawpack_to_hq(&pack_dir, &root_dir).await
        }
        PackCommand::UpdateRawpackToHq {
            pack_dir,
            root_dir,
            sync_dir,
        } => pack_update_rawpack_to_hq(&pack_dir, &root_dir, &sync_dir).await,
    }
}

fn get_audio_opts() -> AudioTransferOpts {
    AudioTransferOpts {
        remove_origin_on_success: true,
        remove_origin_on_failure: false,
        remove_existing_target_file: true,
        stop_on_error: true,
    }
}

async fn wav_to_flac(root_dir: &Path) -> crate::Result<()> {
    let opts = get_audio_opts();
    audio::bms_folder_transfer_audio(
        root_dir,
        &[String::from("wav")],
        &[AUDIO_PRESET_FLAC.clone(), AUDIO_PRESET_FLAC_FFMPEG.clone()],
        &opts,
    )
    .await?;
    Ok(())
}

async fn flac_to_ogg(root_dir: &Path) -> crate::Result<()> {
    let opts = get_audio_opts();
    audio::bms_folder_transfer_audio(
        root_dir,
        &[String::from("flac")],
        std::slice::from_ref(&AUDIO_PRESET_OGG_Q10),
        &opts,
    )
    .await?;
    Ok(())
}

async fn video_to_lq(root_dir: &Path) -> crate::Result<()> {
    video::bms_folder_transfer_video(
        root_dir,
        &[String::from("mp4")],
        &[
            VIDEO_PRESET_MPEG1VIDEO_512X512.clone(),
            VIDEO_PRESET_WMV2_512X512.clone(),
            VIDEO_PRESET_AVI_512X512.clone(),
        ],
        false,
        true,
    )
    .await?;
    Ok(())
}

async fn pack_raw_to_hq(root_dir: &Path) -> crate::Result<()> {
    wav_to_flac(root_dir).await?;
    bigpack::remove_unneed_media_files(root_dir, 0);
    Ok(())
}

async fn pack_hq_to_lq(root_dir: &Path) -> crate::Result<()> {
    flac_to_ogg(root_dir).await?;
    video_to_lq(root_dir).await?;
    Ok(())
}

async fn pack_setup_rawpack_to_hq(pack_dir: &Path, root_dir: &Path) -> crate::Result<()> {
    if root_dir.is_dir() {
        return Err(crate::AppError::InvalidArg(format!(
            "{} already exists",
            root_dir.display()
        )));
    }
    std::fs::create_dir(root_dir)?;

    let cache_dir = std::env::temp_dir().join("bms-toolbox-setup-cache");
    rawpack::unzip_numeric_to_bms_folder(pack_dir, &cache_dir, root_dir);
    let _ = std::fs::remove_dir_all(&cache_dir);

    folder::append_name_by_bms(root_dir);

    wav_to_flac(root_dir).await?;
    bigpack::remove_unneed_media_files(root_dir, 0);
    Ok(())
}

async fn pack_update_rawpack_to_hq(
    pack_dir: &Path,
    root_dir: &Path,
    sync_dir: &Path,
) -> crate::Result<()> {
    if root_dir.is_dir() {
        return Err(crate::AppError::InvalidArg(format!(
            "{} already exists",
            root_dir.display()
        )));
    }
    std::fs::create_dir(root_dir)?;

    let cache_dir = std::env::temp_dir().join("bms-toolbox-update-cache");
    rawpack::unzip_numeric_to_bms_folder(pack_dir, &cache_dir, root_dir);
    let _ = std::fs::remove_dir_all(&cache_dir);

    folder::copy_numbered_workdir_names(sync_dir, root_dir);

    wav_to_flac(root_dir).await?;
    bigpack::remove_unneed_media_files(root_dir, 0);

    sync::sync_folder(root_dir, sync_dir, &sync::SYNC_PRESET_FOR_APPEND);
    fs::remove_empty_folder(root_dir);
    Ok(())
}
