use sha2::{Digest, Sha512};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use tokio::sync::Semaphore;

/// Execution mode for a soft sync operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SoftSyncExec {
    /// Dry run; do not copy any files.
    None = 0,
    /// Copy files from source to destination.
    #[default]
    Copy = 1,
}

/// Preset configuration for a soft sync operation.
///
/// Controls which files are compared, how equality is determined (size, mtime, SHA-512),
/// whether extra files in the destination are removed, and whether identical source files
/// are deleted after syncing.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SoftSyncPreset {
    /// Name of this preset (informational, not read internally).
    #[allow(dead_code)]
    pub name: String,
    /// File extensions (without dot) allowed for syncing.
    pub allow_src_exts: Vec<String>,
    /// File extensions (without dot) disallowed for syncing.
    pub disallow_src_exts: Vec<String>,
    /// Whether to allow extensions not listed in `allow_src_exts` or `disallow_src_exts`.
    pub allow_other_exts: bool,
    /// Pairs of extension groups that deactivate syncing when a bound file exists in the destination.
    pub no_activate_ext_bound_pairs: Vec<(Vec<String>, Vec<String>)>,
    /// Whether to remove files in the destination that do not exist in the source.
    pub remove_dst_extra_files: bool,
    /// Whether to compare file sizes to determine if files are identical.
    pub check_file_size: bool,
    /// Whether to compare modification timestamps to determine if files are identical.
    pub check_file_mtime: bool,
    /// Whether to compare SHA-512 hashes to determine if files are identical.
    pub check_file_sha512: bool,
    /// Whether to remove source files that are identical to their destination counterpart.
    pub remove_src_same_files: bool,
    /// Execution mode for this sync operation.
    pub exec: SoftSyncExec,
}

impl SoftSyncPreset {
    /// Create a new preset with the given name and sensible defaults.
    ///
    /// Defaults: all extensions allowed, file size + mtime checking enabled,
    /// SHA-512 checking disabled, destination extra files removed,
    /// source same files not removed, execution mode is [`SoftSyncExec::Copy`].
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

async fn read_dir_entries(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut entries = tokio::fs::read_dir(dir).await?;
    let mut list = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        list.push(entry.path());
    }
    Ok(list)
}

fn format_file_names(paths: &[PathBuf]) -> Vec<&str> {
    paths
        .iter()
        .filter_map(|p| p.file_name())
        .filter_map(|n| n.to_str())
        .collect()
}

fn log_op(label: &str, paths: &[PathBuf]) {
    if !paths.is_empty() {
        println!("{label}: {:?}", format_file_names(paths));
    }
}

/// Compute the SHA-512 hex digest of a file.
///
/// Returns an empty string if the file does not exist or cannot be read.
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

/// Preset for "append" mode sync: checks size and SHA-512, skips mtime,
/// removes source same files, preserves destination extra files, dry-run only.
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

type SyncTaskResult = (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>);

async fn process_src_file(
    src_path: PathBuf,
    dst_dir: &Path,
    preset: &SoftSyncPreset,
    sem: Arc<Semaphore>,
) -> Result<SyncTaskResult, std::io::Error> {
    let _permit = sem.acquire().await.unwrap();
    let dst_path = dst_dir.join(src_path.file_name().unwrap_or_default());
    let ext = crate::fs::utils::get_ext(&src_path);

    let mut ext_check_passed = preset.allow_other_exts;
    if preset.allow_src_exts.iter().any(|e| *e == ext) {
        ext_check_passed = true;
    }
    if preset.disallow_src_exts.iter().any(|e| *e == ext) {
        ext_check_passed = false;
    }
    if !ext_check_passed {
        return Ok((Vec::new(), Vec::new(), Vec::new()));
    }

    for (ext_bound_from_list, ext_bound_to_list) in &preset.no_activate_ext_bound_pairs {
        if !ext_bound_from_list.iter().any(|e| *e == ext) {
            continue;
        }
        for ext_bound_to in ext_bound_to_list {
            let normalized = if ext_bound_to.starts_with('.') {
                ext_bound_to.clone()
            } else {
                format!(".{ext_bound_to}")
            };
            let bound_path = dst_path.with_extension(normalized.trim_start_matches('.'));
            if tokio::fs::metadata(&bound_path)
                .await
                .is_ok_and(|m| m.is_file())
            {
                return Ok((Vec::new(), Vec::new(), Vec::new()));
            }
        }
    }

    let dst_metadata = tokio::fs::metadata(&dst_path).await;
    let dst_file_exists = dst_metadata.as_ref().is_ok_and(std::fs::Metadata::is_file);
    let mut is_same_file = dst_file_exists;

    if preset.check_file_size && is_same_file && dst_file_exists {
        let src_size = tokio::fs::metadata(&src_path).await?.len();
        let dst_size = dst_metadata.as_ref().unwrap().len();
        is_same_file = src_size == dst_size;
    }
    if preset.check_file_mtime && is_same_file && dst_file_exists {
        let src_mtime = tokio::fs::metadata(&src_path).await?.modified()?;
        let dst_mtime = dst_metadata.as_ref().unwrap().modified()?;
        is_same_file = src_mtime == dst_mtime;
    }
    if preset.check_file_sha512 && is_same_file && dst_file_exists {
        is_same_file = get_file_sha512(&src_path).await == get_file_sha512(&dst_path).await;
    }

    let mut copy_files = Vec::new();
    let mut remove_files = Vec::new();

    if !dst_file_exists || !is_same_file {
        let src_mtime = tokio::fs::metadata(&src_path).await?.modified();
        match preset.exec {
            SoftSyncExec::None => {}
            SoftSyncExec::Copy => {
                copy_files.push(src_path.clone());
                tokio::fs::copy(&src_path, &dst_path).await?;
                if let Ok(mtime) = src_mtime {
                    let _ = filetime::set_file_mtime(
                        &dst_path,
                        filetime::FileTime::from_system_time(mtime),
                    );
                }
            }
        }
    }

    if preset.remove_src_same_files
        && dst_file_exists
        && is_same_file
        && tokio::fs::metadata(&src_path)
            .await
            .is_ok_and(|m| m.is_file())
    {
        remove_files.push(src_path.clone());
        let _ = tokio::fs::remove_file(&src_path).await;
    }

    Ok((copy_files, Vec::new(), remove_files))
}

/// Synchronize files from `src_dir` to `dst_dir` according to the given preset.
///
/// Compares files by size, mtime, and/or SHA-512 as configured in the preset,
/// then copies new or changed files, and optionally removes extra files in the
/// destination or same files in the source.
///
/// # Errors
///
/// Returns an error if directory listing, file stat, or file I/O fails.
///
/// # Panics
///
/// Panics if the internal semaphore is closed, which should not happen under
/// normal operation.
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

    let src_list = read_dir_entries(src_dir).await?;
    let dst_list = read_dir_entries(dst_dir).await?;

    let mut src_copy_files: Vec<PathBuf> = Vec::new();
    let mut src_remove_files: Vec<PathBuf> = Vec::new();
    let mut dst_remove_files: Vec<PathBuf> = Vec::new();
    let mut dst_remove_dirs: Vec<PathBuf> = Vec::new();

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut handles = Vec::new();

    for src_path in &src_list {
        if !src_path.is_file() {
            continue;
        }

        let sem = semaphore.clone();
        let src_path_clone = src_path.clone();
        let preset_clone = preset.clone();
        let dst_dir_clone = dst_dir.to_path_buf();

        let handle = tokio::spawn(async move {
            process_src_file(src_path_clone, &dst_dir_clone, &preset_clone, sem).await
        });
        handles.push(handle);
    }

    for handle in handles {
        if let Ok(result) = handle.await {
            match result {
                Ok((copy, _move, remove)) => {
                    src_copy_files.extend(copy);
                    src_remove_files.extend(remove);
                }
                Err(e) => return Err(e),
            }
        }
    }

    for src_path in &src_list {
        if src_path.is_dir() {
            let dst_path = dst_dir.join(src_path.file_name().unwrap_or_default());
            if !dst_path.is_dir() {
                tokio::fs::create_dir(&dst_path).await?;
            }
            Box::pin(sync_folder(src_path, &dst_path, preset, max_concurrent)).await?;
        }
    }

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

    if !src_copy_files.is_empty()
        || !src_remove_files.is_empty()
        || !dst_remove_files.is_empty()
        || !dst_remove_dirs.is_empty()
    {
        println!("{} -> {}:", src_dir.display(), dst_dir.display());
        log_op("Src copy", &src_copy_files);
        log_op("Src remove", &src_remove_files);
        log_op("Dst remove", &dst_remove_files);
        log_op("Dst remove dir", &dst_remove_dirs);
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

        tokio::fs::write(src_dir.join("test.txt"), "content")
            .await
            .unwrap();

        let preset = SoftSyncPreset::default();
        let result = sync_folder(&src_dir, &dst_dir, &preset, 8).await;
        assert!(result.is_ok());

        let _ = tokio::fs::remove_dir_all(&src_dir).await;
        let _ = tokio::fs::remove_dir_all(&dst_dir).await;
    }
}
