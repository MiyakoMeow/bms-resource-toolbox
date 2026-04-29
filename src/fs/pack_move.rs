//! Directory move and merge operations.
//!
//! This module provides utilities for moving files and directories
//! between locations with conflict resolution.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tracing::info;

/// Check if two files have the same content.
#[must_use]
pub fn is_same_content(file_a: &Path, file_b: &Path) -> bool {
    if !file_a.is_file() || !file_b.is_file() {
        return false;
    }
    match (std::fs::read(file_a), std::fs::read(file_b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Options for move operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct MoveOptions {
    /// Whether to print move info.
    pub print_info: bool,
}

/// Action to take when a file conflict occurs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplaceAction {
    /// Skip the file
    Skip = 0,
    /// Replace the destination file
    #[default]
    Replace = 1,
    /// Rename the source file
    #[allow(dead_code)]
    Rename = 2,
    /// Check if content differs, then rename or replace
    CheckReplace = 12,
}

/// Options for handling file conflicts
#[derive(Debug, Clone)]
pub struct ReplaceOptions {
    /// Extension-specific replace actions
    pub ext: HashMap<String, ReplaceAction>,
    /// Default action for extensions not in `ext`
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

/// Replace options for update pack operations.
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

/// Default move options.
pub const DEFAULT_MOVE_OPTIONS: MoveOptions = MoveOptions { print_info: false };

/// Default replace options.
pub static DEFAULT_REPLACE_OPTIONS: LazyLock<ReplaceOptions> = LazyLock::new(|| ReplaceOptions {
    ext: HashMap::new(),
    default: ReplaceAction::Replace,
});

/// Check if a directory contains any files (recursive)
///
/// This matches Python's `is_dir_having_file` behavior:
/// - Recursively checks subdirectories
/// - Only counts non-empty files (size > 0)
#[must_use]
pub fn is_dir_having_file(dir: &Path) -> bool {
    fn check_recursive(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                // Check if file is non-empty (matches Python behavior)
                if let Ok(metadata) = path.metadata()
                    && metadata.len() > 0
                {
                    return true;
                }
            } else if path.is_dir() && check_recursive(&path) {
                return true;
            }
        }

        false
    }

    if !dir.is_dir() {
        return false;
    }

    check_recursive(dir)
}

/// Move elements (files and directories) from source to destination.
///
/// If conflict exists, handles it based on `ReplaceOptions`.
///
/// # Errors
///
/// Returns [`std::io::Error`] if:
/// - `src` is not a directory
/// - directory operations fail
#[expect(clippy::needless_pass_by_value)]
pub fn move_elements_across_dir(
    src: &Path,
    dst: &Path,
    options: MoveOptions,
    replace_options: ReplaceOptions,
) -> Result<(), std::io::Error> {
    if !src.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Source is not a directory",
        ));
    }

    if !dst.is_dir() {
        std::fs::create_dir_all(dst)?;
        return move_dir_recursive(src, dst);
    }

    let mut next_folder_paths: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut write_ops: Vec<(PathBuf, PathBuf)> = Vec::new();

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let filename = entry.file_name();
        let dst_path = dst.join(&filename);

        if src_path.is_file() {
            if let Some((planned_src, planned_dst)) =
                plan_move_file(&src_path, &dst_path, &replace_options)
            {
                write_ops.push((planned_src, planned_dst));
            }
        } else if src_path.is_dir() {
            next_folder_paths.push((src_path, dst_path));
        }
    }

    for (src_path, final_dst_path) in write_ops {
        if options.print_info {
            info!("Moving {:?} -> {:?}", src_path, final_dst_path);
        }
        move_file(&src_path, &final_dst_path)?;
    }

    for (src_path, dst_path) in next_folder_paths {
        move_elements_across_dir(&src_path, &dst_path, options, replace_options.clone())?;
    }

    let should_clean = replace_options.default != ReplaceAction::Skip || !is_dir_having_file(src);
    if should_clean && let Err(e) = std::fs::remove_dir_all(src) {
        tracing::warn!("Failed to remove source directory {:?}: {}", src, e);
    }

    Ok(())
}

/// Plan a file move operation based on `ReplaceOptions`
fn plan_move_file(
    ori_path: &Path,
    dst_path: &Path,
    replace_options: &ReplaceOptions,
) -> Option<(PathBuf, PathBuf)> {
    let file_ext = ori_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
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
        ReplaceAction::Rename => {
            // Find a new name that doesn't conflict
            for i in 0..100 {
                let stem = dst_path.file_stem().unwrap_or_default().to_string_lossy();
                let ext = dst_path
                    .extension()
                    .map(|e| e.to_string_lossy())
                    .unwrap_or_default();
                let new_name = if ext.is_empty() {
                    format!("{stem}.{i}")
                } else {
                    format!("{stem}.{i}.{ext}")
                };
                let new_dst_path = dst_path.with_file_name(new_name);
                if !new_dst_path.exists() {
                    return Some((ori_path.to_path_buf(), new_dst_path));
                }
            }
            None
        }
        ReplaceAction::CheckReplace => {
            if !dst_path.is_file() {
                Some((ori_path.to_path_buf(), dst_path.to_path_buf()))
            } else if is_same_content(ori_path, dst_path) {
                // Same content, still move to unify location
                Some((ori_path.to_path_buf(), dst_path.to_path_buf()))
            } else {
                // Different content, rename
                for i in 0..100 {
                    let stem = dst_path.file_stem().unwrap_or_default().to_string_lossy();
                    let ext = dst_path
                        .extension()
                        .map(|e| e.to_string_lossy())
                        .unwrap_or_default();
                    let new_name = if ext.is_empty() {
                        format!("{stem}.{i}")
                    } else {
                        format!("{stem}.{i}.{ext}")
                    };
                    let new_dst_path = dst_path.with_file_name(new_name);
                    if !new_dst_path.exists() {
                        return Some((ori_path.to_path_buf(), new_dst_path));
                    }
                }
                None
            }
        }
    }
}

use super::utils::copy_dir_recursive;

/// Move a file (handles cross-device moves)
fn move_file(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::rename(src, dst).or_else(|_| {
        if src.is_dir() {
            copy_dir_recursive(src, dst)?;
            std::fs::remove_dir_all(src)
        } else {
            std::fs::copy(src, dst)?;
            std::fs::remove_file(src)
        }
    })
}

/// Move directory recursively
fn move_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::rename(src, dst).or_else(|_| {
        std::fs::create_dir_all(dst)?;
        copy_dir_recursive(src, dst)?;
        std::fs::remove_dir_all(src)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_dir_having_file() {
        let temp_dir = std::env::temp_dir();
        assert!(is_dir_having_file(&temp_dir));
        assert!(!is_dir_having_file(&PathBuf::from("/nonexistent")));
    }
}
