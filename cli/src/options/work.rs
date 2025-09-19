use std::{collections::VecDeque, path::Path, str::FromStr};

use clap::ValueEnum;
use smol::{fs, io, stream::StreamExt};

use crate::{
    bms::get_dir_bms_info,
    fs::{
        get_vaild_fs_name,
        moving::{ReplacePreset, move_elements_across_dir, replace_options_from_preset},
    },
};

pub const DEFAULT_TITLE: &str = "!!! UnknownTitle !!!";
pub const DEFAULT_ARTIST: &str = "!!! UnknownArtist !!!";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BmsFolderSetNameType {
    /// Suitable for cases where you want to directly replace directory name with "Title [Artist]"
    ReplaceTitleArtist = 0,
    /// Suitable for cases where you want to append "Title [Artist]" after work folder name
    AppendTitleArtist = 1,
    /// Suitable for cases where you want to append " [Artist]" after work folder name
    AppendArtist = 2,
}

impl FromStr for BmsFolderSetNameType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "replace" | "replace_title_artist" => Ok(BmsFolderSetNameType::ReplaceTitleArtist),
            "append" | "append_title_artist" => Ok(BmsFolderSetNameType::AppendTitleArtist),
            "append_artist" => Ok(BmsFolderSetNameType::AppendArtist),
            _ => Err(format!(
                "Unknown set type: {}. Valid values are: replace, append, append_artist",
                s
            )),
        }
    }
}

impl ValueEnum for BmsFolderSetNameType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::ReplaceTitleArtist,
            Self::AppendTitleArtist,
            Self::AppendArtist,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let name = match self {
            BmsFolderSetNameType::ReplaceTitleArtist => "replace_title_artist",
            BmsFolderSetNameType::AppendTitleArtist => "append_title_artist",
            BmsFolderSetNameType::AppendArtist => "append_artist",
        };
        Some(clap::builder::PossibleValue::new(name))
    }
}

/// Check if directory name already follows the "XX [XX]" pattern
fn is_already_formatted(dir_name: &str, set_type: BmsFolderSetNameType) -> bool {
    match set_type {
        BmsFolderSetNameType::ReplaceTitleArtist => {
            // 检查是否已经是 "Title [Artist]" 格式
            dir_name.contains(" [") && dir_name.ends_with("]")
        }
        BmsFolderSetNameType::AppendTitleArtist => {
            // 检查是否已经包含 "Title [Artist]" 格式
            dir_name.contains(" [") && dir_name.ends_with("]")
        }
        BmsFolderSetNameType::AppendArtist => {
            // 检查是否已经包含 " [Artist]" 格式
            dir_name.contains(" [") && dir_name.ends_with("]")
        }
    }
}

/// This script is suitable for cases where you want to append "Title [Artist]" after work folder name
pub async fn set_name_by_bms(
    work_dir: &Path,
    set_type: BmsFolderSetNameType,
    dry_run: bool,
    replace_preset: ReplacePreset,
    skip_already_formatted: bool,
) -> io::Result<()> {
    if dry_run {
        log::info!("[dry-run] Start: work::set_name_by_bms");
    }
    let Some(bms_info) = get_dir_bms_info(work_dir).await? else {
        log::info!("Bms file not found, skipping: {}", work_dir.display());
        return Ok(());
    };
    let title = bms_info.header.title.unwrap_or(DEFAULT_TITLE.to_string());
    let artist = bms_info.header.artist.unwrap_or(DEFAULT_ARTIST.to_string());
    let work_dir_name = work_dir
        .file_name()
        .ok_or(io::Error::other("Dir name not exists"))?
        .to_string_lossy();

    // 如果启用了跳过已格式化目录的选项，检查目录名是否已经是目标格式
    if skip_already_formatted && is_already_formatted(&work_dir_name, set_type) {
        if dry_run {
            log::info!(
                "[dry-run] Directory already formatted, skipping: {}",
                work_dir.display()
            );
        }
        return Ok(());
    }

    let target_dir_name = match set_type {
        BmsFolderSetNameType::ReplaceTitleArtist => format!("{title} [{artist}]"),
        BmsFolderSetNameType::AppendTitleArtist => format!("{work_dir_name} {title} [{artist}]"),
        BmsFolderSetNameType::AppendArtist => format!("{work_dir_name} [{artist}]"),
    };
    let target_dir_name = get_vaild_fs_name(&target_dir_name);
    let target_work_dir = work_dir
        .parent()
        .ok_or(io::Error::other("Dir name not exists"))?
        .join(target_dir_name);

    // 如果源目录与目标目录相同，则跳过操作
    if work_dir == target_work_dir {
        if dry_run {
            log::info!(
                "[dry-run] Source and target directories are the same, skipping: {}",
                work_dir.display()
            );
        }
        return Ok(());
    }

    log::info!(
        "Rename work dir by moving content: {} -> {}",
        work_dir.display(),
        target_work_dir.display()
    );
    if !dry_run {
        move_elements_across_dir(
            work_dir,
            target_work_dir,
            replace_options_from_preset(replace_preset),
        )
        .await?;
    }
    if dry_run {
        log::info!("[dry-run] End: work::set_name_by_bms");
    }
    Ok(())
}

pub async fn undo_set_name_by_bms(
    work_dir: &Path,
    set_type: BmsFolderSetNameType,
    dry_run: bool,
) -> io::Result<()> {
    if dry_run {
        log::info!("[dry-run] Start: work::undo_set_name_by_bms");
    }
    let work_dir_name = work_dir
        .file_name()
        .ok_or(io::Error::other("Dir name not exists"))?
        .to_string_lossy();

    // 根据不同的set_type，提取原始目录名
    let original_dir_name = match set_type {
        BmsFolderSetNameType::ReplaceTitleArtist => {
            // 对于ReplaceTitleArtist，原始名称应该是第一个单词
            work_dir_name
                .split_whitespace()
                .next()
                .unwrap_or(&work_dir_name)
        }
        BmsFolderSetNameType::AppendTitleArtist => {
            // 对于AppendTitleArtist，原始名称是第一个单词
            work_dir_name
                .split_whitespace()
                .next()
                .unwrap_or(&work_dir_name)
        }
        BmsFolderSetNameType::AppendArtist => {
            // 对于AppendArtist，原始名称是第一个单词
            work_dir_name
                .split_whitespace()
                .next()
                .unwrap_or(&work_dir_name)
        }
    };

    // 确保至少保留1个单词
    let original_dir_name = if original_dir_name.is_empty() {
        &work_dir_name
    } else {
        original_dir_name
    };

    let new_dir_path = work_dir
        .parent()
        .ok_or(io::Error::other("Dir name not exists"))?
        .join(original_dir_name);

    // 如果源目录与目标目录相同，则跳过操作
    if work_dir == new_dir_path {
        if dry_run {
            log::info!(
                "[dry-run] Source and target directories are the same, skipping: {}",
                work_dir.display()
            );
        }
        return Ok(());
    }

    // 检查目标目录是否已存在，如果存在则添加数字后缀
    let mut final_dir_path = new_dir_path.clone();
    let mut counter = 1;
    while final_dir_path.exists() {
        let new_name = format!("{}_{}", original_dir_name, counter);
        final_dir_path = work_dir
            .parent()
            .ok_or(io::Error::other("Dir name not exists"))?
            .join(new_name);
        counter += 1;
    }

    log::info!(
        "Undo rename: {} -> {}",
        work_dir.display(),
        final_dir_path.display()
    );

    if !dry_run {
        // 仅使用fs::rename，不使用move_elements_across_dir
        fs::rename(work_dir, &final_dir_path).await?;
    }

    if dry_run {
        log::info!("[dry-run] End: work::undo_set_name_by_bms");
    }
    Ok(())
}

/// Remove all 0-byte files in work_dir and its subdirectories (loop version, smol 2).
pub async fn remove_zero_sized_media_files(
    work_dir: impl AsRef<Path>,
    dry_run: bool,
) -> io::Result<()> {
    if dry_run {
        log::info!("[dry-run] Start: work::remove_zero_sized_media_files");
    }
    let mut stack = VecDeque::new();
    stack.push_back(work_dir.as_ref().to_path_buf());

    // Store async deletion tasks
    let mut tasks = Vec::new();

    while let Some(dir) = stack.pop_back() {
        let mut entries = fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let path = entry.path();
            let meta = entry.metadata().await?;

            if meta.is_file() && meta.len() == 0 {
                // Async deletion, task handle goes into Vec
                if dry_run {
                    log::info!("Would remove empty file: {}", path.display());
                } else {
                    tasks.push(smol::spawn(async move {
                        fs::remove_file(&path).await?;
                        Ok::<(), io::Error>(())
                    }));
                }
            } else if meta.is_dir() {
                // Continue pushing to stack
                stack.push_back(path);
            }
        }
    }

    if !dry_run {
        // Wait for all deletion tasks to complete
        for task in tasks {
            task.await?;
        }
    }

    if dry_run {
        log::info!("[dry-run] End: work::remove_zero_sized_media_files");
    }

    Ok(())
}
