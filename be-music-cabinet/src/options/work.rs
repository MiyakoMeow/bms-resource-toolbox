use std::{collections::VecDeque, path::Path};

use smol::{fs, io, stream::StreamExt};

use crate::{
    bms::get_dir_bms_info,
    fs::{
        get_vaild_fs_name,
        moving::{move_elements_across_dir, replace_options_update_pack},
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

/// This script is suitable for cases where you want to append "Title [Artist]" after work folder name
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
    let target_dir_name = get_vaild_fs_name(&target_dir_name);
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

/// Remove all 0-byte files in work_dir and its subdirectories (loop version, smol 2).
pub async fn remove_zero_sized_media_files(work_dir: impl AsRef<Path>) -> io::Result<()> {
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
                tasks.push(smol::spawn(async move {
                    fs::remove_file(&path).await?;
                    Ok::<(), io::Error>(())
                }));
            } else if meta.is_dir() {
                // Continue pushing to stack
                stack.push_back(path);
            }
        }
    }

    // Wait for all deletion tasks to complete
    for task in tasks {
        task.await?;
    }

    Ok(())
}
