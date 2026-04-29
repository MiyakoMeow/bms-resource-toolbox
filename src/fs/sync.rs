//! Directory synchronization utilities.
//!
//! This module provides functions for synchronizing files
//! between directories with various comparison strategies.

use sha2::{Digest, Sha512};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use tokio::sync::Semaphore;
use tracing::info;

/// Soft sync execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SoftSyncExec {
    /// No operation.
    None = 0,
    /// Copy files from source to destination.
    #[default]
    Copy = 1,
    /// Move files from source to destination.
    #[allow(dead_code)]
    Move = 2,
}

/// Soft sync preset configuration
#[expect(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct SoftSyncPreset {
    /// Name of the preset
    #[allow(dead_code)]
    pub name: String,
    /// Allowed source extensions
    pub allow_src_exts: Vec<String>,
    /// Disallowed source extensions
    pub disallow_src_exts: Vec<String>,
    /// Allow extensions not in `allow_src_exts`
    pub allow_other_exts: bool,
    /// Extension bound pairs that should not activate sync
    pub no_activate_ext_bound_pairs: Vec<(Vec<String>, Vec<String>)>,
    /// Remove extra files in destination
    pub remove_dst_extra_files: bool,
    /// Check file size when comparing
    pub check_file_size: bool,
    /// Check file mtime when comparing
    pub check_file_mtime: bool,
    /// Check file sha512 when comparing
    pub check_file_sha512: bool,
    /// Remove source files that are the same as destination
    pub remove_src_same_files: bool,
    /// Execution mode (copy/move/none)
    pub exec: SoftSyncExec,
}

impl SoftSyncPreset {
    /// Create a new soft sync preset with default values
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            allow_src_exts: Vec::new(),
            disallow_src_exts: Vec::new(),
            allow_other_exts: true,
            no_activate_ext_bound_pairs: Vec::new(),
            remove_dst_extra_files: true,
            check_file_size: true,
            check_file_mtime: true,
            check_file_sha512: false,
            remove_src_same_files: false,
            exec: SoftSyncExec::Copy,
        }
    }
}

impl Default for SoftSyncPreset {
    fn default() -> Self {
        Self::new("本地文件同步预设")
    }
}

/// Get SHA-512 hash of a file - async version
pub async fn get_file_sha512(file_path: &Path) -> String {
    if !file_path.is_file() {
        return String::new();
    }
    match tokio::fs::read(file_path).await {
        Ok(bytes) => {
            let mut hasher = Sha512::new();
            hasher.update(&bytes);
            let result = hasher.finalize();
            let mut hex_string = String::with_capacity(result.len() * 2);
            for byte in result {
                use std::fmt::Write;
                let _ = write!(hex_string, "{byte:02x}");
            }
            hex_string
        }
        Err(_) => String::new(),
    }
}

/// Default sync preset
#[allow(dead_code)]
pub static SYNC_PRESET_DEFAULT: LazyLock<SoftSyncPreset> = LazyLock::new(SoftSyncPreset::default);

/// Sync preset for append mode (update packs)
pub static SYNC_PRESET_FOR_APPEND: LazyLock<SoftSyncPreset> = LazyLock::new(|| SoftSyncPreset {
    name: "同步预设（用于更新包）".to_string(),
    check_file_size: true,
    check_file_mtime: false,
    check_file_sha512: true,
    remove_src_same_files: true,
    remove_dst_extra_files: false,
    exec: SoftSyncExec::None,
    ..Default::default()
});

/// Sync preset for FLAC files
#[allow(dead_code)]
pub static SYNC_PRESET_FLAC: LazyLock<SoftSyncPreset> = LazyLock::new(|| SoftSyncPreset {
    name: "FLAC Sync".to_string(),
    allow_src_exts: vec!["flac".to_string()],
    allow_other_exts: false,
    remove_dst_extra_files: false,
    exec: SoftSyncExec::Copy,
    ..Default::default()
});

/// Sync preset for MP4/AVI files
#[allow(dead_code)]
pub static SYNC_PRESET_MP4_AVI: LazyLock<SoftSyncPreset> = LazyLock::new(|| SoftSyncPreset {
    name: "MP4/AVI Sync".to_string(),
    allow_src_exts: vec!["mp4".to_string(), "avi".to_string()],
    allow_other_exts: false,
    remove_dst_extra_files: false,
    exec: SoftSyncExec::Copy,
    ..Default::default()
});

/// Sync preset for cache directories
#[allow(dead_code)]
pub static SYNC_PRESET_CACHE: LazyLock<SoftSyncPreset> = LazyLock::new(|| SoftSyncPreset {
    name: "Cache Sync".to_string(),
    allow_src_exts: vec!["mp4".to_string(), "avi".to_string(), "flac".to_string()],
    allow_other_exts: false,
    remove_dst_extra_files: false,
    exec: SoftSyncExec::None,
    ..Default::default()
});

/// Synchronize files from src to dst based on `SoftSyncPreset` - async version
///
/// # Arguments
///
/// * `src_dir` - Source directory path
/// * `dst_dir` - Destination directory path
/// * `preset` - Sync configuration preset
/// * `max_concurrent` - Maximum number of concurrent file operations
///
/// # Errors
///
/// Returns [`std::io::Error`] if:
/// - `src_dir` is not a directory
/// - `dst_dir` creation fails
/// - reading directories fails
///
/// # Panics
///
/// Panics if the semaphore acquisition fails (should not happen in normal operation).
#[expect(clippy::too_many_lines)]
pub async fn sync_folder(
    src_dir: &Path,
    dst_dir: &Path,
    preset: &SoftSyncPreset,
    max_concurrent: usize,
) -> Result<(), std::io::Error> {
    if !src_dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Source is not a directory",
        ));
    }

    if !dst_dir.is_dir() {
        tokio::fs::create_dir_all(dst_dir).await?;
    }

    let mut src_list: Vec<PathBuf> = Vec::new();
    let mut src_entries = tokio::fs::read_dir(src_dir).await?;
    while let Ok(Some(entry)) = src_entries.next_entry().await {
        src_list.push(entry.path());
    }

    let mut dst_list: Vec<PathBuf> = Vec::new();
    let mut dst_entries = tokio::fs::read_dir(dst_dir).await?;
    while let Ok(Some(entry)) = dst_entries.next_entry().await {
        dst_list.push(entry.path());
    }

    // Track operations for logging
    let mut src_copy_files: Vec<PathBuf> = Vec::new();
    let mut src_move_files: Vec<PathBuf> = Vec::new();
    let mut src_remove_files: Vec<PathBuf> = Vec::new();
    let mut dst_remove_files: Vec<PathBuf> = Vec::new();
    let mut dst_remove_dirs: Vec<PathBuf> = Vec::new();

    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    // Process source files concurrently
    let mut handles = Vec::new();
    for src_path in &src_list {
        if !src_path.is_file() {
            continue;
        }

        let dst_path = dst_dir.join(src_path.file_name().unwrap_or_default());
        let sem = semaphore.clone();
        let src_path_clone = src_path.clone();
        let preset_clone = preset.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            // Check extension
            let ext = src_path_clone
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let mut ext_check_passed = preset_clone.allow_other_exts;
            if preset_clone.allow_src_exts.iter().any(|e| e == &ext) {
                ext_check_passed = true;
            }
            if preset_clone.disallow_src_exts.iter().any(|e| e == &ext) {
                ext_check_passed = false;
            }
            if !ext_check_passed {
                return Ok((Vec::new(), Vec::new(), Vec::new()));
            }

            // Check extension bounds
            let mut ext_in_bound = false;
            for (ext_bound_from_list, ext_bound_to_list) in
                &preset_clone.no_activate_ext_bound_pairs
            {
                if !ext_bound_from_list.iter().any(|e| e == &ext) {
                    continue;
                }
                for ext_bound_to in ext_bound_to_list {
                    let normalized_suffix = if ext_bound_to.starts_with('.') {
                        ext_bound_to.clone()
                    } else {
                        format!(".{ext_bound_to}")
                    };
                    let bound_file_path =
                        dst_path.with_extension(normalized_suffix.trim_start_matches('.'));
                    if tokio::fs::metadata(&bound_file_path)
                        .await
                        .is_ok_and(|m| m.is_file())
                    {
                        ext_in_bound = true;
                        break;
                    }
                }
                if ext_in_bound {
                    break;
                }
            }
            if ext_in_bound {
                return Ok((Vec::new(), Vec::new(), Vec::new()));
            }

            // Check if destination exists and compare
            let dst_metadata = tokio::fs::metadata(&dst_path).await;
            let dst_file_exists = dst_metadata.is_ok();
            let mut is_same_file = dst_file_exists;

            if preset_clone.check_file_size && is_same_file && dst_file_exists {
                let src_size = tokio::fs::metadata(&src_path_clone).await?.len();
                let dst_size = dst_metadata.as_ref().unwrap().len();
                is_same_file = is_same_file && src_size == dst_size;
            }
            if preset_clone.check_file_mtime && is_same_file && dst_file_exists {
                let src_mtime = tokio::fs::metadata(&src_path_clone).await?.modified()?;
                let dst_mtime = dst_metadata.as_ref().unwrap().modified()?;
                is_same_file = is_same_file && src_mtime == dst_mtime;
            }
            if preset_clone.check_file_sha512 && is_same_file && dst_file_exists {
                let src_value = get_file_sha512(&src_path_clone).await;
                let dst_value = get_file_sha512(&dst_path).await;
                is_same_file = is_same_file && src_value == dst_value && !src_value.is_empty();
            }

            let mut copy_files = Vec::new();
            let mut move_files = Vec::new();
            let mut remove_files = Vec::new();

            // Execute sync
            if !dst_file_exists || !is_same_file {
                let src_mtime = tokio::fs::metadata(&src_path_clone).await?.modified();
                match preset_clone.exec {
                    SoftSyncExec::None => {}
                    SoftSyncExec::Copy => {
                        copy_files.push(src_path_clone.clone());
                        tokio::fs::copy(&src_path_clone, &dst_path).await?;
                        if let Ok(mtime) = src_mtime {
                            let _ = filetime::set_file_mtime(
                                &dst_path,
                                filetime::FileTime::from_system_time(mtime),
                            );
                        }
                    }
                    SoftSyncExec::Move => {
                        move_files.push(src_path_clone.clone());
                        if tokio::fs::rename(&src_path_clone, &dst_path).await.is_err() {
                            tokio::fs::copy(&src_path_clone, &dst_path).await?;
                            tokio::fs::remove_file(&src_path_clone).await?;
                        }
                        if let Ok(mtime) = src_mtime {
                            let _ = filetime::set_file_mtime(
                                &dst_path,
                                filetime::FileTime::from_system_time(mtime),
                            );
                        }
                    }
                }
            }

            // Remove same source files
            if preset_clone.remove_src_same_files
                && dst_file_exists
                && is_same_file
                && tokio::fs::metadata(&src_path_clone)
                    .await
                    .is_ok_and(|m| m.is_file())
            {
                remove_files.push(src_path_clone.clone());
                let _ = tokio::fs::remove_file(&src_path_clone).await;
            }

            Ok((copy_files, move_files, remove_files))
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        if let Ok(result) = handle.await {
            match result {
                Ok((copy, move_f, remove)) => {
                    src_copy_files.extend(copy);
                    src_move_files.extend(move_f);
                    src_remove_files.extend(remove);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    // Process destination extra files
    if preset.remove_dst_extra_files {
        for dst_path in &dst_list {
            let src_path = src_dir.join(dst_path.file_name().unwrap_or_default());

            if dst_path.is_dir() {
                if !src_path.is_dir() {
                    dst_remove_dirs.push(dst_path.clone());
                    let _ = tokio::fs::remove_dir_all(dst_path).await;
                }
            } else if dst_path.is_file() && !src_path.is_file() {
                dst_remove_files.push(dst_path.clone());
                let _ = tokio::fs::remove_file(dst_path).await;
            }
        }
    }

    // Recurse into subdirectories
    for src_path in &src_list {
        if src_path.is_dir() {
            let dst_path = dst_dir.join(src_path.file_name().unwrap_or_default());
            if !dst_path.is_dir() {
                tokio::fs::create_dir_all(&dst_path).await?;
            }
            Box::pin(sync_folder(src_path, &dst_path, preset, max_concurrent)).await?;
        }
    }

    // Log operations
    if !src_copy_files.is_empty()
        || !src_move_files.is_empty()
        || !src_remove_files.is_empty()
        || !dst_remove_files.is_empty()
        || !dst_remove_dirs.is_empty()
    {
        info!("{} -> {}:", src_dir.display(), dst_dir.display());
        if !src_copy_files.is_empty() {
            let names: Vec<_> = src_copy_files
                .iter()
                .filter_map(|p| p.file_name())
                .filter_map(|n| n.to_str())
                .collect();
            info!("Src copy: {:?}", names);
        }
        if !src_move_files.is_empty() {
            let names: Vec<_> = src_move_files
                .iter()
                .filter_map(|p| p.file_name())
                .filter_map(|n| n.to_str())
                .collect();
            info!("Src move: {:?}", names);
        }
        if !src_remove_files.is_empty() {
            let names: Vec<_> = src_remove_files
                .iter()
                .filter_map(|p| p.file_name())
                .filter_map(|n| n.to_str())
                .collect();
            info!("Src remove: {:?}", names);
        }
        if !dst_remove_files.is_empty() {
            let names: Vec<_> = dst_remove_files
                .iter()
                .filter_map(|p| p.file_name())
                .filter_map(|n| n.to_str())
                .collect();
            info!("Dst remove: {:?}", names);
        }
        if !dst_remove_dirs.is_empty() {
            let names: Vec<_> = dst_remove_dirs
                .iter()
                .filter_map(|p| p.file_name())
                .filter_map(|n| n.to_str())
                .collect();
            info!("Dst remove dir: {:?}", names);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_sync_preset() {
        let preset = SoftSyncPreset::default();
        assert_eq!(preset.name, "本地文件同步预设");
        assert!(preset.allow_other_exts);
        assert!(preset.check_file_size);
    }

    #[test]
    async fn test_sync_folder_basic() {
        let temp_dir = std::env::temp_dir();
        let src_dir = temp_dir.join("sync_src");
        let dst_dir = temp_dir.join("sync_dst");

        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::create_dir_all(&dst_dir).await.unwrap();

        // Create a test file
        tokio::fs::write(src_dir.join("test.txt"), "content")
            .await
            .unwrap();

        let preset = SoftSyncPreset::default();
        let result = sync_folder(&src_dir, &dst_dir, &preset, 8).await;
        assert!(result.is_ok());

        // Cleanup
        let _ = tokio::fs::remove_dir_all(&src_dir).await;
        let _ = tokio::fs::remove_dir_all(&dst_dir).await;
    }
}
