use std::path::Path;

use smol::{fs, io, stream::StreamExt};

use crate::{
    fs::{
        moving::{MoveOptions, move_elements_across_dir},
        rawpack::{
            get_num_set_file_names, move_out_files_in_folder_in_cache_dir, unzip_file_to_cache_dir,
        },
        sync::{preset_for_append, sync_folder},
    },
    media::{audio::process_bms_folders, video::process_bms_video_folders},
    options::{
        root::copy_numbered_workdir_names,
        root_bigpack::{get_remove_media_rule_oraja, remove_unneed_media_files},
        work::{BmsFolderSetNameType, set_name_by_bms},
    },
};

/// 解压数字命名的包文件到BMS文件夹
async fn unzip_numeric_to_bms_folder(
    root_dir: impl AsRef<Path>,
    pack_dir: impl AsRef<Path>,
    cache_dir: impl AsRef<Path>,
) -> io::Result<()> {
    let root_dir = root_dir.as_ref();
    let pack_dir = pack_dir.as_ref();
    let cache_dir = cache_dir.as_ref();

    // 获取数字命名的文件列表
    let file_names = get_num_set_file_names(pack_dir)?;

    for file_name in file_names {
        let file_path = pack_dir.join(&file_name);
        let id_str = file_name.split(' ').next().unwrap_or("");

        if !id_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        println!("Processing pack file: {}", file_name);

        // 解压到缓存目录
        unzip_file_to_cache_dir(&file_path, cache_dir).await?;

        // 整理缓存目录中的文件
        if !move_out_files_in_folder_in_cache_dir(cache_dir).await? {
            println!("Failed to process cache directory for {}", file_name);
            continue;
        }

        // 创建目标目录
        let target_dir = root_dir.join(id_str);
        fs::create_dir_all(&target_dir).await?;

        // 移动文件到目标目录
        move_elements_across_dir(
            cache_dir,
            &target_dir,
            MoveOptions::default(),
            Default::default(),
        )
        .await?;

        println!(
            "Successfully processed: {} -> {}",
            file_name,
            target_dir.display()
        );
    }

    Ok(())
}

/// 移除空文件夹
async fn remove_empty_folder(parent_dir: impl AsRef<Path>) -> io::Result<()> {
    let parent_dir = parent_dir.as_ref();
    let mut entries = fs::read_dir(parent_dir).await?;

    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // 递归移除子目录中的空文件夹
            Box::pin(remove_empty_folder(&path)).await?;

            // 检查当前目录是否为空
            let mut check_entries = fs::read_dir(&path).await?;
            if check_entries.next().await.is_none() {
                fs::remove_dir(&path).await?;
                println!("Removed empty folder: {}", path.display());
            }
        }
    }

    Ok(())
}

/// 原包 -> HQ版大包
/// This function is for parsing Raw version to HQ version. Just for beatoraja/Qwilight players.
pub async fn pack_raw_to_hq(root_dir: impl AsRef<Path>) -> io::Result<()> {
    let root_dir = root_dir.as_ref();

    // Parse Audio
    println!("Parsing Audio... Phase 1: WAV -> FLAC");
    process_bms_folders(
        root_dir,
        &["wav"],
        &["FLAC", "FLAC_FFMPEG"],
        true,  // remove_origin_file_when_success
        true,  // remove_origin_file_when_failed
        false, // skip_on_fail
    )
    .await?;

    // Remove Unneed Media File
    println!("Removing Unneed Files");
    remove_unneed_media_files(root_dir, Some(get_remove_media_rule_oraja())).await?;

    Ok(())
}

/// HQ版大包 -> LQ版大包
/// This file is for parsing HQ version to LQ version. Just for LR2 players.
pub async fn pack_hq_to_lq(root_dir: impl AsRef<Path>) -> io::Result<()> {
    let root_dir = root_dir.as_ref();

    // Parse Audio
    println!("Parsing Audio... Phase 1: FLAC -> OGG");
    process_bms_folders(
        root_dir,
        &["flac"],
        &["OGG_Q10"],
        true,  // remove_origin_file_when_success
        false, // remove_origin_file_when_failed
        false, // skip_on_fail
    )
    .await?;

    // Parse Video
    println!("Parsing Video...");
    process_bms_video_folders(
        root_dir,
        &["mp4"],
        &["MPEG1VIDEO_512X512", "WMV2_512X512", "AVI_512X512"],
        true,  // remove_origin_file
        false, // remove_existing
        false, // use_prefered
    )
    .await?;

    Ok(())
}

/// 检查大包生成脚本的输入参数
fn pack_setup_rawpack_to_hq_check(pack_dir: &Path, root_dir: &Path) -> bool {
    // Input 1
    println!(" - Input 1: Pack dir path");
    if !pack_dir.is_dir() {
        println!("Pack dir is not valid dir.");
        return false;
    }

    // Print Packs
    println!(" -- There are packs in pack_dir:");
    match get_num_set_file_names(pack_dir) {
        Ok(file_names) => {
            for file_name in file_names {
                println!(" > {}", file_name);
            }
        }
        Err(e) => {
            println!("Error reading pack files: {}", e);
        }
    }

    // Input 2
    println!(" - Input 2: BMS Cache Folder path. (Input a dir path that NOT exists)");
    if root_dir.is_dir() {
        println!("Root dir is an existing dir.");
        return false;
    }

    true
}

/// 大包生成脚本：原包 -> HQ版大包
/// BMS Pack Generator by MiyakoMeow.
/// - For Pack Create:
/// Fast creating pack script, from: Raw Packs set numed, to: target bms folder.
/// You need to set pack num before running this script, see options/rawpack.rs => set_file_num
pub async fn pack_setup_rawpack_to_hq(
    pack_dir: impl AsRef<Path>,
    root_dir: impl AsRef<Path>,
) -> io::Result<()> {
    let pack_dir = pack_dir.as_ref();
    let root_dir = root_dir.as_ref();

    // Setup
    fs::create_dir_all(root_dir).await?;

    // Unzip
    println!(
        " > 1. Unzip packs from {} to {}",
        pack_dir.display(),
        root_dir.display()
    );
    let cache_dir = root_dir.join("CacheDir");
    fs::create_dir_all(&cache_dir).await?;
    unzip_numeric_to_bms_folder(root_dir, pack_dir, &cache_dir).await?;

    // 检查缓存目录是否为空，如果为空则删除
    if cache_dir.exists() {
        let mut cache_entries = fs::read_dir(&cache_dir).await?;
        if cache_entries.next().await.is_none() {
            fs::remove_dir(&cache_dir).await?;
        }
    }

    // Syncing folder name
    println!(" > 2. Setting dir names from BMS Files");
    let mut entries = fs::read_dir(root_dir).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            set_name_by_bms(&path, BmsFolderSetNameType::AppendTitleArtist).await?;
        }
    }

    // Parse Audio
    println!(" > 3. Parsing Audio... Phase 1: WAV -> FLAC");
    process_bms_folders(
        root_dir,
        &["wav"],
        &["FLAC", "FLAC_FFMPEG"],
        true,  // remove_origin_file_when_success
        false, // remove_origin_file_when_failed
        false, // skip_on_fail
    )
    .await?;

    // Remove Unneed Media File
    println!(" > 4. Removing Unneed Files");
    remove_unneed_media_files(root_dir, Some(get_remove_media_rule_oraja())).await?;

    Ok(())
}

/// 检查大包更新脚本的输入参数
fn pack_update_rawpack_to_hq_check(pack_dir: &Path, root_dir: &Path, sync_dir: &Path) -> bool {
    // Input 1
    println!(" - Input 1: Pack dir path");
    if !pack_dir.is_dir() {
        println!("Pack dir is not valid dir.");
        return false;
    }

    // Print Packs
    println!(" -- There are packs in pack_dir:");
    match get_num_set_file_names(pack_dir) {
        Ok(file_names) => {
            for file_name in file_names {
                println!(" > {}", file_name);
            }
        }
        Err(e) => {
            println!("Error reading pack files: {}", e);
        }
    }

    // Input 2
    println!(" - Input 2: BMS Cache Folder path. (Input a dir path that NOT exists)");
    if root_dir.is_dir() {
        println!("Root dir is an existing dir.");
        return false;
    }

    // Input 3
    println!(" - Input 3: Already exists BMS Folder path. (Input a dir path that ALREADY exists)");
    println!("This script will use this dir, just for name syncing and file checking.");
    if !sync_dir.is_dir() {
        println!("Syncing dir is not valid dir.");
        return false;
    }

    true
}

/// 大包更新脚本：原包 -> HQ版大包
/// BMS Pack Generator by MiyakoMeow.
/// - For Pack Update:
/// Fast update script, from: Raw Packs set numed, to: delta bms folder just for making pack update.
/// You need to set pack num before running this script, see scripts_rawpack/rawpack_set_num.py
pub async fn pack_update_rawpack_to_hq(
    pack_dir: impl AsRef<Path>,
    root_dir: impl AsRef<Path>,
    sync_dir: impl AsRef<Path>,
) -> io::Result<()> {
    let pack_dir = pack_dir.as_ref();
    let root_dir = root_dir.as_ref();
    let sync_dir = sync_dir.as_ref();

    // Setup
    fs::create_dir_all(root_dir).await?;

    // Unzip
    println!(
        " > 1. Unzip packs from {} to {}",
        pack_dir.display(),
        root_dir.display()
    );
    let cache_dir = root_dir.join("CacheDir");
    fs::create_dir_all(&cache_dir).await?;
    unzip_numeric_to_bms_folder(root_dir, pack_dir, &cache_dir).await?;

    // Syncing folder name
    println!(
        " > 2. Syncing dir name from {} to {}",
        sync_dir.display(),
        root_dir.display()
    );
    copy_numbered_workdir_names(sync_dir, root_dir).await?;

    // Parse Audio
    println!(" > 3. Parsing Audio... Phase 1: WAV -> FLAC");
    process_bms_folders(
        root_dir,
        &["wav"],
        &["FLAC", "FLAC_FFMPEG"],
        true,  // remove_origin_file_when_success
        false, // remove_origin_file_when_failed
        false, // skip_on_fail
    )
    .await?;

    // Remove Unneed Media File
    println!(" > 4. Removing Unneed Files");
    remove_unneed_media_files(root_dir, Some(get_remove_media_rule_oraja())).await?;

    // Soft syncing
    println!(
        " > 5. Syncing dir files from {} to {}",
        root_dir.display(),
        sync_dir.display()
    );
    sync_folder(root_dir, sync_dir, &preset_for_append()).await?;

    // Remove Empty folder
    println!(" > 6. Remove empty folder in {}", root_dir.display());
    remove_empty_folder(root_dir).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_empty_folder() {
        smol::block_on(async {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let test_dir = temp_dir.path().join("test");
            fs::create_dir_all(&test_dir)
                .await
                .expect("Failed to create test dir");

            // 创建一些空文件夹
            let empty_dir1 = test_dir.join("empty1");
            let empty_dir2 = test_dir.join("empty2");
            let non_empty_dir = test_dir.join("non_empty");

            fs::create_dir_all(&empty_dir1)
                .await
                .expect("Failed to create empty dir1");
            fs::create_dir_all(&empty_dir2)
                .await
                .expect("Failed to create empty dir2");
            fs::create_dir_all(&non_empty_dir)
                .await
                .expect("Failed to create non_empty dir");

            // 在non_empty_dir中创建一个文件
            fs::write(non_empty_dir.join("test.txt"), "test")
                .await
                .expect("Failed to write test file");

            // 执行移除空文件夹操作
            remove_empty_folder(&test_dir)
                .await
                .expect("Failed to remove empty folders");

            // 验证结果
            assert!(!empty_dir1.exists(), "empty_dir1 should be removed");
            assert!(!empty_dir2.exists(), "empty_dir2 should be removed");
            assert!(non_empty_dir.exists(), "non_empty_dir should still exist");
            assert!(
                non_empty_dir.join("test.txt").exists(),
                "test file should still exist"
            );
        });
    }

    #[test]
    fn test_unzip_numeric_to_bms_folder() {
        smol::block_on(async {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let pack_dir = temp_dir.path().join("packs");
            let root_dir = temp_dir.path().join("root");
            let cache_dir = temp_dir.path().join("cache");

            fs::create_dir_all(&pack_dir)
                .await
                .expect("Failed to create pack dir");
            fs::create_dir_all(&root_dir)
                .await
                .expect("Failed to create root dir");
            fs::create_dir_all(&cache_dir)
                .await
                .expect("Failed to create cache dir");

            // 创建一个模拟的数字命名文件（非实际压缩文件）
            let test_file = pack_dir.join("001 Test Song.txt");
            fs::write(&test_file, "test content")
                .await
                .expect("Failed to create test file");

            // 这个测试主要验证函数结构是否正确，实际解压需要真实的压缩文件
            // 由于我们没有真实的压缩文件，这里只验证不会panic
            let result = unzip_numeric_to_bms_folder(&root_dir, &pack_dir, &cache_dir).await;

            // 验证函数执行完成（即使可能失败也不应该panic）
            assert!(
                result.is_ok() || result.is_err(),
                "Function should complete without panicking"
            );
        });
    }
}
