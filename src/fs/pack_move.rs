use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[must_use]
pub async fn is_same_content(file_a: &Path, file_b: &Path) -> bool {
    if !file_a.is_file() || !file_b.is_file() {
        return false;
    }
    match (tokio::fs::read(file_a).await, tokio::fs::read(file_b).await) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MoveOptions {
    pub print_info: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplaceAction {
    Skip = 0,
    #[default]
    Replace = 1,
    CheckReplace = 12,
}

#[derive(Debug, Clone)]
pub struct ReplaceOptions {
    pub ext: HashMap<String, ReplaceAction>,
    pub default: ReplaceAction,
}

impl Default for ReplaceOptions {
    fn default() -> Self {
        Self {
            ext: HashMap::new(),
            default: ReplaceAction::Replace,
        }
    }
}

pub static REPLACE_OPTION_UPDATE_PACK: LazyLock<ReplaceOptions> =
    LazyLock::new(|| ReplaceOptions {
        ext: HashMap::from([
            (String::from("bms"), ReplaceAction::CheckReplace),
            (String::from("bml"), ReplaceAction::CheckReplace),
            (String::from("bme"), ReplaceAction::CheckReplace),
            (String::from("pms"), ReplaceAction::CheckReplace),
            (String::from("txt"), ReplaceAction::CheckReplace),
            (String::from("bmson"), ReplaceAction::CheckReplace),
        ]),
        default: ReplaceAction::Replace,
    });

pub const DEFAULT_MOVE_OPTIONS: MoveOptions = MoveOptions { print_info: false };

pub static DEFAULT_REPLACE_OPTIONS: LazyLock<ReplaceOptions> = LazyLock::new(|| ReplaceOptions {
    ext: HashMap::new(),
    default: ReplaceAction::Replace,
});

#[must_use]
pub async fn is_dir_having_file(dir: &Path) -> bool {
    async fn check_recursive(dir: &Path) -> bool {
        let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
            return false;
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file()
                && let Ok(metadata) = tokio::fs::metadata(&path).await
                && metadata.len() > 0
            {
                return true;
            } else if path.is_dir() && Box::pin(check_recursive(&path)).await {
                return true;
            }
        }

        false
    }

    if !dir.is_dir() {
        return false;
    }

    check_recursive(dir).await
}

pub async fn move_elements_across_dir(
    src: &Path,
    dst: &Path,
    options: MoveOptions,
    replace_options: &ReplaceOptions,
) -> Result<(), std::io::Error> {
    if let (Ok(src_canon), Ok(dst_canon)) = (
        tokio::fs::canonicalize(src).await,
        tokio::fs::canonicalize(dst).await,
    ) && src_canon == dst_canon
    {
        return Ok(());
    }

    if !src.is_dir() {
        return Ok(());
    }

    if !dst.is_dir() {
        tokio::fs::create_dir_all(dst).await?;
        return Box::pin(move_dir_recursive(src, dst)).await;
    }

    let mut next_folder_paths: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut write_ops: Vec<(PathBuf, PathBuf)> = Vec::new();

    let mut entries = tokio::fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let filename = entry.file_name();
        let dst_path = dst.join(&filename);

        if src_path.is_file() {
            if let Some((planned_src, planned_dst)) =
                plan_move_file(&src_path, &dst_path, replace_options).await
            {
                write_ops.push((planned_src, planned_dst));
            }
        } else if src_path.is_dir() {
            next_folder_paths.push((src_path, dst_path));
        }
    }

    for (src_path, final_dst_path) in write_ops {
        if options.print_info {
            println!("Moving {src_path:?} -> {final_dst_path:?}");
        }
        move_file(&src_path, &final_dst_path).await?;
    }

    for (src_path, dst_path) in next_folder_paths {
        Box::pin(move_elements_across_dir(
            &src_path,
            &dst_path,
            options,
            replace_options,
        ))
        .await?;
    }

    let should_clean =
        replace_options.default != ReplaceAction::Skip || !is_dir_having_file(src).await;
    if should_clean && let Err(e) = tokio::fs::remove_dir_all(src).await {
        println!("Failed to remove source directory {src:?}: {e}");
    }

    Ok(())
}

async fn plan_move_file(
    ori_path: &Path,
    dst_path: &Path,
    replace_options: &ReplaceOptions,
) -> Option<(PathBuf, PathBuf)> {
    let file_ext = ori_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();
    let action = replace_options
        .ext
        .get(&file_ext)
        .copied()
        .unwrap_or(replace_options.default);

    match action {
        ReplaceAction::Skip => {
            if dst_path.is_file() {
                return None;
            }
            Some((ori_path.to_path_buf(), dst_path.to_path_buf()))
        }
        ReplaceAction::Replace => Some((ori_path.to_path_buf(), dst_path.to_path_buf())),
        ReplaceAction::CheckReplace => {
            if !dst_path.is_file() || is_same_content(ori_path, dst_path).await {
                return Some((ori_path.to_path_buf(), dst_path.to_path_buf()));
            }
            for i in 0..100 {
                let stem = dst_path.file_stem().unwrap_or_default().to_string_lossy();
                let ext = dst_path
                    .extension()
                    .map(|e| e.to_string_lossy())
                    .unwrap_or_default();
                let new_name = format!("{stem}.{i}.{ext}");
                let new_dst_path = dst_path.with_file_name(new_name);
                if new_dst_path.is_file() {
                    if is_same_content(ori_path, &new_dst_path).await {
                        return None;
                    }
                    continue;
                }
                return Some((ori_path.to_path_buf(), new_dst_path));
            }
            None
        }
    }
}

use super::utils::copy_dir_recursive;

async fn move_file(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    match tokio::fs::rename(src, dst).await {
        Ok(()) => Ok(()),
        Err(_) => {
            if src.is_dir() {
                copy_dir_recursive(src, dst).await?;
                tokio::fs::remove_dir_all(src).await
            } else {
                tokio::fs::copy(src, dst).await?;
                tokio::fs::remove_file(src).await
            }
        }
    }
}

async fn move_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if tokio::fs::rename(src, dst).await.is_ok() {
        return Ok(());
    }
    tokio::fs::create_dir_all(dst).await?;
    copy_dir_recursive(src, dst).await?;
    tokio::fs::remove_dir_all(src).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn create_test_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let unique_name = format!(
            "test_is_dir_having_file_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = temp_dir.join(unique_name);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn cleanup_test_dir(path: &Path) {
        let _ = std::fs::remove_dir_all(path);
    }

    #[tokio::test]
    async fn test_is_dir_having_file() {
        let dir = create_test_dir();
        let file_path = dir.join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        assert!(is_dir_having_file(&dir).await);

        assert!(!is_dir_having_file(&PathBuf::from("/nonexistent")).await);

        let empty_dir = create_test_dir();
        assert!(!is_dir_having_file(&empty_dir).await);

        cleanup_test_dir(&dir);
        cleanup_test_dir(&empty_dir);
    }
}
