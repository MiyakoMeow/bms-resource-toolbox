use std::path::Path;

use smol::{fs, io};

use crate::{
    bms::get_dir_bms_info,
    fs::moving::{
        move_elements_across_dir, replace_options_update_pack
    },
};

pub const DEFAULT_TITLE: &str = "!!! UnknownTitle !!!";
pub const DEFAULT_ARTIST: &str = "!!! UnknownArtist !!!";

/// 该脚本适用于希望在作品文件夹名后添加“标题 [艺术家]”的情况
pub async fn set_name_by_bms(work_dir: &Path) -> io::Result<()> {
    let bms_info = get_dir_bms_info(work_dir)
        .await?
        .ok_or(io::Error::other("Bms file not found"))?;
    let title = bms_info.header.title.unwrap_or(DEFAULT_TITLE.to_string());
    let artist = bms_info.header.artist.unwrap_or(DEFAULT_ARTIST.to_string());
    let target_dir_name = format!("{title} [{artist}]");
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

/// 该脚本适用于希望在作品文件夹名后添加“ [艺术家]”的情况
pub async fn append_artist_by_bms(work_dir: &Path) -> io::Result<()> {
    let bms_info = get_dir_bms_info(work_dir)
        .await?
        .ok_or(io::Error::other("Bms file not found"))?;
    let work_dir_name = work_dir
        .file_name()
        .ok_or(io::Error::other("Dir name not exists"))?
        .to_string_lossy();
    let title = work_dir_name;
    let artist = bms_info.header.artist.unwrap_or(DEFAULT_ARTIST.to_string());
    let target_dir_name = format!("{title} [{artist}]");
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

/// 该脚本适用于希望在作品文件夹名后添加“标题 [艺术家]”的情况
pub async fn append_name_by_bms(work_dir: &Path) -> io::Result<()> {
    let bms_info = get_dir_bms_info(work_dir)
        .await?
        .ok_or(io::Error::other("Bms file not found"))?;
    let work_dir_name = work_dir
        .file_name()
        .ok_or(io::Error::other("Dir name not exists"))?
        .to_string_lossy();
    let title = bms_info.header.title.unwrap_or(DEFAULT_TITLE.to_string());
    let artist = bms_info.header.artist.unwrap_or(DEFAULT_ARTIST.to_string());
    let target_dir_name = format!("{work_dir_name} {title} [{artist}]");
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
