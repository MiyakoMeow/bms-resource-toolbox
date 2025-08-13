use std::{collections::VecDeque, path::Path};

use smol::{fs, io, stream::StreamExt};

use crate::{
    bms::get_dir_bms_info,
    fs::moving::{move_elements_across_dir, replace_options_update_pack},
};

pub const DEFAULT_TITLE: &str = "!!! UnknownTitle !!!";
pub const DEFAULT_ARTIST: &str = "!!! UnknownArtist !!!";

pub enum BmsFolderSetNameType {
    /// 适用于希望直接替换目录名为“标题 [艺术家]”的情况
    ReplaceTitleArtist = 0,
    /// 适用于希望在作品文件夹名后添加“标题 [艺术家]”的情况
    AppendTitleArtist = 1,
    /// 适用于希望在作品文件夹名后添加“ [艺术家]”的情况
    AppendArtist = 2,
}

/// 该脚本适用于希望在作品文件夹名后添加“标题 [艺术家]”的情况
pub async fn set_name_by_bms(work_dir: &Path, set_type: BmsFolderSetNameType) -> io::Result<()> {
    let bms_info = get_dir_bms_info(work_dir)
        .await?
        .ok_or(io::Error::other("Bms file not found"))?;
    let title = bms_info.header.title.unwrap_or(DEFAULT_TITLE.to_string());
    let artist = bms_info.header.artist.unwrap_or(DEFAULT_ARTIST.to_string());
    let work_dir_name = work_dir
        .file_name()
        .ok_or(io::Error::other("Dir name not exists"))?
        .to_string_lossy();
    let target_dir_name = match set_type {
        BmsFolderSetNameType::ReplaceTitleArtist => format!("{title} [{artist}]"),
        BmsFolderSetNameType::AppendTitleArtist => format!("{work_dir_name} {title} [{artist}]"),
        BmsFolderSetNameType::AppendArtist => format!("{work_dir_name} [{artist}]"),
    };
    let target_work_dir = work_dir
        .parent()
        .ok_or(io::Error::other("Dir name not exists"))?
        .join(target_dir_name);
    fs::DirBuilder::new()
        .recursive(true)
        .create(&target_work_dir)
        .await?;
    move_elements_across_dir(
        work_dir,
        target_work_dir,
        Default::default(),
        replace_options_update_pack(),
    )
    .await?;
    Ok(())
}

pub async fn undo_set_name(work_dir: &Path, set_type: BmsFolderSetNameType) -> io::Result<()> {
    let work_dir_name = work_dir
        .file_name()
        .ok_or(io::Error::other("Dir name not exists"))?
        .to_string_lossy();
    let new_dir_name = match set_type {
        BmsFolderSetNameType::ReplaceTitleArtist => {
            work_dir_name.split(" ").next().unwrap_or(&work_dir_name)
        }
        BmsFolderSetNameType::AppendTitleArtist => {
            work_dir_name.split(" ").next().unwrap_or(&work_dir_name)
        }
        BmsFolderSetNameType::AppendArtist => {
            work_dir_name.split(" ").next().unwrap_or(&work_dir_name)
        }
    };
    let new_dir_path = work_dir
        .parent()
        .ok_or(io::Error::other("Dir name not exists"))?
        .join(new_dir_name);
    fs::rename(work_dir, new_dir_path).await?;
    Ok(())
}

/// 删除 work_dir 及其子目录中所有 0 字节文件（循环版，smol 2）。
pub async fn remove_zero_sized_media_files(work_dir: impl AsRef<Path>) -> io::Result<()> {
    let mut stack = VecDeque::new();
    stack.push_back(work_dir.as_ref().to_path_buf());

    // 存放异步删除任务
    let mut tasks = Vec::new();

    while let Some(dir) = stack.pop_back() {
        let mut entries = fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let path = entry.path();
            let meta = entry.metadata().await?;

            if meta.is_file() && meta.len() == 0 {
                // 异步删除，任务句柄放进 Vec
                tasks.push(smol::spawn(async move {
                    fs::remove_file(&path).await?;
                    Ok::<(), io::Error>(())
                }));
            } else if meta.is_dir() {
                // 继续压栈
                stack.push_back(path);
            }
        }
    }

    // 等待所有删除任务完成
    for task in tasks {
        task.await?;
    }

    Ok(())
}
