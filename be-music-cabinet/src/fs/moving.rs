use std::{
    collections::HashMap,
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

/// 递归移动目录内容
pub async fn move_elements_across_dir(
    dir_path_ori: impl AsRef<Path>,
    dir_path_dst: impl AsRef<Path>,
    options: MoveOptions,
    replace_options: ReplaceOptions,
) -> io::Result<()> {
    let dir_path_ori = dir_path_ori.as_ref();
    let dir_path_dst = dir_path_dst.as_ref();

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

    // 收集待处理条目（文件 / 子目录）
    let mut entries = fs::read_dir(&dir_path_ori).await?;
    let mut tasks = Vec::new();

    // 递归处理子目录时需要再次调用的路径
    let next_folder_paths = Arc::new(Mutex::new(Vec::new()));

    while let Some(entry) = StreamExt::next(&mut entries).await {
        let entry = entry?;
        let src = entry.path();
        let dst = dir_path_dst.join(entry.file_name());

        let rep = replace_options.clone();
        let next_folder_paths = Arc::clone(&next_folder_paths);

        tasks.push(smol::spawn(async move {
            let next_folder_paths = Arc::clone(&next_folder_paths);
            move_action(&src, &dst, options, rep, {
                let mut next = next_folder_paths.lock_arc().await;
                move |ori: PathBuf, dst: PathBuf| next.push((ori, dst))
            })
            .await
        }));
    }

    // 等待所有任务完成
    for t in tasks {
        t.await?;
    }

    // 递归处理子目录
    for (ori, dst) in next_folder_paths.lock_arc().await.iter() {
        move_elements_across_dir(ori, dst, options, replace_options.clone()).await?;
    }

    // 清理空目录
    if replace_options.default != ReplaceAction::Skip || !is_dir_having_file(dir_path_ori).await? {
        if let Err(e) = fs::remove_dir_all(&dir_path_ori).await {
            eprintln!(" x PermissionError! ({}) - {}", dir_path_ori.display(), e);
        }
    }

    Ok(())
}

/// 单个文件/目录的移动入口
async fn move_action(
    src: &Path,
    dst: &Path,
    options: MoveOptions,
    rep: ReplaceOptions,
    mut push_child: impl FnMut(PathBuf, PathBuf),
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
            // 推迟到下一轮递归
            push_child(src.to_path_buf(), dst.to_path_buf());
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

/// 带重试的“重命名”移动
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
