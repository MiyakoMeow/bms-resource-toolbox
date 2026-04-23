use std::collections::HashMap;
use std::path::{Path, PathBuf};

use zip::HasZipMetadata;

use crate::bms::CHART_FILE_EXTS;
use crate::fs::move_ops::{self, MoveOptions, ReplaceAction, ReplaceOptions};
use crate::AppError;

#[allow(clippy::missing_errors_doc, clippy::unnecessary_debug_formatting)]
fn safe_join(base_dir: &Path, relative: &Path) -> crate::Result<PathBuf> {
    let rel_str = relative.to_string_lossy();
    let rel_cleaned = rel_str.trim_start_matches('/');
    let candidate = base_dir.join(rel_cleaned);
    let candidate = std::path::absolute(&candidate).unwrap_or(candidate);
    let base = std::path::absolute(base_dir).unwrap_or_else(|_| base_dir.to_path_buf());
    if !candidate.starts_with(&base) {
        return Err(AppError::InvalidArg(format!("Unsafe path: {relative:?}")));
    }
    Ok(candidate)
}

fn set_mtime(path: &Path, year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) {
    let Ok(dt) = chrono_like_timestamp(year, month, day, hour, minute, second) else {
        return;
    };
    let ft = filetime::FileTime::from_unix_time(dt, 0);
    if let Err(e) = filetime::set_file_mtime(path, ft) {
        tracing::debug!("set_mtime failed for {}: {e}", path.display());
    }
}

fn chrono_like_timestamp(
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> Result<i64, ()> {
    let year = i32::from(year);
    let month = i32::from(month);
    let day = i32::from(day);
    let hour = i32::from(hour);
    let minute = i32::from(minute);
    let second = i32::from(second);

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(());
    }

    let is_leap = (year % 4 == 0) && ((year % 100 != 0) || (year % 400 == 0));
    let days_in_months: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let days_in_feb = if is_leap { 29 } else { 28 };

    let mut days = (year - 1970) * 365;
    days += (year - 1969) / 4;
    days -= (year - 1901) / 100;
    days += (year - 1601) / 400;

    for i in 0..(month - 1) {
        let idx = usize::try_from(i).unwrap_or(0);
        days += if i == 1 { days_in_feb } else { days_in_months[idx] };
    }
    days += day - 1;

    Ok(i64::from(days) * 86400 + i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second))
}

fn try_decode_cp932(raw_bytes: &[u8]) -> Option<String> {
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(raw_bytes);
    if had_errors {
        return None;
    }
    let s = decoded.into_owned();
    let has_jp = s.chars().any(|c| {
        ('\u{3040}'..='\u{30ff}').contains(&c) || ('\u{3400}'..='\u{9fff}').contains(&c)
    });
    if has_jp { Some(s) } else { None }
}

fn detect_use_cp932(archive: &mut zip::ZipArchive<std::fs::File>) -> bool {
    let count = archive.len();
    for i in 0..count {
        let Ok(file) = archive.by_index(i) else { continue };
        if file.get_metadata().is_utf8 {
            continue;
        }
        let raw = file.name_raw();
        if try_decode_cp932(raw).is_some() {
            return true;
        }
    }
    false
}

fn decode_entry_name(file: &zip::read::ZipFile, use_cp932: bool) -> String {
    if file.get_metadata().is_utf8 {
        return file.name().to_owned();
    }
    if use_cp932
        && let Some(s) = try_decode_cp932(file.name_raw())
    {
        return s;
    }
    file.name().to_owned()
}

pub fn unzip_zip_to_dir(file_path: &Path, cache_dir: &Path) -> crate::Result<()> {
    println!("Extracting {} to {} (zip)", file_path.display(), cache_dir.display());
    let mut archive = zip::ZipArchive::new(std::fs::File::open(file_path)?)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;

    let use_cp932 = detect_use_cp932(&mut archive);

    let file_count = archive.len();
    for i in 0..file_count {
        let mut file = archive.by_index(i)
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;

        let rel_name = decode_entry_name(&file, use_cp932);
        let out_path = safe_join(cache_dir, Path::new(&rel_name))?;

        let is_dir = file.is_dir() || rel_name.ends_with('/');
        if is_dir {
            std::fs::create_dir_all(&out_path)?;
            if let Some(dt) = file.last_modified() {
                set_mtime(&out_path, dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second());
            }
            continue;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut buf = Vec::with_capacity(usize::try_from(file.size()).unwrap_or(0));
        std::io::Read::read_to_end(&mut file, &mut buf)?;
        std::fs::write(&out_path, &buf)?;

        if let Some(dt) = file.last_modified() {
            set_mtime(&out_path, dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second());
        }
    }

    Ok(())
}

pub fn unzip_7z_to_dir(file_path: &Path, cache_dir: &Path) -> crate::Result<()> {
    println!("Extracting {} to {} (7z)", file_path.display(), cache_dir.display());
    let status = std::process::Command::new("7z")
        .arg("x")
        .arg(file_path)
        .arg(format!("-o{}", cache_dir.display()))
        .arg("-y")
        .status()?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(AppError::Process { command: "7z x".into(), code });
    }
    Ok(())
}

pub fn unzip_rar_to_dir(file_path: &Path, cache_dir: &Path) -> crate::Result<()> {
    println!("Extracting {} to {} (RAR)", file_path.display(), cache_dir.display());
    let status = std::process::Command::new("unrar")
        .arg("x")
        .arg("-o+")
        .arg(file_path)
        .arg(cache_dir)
        .status()?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(AppError::Process { command: "unrar x".into(), code });
    }
    Ok(())
}

pub fn unzip_file_to_cache_dir(file_path: &Path, cache_dir: &Path) -> crate::Result<()> {
    let file_suffix = file_path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match file_suffix.as_str() {
        "zip" => unzip_zip_to_dir(file_path, cache_dir),
        "7z" => unzip_7z_to_dir(file_path, cache_dir),
        "rar" => unzip_rar_to_dir(file_path, cache_dir),
        _ => {
            let file_name = file_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let parts: Vec<&str> = file_name.splitn(2, ' ').collect();
            let target_name = parts.get(1).map(|s| (*s).to_string()).unwrap_or(file_name);
            let target_path = cache_dir.join(&target_name);
            println!("Copying {} to {}", file_path.display(), target_path.display());
            std::fs::copy(file_path, &target_path)?;
            Ok(())
        }
    }
}

#[must_use]
pub fn get_num_set_file_names(pack_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(pack_dir) else {
        return vec![];
    };

    let mut names: Vec<(u64, String)> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let id_str = name.split(' ').next()?;
            id_str.parse::<u64>().ok().map(|id| (id, name))
        })
        .collect();

    names.sort_by_key(|(id, _)| *id);
    names.into_iter().map(|(_, name)| name).collect()
}

pub fn move_out_files_in_folder_in_cache_dir(cache_dir: &Path) -> crate::Result<bool> {
    let move_opts = MoveOptions { print_info: true };
    let replace_opts = ReplaceOptions {
        ext: HashMap::new(),
        default: ReplaceAction::Replace,
    };

    let mut file_ext_count: HashMap<String, Vec<String>> = HashMap::new();
    let mut done = false;
    let mut error = false;

    loop {
        file_ext_count.clear();
        let mut cache_folder_count: usize = 0;
        let mut cache_file_count: usize = 0;
        let mut inner_dir_name: Option<String> = None;

        let Ok(entries) = std::fs::read_dir(cache_dir) else {
            return Ok(false);
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                if name == "__MACOSX" {
                    let _ = std::fs::remove_dir_all(&path);
                    continue;
                }
                cache_folder_count += 1;
                inner_dir_name = Some(name.clone());
            }

            if path.is_file() {
                cache_file_count += 1;
                let ext = name.rsplit('.').next().unwrap_or("").to_string();
                file_ext_count.entry(ext).or_default().push(name);
            }
        }

        if cache_folder_count == 0 {
            done = true;
        }

        if cache_folder_count == 1 && cache_file_count >= 10 {
            done = true;
        }

        if cache_folder_count > 1 {
            let has_bms = has_chart_files(cache_dir);
            if has_bms {
                done = true;
            } else {
                println!(" !_! {}: has more than 1 folders, please do it manually.", cache_dir.display());
                error = true;
            }
        }

        if done || error {
            break;
        }

        if let Some(ref inner_name) = inner_dir_name {
            let inner_dir_path = cache_dir.join(inner_name);
            let inner_inner = inner_dir_path.join(inner_name);
            if inner_inner.is_dir() {
                println!(" - Renaming inner inner dir name: {}", inner_inner.display());
                let new_path = inner_inner.with_file_name(format!("{inner_name}-rep"));
                std::fs::rename(&inner_inner, &new_path)?;
            }
            println!(" - Moving inner files in {} to {}", inner_dir_path.display(), cache_dir.display());
            move_ops::move_elements_across_dir(&inner_dir_path, cache_dir, &move_opts, &replace_opts);
            let _ = std::fs::remove_dir(&inner_dir_path);
        }
    }

    if error {
        return Ok(false);
    }

    if cache_dir_is_empty(cache_dir) {
        println!(" !_! {}: Cache is Empty!", cache_dir.display());
        let _ = std::fs::remove_dir(cache_dir);
        return Ok(false);
    }

    if let Some(mp4_list) = file_ext_count.get("mp4")
        && mp4_list.len() > 1
    {
        println!(" - Tips: {} has more than 1 mp4 files! {:?}", cache_dir.display(), mp4_list);
    }

    Ok(true)
}

fn has_chart_files(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let lower = format!(".{ext}").to_lowercase();
                if CHART_FILE_EXTS.contains(&lower.as_str()) {
                    return true;
                }
            }
        } else if path.is_dir() && has_chart_files(&path) {
            return true;
        }
    }
    false
}

fn cache_dir_is_empty(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return true;
    };
    entries.flatten().next().is_none()
}
