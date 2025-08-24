pub mod bms_event;
pub mod work;

use std::{cell::LazyCell, collections::HashMap, fs::FileType, path::Path};

use crate::fs::lock::acquire_disk_lock;
use bms_rs::{
    bms::{BmsOutput, BmsWarning, parse_bms, prelude::PlayingWarning},
    bmson::{Bmson, bmson_to_bms::BmsonToBmsOutput},
    parse::model::Bms,
};
use smol::{fs, io, stream::StreamExt};
use work::extract_work_name;

pub const BMS_FILE_EXTS: &[&str] = &["bms", "bme", "bml", "pms"];
pub const BMSON_FILE_EXTS: &[&str] = &["bmson"];
#[allow(clippy::declare_interior_mutable_const)]
pub const CHART_FILE_EXTS: LazyCell<Vec<&str>> = LazyCell::new(|| {
    BMS_FILE_EXTS
        .iter()
        .chain(BMSON_FILE_EXTS)
        .copied()
        .collect()
});

pub const AUDIO_FILE_EXTS: &[&str] = &["flac", "ape", "ogg", "wav", "mp3"];
pub const VIDEO_FILE_EXTS: &[&str] = &["webm", "mp4", "mkv", "avi", "wmv", "mpg", "mpeg"];
pub const IMAGE_FILE_EXTS: &[&str] = &["webp", "jpg", "png", "bmp", "svg"];

#[allow(clippy::declare_interior_mutable_const)]
pub const MEDIA_FILE_EXTS_FILE_EXTS: LazyCell<Vec<&str>> = LazyCell::new(|| {
    AUDIO_FILE_EXTS
        .iter()
        .chain(VIDEO_FILE_EXTS)
        .chain(IMAGE_FILE_EXTS)
        .copied()
        .collect::<Vec<_>>()
});

pub async fn parse_bms_file(file: &Path) -> io::Result<BmsOutput> {
    // Acquire disk lock for file reading
    let bytes = {
        let _lock_guard = acquire_disk_lock(file).await;
        fs::read(file).await?
    }; // 锁在这里被 drop，文件读取完成后立即释放

    let (str, _encoding, _has_error) = encoding_rs::SHIFT_JIS.decode(&bytes);
    Ok(parse_bms(&str))
}

pub async fn parse_bmson_file(file: &Path) -> io::Result<BmsOutput> {
    // Acquire disk lock for file reading
    let bytes = {
        let _lock_guard = acquire_disk_lock(file).await;
        fs::read(file).await?
    }; // 锁在这里被 drop，文件读取完成后立即释放

    let bmson: Bmson = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    let BmsonToBmsOutput {
        bms,
        warnings: _,
        playing_warnings,
        playing_errors,
    }: BmsonToBmsOutput = Bms::from_bmson(bmson);
    Ok(BmsOutput {
        bms,
        warnings: playing_warnings
            .into_iter()
            .map(BmsWarning::PlayingWarning)
            .chain(playing_errors.into_iter().map(BmsWarning::PlayingError))
            .collect(),
    })
}

pub async fn get_dir_bms_list(dir: &Path) -> io::Result<Vec<BmsOutput>> {
    // Collect all BMS files first
    let mut bms_files = Vec::new();
    let mut dir_entry = fs::read_dir(dir).await?;
    while let Some(entry) = dir_entry.next().await {
        let entry = entry?;
        let file_type: FileType = entry.file_type().await?;
        if file_type.is_dir() {
            continue;
        }
        let file_path = entry.path();
        let ext = file_path.extension().and_then(|p| p.to_str()).unwrap_or("");
        if BMS_FILE_EXTS.contains(&ext) || BMSON_FILE_EXTS.contains(&ext) {
            bms_files.push(file_path);
        }
    }

    // Process files in parallel using disk locks
    let futures: Vec<_> = bms_files
        .into_iter()
        .map(|file_path| async move {
            let ext = file_path.extension().and_then(|p| p.to_str()).unwrap_or("");
            let bms = if BMS_FILE_EXTS.contains(&ext) {
                parse_bms_file(&file_path).await?
            } else if BMSON_FILE_EXTS.contains(&ext) {
                parse_bmson_file(&file_path).await?
            } else {
                return Ok::<std::option::Option<bms_rs::bms::BmsOutput>, io::Error>(
                    None::<BmsOutput>,
                );
            };

            // Check playing error
            if bms.warnings.iter().any(|warning| {
                matches!(
                    warning,
                    BmsWarning::PlayingError(_)
                        | BmsWarning::PlayingWarning(PlayingWarning::NoPlayableNotes)
                )
            }) {
                return Ok(None);
            }

            Ok(Some(bms))
        })
        .collect();

    // Wait for all tasks to complete
    let results = futures::future::join_all(futures).await;

    // Collect successful results
    let mut bms_list = Vec::new();
    for result in results {
        if let Ok(Some(bms)) = result {
            bms_list.push(bms);
        }
    }

    Ok(bms_list)
}

/// Get BMS information for an entire directory (information integration)
pub async fn get_dir_bms_info(dir: &Path) -> io::Result<Option<Bms>> {
    let bms_list = get_dir_bms_list(dir).await?;
    if bms_list.is_empty() {
        return Ok(None);
    }
    let mut bms = Bms::default();
    // Header
    let titles: Vec<_> = bms_list
        .iter()
        .filter_map(|BmsOutput { bms, .. }| bms.header.title.as_deref())
        .collect();
    let title = extract_work_name(titles.as_slice(), true, &[]);
    bms.header.title = Some(title);
    let artists: Vec<_> = bms_list
        .iter()
        .filter_map(|BmsOutput { bms, .. }| bms.header.artist.as_deref())
        .collect();
    let artist = extract_work_name(
        artists.as_slice(),
        true,
        &[
            "/", ":", "：", "-", "obj", "obj.", "Obj", "Obj.", "OBJ", "OBJ.",
        ],
    );
    bms.header.artist = Some(artist);
    let genres: Vec<_> = bms_list
        .iter()
        .filter_map(|BmsOutput { bms, .. }| bms.header.genre.as_deref())
        .collect();
    let genre = extract_work_name(genres.as_slice(), true, &[]);
    bms.header.genre = Some(genre);
    // Defines
    bms.notes.wav_files = bms_list
        .iter()
        .fold(HashMap::new(), |mut map, BmsOutput { bms, .. }| {
            map.extend(bms.notes.wav_files.clone());
            map
        });
    bms.graphics.bmp_files =
        bms_list
            .iter()
            .fold(HashMap::new(), |mut map, BmsOutput { bms, .. }| {
                map.extend(bms.graphics.bmp_files.clone());
                map
            });
    Ok(Some(bms))
}

/// work_dir: work directory, must contain BMS files
pub async fn is_work_dir(dir: &Path) -> io::Result<bool> {
    // Collect all files first
    let mut files = Vec::new();
    let mut read_dir = fs::read_dir(dir).await?;
    while let Some(entry) = read_dir.next().await {
        let entry = entry?;
        let file_type: FileType = entry.file_type().await?;
        if file_type.is_file() {
            files.push(entry.path());
        }
    }

    // Check files in parallel
    let futures: Vec<_> = files
        .into_iter()
        .map(|file_path| async move {
            #[allow(clippy::borrow_interior_mutable_const)]
            let has_chart_ext = file_path
                .extension()
                .and_then(|s| s.to_str())
                .filter(|s| CHART_FILE_EXTS.contains(s))
                .is_some();
            Ok::<bool, io::Error>(has_chart_ext)
        })
        .collect();

    // Wait for all tasks to complete
    let results = futures::future::join_all(futures).await;

    // Return true if any file has chart extension
    for result in results {
        if result? {
            return Ok(true);
        }
    }

    Ok(false)
}

/// root_dir: work collection directory, parent of work_dir
pub async fn is_root_dir(dir: &Path) -> io::Result<bool> {
    // Collect all directories first
    let mut dirs = Vec::new();
    let mut read_dir = fs::read_dir(dir).await?;
    while let Some(entry) = read_dir.next().await {
        let entry = entry?;
        let file_type: FileType = entry.file_type().await?;
        if file_type.is_dir() {
            dirs.push(entry.path());
        }
    }

    // Check directories in parallel
    let futures: Vec<_> = dirs
        .into_iter()
        .map(|dir_path| async move { is_work_dir(&dir_path).await })
        .collect();

    // Wait for all tasks to complete
    let results = futures::future::join_all(futures).await;

    // Return true if any directory is a work directory
    for result in results {
        if result? {
            return Ok(true);
        }
    }

    Ok(false)
}
