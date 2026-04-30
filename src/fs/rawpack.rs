//! Archive extraction utilities.
//!
//! This module handles extraction of compressed archives
//! including zip, 7z, and rar formats.

#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::items_after_statements
)]

use std::collections::HashMap;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use tracing::info;

use crate::bms::types::CHART_FILE_EXTS;
use crate::fs::pack_move::{
    DEFAULT_MOVE_OPTIONS, DEFAULT_REPLACE_OPTIONS, move_elements_across_dir,
};

/// Safely join a path component to a base directory, preventing path traversal attacks.
/// Returns `None` if the joined path would escape the base directory.
#[cfg(test)]
fn safe_join(base: &Path, component: &str) -> Option<PathBuf> {
    let decoded = component.replace('\\', "/");
    let path = PathBuf::from(&decoded);

    if path.is_absolute() {
        return None;
    }

    let mut current = base.to_path_buf();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                current.pop();
                if !current.starts_with(base) {
                    return None;
                }
            }
            std::path::Component::Normal(name) => {
                current.push(name);
            }
            _ => return None,
        }
    }

    if current.starts_with(base) {
        Some(current)
    } else {
        None
    }
}

/// Set the modification time of a file.
#[expect(dead_code)]
async fn set_mtime(path: &Path, mtime: u32) -> anyhow::Result<()> {
    let _file = tokio::fs::File::open(path).await?;
    let times = filetime::FileTime::from_unix_time(i64::from(mtime), 0);
    filetime::set_file_mtime(path, times)?;
    Ok(())
}

/// Get numbered file names from a directory.
///
/// Matches Python behavior: files whose first space-delimited token is all digits.
#[must_use]
pub fn get_num_set_file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if !entry.path().is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let id_str = name.split(' ').next().unwrap_or("");
            if id_str.is_empty() || !id_str.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            names.push(name);
        }
    }

    names.sort();
    names
}

/// Extract numeric-prefixed archives to BMS folder structure
#[expect(dead_code)]
pub(crate) fn extract_numeric_to_bms_folder(
    pack_dir: &Path,
    cache_dir: &Path,
    root_dir: &Path,
) -> Result<(), std::io::Error> {
    use tokio::runtime::Runtime;

    if !pack_dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Pack dir is not a directory",
        ));
    }

    std::fs::create_dir_all(cache_dir)?;
    std::fs::create_dir_all(root_dir)?;

    let file_names = get_num_set_file_names(pack_dir);
    info!("Found {} pack files", file_names.len());

    // Create runtime for async extraction
    let rt = Runtime::new()?;

    for file_name in file_names {
        let pack_path = pack_dir.join(&file_name);
        info!("Extracting: {}", file_name);

        // Determine archive type and extract
        let ext = pack_path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .unwrap_or_default();

        match ext.as_str() {
            "zip" => {
                extract_zip(&pack_path, cache_dir)?;
            }
            "7z" => {
                let pack_path_buf = pack_path.clone();
                let cache_dir_buf = cache_dir.to_path_buf();
                rt.block_on(async { extract_7z(&pack_path_buf, &cache_dir_buf).await })?;
            }
            "rar" => {
                let pack_path_buf = pack_path.clone();
                let cache_dir_buf = cache_dir.to_path_buf();
                rt.block_on(async { extract_rar(&pack_path_buf, &cache_dir_buf).await })?;
            }
            _ => {
                let target_file_name = file_name
                    .split_once(' ')
                    .map_or(file_name.as_str(), |(_, rest)| rest);
                let target_file_path = cache_dir.join(target_file_name);
                info!("Copying {} to {}", file_name, target_file_path.display());
                std::fs::copy(&pack_path, &target_file_path)?;
            }
        }
    }

    Ok(())
}

/// Extract archive files (zip, 7z, rar)
pub async fn extract_archive(archive_path: &Path, output_dir: &Path) -> Result<(), std::io::Error> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    std::fs::create_dir_all(output_dir)?;

    match ext.as_str() {
        "zip" => extract_zip(archive_path, output_dir),
        "7z" => extract_7z(archive_path, output_dir).await,
        "rar" => extract_rar(archive_path, output_dir).await,
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported archive format: {ext}"),
        )),
    }
}

fn extract_zip(archive_path: &Path, output_dir: &Path) -> Result<(), std::io::Error> {
    use zip::ZipArchive;

    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Detect if we need CP932 (Shift-JIS) decoding
    // This matches Python's behavior: check non-UTF-8 entries for Japanese characters
    let use_cp932 = detect_cp932_encoding(&mut archive);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // Decode filename: try enclosed_name first, then CP932 if needed
        let decoded_name = if let Some(enclosed) = file.enclosed_name() {
            enclosed.to_string_lossy().to_string()
        } else if use_cp932 {
            // Try CP437 -> CP932 conversion on the name
            decode_cp932_filename(file.name()).unwrap_or_else(|| file.name().to_string())
        } else {
            file.name().to_string()
        };

        let outpath = output_dir.join(&decoded_name);

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// Detect if ZIP file uses CP932 (Shift-JIS) encoding
///
/// This matches Python's behavior:
/// 1. Check non-UTF-8 entries (using `name_raw` for detection)
/// 2. Try CP437 -> CP932 conversion
/// 3. Check if result contains Japanese/CJK characters
fn detect_cp932_encoding(archive: &mut zip::ZipArchive<std::fs::File>) -> bool {
    for i in 0..archive.len() {
        let Ok(file) = archive.by_index(i) else {
            continue;
        };

        // Get the raw name bytes
        let raw_name_bytes = file.name_raw();
        let name = file.name();

        // If name contains non-ASCII characters that look garbled (CP437 decode of Shift-JIS),
        // try re-decoding as Shift-JIS
        if let Some(sjis_name) = try_decode_shift_jis(raw_name_bytes)
            && contains_japanese_or_cjk(&sjis_name)
        {
            return true;
        }

        if let Some(sjis_name) = decode_cp932_filename(name)
            && contains_japanese_or_cjk(&sjis_name)
        {
            return true;
        }
    }

    false
}

/// Try to decode bytes as Shift-JIS
fn try_decode_shift_jis(bytes: &[u8]) -> Option<String> {
    use encoding_rs::SHIFT_JIS;
    let (decoded, _, had_errors) = SHIFT_JIS.decode(bytes);
    if had_errors {
        None
    } else {
        Some(decoded.to_string())
    }
}

/// Decode filename from CP437 to CP932 (Shift-JIS)
///
/// This matches Python's `_try_decode_cp932_from_cp437` behavior.
fn decode_cp932_filename(name: &str) -> Option<String> {
    // Convert CP437 string to bytes
    let cp437_bytes = encode_cp437(name)?;

    // Try CP932 (Shift-JIS) decoding
    try_decode_shift_jis(&cp437_bytes)
}

/// Encode string to CP437 bytes
///
/// CP437 is the default encoding for ZIP files on DOS/Windows.
#[expect(clippy::too_many_lines)]
fn encode_cp437(name: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    for ch in name.chars() {
        let byte = if ch.is_ascii() {
            ch as u8
        } else {
            // CP437 extended characters (0x80-0xFF)
            match ch {
                'Ç' => 0x80,
                'ü' => 0x81,
                'é' => 0x82,
                'â' => 0x83,
                'ä' => 0x84,
                'à' => 0x85,
                'å' => 0x86,
                'ç' => 0x87,
                'ê' => 0x88,
                'ë' => 0x89,
                'è' => 0x8A,
                'ï' => 0x8B,
                'î' => 0x8C,
                'ì' => 0x8D,
                'Ä' => 0x8E,
                'Å' => 0x8F,
                'É' => 0x90,
                'æ' => 0x91,
                'Æ' => 0x92,
                'ô' => 0x93,
                'ö' => 0x94,
                'ò' => 0x95,
                'û' => 0x96,
                'ù' => 0x97,
                'ÿ' => 0x98,
                'Ö' => 0x99,
                'Ü' => 0x9A,
                '¢' => 0x9B,
                '£' => 0x9C,
                '¥' => 0x9D,
                '₧' => 0x9E,
                'ƒ' => 0x9F,
                'á' => 0xA0,
                'í' => 0xA1,
                'ó' => 0xA2,
                'ú' => 0xA3,
                'ñ' => 0xA4,
                'Ñ' => 0xA5,
                'ª' => 0xA6,
                'º' => 0xA7,
                '¿' => 0xA8,
                '⌐' => 0xA9,
                '¬' => 0xAA,
                '½' => 0xAB,
                '¼' => 0xAC,
                '¡' => 0xAD,
                '«' => 0xAE,
                '»' => 0xAF,
                '░' => 0xB0,
                '▒' => 0xB1,
                '▓' => 0xB2,
                '│' => 0xB3,
                '┤' => 0xB4,
                '╡' => 0xB5,
                '╢' => 0xB6,
                '╖' => 0xB7,
                '╕' => 0xB8,
                '╣' => 0xB9,
                '║' => 0xBA,
                '╗' => 0xBB,
                '╝' => 0xBC,
                '╜' => 0xBD,
                '╛' => 0xBE,
                '┐' => 0xBF,
                '└' => 0xC0,
                '┴' => 0xC1,
                '┬' => 0xC2,
                '├' => 0xC3,
                '─' => 0xC4,
                '┼' => 0xC5,
                '╞' => 0xC6,
                '╟' => 0xC7,
                '╚' => 0xC8,
                '╔' => 0xC9,
                '╩' => 0xCA,
                '╦' => 0xCB,
                '╠' => 0xCC,
                '═' => 0xCD,
                '╬' => 0xCE,
                '╧' => 0xCF,
                '╨' => 0xD0,
                '╤' => 0xD1,
                '╥' => 0xD2,
                '╙' => 0xD3,
                '╘' => 0xD4,
                '╒' => 0xD5,
                '╓' => 0xD6,
                '╫' => 0xD7,
                '╪' => 0xD8,
                '┘' => 0xD9,
                '┌' => 0xDA,
                '█' => 0xDB,
                '▄' => 0xDC,
                '▌' => 0xDD,
                '▐' => 0xDE,
                '▀' => 0xDF,
                'α' => 0xE0,
                'ß' => 0xE1,
                'Γ' => 0xE2,
                'π' => 0xE3,
                'Σ' => 0xE4,
                'σ' => 0xE5,
                'µ' => 0xE6,
                'τ' => 0xE7,
                'Φ' => 0xE8,
                'Θ' => 0xE9,
                'Ω' => 0xEA,
                'δ' => 0xEB,
                '∞' => 0xEC,
                'φ' => 0xED,
                'ε' => 0xEE,
                '∩' => 0xEF,
                '≡' => 0xF0,
                '±' => 0xF1,
                '≥' => 0xF2,
                '≤' => 0xF3,
                '⌠' => 0xF4,
                '⌡' => 0xF5,
                '÷' => 0xF6,
                '≈' => 0xF7,
                '°' => 0xF8,
                '∙' => 0xF9,
                '·' => 0xFA,
                '√' => 0xFB,
                'ⁿ' => 0xFC,
                '²' => 0xFD,
                '■' => 0xFE,
                _ => return None,
            }
        };
        bytes.push(byte);
    }
    Some(bytes)
}

/// Check if string contains Japanese or CJK characters
fn contains_japanese_or_cjk(name: &str) -> bool {
    name.chars().any(|ch| {
        matches!(ch,
            '\u{3040}'..='\u{309F}' |  // Hiragana
            '\u{30A0}'..='\u{30FF}' |  // Katakana
            '\u{3400}'..='\u{9FFF}' |  // CJK Unified Ideographs
            '\u{F900}'..='\u{FAFF}' |  // CJK Compatibility Ideographs
            '\u{FE30}'..='\u{FE4F}' |  // CJK Compatibility Forms
            '\u{20000}'..='\u{2A6DF}' // CJK Unified Ideographs Extension B
        )
    })
}

async fn extract_7z(archive_path: &Path, output_dir: &Path) -> Result<(), std::io::Error> {
    let archive_path = archive_path.to_path_buf();
    let output_dir = output_dir.to_path_buf();
    let result = tokio::task::spawn_blocking(move || {
        sevenz_rust::decompress_file(&archive_path, &output_dir)
    })
    .await;
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(std::io::Error::other(format!("7z error: {e}"))),
        Err(e) => Err(std::io::Error::other(format!("Join error: {e}"))),
    }
}

async fn extract_rar(archive_path: &Path, output_dir: &Path) -> Result<(), std::io::Error> {
    // Use unrar crate or fall back to external tool
    use tokio::process::Command;

    let output = Command::new("unrar")
        .args([
            "x",
            "-o+",
            &archive_path.to_string_lossy(),
            &output_dir.to_string_lossy(),
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(std::io::Error::other("Failed to extract rar archive"));
    }
    Ok(())
}

/// Move out files from nested folders in cache directory
///
/// This replicates Python's `move_out_files_in_folder_in_cache_dir(cache_dir_path)`:
/// - Iteratively unpacks nested folder structures
/// - Removes __MACOSX directories
/// - Handles single inner folder case (if >= 10 files, considered "done")
/// - If multiple inner folders, checks for BMS files to determine if state is acceptable
/// - Moves files out of inner directories to the cache root
///
/// Returns `true` on success, `false` on error or empty cache
#[expect(clippy::too_many_lines)]
pub fn move_out_files_in_folder_in_cache_dir(cache_dir_path: &Path) -> bool {
    let mut error = false;

    loop {
        let mut file_ext_count: HashMap<String, Vec<String>> = HashMap::new();
        let mut cache_folder_count: usize = 0;
        let mut cache_file_count: usize = 0;
        let mut inner_dir_name: Option<String> = None;

        // Scan cache directory
        let Ok(entries) = std::fs::read_dir(cache_dir_path) else {
            break;
        };

        for entry in entries.flatten() {
            let cache_path = entry.path();
            let cache_name = cache_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if cache_path.is_dir() {
                // Remove __MACOSX directory
                if cache_name == "__MACOSX" {
                    info!("Removing __MACOSX directory: {:?}", cache_path);
                    if let Err(e) = std::fs::remove_dir_all(&cache_path) {
                        info!("Failed to remove __MACOSX: {}", e);
                    }
                    continue;
                }
                // Normal directory
                cache_folder_count += 1;
                inner_dir_name = Some(cache_name.to_string());
            }

            if cache_path.is_file() {
                cache_file_count += 1;
                // Count extensions
                let ext = cache_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_string();
                file_ext_count
                    .entry(ext)
                    .or_default()
                    .push(cache_name.to_string());
            }
        }

        // Determine state
        let done;
        if cache_folder_count == 0 || (cache_folder_count == 1 && cache_file_count >= 10) {
            done = true;
        } else if cache_folder_count > 1 {
            // Check if there are BMS files anywhere in cache_dir
            let mut has_bms = false;
            for entry in walkdir::WalkDir::new(cache_dir_path).into_iter().flatten() {
                let path = entry.path();
                if path.is_file()
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && CHART_FILE_EXTS
                        .iter()
                        .any(|ext| name.to_lowercase().ends_with(ext))
                {
                    has_bms = true;
                    break;
                }
            }
            if has_bms {
                done = true;
            } else {
                info!(
                    " !_! {}: has more than 1 folders, please do it manually.",
                    cache_dir_path.display()
                );
                error = true;
                done = false;
            }
        } else {
            done = false;
        }

        if done || error {
            break;
        }

        // Move files out of inner directory
        if let Some(ref inner_name) = inner_dir_name {
            let inner_dir_path = cache_dir_path.join(inner_name);
            // Avoid two floor same name: if inner_dir_path/inner_name exists, rename it
            let inner_inner_dir_path = inner_dir_path.join(inner_name);
            if inner_inner_dir_path.is_dir() {
                info!(
                    " - Renaming inner inner dir name: {:?}",
                    inner_inner_dir_path
                );
                let new_path = inner_inner_dir_path.with_file_name(format!("{inner_name}-rep"));
                if let Err(e) = std::fs::rename(&inner_inner_dir_path, &new_path) {
                    info!("Failed to rename inner inner dir: {}", e);
                }
            }
            // Move files
            info!(
                " - Moving inner files in {:?} to {:?}",
                inner_dir_path, cache_dir_path
            );
            if let Err(e) = move_elements_across_dir(
                &inner_dir_path,
                cache_dir_path,
                DEFAULT_MOVE_OPTIONS,
                DEFAULT_REPLACE_OPTIONS.clone(),
            ) {
                info!("Failed to move elements: {}", e);
            }
            // Try to remove the now-empty inner directory
            let _ = std::fs::remove_dir(&inner_dir_path);
        }
    }

    // Final checks
    let (final_folder_count, final_file_count) = count_cache_contents(cache_dir_path);

    if error {
        return false;
    }

    if final_folder_count == 0 && final_file_count == 0 {
        info!(" !_! {}: Cache is Empty!", cache_dir_path.display());
        // Try to remove the empty cache directory
        let _ = std::fs::remove_dir(cache_dir_path);
        return false;
    }

    // Check for multiple mp4 files
    let mp4_files: usize = file_ext_count_at_path(cache_dir_path)
        .get("mp4")
        .map_or(0, std::vec::Vec::len);
    if mp4_files > 1 {
        info!(
            " - Tips: {} has more than 1 mp4 files!",
            cache_dir_path.display()
        );
    }

    true
}

/// Count folders and files in a directory
fn count_cache_contents(dir: &Path) -> (usize, usize) {
    let mut folder_count = 0;
    let mut file_count = 0;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                folder_count += 1;
            } else if entry.path().is_file() {
                file_count += 1;
            }
        }
    }

    (folder_count, file_count)
}

/// Get file extension counts in a directory
fn file_ext_count_at_path(dir: &Path) -> HashMap<String, Vec<String>> {
    let mut ext_count: HashMap<String, Vec<String>> = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_string();
                ext_count.entry(ext).or_default().push(name);
            }
        }
    }

    ext_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_num_set_file_names() {
        let temp_dir = std::env::temp_dir();
        let _names = get_num_set_file_names(&temp_dir);
    }

    #[test]
    fn test_safe_join_normal() {
        let base = PathBuf::from("/tmp/test");
        let result = safe_join(&base, "file.txt").unwrap();
        assert_eq!(result, PathBuf::from("/tmp/test/file.txt"));
    }

    #[test]
    fn test_safe_join_traversal() {
        let base = PathBuf::from("/tmp/test");
        let result = safe_join(&base, "../etc/passwd");
        assert!(result.is_none());
    }

    #[test]
    fn test_safe_join_absolute() {
        let base = PathBuf::from("/tmp/test");
        let result = safe_join(&base, "/etc/passwd");
        assert!(result.is_none());
    }
}
