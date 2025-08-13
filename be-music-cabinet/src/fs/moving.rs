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

use super::{is_dir_having_file, is_file_same_content};

/// 与 Python 同名枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplaceAction {
    Skip = 0,
    #[default]
    Replace = 1,
    Rename = 2,
    /// 先比较内容再决定
    CheckReplace = 12,
}

/// 替换策略
#[derive(Debug, Default, Clone)]
pub struct ReplaceOptions {
    /// 按扩展名指定策略
    pub ext: HashMap<String, ReplaceAction>,
    /// 默认策略
    pub default: ReplaceAction,
}

impl ReplaceOptions {
    /// 获得某文件对应的策略
    fn for_path(&self, path: &Path) -> ReplaceAction {
        path.extension()
            .and_then(|s| s.to_str())
            .and_then(|ext| self.ext.get(ext).copied())
            .unwrap_or(self.default)
    }
}

/// 默认更新包策略
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

/// 移动选项
#[derive(Debug, Default, Clone, Copy)]
pub struct MoveOptions {
    pub print_info: bool,
}

/// 递归移动目录内容（使用循环而非递归）
pub async fn move_elements_across_dir(
    dir_path_ori: impl AsRef<Path>,
    dir_path_dst: impl AsRef<Path>,
    options: MoveOptions,
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
    if !fs::metadata(&dir_path_dst).await.is_ok_and(|m| m.is_dir()) {
        fs::create_dir_all(&dir_path_dst).await?;
    }

    // 如果目标目录不存在，直接 mv 整个目录
    if !fs::metadata(&dir_path_dst).await.is_ok_and(|m| m.is_dir()) {
        fs::rename(&dir_path_ori, &dir_path_dst).await?;
        return Ok(());
    }

    // 使用队列管理待处理的目录
    let mut pending_dirs = VecDeque::new();
    pending_dirs.push_back((dir_path_ori.to_path_buf(), dir_path_dst.to_path_buf()));

    while let Some((current_ori, current_dst)) = pending_dirs.pop_front() {
        // 处理当前目录
        let next_dirs =
            process_directory(&current_ori, &current_dst, options, &replace_options).await?;

        // 将新发现的子目录添加到队列中
        for (ori, dst) in next_dirs {
            pending_dirs.push_back((ori, dst));
        }

        // 清理空目录
        if (replace_options.default != ReplaceAction::Skip
            || !is_dir_having_file(&current_ori).await?)
            && let Err(e) = fs::remove_dir_all(&current_ori).await
        {
            eprintln!(" x PermissionError! ({}) - {}", current_ori.display(), e);
        }
    }

    Ok(())
}

/// 处理单个目录，返回需要进一步处理的子目录
async fn process_directory(
    dir_path_ori: &Path,
    dir_path_dst: &Path,
    options: MoveOptions,
    replace_options: &ReplaceOptions,
) -> io::Result<Vec<(PathBuf, PathBuf)>> {
    // 收集待处理条目（文件 / 子目录）
    let mut entries = fs::read_dir(dir_path_ori).await?;
    let mut tasks = Vec::new();
    let next_folder_paths = Arc::new(Mutex::new(Vec::new()));

    while let Some(entry) = StreamExt::next(&mut entries).await {
        let entry = entry?;
        let src = entry.path();
        let dst = dir_path_dst.join(entry.file_name());

        let rep = replace_options.clone();
        let next_folder_paths = Arc::clone(&next_folder_paths);

        tasks.push(smol::spawn(async move {
            let next_folder_paths = Arc::clone(&next_folder_paths);
            move_action(
                &src,
                &dst,
                options,
                rep,
                move |ori: PathBuf, dst: PathBuf| {
                    let next_folder_paths = Arc::clone(&next_folder_paths);
                    smol::spawn(async move {
                        let mut next = next_folder_paths.lock_arc().await;
                        next.push((ori, dst));
                    })
                },
            )
            .await
        }));
    }

    // 等待所有任务完成
    for t in tasks {
        t.await?;
    }

    // 返回需要进一步处理的子目录
    Ok(next_folder_paths.lock_arc().await.clone())
}

/// 单个文件/目录的移动入口
async fn move_action(
    src: &Path,
    dst: &Path,
    options: MoveOptions,
    rep: ReplaceOptions,
    mut push_child: impl FnMut(PathBuf, PathBuf) -> smol::Task<()>,
) -> io::Result<()> {
    if options.print_info {
        println!(" - Moving from {} to {}", src.display(), dst.display());
    }

    let md = fs::metadata(&src).await?;
    if md.is_file() {
        move_file(src, dst, &rep).await?;
    } else if md.is_dir() {
        // 如果目标目录不存在，直接移动
        if !fs::metadata(&dst).await.is_ok_and(|m| m.is_dir()) {
            fs::rename(src, dst).await?;
        } else {
            // 推迟到下一轮处理
            push_child(src.to_path_buf(), dst.to_path_buf()).await;
        }
    }
    Ok(())
}

/// 移动单个文件，根据策略处理冲突
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
                // 内容相同直接覆盖
                fs::rename(src, dst).await
            } else {
                move_file_rename(src, dst).await
            }
        }
    }
}

/// 带重试的"重命名"移动
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
            // 已存在同名且内容相同，跳过
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

    /// 创建测试目录结构
    async fn create_test_structure(base_dir: &Path) -> io::Result<()> {
        // 创建子目录
        let sub_dir = base_dir.join("subdir");
        fs::create_dir_all(&sub_dir).await?;

        // 创建文件
        fs::write(base_dir.join("file1.txt"), "content1").await?;
        fs::write(base_dir.join("file2.bms"), "content2").await?;
        fs::write(sub_dir.join("file3.txt"), "content3").await?;

        // 创建嵌套目录
        let nested_dir = sub_dir.join("nested");
        fs::create_dir_all(&nested_dir).await?;
        fs::write(nested_dir.join("file4.txt"), "content4").await?;

        Ok(())
    }

    /// 验证目录结构
    async fn verify_structure(dir: &Path, expected_files: &[(&str, &str)]) -> io::Result<()> {
        for (file_path, expected_content) in expected_files {
            let full_path = dir.join(file_path);
            assert!(full_path.exists(), "文件不存在: {}", full_path.display());

            let content = fs::read_to_string(&full_path).await?;
            assert_eq!(
                &content,
                expected_content,
                "文件内容不匹配: {}",
                full_path.display()
            );
        }
        Ok(())
    }

    /// 清理测试目录
    async fn cleanup_test_dir(dir: &TempDir) {
        if let Err(e) = fs::remove_dir_all(dir.path()).await {
            eprintln!("清理测试目录失败: {e}");
        }
    }

    #[test]
    fn test_move_elements_across_dir_basic() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("创建临时目录失败");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // 创建源目录结构
            fs::create_dir_all(&src_dir).await.expect("创建源目录失败");
            create_test_structure(&src_dir)
                .await
                .expect("创建测试结构失败");

            // 执行移动
            let options = MoveOptions { print_info: false };
            let replace_options = ReplaceOptions::default();

            move_elements_across_dir(&src_dir, &dst_dir, options, replace_options)
                .await
                .expect("移动操作失败");

            // 验证结果
            let expected_files = [
                ("file1.txt", "content1"),
                ("file2.bms", "content2"),
                ("subdir/file3.txt", "content3"),
                ("subdir/nested/file4.txt", "content4"),
            ];

            verify_structure(&dst_dir, &expected_files)
                .await
                .expect("验证结构失败");

            // 验证源目录已被清理
            assert!(!src_dir.exists(), "源目录应该被删除");

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_skip_existing() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("创建临时目录失败");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // 创建源目录结构
            fs::create_dir_all(&src_dir).await.expect("创建源目录失败");
            create_test_structure(&src_dir)
                .await
                .expect("创建测试结构失败");

            // 在目标目录创建同名文件
            fs::create_dir_all(&dst_dir)
                .await
                .expect("创建目标目录失败");
            fs::write(dst_dir.join("file1.txt"), "existing_content")
                .await
                .expect("创建文件失败");

            // 使用 Skip 策略
            let options = MoveOptions { print_info: false };
            let replace_options = ReplaceOptions {
                default: ReplaceAction::Skip,
                ..Default::default()
            };

            move_elements_across_dir(&src_dir, &dst_dir, options, replace_options)
                .await
                .expect("移动操作失败");

            // 验证目标文件保持原内容
            let content = fs::read_to_string(dst_dir.join("file1.txt"))
                .await
                .expect("读取文件失败");
            assert_eq!(content, "existing_content", "文件内容应该保持不变");

            // 验证其他文件被移动
            assert!(dst_dir.join("file2.bms").exists());
            assert!(dst_dir.join("subdir").exists());

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_rename_conflict() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("创建临时目录失败");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // 创建源目录结构
            fs::create_dir_all(&src_dir).await.expect("创建源目录失败");
            fs::write(src_dir.join("file1.txt"), "new_content")
                .await
                .expect("创建文件失败");

            // 在目标目录创建同名文件
            fs::create_dir_all(&dst_dir)
                .await
                .expect("创建目标目录失败");
            fs::write(dst_dir.join("file1.txt"), "existing_content")
                .await
                .expect("创建文件失败");

            // 使用 Rename 策略
            let options = MoveOptions { print_info: false };
            let replace_options = ReplaceOptions {
                default: ReplaceAction::Rename,
                ..Default::default()
            };

            move_elements_across_dir(&src_dir, &dst_dir, options, replace_options)
                .await
                .expect("移动操作失败");

            // 验证原文件存在
            let content = fs::read_to_string(dst_dir.join("file1.txt"))
                .await
                .expect("读取文件失败");
            assert_eq!(content, "existing_content", "原文件应该保持不变");

            // 验证新文件被重命名
            assert!(dst_dir.join("file1.1.txt").exists(), "应该创建重命名文件");

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_check_replace() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("创建临时目录失败");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // 创建源目录结构
            fs::create_dir_all(&src_dir).await.expect("创建源目录失败");
            fs::write(src_dir.join("file1.txt"), "same_content")
                .await
                .expect("创建文件失败");

            // 在目标目录创建同名文件，内容相同
            fs::create_dir_all(&dst_dir)
                .await
                .expect("创建目标目录失败");
            fs::write(dst_dir.join("file1.txt"), "same_content")
                .await
                .expect("创建文件失败");

            // 使用 CheckReplace 策略
            let options = MoveOptions { print_info: false };
            let replace_options = ReplaceOptions {
                default: ReplaceAction::CheckReplace,
                ..Default::default()
            };

            move_elements_across_dir(&src_dir, &dst_dir, options, replace_options)
                .await
                .expect("移动操作失败");

            // 验证文件被覆盖（因为内容相同）
            let content = fs::read_to_string(dst_dir.join("file1.txt"))
                .await
                .expect("读取文件失败");
            assert_eq!(content, "same_content", "文件内容应该保持不变");

            // 验证源目录被清理
            assert!(!src_dir.exists(), "源目录应该被删除");

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_same_directory() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("创建临时目录失败");
            let src_dir = temp_dir.path().join("src");

            // 创建源目录结构
            fs::create_dir_all(&src_dir).await.expect("创建源目录失败");
            create_test_structure(&src_dir)
                .await
                .expect("创建测试结构失败");

            // 尝试移动到同一目录
            let options = MoveOptions { print_info: false };
            let replace_options = ReplaceOptions::default();

            let result =
                move_elements_across_dir(&src_dir, &src_dir, options, replace_options).await;
            assert!(result.is_ok(), "移动到同一目录应该成功");

            // 验证目录结构保持不变
            assert!(src_dir.exists(), "源目录应该仍然存在");
            assert!(src_dir.join("file1.txt").exists(), "文件应该仍然存在");

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_nonexistent_source() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("创建临时目录失败");
            let src_dir = temp_dir.path().join("nonexistent");
            let dst_dir = temp_dir.path().join("dst");

            let options = MoveOptions { print_info: false };
            let replace_options = ReplaceOptions::default();

            let result =
                move_elements_across_dir(&src_dir, &dst_dir, options, replace_options).await;
            assert!(result.is_ok(), "移动不存在的目录应该成功（无操作）");

            cleanup_test_dir(&temp_dir).await;
        });
    }

    #[test]
    fn test_move_elements_across_dir_with_ext_specific_rules() {
        smol::block_on(async {
            let temp_dir = tempdir().expect("创建临时目录失败");
            let src_dir = temp_dir.path().join("src");
            let dst_dir = temp_dir.path().join("dst");

            // 创建源目录结构
            fs::create_dir_all(&src_dir).await.expect("创建源目录失败");
            fs::write(src_dir.join("file1.txt"), "content1")
                .await
                .expect("创建文件失败");
            fs::write(src_dir.join("file2.bms"), "content2")
                .await
                .expect("创建文件失败");
            fs::write(src_dir.join("file3.other"), "content3")
                .await
                .expect("创建文件失败");

            // 在目标目录创建冲突文件
            fs::create_dir_all(&dst_dir)
                .await
                .expect("创建目标目录失败");
            fs::write(dst_dir.join("file1.txt"), "existing_txt")
                .await
                .expect("创建文件失败");
            fs::write(dst_dir.join("file2.bms"), "existing_bms")
                .await
                .expect("创建文件失败");
            fs::write(dst_dir.join("file3.other"), "existing_other")
                .await
                .expect("创建文件失败");

            // 使用特定扩展名规则
            let options = MoveOptions { print_info: false };
            let mut replace_options = ReplaceOptions::default();
            replace_options
                .ext
                .insert("txt".to_string(), ReplaceAction::Skip);
            replace_options
                .ext
                .insert("bms".to_string(), ReplaceAction::Rename);
            replace_options.default = ReplaceAction::Replace;

            move_elements_across_dir(&src_dir, &dst_dir, options, replace_options)
                .await
                .expect("移动操作失败");

            // 验证 txt 文件被跳过
            let content = fs::read_to_string(dst_dir.join("file1.txt"))
                .await
                .expect("读取文件失败");
            assert_eq!(content, "existing_txt", "txt 文件应该被跳过");

            // 验证 bms 文件被重命名
            assert!(dst_dir.join("file2.1.bms").exists(), "bms 文件应该被重命名");

            // 验证 other 文件被替换
            let content = fs::read_to_string(dst_dir.join("file3.other"))
                .await
                .expect("读取文件失败");
            assert_eq!(content, "content3", "other 文件应该被替换");

            cleanup_test_dir(&temp_dir).await;
        })
    }
}
