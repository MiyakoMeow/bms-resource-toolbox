pub mod bms_event;
pub mod work;

use std::{cell::LazyCell, collections::HashMap, fs::FileType, path::Path};

use blocking::unblock;
use bms_rs::{bms::prelude::*, bmson::bmson_to_bms::BmsonToBmsOutput};
use futures::stream::{self, StreamExt as FuturesStreamExt};
use smol::{fs, io};

use self::work::extract_work_name;

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

/// 仅负责读取 BMS 文件（异步 IO）
async fn read_bms_file(file: &Path) -> io::Result<Vec<u8>> {
    let bytes = { fs::read(file).await? };
    Ok(bytes)
}

/// 仅负责解析 BMS 字节（CPU 密集，单线程执行）
fn parse_bms_bytes(bytes: &[u8]) -> BmsOutput {
    let (str, _encoding, _has_error) = encoding_rs::SHIFT_JIS.decode(bytes);
    parse_bms(&str)
}

/// 包装：读取 + 解析
pub async fn parse_bms_file(file: &Path) -> io::Result<BmsOutput> {
    let bytes = read_bms_file(file).await?;
    // 解析阶段保持单线程（不在此处并发）
    Ok(parse_bms_bytes(&bytes))
}

/// 仅负责读取 BMSON 文件（异步 IO）
async fn read_bmson_file(file: &Path) -> io::Result<Vec<u8>> {
    let bytes = { fs::read(file).await? };
    Ok(bytes)
}

/// 仅负责解析 BMSON 字节（CPU 密集，单线程执行）
fn parse_bmson_bytes(bytes: &[u8]) -> io::Result<BmsOutput> {
    let Some(bmson) = serde_json::from_slice(bytes).map_err(io::Error::other)? else {
        let output = BmsOutput {
            bms: Bms::default(),
            warnings: vec![BmsWarning::PlayingError(PlayingError::NoNotes)],
        };
        return Ok(output);
    };
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

/// 包装：读取 + 解析
pub async fn parse_bmson_file(file: &Path) -> io::Result<BmsOutput> {
    let bytes = read_bmson_file(file).await?;
    parse_bmson_bytes(&bytes)
}

pub async fn get_dir_bms_list(dir: &Path) -> io::Result<Vec<BmsOutput>> {
    // 收集候选文件
    let mut bms_files = Vec::new();
    let mut dir_entry = fs::read_dir(dir).await?;
    while let Some(entry) = smol::stream::StreamExt::next(&mut dir_entry).await {
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

    // Stage 1: 并发读取（IO 密集）
    let read_concurrency: usize = 16;
    #[derive(Debug)]
    enum PendingParse {
        Bms(Vec<u8>),
        Bmson(Vec<u8>),
    }
    // Stage 2: 解析并发度（CPU 密集）
    let parse_concurrency: usize = num_cpus::get().max(1);

    let parsed_list: Vec<BmsOutput> = stream::iter(bms_files)
        .map(|file_path| async move {
            let ext = file_path.extension().and_then(|p| p.to_str()).unwrap_or("");
            if BMS_FILE_EXTS.contains(&ext) {
                let bytes = read_bms_file(&file_path).await?;
                Ok::<Option<PendingParse>, io::Error>(Some(PendingParse::Bms(bytes)))
            } else if BMSON_FILE_EXTS.contains(&ext) {
                let bytes = read_bmson_file(&file_path).await?;
                Ok(Some(PendingParse::Bmson(bytes)))
            } else {
                Ok(None)
            }
        })
        .buffer_unordered(read_concurrency)
        .filter_map(|res| async move { res.ok().flatten() })
        // Stage 2: 解析（CPU 密集，受限并发）
        .map(|pending| async move {
            let parsed: io::Result<BmsOutput> = match pending {
                PendingParse::Bms(bytes) => {
                    let r = unblock(move || parse_bms_bytes(&bytes)).await;
                    Ok(r)
                }
                PendingParse::Bmson(bytes) => unblock(move || parse_bmson_bytes(&bytes)).await,
            };
            if let Ok(out) = parsed {
                (!out.warnings.iter().any(|warning| {
                    matches!(
                        warning,
                        BmsWarning::PlayingError(_)
                            | BmsWarning::PlayingWarning(PlayingWarning::NoPlayableNotes)
                    )
                }))
                .then_some(out)
            } else {
                None
            }
        })
        .buffer_unordered(parse_concurrency)
        .filter_map(|v| async move { v })
        .collect::<Vec<BmsOutput>>()
        .await;

    Ok(parsed_list)
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
    while let Some(entry) = smol::stream::StreamExt::next(&mut read_dir).await {
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
    while let Some(entry) = smol::stream::StreamExt::next(&mut read_dir).await {
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
