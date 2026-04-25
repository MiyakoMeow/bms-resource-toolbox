//! Directory synchronization utilities.
//!
//! This module provides functions for synchronizing files
//! between directories with various comparison strategies.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::items_after_statements)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;
use sha2::{Sha512, Digest};
use std::sync::LazyLock;

/// Synchronization preset configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SyncPreset {
    /// Name of the preset
    pub name: String,
    /// Comparison method: "name", "size", or "hash"
    pub compare_by: String,
    /// Whether to remove empty directories after sync
    pub remove_empty: bool,
}

impl SyncPreset {
    /// Create a new sync preset.
    #[must_use] 
    pub fn new(name: &str, compare_by: &str, remove_empty: bool) -> Self {
        Self {
            name: name.to_string(),
            compare_by: compare_by.to_string(),
            remove_empty,
        }
    }
}

/// Create a sync preset for append mode.
#[must_use] 
pub fn sync_preset_for_append() -> SyncPreset {
    SyncPreset::new("append", "name", false)
}

/// Sync preset for appending files by name.
pub static SYNC_PRESET_FOR_APPEND: LazyLock<SyncPreset> = LazyLock::new(sync_preset_for_append);

/// Synchronize files from src to dst based on preset
pub async fn sync_folder(src_dir: &Path, dst_dir: &Path, preset: &SyncPreset) -> Result<(), std::io::Error> {
    if !src_dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Source is not a directory",
        ));
    }

    if !dst_dir.is_dir() {
        std::fs::create_dir_all(dst_dir)?;
    }

    #[allow(clippy::match_same_arms)]
    match preset.compare_by.as_str() {
        "name" => sync_by_name(src_dir, dst_dir),
        "size" => sync_by_size(src_dir, dst_dir),
        "hash" => sync_by_hash(src_dir, dst_dir),
        _ => sync_by_name(src_dir, dst_dir),
    }
}

fn sync_by_name(src_dir: &Path, dst_dir: &Path) -> Result<(), std::io::Error> {
    for entry in walkdir::WalkDir::new(src_dir).into_iter().map_while(std::result::Result::ok) {
        let src_path = entry.path();
        if src_path.is_file() {
            let rel_path = src_path.strip_prefix(src_dir).unwrap_or(src_path);
            let dst_path = dst_dir.join(rel_path);

            if !dst_path.exists() {
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                info!("Copying {:?} -> {:?}", src_path, dst_path);
                std::fs::copy(src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}

fn sync_by_size(src_dir: &Path, dst_dir: &Path) -> Result<(), std::io::Error> {
    for entry in walkdir::WalkDir::new(src_dir).into_iter().map_while(std::result::Result::ok) {
        let src_path = entry.path();
        if src_path.is_file() {
            let rel_path = src_path.strip_prefix(src_dir).unwrap_or(src_path);
            let dst_path = dst_dir.join(rel_path);

            if !dst_path.exists() || should_copy_by_size(src_path, &dst_path)? {
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                info!("Copying {:?} -> {:?}", src_path, dst_path);
                std::fs::copy(src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}

fn sync_by_hash(src_dir: &Path, dst_dir: &Path) -> Result<(), std::io::Error> {
    let mut src_hashes: HashMap<PathBuf, String> = HashMap::new();

    // Build hash map of source files
    for entry in walkdir::WalkDir::new(src_dir).into_iter().map_while(std::result::Result::ok) {
        let src_path = entry.path();
        if src_path.is_file() {
            let hash = compute_file_hash(src_path)?;
            src_hashes.insert(src_path.to_path_buf(), hash);
        }
    }

    // Build hash map of destination files
    let mut dst_hashes: HashMap<PathBuf, String> = HashMap::new();
    if dst_dir.is_dir() {
        for entry in walkdir::WalkDir::new(dst_dir).into_iter().filter_map(std::result::Result::ok) {
            let dst_path = entry.path();
            if dst_path.is_file() {
                let hash = compute_file_hash(dst_path)?;
                dst_hashes.insert(dst_path.to_path_buf(), hash);
            }
        }
    }

    // Copy files that don't exist or have different hash
    for (src_path, src_hash) in src_hashes {
        let rel_path = src_path.strip_prefix(src_dir).unwrap_or(&src_path);
        let dst_path = dst_dir.join(rel_path);

        let needs_copy = if let Some(dst_hash) = dst_hashes.get(&dst_path) {
            src_hash != *dst_hash
        } else {
            true
        };

        if needs_copy {
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            info!("Copying {:?} -> {:?}", src_path, dst_path);
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn should_copy_by_size(src: &Path, dst: &Path) -> Result<bool, std::io::Error> {
    let src_meta = src.metadata()?;
    let dst_meta = dst.metadata()?;
    Ok(src_meta.len() != dst_meta.len())
}

fn compute_file_hash(path: &Path) -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha512::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_preset() {
        assert_eq!(SYNC_PRESET_FOR_APPEND.name, "append");
    }
}
