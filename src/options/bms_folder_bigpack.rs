//! BMS big pack operations.
//!
//! This module provides functions for managing large BMS packs
//! including folder splitting, merging, and media file removal.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tracing::info;
use regex::Regex;

use crate::fs::pack_move::{move_elements_across_dir, REPLACE_OPTION_UPDATE_PACK, MoveOptions, ReplaceOptions, is_dir_having_file};

/// Regular expression for Japanese Hiragana
#[allow(dead_code)]
const RE_JAPANESE_HIRAGANA: &str = r"[぀-ゟ]+";
/// Regular expression for Japanese Katakana
#[allow(dead_code)]
const RE_JAPANESE_KATAKANA: &str = r"[゠-ヿ]+";
/// Regular expression for Chinese characters
#[allow(dead_code)]
const RE_CHINESE_CHARACTER: &str = r"[一-龥]+";

#[allow(dead_code)]
static RE_HIRAGANA: LazyLock<Regex> = LazyLock::new(|| Regex::new(RE_JAPANESE_HIRAGANA).unwrap());
#[allow(dead_code)]
static RE_KATAKANA: LazyLock<Regex> = LazyLock::new(|| Regex::new(RE_JAPANESE_KATAKANA).unwrap());
#[allow(dead_code)]
static RE_CHINESE: LazyLock<Regex> = LazyLock::new(|| Regex::new(RE_CHINESE_CHARACTER).unwrap());

/// First character classification rules
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FirstCharRule {
    pub name: String,
    pub check: fn(&str) -> bool,
}

#[allow(dead_code)]
fn _check_digit(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_digit())
}

fn _check_range(name: &str, start: char, end: char) -> bool {
    name.chars()
        .next()
        .is_some_and(|c| c.to_ascii_uppercase() >= start && c.to_ascii_uppercase() <= end)
}

/// Find the first character rule group for a name
#[allow(dead_code)]
fn find_first_char_rule(name: &str) -> String {
    if name.is_empty() {
        return "未分类".to_string();
    }

    let first_char = name.chars().next().unwrap();

    // Check digit
    if first_char.is_ascii_digit() {
        return "0-9".to_string();
    }

    // Check ASCII letter ranges
    if first_char.is_ascii_alphabetic() {
        let upper = first_char.to_ascii_uppercase();
        if ('A'..='D').contains(&upper) {
            return "ABCD".to_string();
        }
        if ('E'..='K').contains(&upper) {
            return "EFGHIJK".to_string();
        }
        if ('L'..='Q').contains(&upper) {
            return "LMNOPQ".to_string();
        }
        if ('R'..='T').contains(&upper) {
            return "RST".to_string();
        }
        if ('U'..='Z').contains(&upper) {
            return "UVWXYZ".to_string();
        }
    }

    // Check Japanese/Chinese
    let c_str = first_char.to_string();
    if RE_HIRAGANA.is_match(&c_str) {
        return "平假名".to_string();
    }
    if RE_KATAKANA.is_match(&c_str) {
        return "片假名".to_string();
    }
    if RE_CHINESE.is_match(&c_str) {
        return "汉字".to_string();
    }

    "+".to_string()
}

/// Split folders in `root_dir` into subdirectories based on first character
pub fn split_folders_with_first_char(root_dir: &Path) -> Result<(), std::io::Error> {
    if !root_dir.is_dir() {
        info!("{} is not a dir! Aborting...", root_dir.display());
        return Ok(());
    }

    let root_folder_name = root_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if root_folder_name.ends_with(']') {
        info!("{} endswith ']'. Aborting...", root_dir.display());
        return Ok(());
    }

    let parent_dir = match root_dir.parent() {
        Some(p) => p,
        None => return Ok(()),
    };

    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let element_path = entry.path();
        if !element_path.is_dir() {
            continue;
        }

        let element_name = element_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let rule = find_first_char_rule(element_name);
        let target_dir = parent_dir.join(format!("{root_folder_name} [{rule}]"));

        if !target_dir.is_dir() {
            std::fs::create_dir_all(&target_dir)?;
        }

        let target_path = target_dir.join(element_name);
        info!("Moving {:?} -> {:?}", element_path, target_path);
        std::fs::rename(&element_path, &target_path)?;
    }

    // Remove the original folder if empty
    if !is_dir_having_file(root_dir) {
        let _ = std::fs::remove_dir(root_dir);
    }

    Ok(())
}

/// Undo split pack operation - move folders back from categorized subdirs
pub fn undo_split_pack(root_dir: &Path) -> Result<(), std::io::Error> {
    let root_folder_name = root_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let parent_dir = match root_dir.parent() {
        Some(p) => p,
        None => return Ok(()),
    };

    // Find folders that start with root_folder_name [ and end with ]
    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(parent_dir) {
        for entry in entries.flatten() {
            let folder_path = entry.path();
            if !folder_path.is_dir() {
                continue;
            }
            let folder_name = folder_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if folder_name.starts_with(&format!("{root_folder_name} [")) && folder_name.ends_with(']') {
                info!(" - {:?} <- {:?}", root_dir, folder_path);
                pairs.push((folder_path, root_dir.to_path_buf()));
            }
        }
    }

    if pairs.is_empty() {
        return Ok(());
    }

    // Confirm with user (in Rust we just proceed)
    for (from, to) in &pairs {
        move_elements_across_dir(from, to, MoveOptions::default(), ReplaceOptions::default())?;
    }

    Ok(())
}

/// Merge split folders back together
pub fn merge_split_folders(root_dir: &Path) -> Result<(), std::io::Error> {
    let dir_names: Vec<String> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    let mut pairs: Vec<(String, String)> = Vec::new();

    for dir_name in &dir_names {
        let dir_path = root_dir.join(dir_name);
        if !dir_path.is_dir() {
            continue;
        }

        // Check if ends with "]"
        if !dir_name.ends_with(']') {
            continue;
        }

        // Find dir_name_without_artist
        if let Some(bracket_pos) = dir_name.rfind('[') {
            if bracket_pos < 2 {
                continue;
            }
            let dir_name_without_artist = &dir_name[..bracket_pos - 1];
            if dir_name_without_artist.is_empty() {
                continue;
            }

            let dir_path_without_artist = root_dir.join(dir_name_without_artist);
            if !dir_path_without_artist.is_dir() {
                continue;
            }

            // Check for multiple folders with same prefix
            let matching_dirs: Vec<_> = dir_names.iter()
                .filter(|n| n.starts_with(&format!("{dir_name_without_artist} [")))
                .collect();

            if matching_dirs.len() > 2 {
                info!(" !_! {} have more than 2 folders: {:?}", dir_name_without_artist, matching_dirs);
                continue;
            }

            pairs.push((dir_name.clone(), dir_name_without_artist.to_string()));
        }
    }

    // Check for duplicates
    let mut last_from = String::new();
    let mut duplicates: Vec<String> = Vec::new();
    for (_target, from_dir_name) in &pairs {
        if last_from == *from_dir_name {
            duplicates.push(from_dir_name.clone());
        }
        last_from = from_dir_name.clone();
    }

    if !duplicates.is_empty() {
        info!("Duplicate! {:?}", duplicates);
        return Ok(());
    }

    // Print pairs and proceed
    for (target, from) in &pairs {
        info!("- Find Dir pair: {} <- {}", target, from);
    }

    for (target_dir_name, from_dir_name) in pairs {
        let from_dir_path = root_dir.join(&from_dir_name);
        let target_dir_path = root_dir.join(&target_dir_name);
        info!(" - Moving: {} <- {}", target_dir_name, from_dir_name);
        move_elements_across_dir(&from_dir_path, &target_dir_path, MoveOptions::default(), ReplaceOptions::default())?;
    }

    Ok(())
}

/// Move works from one pack directory to another
pub fn move_works_in_pack(root_dir_from: &Path, root_dir_to: &Path) -> Result<(), std::io::Error> {
    if root_dir_from == root_dir_to {
        return Ok(());
    }

    let mut move_count = 0;

    if let Ok(entries) = std::fs::read_dir(root_dir_from) {
        for entry in entries.flatten() {
            let bms_dir = entry.path();
            if !bms_dir.is_dir() {
                continue;
            }

            let bms_dir_name = bms_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            info!("Moving: {}", bms_dir_name);

            let dst_bms_dir = root_dir_to.join(bms_dir_name);
            move_elements_across_dir(&bms_dir, &dst_bms_dir, MoveOptions::default(), REPLACE_OPTION_UPDATE_PACK.clone())?;
            move_count += 1;
        }
    }

    if move_count > 0 {
        info!("Move {} songs.", move_count);
        return Ok(());
    }

    // Deal with song dir if no subdirs
    move_elements_across_dir(root_dir_from, root_dir_to, MoveOptions::default(), REPLACE_OPTION_UPDATE_PACK.clone())?;

    Ok(())
}

/// Media file removal rule
pub type RemoveMediaRule = Vec<(Vec<&'static str>, Vec<&'static str>)>;

/// ORAJA removal rule - remove redundant video files and prefer specific formats
#[must_use] 
pub fn get_remove_media_rule_oraja() -> RemoveMediaRule {
    vec![
        (vec!["mp4"], vec!["avi", "wmv", "mpg", "mpeg"]),
        (vec!["avi"], vec!["wmv", "mpg", "mpeg"]),
        (vec!["flac", "wav"], vec!["ogg"]),
        (vec!["flac"], vec!["wav"]),
        (vec!["mpg"], vec!["wmv"]),
    ]
}

/// WAV fill FLAC rule
#[must_use] 
pub fn get_remove_media_rule_wav_fill_flac() -> RemoveMediaRule {
    vec![(vec!["wav"], vec!["flac"])]
}

/// MPG fill WMV rule
#[must_use] 
pub fn get_remove_media_rule_mpg_fill_wmv() -> RemoveMediaRule {
    vec![(vec!["mpg"], vec!["wmv"])]
}

/// Remove unneeded media files according to rule in a work directory
fn workdir_remove_unneed_media_files(work_dir: &Path, rule: &RemoveMediaRule) -> Result<(), std::io::Error> {
    let mut remove_pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut removed_files: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    let entries: Vec<_> = std::fs::read_dir(work_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in &entries {
        let check_file_path = entry.path();
        if !check_file_path.is_file() {
            continue;
        }

        let file_ext = check_file_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        for (upper_exts, lower_exts) in rule {
            if !upper_exts.iter().any(|e| e.to_lowercase() == file_ext) {
                continue;
            }

            // File is empty?
            if check_file_path.metadata().map(|m| m.len() == 0).unwrap_or(false) {
                info!(" - !x!: File {:?} is Empty! Skipping...", check_file_path);
                continue;
            }

            // File is in upper_exts, search for file in lower_exts
            for lower_ext in lower_exts {
                let replacing_file_path = check_file_path.with_extension(*lower_ext);
                if !replacing_file_path.is_file() {
                    continue;
                }
                if removed_files.contains(&replacing_file_path) {
                    continue;
                }
                remove_pairs.push((check_file_path.clone(), replacing_file_path.clone()));
                removed_files.insert(replacing_file_path);
            }
        }
    }

    // Remove files
    for (check_file_path, replacing_file_path) in &remove_pairs {
        info!("- Remove file {:?}, because {:?} exists.", replacing_file_path.file_name(), check_file_path.file_name());
        let _ = std::fs::remove_file(replacing_file_path);
    }

    // Remove zero-sized media files
    remove_zero_sized_media_files(work_dir)?;

    // Count extensions for mp4 warning
    let mut ext_count: HashMap<String, Vec<String>> = HashMap::new();
    let entries: Vec<_> = std::fs::read_dir(work_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in &entries {
        let count_file_path = entry.path();
        if !count_file_path.is_file() {
            continue;
        }
        let file_ext = count_file_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let file_name = count_file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        ext_count.entry(file_ext).or_default().push(file_name);
    }

    if let Some(mp4_files) = ext_count.get("mp4")
        && mp4_files.len() > 1 {
            info!(" - Tips: {:?} has more than 1 mp4 files!", work_dir);
        }

    Ok(())
}

/// Remove unneeded media files from all works in `root_dir`
#[allow(dead_code)]
pub fn remove_unneed_media_files(root_dir: &Path, rule: Option<RemoveMediaRule>) -> Result<(), std::io::Error> {
    let rule = rule.unwrap_or_else(get_remove_media_rule_oraja);

    let entries: Vec<_> = std::fs::read_dir(root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let bms_dir_path = entry.path();
        if !bms_dir_path.is_dir() {
            continue;
        }
        workdir_remove_unneed_media_files(&bms_dir_path, &rule)?;
    }

    Ok(())
}

/// Remove zero-sized media files and temp files
#[allow(dead_code)]
pub fn remove_zero_sized_media_files(current_dir: &Path) -> Result<(), std::io::Error> {
    const TEMP_FILES: &[&str] = &["desktop.ini", "thumbs.db", ".ds_store"];
    const MEDIA_EXTS: &[&str] = &["flac", "ogg", "wav", "mp4", "mkv", "avi", "wmv", "mpg", "mpeg", "jpg", "png", "bmp", "svg"];

    let mut next_dirs: Vec<PathBuf> = Vec::new();

    let entries: Vec<_> = std::fs::read_dir(current_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in &entries {
        let element_path = entry.path();
        let element_name = element_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if element_path.is_file() {
            // Check if temp file
            let is_temp = TEMP_FILES.contains(&element_name.to_lowercase().as_str())
                || element_name.starts_with(".trash-")
                || element_name.starts_with("._");

            if is_temp {
                info!(" - Remove temp file: {:?}", element_path);
                let _ = std::fs::remove_file(&element_path);
                continue;
            }

            // Check if zero-sized media file
            let ext = element_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !MEDIA_EXTS.contains(&ext.as_str()) {
                continue;
            }

            if element_path.metadata().map(|m| m.len() == 0).unwrap_or(false) {
                info!(" - Remove empty file: {:?}", element_path);
                let _ = std::fs::remove_file(&element_path);
            }
        } else if element_path.is_dir() {
            next_dirs.push(element_path);
        }
    }

    // Recurse into subdirectories
    for next_dir in next_dirs {
        remove_zero_sized_media_files(&next_dir)?;
    }

    Ok(())
}

/// Move works out one level (un-nest subdirectories)
#[allow(dead_code)]
pub fn move_out_works(target_root_dir: &Path) -> Result<(), std::io::Error> {
    let entries: Vec<_> = std::fs::read_dir(target_root_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let root_dir_path = entry.path();
        if !root_dir_path.is_dir() {
            continue;
        }

        let work_entries: Vec<_> = std::fs::read_dir(&root_dir_path)?
            .filter_map(std::result::Result::ok)
            .collect();

        for work_entry in work_entries {
            let work_dir_path = work_entry.path();
            if !work_dir_path.is_dir() {
                continue;
            }

            let work_dir_name = work_dir_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let target_work_dir_path = target_root_dir.join(work_dir_name);

            move_elements_across_dir(&work_dir_path, &target_work_dir_path, MoveOptions::default(), REPLACE_OPTION_UPDATE_PACK.clone())?;
        }

        // Remove empty root_dir if not having files
        if !is_dir_having_file(&root_dir_path) {
            let _ = std::fs::remove_dir(&root_dir_path);
        }
    }

    Ok(())
}

/// Move works with same name from one dir to another
#[allow(dead_code)]
pub fn move_works_with_same_name(root_dir_from: &Path, root_dir_to: &Path) -> Result<(), std::io::Error> {
    if !root_dir_from.is_dir() || !root_dir_to.is_dir() {
        return Ok(());
    }

    // Get source subdirectories
    let from_subdirs: Vec<(String, PathBuf)> = std::fs::read_dir(root_dir_from)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|n| (n.to_string(), e.path())))
        .collect();

    // Get target subdirectories
    let to_subdirs: Vec<String> = std::fs::read_dir(root_dir_to)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();

    // Find matching pairs
    for (from_name, from_path) in &from_subdirs {
        for to_name in &to_subdirs {
            if to_name.starts_with(from_name) {
                let to_path = root_dir_to.join(to_name);
                info!(" -> {} => {}", from_name, to_name);
                pairs.push((from_path.clone(), to_path));
                break;
            }
        }
    }

    // Execute moves
    for (_, from_path, _, to_path) in pairs.iter().map(|(f, t)| (f.clone(), f.clone(), t.clone(), t.clone())) {
        info!("合并: {:?} -> {:?}", from_path, to_path);
        move_elements_across_dir(&from_path, &to_path, MoveOptions::default(), REPLACE_OPTION_UPDATE_PACK.clone())?;
    }

    Ok(())
}

/// Move works with same name to sibling directories
#[allow(dead_code)]
pub fn move_works_with_same_name_to_siblings(root_dir_from: &Path) -> Result<(), std::io::Error> {
    if !root_dir_from.is_dir() {
        return Ok(());
    }

    let parent_dir = match root_dir_from.parent() {
        Some(p) => p,
        None => return Ok(()),
    };

    let root_base_name = root_dir_from.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Get source subdirectories
    let from_subdirs: Vec<(String, PathBuf)> = std::fs::read_dir(root_dir_from)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|n| (n.to_string(), e.path())))
        .collect();

    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();

    // Iterate sibling directories
    if let Ok(siblings) = std::fs::read_dir(parent_dir) {
        for sibling in siblings.flatten() {
            let sibling_path = sibling.path();
            if !sibling_path.is_dir() {
                continue;
            }

            let sibling_name = sibling_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if sibling_name == root_base_name {
                continue;
            }

            // Get sibling's subdirectories
            let to_subdirs: Vec<String> = std::fs::read_dir(&sibling_path)?
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .collect();

            // Find matching pairs
            for (from_name, from_path) in &from_subdirs {
                for to_name in &to_subdirs {
                    if to_name.starts_with(from_name) {
                        let target_path = sibling_path.join(to_name);
                        info!(" -> {} => {:?}", from_name, target_path);
                        pairs.push((from_path.clone(), target_path));
                        break;
                    }
                }
            }
        }
    }

    // Execute moves
    for (from_path, target_path) in &pairs {
        info!("合并: {:?} -> {:?}", from_path, target_path);
        move_elements_across_dir(from_path, target_path, MoveOptions::default(), REPLACE_OPTION_UPDATE_PACK.clone())?;
    }

    Ok(())
}
