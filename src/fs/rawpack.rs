use chrono::TimeZone;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::bms::types::CHART_FILE_EXTS;
use crate::fs::pack_move::{
    DEFAULT_MOVE_OPTIONS, DEFAULT_REPLACE_OPTIONS, move_elements_across_dir,
};

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

    if !current.starts_with(base) {
        return None;
    }

    if let Some(parent) = current.parent()
        && parent.exists()
        && let (Ok(resolved), Ok(resolved_base)) = (parent.canonicalize(), base.canonicalize())
        && !resolved.starts_with(&resolved_base)
    {
        return None;
    }

    Some(current)
}

fn set_mtime(path: &Path, dt: Option<zip::DateTime>) {
    let Some(dt) = dt else { return };
    let local_dt = chrono::Local.with_ymd_and_hms(
        i32::from(dt.year()),
        u32::from(dt.month()),
        u32::from(dt.day()),
        u32::from(dt.hour()),
        u32::from(dt.minute()),
        u32::from(dt.second()),
    );
    if let Some(dt) = local_dt.single() {
        let ft = filetime::FileTime::from_unix_time(dt.timestamp(), 0);
        let _ = filetime::set_file_mtime(path, ft);
    }
}

#[must_use]
pub async fn get_num_set_file_names(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();

    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return names;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        if !entry.path().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let id_str = name.split(' ').next().unwrap_or("");
        if id_str.is_empty()
            || !id_str
                .chars()
                .all(|c| c.is_ascii_digit() || ('\u{FF10}'..='\u{FF19}').contains(&c))
        {
            continue;
        }
        names.push(name);
    }
    names
}

pub async fn extract_archive(archive_path: &Path, output_dir: &Path) -> Result<(), std::io::Error> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    tokio::fs::create_dir_all(output_dir).await?;

    match ext.as_str() {
        "zip" => {
            let archive_path = archive_path.to_path_buf();
            let output_dir = output_dir.to_path_buf();
            match tokio::task::spawn_blocking(move || extract_zip(&archive_path, &output_dir)).await
            {
                Ok(result) => result,
                Err(e) => Err(std::io::Error::other(format!("Join error: {e}"))),
            }
        }
        "7z" => extract_7z(archive_path, output_dir).await,
        "rar" => extract_rar(archive_path, output_dir).await,
        _ => {
            let target_path = output_dir.join(archive_path.file_name().unwrap_or_default());
            tokio::fs::copy(archive_path, &target_path).await?;
            Ok(())
        }
    }
}

fn extract_zip(archive_path: &Path, output_dir: &Path) -> Result<(), std::io::Error> {
    use zip::ZipArchive;

    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    let use_cp932 = detect_cp932_encoding(&mut archive);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let decoded_name = if let Some(enclosed) = file.enclosed_name() {
            enclosed.to_string_lossy().to_string()
        } else if use_cp932 {
            decode_cp932_filename(file.name()).unwrap_or_else(|| file.name().to_string())
        } else {
            file.name().to_string()
        };

        let Some(outpath) = safe_join(output_dir, &decoded_name) else {
            continue;
        };

        let dt = file.last_modified();

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
            set_mtime(&outpath, dt);
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
            set_mtime(&outpath, dt);
        }
    }

    Ok(())
}

fn detect_cp932_encoding(archive: &mut zip::ZipArchive<std::fs::File>) -> bool {
    for i in 0..archive.len() {
        let Ok(file) = archive.by_index(i) else {
            continue;
        };

        let raw_name_bytes = file.name_raw();
        let name = file.name();

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

fn try_decode_shift_jis(bytes: &[u8]) -> Option<String> {
    use encoding_rs::SHIFT_JIS;
    let (decoded, _, had_errors) = SHIFT_JIS.decode(bytes);
    if had_errors {
        None
    } else {
        Some(decoded.to_string())
    }
}

fn decode_cp932_filename(name: &str) -> Option<String> {
    let cp437_bytes = encode_cp437(name)?;
    try_decode_shift_jis(&cp437_bytes)
}

#[allow(clippy::too_many_lines)]
fn encode_cp437(name: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    for ch in name.chars() {
        let byte = if ch.is_ascii() {
            ch as u8
        } else {
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

fn contains_japanese_or_cjk(name: &str) -> bool {
    name.chars().any(|ch| {
        matches!(ch,
            '\u{3040}'..='\u{309F}' |
            '\u{30A0}'..='\u{30FF}' |
            '\u{3400}'..='\u{9FFF}' |
            '\u{F900}'..='\u{FAFF}' |
            '\u{FE30}'..='\u{FE4F}' |
            '\u{20000}'..='\u{2A6DF}'
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

pub async fn move_out_files_in_folder_in_cache_dir(cache_dir_path: &Path) -> bool {
    let mut error = false;
    let mut file_ext_count: HashMap<String, Vec<String>>;
    loop {
        file_ext_count = HashMap::new();
        let mut cache_folder_count: usize = 0;
        let mut cache_file_count: usize = 0;
        let mut inner_dir_name: Option<String> = None;

        let Ok(mut entries) = tokio::fs::read_dir(cache_dir_path).await else {
            break;
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let cache_path = entry.path();
            let cache_name = cache_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if cache_path.is_dir() {
                if cache_name == "__MACOSX" {
                    println!("Removing __MACOSX directory: {cache_path:?}");
                    if let Err(e) = tokio::fs::remove_dir_all(&cache_path).await {
                        println!("Failed to remove __MACOSX: {e}");
                    }
                    continue;
                }
                cache_folder_count += 1;
                inner_dir_name = Some(cache_name.to_string());
            }

            if cache_path.is_file() {
                cache_file_count += 1;
                let ext = crate::fs::utils::get_ext(&cache_path).to_string();
                file_ext_count
                    .entry(ext)
                    .or_default()
                    .push(cache_name.to_string());
            }
        }

        let done;
        if cache_folder_count == 0 || (cache_folder_count == 1 && cache_file_count >= 10) {
            done = true;
        } else if cache_folder_count > 1 {
            let has_bms = has_chart_file_recursive(cache_dir_path).await;
            if has_bms {
                done = true;
            } else {
                println!(
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

        if let Some(ref inner_name) = inner_dir_name {
            let inner_dir_path = cache_dir_path.join(inner_name);
            let inner_inner_dir_path = inner_dir_path.join(inner_name);
            if inner_inner_dir_path.is_dir() {
                println!(" - Renaming inner inner dir name: {inner_inner_dir_path:?}");
                let new_path = inner_inner_dir_path.with_file_name(format!("{inner_name}-rep"));
                if let Err(e) = tokio::fs::rename(&inner_inner_dir_path, &new_path).await {
                    println!("Failed to rename inner inner dir: {e}");
                }
            }
            println!(" - Moving inner files in {inner_dir_path:?} to {cache_dir_path:?}");
            if let Err(e) = move_elements_across_dir(
                &inner_dir_path,
                cache_dir_path,
                DEFAULT_MOVE_OPTIONS,
                &DEFAULT_REPLACE_OPTIONS,
            )
            .await
            {
                println!("Failed to move elements: {e}");
            }
            let _ = tokio::fs::remove_dir(&inner_dir_path).await;
        }
    }

    let (final_folder_count, final_file_count) = count_cache_contents(cache_dir_path).await;

    if error {
        return false;
    }

    if final_folder_count == 0 && final_file_count == 0 {
        println!(" !_! {}: Cache is Empty!", cache_dir_path.display());
        let _ = tokio::fs::remove_dir(cache_dir_path).await;
        return false;
    }

    let mp4_count = file_ext_count.get("mp4").map_or(0, Vec::len);
    if mp4_count > 1 {
        println!(
            " - Tips: {} has more than 1 mp4 files!",
            cache_dir_path.display()
        );
    }

    true
}

async fn has_chart_file_recursive(dir: &Path) -> bool {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return false;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_file()
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
            && CHART_FILE_EXTS
                .iter()
                .any(|ext| name.to_lowercase().ends_with(ext))
        {
            return true;
        } else if path.is_dir() && Box::pin(has_chart_file_recursive(&path)).await {
            return true;
        }
    }
    false
}

async fn count_cache_contents(dir: &Path) -> (usize, usize) {
    let mut folder_count = 0;
    let mut file_count = 0;

    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.path().is_dir() {
                folder_count += 1;
            } else if entry.path().is_file() {
                file_count += 1;
            }
        }
    }

    (folder_count, file_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_num_set_file_names() {
        let temp_dir = std::env::temp_dir();
        let _names = get_num_set_file_names(&temp_dir).await;
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
