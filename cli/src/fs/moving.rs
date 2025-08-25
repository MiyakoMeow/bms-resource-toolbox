use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
};

use smol::{
    fs,
    io::{self},
    lock::Mutex,
    stream::StreamExt,
};

use crate::bms::{BMS_FILE_EXTS, BMSON_FILE_EXTS};

use super::{is_dir_having_file, is_file_same_content, lock::acquire_disk_lock};
use log::{info, warn};

/// Same name enum as Python
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplaceAction {
    Skip = 0,
    #[default]
    Replace = 1,
    Rename = 2,
    /// Check content first before deciding
    CheckReplace = 12,
}

/// Replacement strategy
#[derive(Debug, Default, Clone)]
pub struct ReplaceOptions {
    /// Strategy specified by extension
    pub ext: HashMap<String, ReplaceAction>,
    /// Default strategy
    pub default: ReplaceAction,
}

impl ReplaceOptions {
    /// Get strategy for a specific file
    fn for_path(&self, path: &Path) -> ReplaceAction {
        path.extension()
            .and_then(|s| s.to_str())
            .and_then(|ext| self.ext.get(ext).copied())
            .unwrap_or(self.default)
    }
}

/// 预设的替换策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplacePreset {
    /// 与 ReplaceOptions::default() 等价
    Default = 0,
    /// 与 replace_options_update_pack() 等价
    UpdatePack = 1,
}

impl std::str::FromStr for ReplacePreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(ReplacePreset::Default),
            "update_pack" | "update-pack" => Ok(ReplacePreset::UpdatePack),
            _ => Err(format!(
                "Unknown preset: {}. Valid values: default, update_pack",
                s
            )),
        }
    }
}

impl clap::ValueEnum for ReplacePreset {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Default, Self::UpdatePack]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let name = match self {
            ReplacePreset::Default => "default",
            ReplacePreset::UpdatePack => "update_pack",
        };
        Some(clap::builder::PossibleValue::new(name))
    }
}

/// 从预设获取具体的 ReplaceOptions
pub fn replace_options_from_preset(preset: ReplacePreset) -> ReplaceOptions {
    match preset {
        ReplacePreset::Default => ReplaceOptions::default(),
        ReplacePreset::UpdatePack => replace_options_update_pack(),
    }
}

/// Default update pack strategy
pub fn replace_options_update_pack() -> ReplaceOptions {
    ReplaceOptions {
        ext: {
            BMS_FILE_EXTS
                .iter()
                .chain(BMSON_FILE_EXTS)
                .chain(&["txt"])
                .map(|ext| (ext.to_string(), ReplaceAction::CheckReplace))
                .collect()
        },
        default: ReplaceAction::Replace,
    }
}

/// Recursively move directory contents (using loops instead of recursion)
pub async fn move_elements_across_dir(
    dir_path_ori: impl AsRef<Path>,
    dir_path_dst: impl AsRef<Path>,
    replace_options: ReplaceOptions,
) -> io::Result<()> {
    let dir_path_ori = dir_path_ori.as_ref();
    let dir_path_dst = dir_path_dst.as_ref();
    if !dir_path_ori.exists() {
        return Ok(());
    }

    if dir_path_ori == dir_path_dst {
        return Ok(());
    }
    if !fs::metadata(&dir_path_ori).await?.is_dir() {
        return Ok(());
    }
    // If target directory doesn't exist, directly move the entire directory.
    // Do this check BEFORE creating the target directory, otherwise we'd
    // unnecessarily enumerate and move children one-by-one.
    match fs::metadata(&dir_path_dst).await {
        Err(_) => {
            fs::rename(&dir_path_ori, &dir_path_dst).await?;
            return Ok(());
        }
        Ok(m) if !m.is_dir() => {
            return Err(io::Error::other(
                "destination path exists and is not a directory",
            ));
        }
        Ok(_) => {}
    }

    // Use queue to manage directories to be processed
    let mut pending_dirs = VecDeque::new();
    pending_dirs.push_back((dir_path_ori.to_path_buf(), dir_path_dst.to_path_buf()));

    while let Some((current_ori, current_dst)) = pending_dirs.pop_front() {
        // Process current directory with adaptive concurrency
        let next_dirs = process_directory(&current_ori, &current_dst, &replace_options).await?;

        // Add newly discovered subdirectories to the queue
        for (ori, dst) in next_dirs {
            pending_dirs.push_back((ori, dst));
        }

        // Clean up empty directories
        if (replace_options.default != ReplaceAction::Skip
            || !is_dir_having_file(&current_ori).await?)
            && let Err(e) = fs::remove_dir_all(&current_ori).await
        {
            warn!(" x PermissionError! ({}) - {}", current_ori.display(), e);
        }
    }

    Ok(())
}

/// Process a single directory, return subdirectories that need further processing
async fn process_directory(
    dir_path_ori: &Path,
    dir_path_dst: &Path,
    replace_options: &ReplaceOptions,
) -> io::Result<Vec<(PathBuf, PathBuf)>> {
    // Collect entries to be processed (files / subdirectories)
    let mut entries = fs::read_dir(dir_path_ori).await?;
    let next_folder_paths = Arc::new(Mutex::new(Vec::new()));
    let mut paths: Vec<(PathBuf, PathBuf)> = Vec::new();

    while let Some(entry) = StreamExt::next(&mut entries).await {
        let entry = entry?;
        let src = entry.path();
        let dst = dir_path_dst.join(entry.file_name());
        paths.push((src, dst));
    }

    // Process all entries under current directory concurrently
    // Use disk locks to control concurrency
    let futures: Vec<_> = paths
        .into_iter()
        .map(|(src, dst)| {
            let rep = replace_options.clone();
            let next_folder_paths = Arc::clone(&next_folder_paths);
            async move {
                // Acquire disk lock for the source path
                let _lock_guard = acquire_disk_lock(&src).await;

                let next_folder_paths_cloned = Arc::clone(&next_folder_paths);
                let _ = move_action(&src, &dst, rep, move |ori: PathBuf, dst: PathBuf| {
                    let next_folder_paths = Arc::clone(&next_folder_paths_cloned);
                    smol::spawn(async move {
                        let mut next = next_folder_paths.lock_arc().await;
                        next.push((ori, dst));
                    })
                })
                .await;
            }
        })
        .collect();

    // Wait for all operations to complete
    futures::future::join_all(futures).await;

    // Return subdirectories that need further processing
    Ok(next_folder_paths.lock_arc().await.clone())
}

/// Entry point for moving a single file/directory
async fn move_action(
    src: &Path,
    dst: &Path,
    rep: ReplaceOptions,
    mut push_child: impl FnMut(PathBuf, PathBuf) -> smol::Task<()>,
) -> io::Result<()> {
    info!(" - Moving from {} to {}", src.display(), dst.display());

    let md = fs::metadata(&src).await?;
    if md.is_file() {
        move_file(src, dst, &rep).await?;
    } else if md.is_dir() {
        // If target directory doesn't exist, move directly
        if !fs::metadata(&dst).await.is_ok_and(|m| m.is_dir()) {
            fs::rename(src, dst).await?;
        } else {
            // Defer to next round of processing
            push_child(src.to_path_buf(), dst.to_path_buf()).await;
        }
    }
    Ok(())
}

/// Move a single file, handle conflicts according to strategy
async fn move_file(src: &Path, dst: &Path, rep: &ReplaceOptions) -> io::Result<()> {
    let action = rep.for_path(src);

    match action {
        ReplaceAction::Replace => fs::rename(src, dst).await,
        ReplaceAction::Skip => {
            if dst.exists() {
                return Ok(());
            }
            fs::rename(src, dst).await
        }
        ReplaceAction::Rename => move_file_rename(src, dst).await,
        ReplaceAction::CheckReplace => {
            if !dst.exists() {
                fs::rename(src, dst).await
            } else if is_file_same_content(src, dst).await? {
                // Same content, directly overwrite
                fs::rename(src, dst).await
            } else {
                move_file_rename(src, dst).await
            }
        }
    }
}

/// "Rename" move with retry
async fn move_file_rename(src: &Path, dst_dir: &Path) -> io::Result<()> {
    let mut dst = dst_dir.to_path_buf();
    let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("");

    let mut _count = 0;
    for i in std::iter::from_fn(|| {
        _count += 1;
        Some(_count)
    }) {
        let name = if i == 0 {
            format!("{stem}.{ext}")
        } else {
            format!("{stem}.{i}.{ext}")
        };
        dst.set_file_name(name);
        if !dst.exists() {
            fs::rename(src, &dst).await?;
            return Ok(());
        }
        if is_file_same_content(src, &dst).await? {
            // File with same name and content already exists, skip
            fs::remove_file(src).await?;
            return Ok(());
        }
    }
    Err(io::Error::other("too many duplicate files"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::{fs, io};
    use tempfile::{TempDir, tempdir};

    /// Create test directory structure
    async fn create_test_structure(base_dir: &Path) -> io::Result<()> {
        // Create subdirectory
        let sub_dir = base_dir.join("subdir");
        fs::create_dir_all(&sub_dir).await?;

        // Create files
        fs::write(base_dir.join("file1.txt"), "content1").await?;
        fs::write(base_dir.join("file2.bms"), "content2").await?;
        fs::write(sub_dir.join("file3.txt"), "content3").await?;

        // Create nested directory
        let nested_dir = sub_dir.join("nested");
        fs::create_dir_all(&nested_dir).await?;
        fs::write(nested_dir.join("file4.txt"), "content4").await?;

        Ok(())
    }

    /// Verify directory structure
    async fn verify_structure(dir: &Path, expected_files: &[(&str, &str)]) -> io::Result<()> {
        for (file_path, expected_content) in expected_files {
            let full_path = dir.join(file_path);
            assert!(
                full_path.exists(),
                "File does not exist: {}",
                full_path.display()
            );

            let content = fs::read_to_string(&full_path).await?;
            assert_eq!(
                &content,
                expected_content,
                "File content mismatch: {}",
                full_path.display()
            );
        }
        Ok(())
    }

    /// Clean up test directory
    async fn cleanup_test_dir(dir: &TempDir) {
        if let Err(e) = fs::remove_dir_all(dir.path()).await {
            eprintln!("Failed to clean up test directory: {e}");
        }
    }

    #[test]
    fn test_move_elements_across_dir_basic() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("Failed to create temp directory");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // Create source directory structure
            fs::create_dir_all(&src_dir)
                .await
                .expect("Failed to create source directory");
            create_test_structure(&src_dir)
                .await
                .expect("Failed to create test structure");

            // Execute move
            let replace_options = ReplaceOptions::default();

            move_elements_across_dir(&src_dir, &dst_dir, replace_options)
                .await
                .expect("Move operation failed");

            // Verify result
            let expected_files = [
                ("file1.txt", "content1"),
                ("file2.bms", "content2"),
                ("subdir/file3.txt", "content3"),
                ("subdir/nested/file4.txt", "content4"),
            ];

            verify_structure(&dst_dir, &expected_files)
                .await
                .expect("Failed to verify structure");

            // Verify source directory has been cleaned up
            assert!(!src_dir.exists(), "Source directory should be deleted");

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_skip_existing() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("Failed to create temp directory");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // Create source directory structure
            fs::create_dir_all(&src_dir)
                .await
                .expect("Failed to create source directory");
            create_test_structure(&src_dir)
                .await
                .expect("Failed to create test structure");

            // Create file with same name in target directory
            fs::create_dir_all(&dst_dir)
                .await
                .expect("Failed to create target directory");
            fs::write(dst_dir.join("file1.txt"), "existing_content")
                .await
                .expect("Failed to create file");

            // Use Skip strategy
            let replace_options = ReplaceOptions {
                default: ReplaceAction::Skip,
                ..Default::default()
            };

            move_elements_across_dir(&src_dir, &dst_dir, replace_options)
                .await
                .expect("Move operation failed");

            // Verify target file keeps original content
            let content = fs::read_to_string(dst_dir.join("file1.txt"))
                .await
                .expect("Failed to read file");
            assert_eq!(
                content, "existing_content",
                "File content should remain unchanged"
            );

            // Verify other files were moved
            assert!(dst_dir.join("file2.bms").exists());
            assert!(dst_dir.join("subdir").exists());

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_rename_conflict() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("Failed to create temp directory");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // Create source directory structure
            fs::create_dir_all(&src_dir)
                .await
                .expect("Failed to create source directory");
            fs::write(src_dir.join("file1.txt"), "new_content")
                .await
                .expect("Failed to create file");

            // Create file with same name in target directory
            fs::create_dir_all(&dst_dir)
                .await
                .expect("Failed to create target directory");
            fs::write(dst_dir.join("file1.txt"), "existing_content")
                .await
                .expect("Failed to create file");

            // Use Rename strategy
            let replace_options = ReplaceOptions {
                default: ReplaceAction::Rename,
                ..Default::default()
            };

            move_elements_across_dir(&src_dir, &dst_dir, replace_options)
                .await
                .expect("Move operation failed");

            // Verify original file exists
            let content = fs::read_to_string(dst_dir.join("file1.txt"))
                .await
                .expect("Failed to read file");
            assert_eq!(
                content, "existing_content",
                "Original file should remain unchanged"
            );

            // Verify new file was renamed
            assert!(
                dst_dir.join("file1.1.txt").exists(),
                "Should create renamed file"
            );

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_check_replace() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("Failed to create temp directory");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // Create source directory structure
            fs::create_dir_all(&src_dir)
                .await
                .expect("Failed to create source directory");
            fs::write(src_dir.join("file1.txt"), "same_content")
                .await
                .expect("Failed to create file");

            // Create file with same name in target directory, same content
            fs::create_dir_all(&dst_dir)
                .await
                .expect("Failed to create target directory");
            fs::write(dst_dir.join("file1.txt"), "same_content")
                .await
                .expect("Failed to create file");

            // Use CheckReplace strategy
            let replace_options = ReplaceOptions {
                default: ReplaceAction::CheckReplace,
                ..Default::default()
            };

            move_elements_across_dir(&src_dir, &dst_dir, replace_options)
                .await
                .expect("Move operation failed");

            // Verify file was overwritten (because content is the same)
            let content = fs::read_to_string(dst_dir.join("file1.txt"))
                .await
                .expect("Failed to read file");
            assert_eq!(
                content, "same_content",
                "File content should remain unchanged"
            );

            // Verify source directory was cleaned up
            assert!(!src_dir.exists(), "Source directory should be deleted");

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_same_directory() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("Failed to create temp directory");
            let src_dir = temp_dir.path().join("src");

            // Create source directory structure
            fs::create_dir_all(&src_dir)
                .await
                .expect("Failed to create source directory");
            create_test_structure(&src_dir)
                .await
                .expect("Failed to create test structure");

            // Try to move to the same directory
            let replace_options = ReplaceOptions::default();
            let result = move_elements_across_dir(&src_dir, &src_dir, replace_options).await;
            assert!(result.is_ok(), "Moving to same directory should succeed");

            // Verify directory structure remains unchanged
            assert!(src_dir.exists(), "Source directory should still exist");
            assert!(
                src_dir.join("file1.txt").exists(),
                "File should still exist"
            );

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_nonexistent_source() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("Failed to create temp directory");
            let src_dir = temp_dir.path().join("nonexistent");
            let dst_dir = temp_dir.path().join("dst");

            let replace_options = ReplaceOptions::default();
            let result = move_elements_across_dir(&src_dir, &dst_dir, replace_options).await;
            assert!(
                result.is_ok(),
                "Moving non-existent directory should succeed (no operation)"
            );

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_with_ext_specific_rules() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("Failed to create temp directory");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // Create source directory structure
            fs::create_dir_all(&src_dir)
                .await
                .expect("Failed to create source directory");
            fs::write(src_dir.join("file1.txt"), "content1")
                .await
                .expect("Failed to create file");
            fs::write(src_dir.join("file2.bms"), "content2")
                .await
                .expect("Failed to create file");
            fs::write(src_dir.join("file3.other"), "content3")
                .await
                .expect("Failed to create file");

            // Create conflicting files in target directory
            fs::create_dir_all(&dst_dir)
                .await
                .expect("Failed to create target directory");
            fs::write(dst_dir.join("file1.txt"), "existing_txt")
                .await
                .expect("Failed to create file");
            fs::write(dst_dir.join("file2.bms"), "existing_bms")
                .await
                .expect("Failed to create file");
            fs::write(dst_dir.join("file3.other"), "existing_other")
                .await
                .expect("Failed to create file");

            // Use specific extension rules
            let mut replace_options = ReplaceOptions::default();
            replace_options
                .ext
                .insert("txt".to_string(), ReplaceAction::Skip);
            replace_options
                .ext
                .insert("bms".to_string(), ReplaceAction::Rename);
            replace_options.default = ReplaceAction::Replace;

            move_elements_across_dir(&src_dir, &dst_dir, replace_options)
                .await
                .expect("Move operation failed");

            // Verify txt file was skipped
            let content = fs::read_to_string(dst_dir.join("file1.txt"))
                .await
                .expect("Failed to read file");
            assert_eq!(content, "existing_txt", "txt file should be skipped");

            // Verify bms file was renamed
            assert!(
                dst_dir.join("file2.1.bms").exists(),
                "bms file should be renamed"
            );

            // Verify other file was replaced
            let content = fs::read_to_string(dst_dir.join("file3.other"))
                .await
                .expect("Failed to read file");
            assert_eq!(content, "content3", "other file should be replaced");

            cleanup_test_dir(&temp_dir).await;
        })
    }
}
