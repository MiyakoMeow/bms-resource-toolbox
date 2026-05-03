//! BMS big pack operations.
//!
//! This module provides functions for managing large BMS packs
//! including folder splitting, merging, and media file removal.

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::fs::pack_move::{
    MoveOptions, REPLACE_OPTION_UPDATE_PACK, ReplaceOptions, is_dir_having_file,
    move_elements_across_dir,
};

/// Regular expression for Japanese Hiragana
const RE_JAPANESE_HIRAGANA: &str = r"[぀-ゟ]+";
/// Regular expression for Japanese Katakana
const RE_JAPANESE_KATAKANA: &str = r"[゠-ヿ]+";
/// Regular expression for Chinese characters
const RE_CHINESE_CHARACTER: &str = r"[一-龥]+";

static RE_HIRAGANA: LazyLock<Regex> = LazyLock::new(|| Regex::new(RE_JAPANESE_HIRAGANA).unwrap());
static RE_KATAKANA: LazyLock<Regex> = LazyLock::new(|| Regex::new(RE_JAPANESE_KATAKANA).unwrap());
static RE_CHINESE: LazyLock<Regex> = LazyLock::new(|| Regex::new(RE_CHINESE_CHARACTER).unwrap());

fn _check_range(name: &str, start: char, end: char) -> bool {
    name.chars()
        .next()
        .is_some_and(|c| c.to_ascii_uppercase() >= start && c.to_ascii_uppercase() <= end)
}

/// Find the first character rule group for a name
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
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn split_folders_with_first_char(root_dir: &Path) -> Result<(), std::io::Error> {
    if !root_dir.is_dir() {
        println!("{} is not a dir! Aborting...", root_dir.display());
        return Ok(());
    }

    let root_folder_name = root_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if root_folder_name.ends_with(']') {
        println!("{} endswith ']'. Aborting...", root_dir.display());
        return Ok(());
    }

    let Some(parent_dir) = root_dir.parent() else {
        return Ok(());
    };

    let mut read_dir = tokio::fs::read_dir(root_dir).await?;
    let mut entries = Vec::new();
    while let Some(entry) = read_dir.next_entry().await? {
        entries.push(entry);
    }

    for entry in entries {
        let element_path = entry.path();

        let element_name = element_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let rule = find_first_char_rule(element_name);
        let target_dir = parent_dir.join(format!("{root_folder_name} [{rule}]"));

        if !target_dir.is_dir() {
            tokio::fs::create_dir_all(&target_dir).await?;
        }

        let target_path = target_dir.join(element_name);
        println!("Moving {element_path:?} -> {target_path:?}");
        tokio::fs::rename(&element_path, &target_path).await?;
    }

    // Remove the original folder if empty
    if !is_dir_having_file(root_dir).await {
        let _ = tokio::fs::remove_dir(root_dir).await;
    }

    Ok(())
}

/// Undo split pack operation - move folders back from categorized subdirs
///
/// This matches Python's `undo_split_pack` behavior:
/// - Shows pairs of directories to merge
/// - Asks for user confirmation before proceeding
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn undo_split_pack(root_dir: &Path) -> Result<(), std::io::Error> {
    let root_folder_name = root_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let Some(parent_dir) = root_dir.parent() else {
        return Ok(());
    };

    // Find folders that start with root_folder_name [ and end with ]
    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();

    if let Ok(mut read_dir) = tokio::fs::read_dir(parent_dir).await {
        while let Some(entry) = read_dir.next_entry().await? {
            let folder_path = entry.path();
            if !folder_path.is_dir() {
                continue;
            }
            let folder_name = folder_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if folder_name.starts_with(&format!("{root_folder_name} ["))
                && folder_name.ends_with(']')
            {
                println!(" - {} <- {}", root_dir.display(), folder_path.display());
                pairs.push((folder_path, root_dir.to_path_buf()));
            }
        }
    }

    if pairs.is_empty() {
        return Ok(());
    }

    // Confirm with user (matches Python behavior)
    if !crate::options::input::input_confirm("Confirm?", false) {
        return Ok(());
    }

    for (from, to) in &pairs {
        move_elements_across_dir(from, to, MoveOptions::default(), &ReplaceOptions::default())
            .await?;
    }

    Ok(())
}

/// Move works from one pack directory to another
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn move_works_in_pack(
    root_dir_from: &Path,
    root_dir_to: &Path,
) -> Result<(), std::io::Error> {
    if root_dir_from == root_dir_to {
        return Ok(());
    }

    let mut move_count = 0;

    if let Ok(mut read_dir) = tokio::fs::read_dir(root_dir_from).await {
        while let Some(entry) = read_dir.next_entry().await? {
            let bms_dir = entry.path();
            if !bms_dir.is_dir() {
                continue;
            }

            let bms_dir_name = bms_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

            println!("Moving: {bms_dir_name}");

            let dst_bms_dir = root_dir_to.join(bms_dir_name);
            move_elements_across_dir(
                &bms_dir,
                &dst_bms_dir,
                MoveOptions::default(),
                &REPLACE_OPTION_UPDATE_PACK,
            )
            .await?;
            move_count += 1;
        }
    }

    if move_count > 0 {
        println!("Move {move_count} songs.");
        return Ok(());
    }

    move_elements_across_dir(
        root_dir_from,
        root_dir_to,
        MoveOptions::default(),
        &REPLACE_OPTION_UPDATE_PACK,
    )
    .await?;

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

static REMOVE_MEDIA_RULE_WAV_FILL_FLAC: LazyLock<RemoveMediaRule> =
    LazyLock::new(|| vec![(vec!["wav"], vec!["flac"])]);
static REMOVE_MEDIA_RULE_MPG_FILL_WMV: LazyLock<RemoveMediaRule> =
    LazyLock::new(|| vec![(vec!["mpg"], vec!["wmv"])]);

static REMOVE_MEDIA_FILE_RULES: LazyLock<Vec<RemoveMediaRule>> = LazyLock::new(|| {
    vec![
        get_remove_media_rule_oraja(),
        REMOVE_MEDIA_RULE_WAV_FILL_FLAC.clone(),
        REMOVE_MEDIA_RULE_MPG_FILL_WMV.clone(),
    ]
});

async fn workdir_remove_unneed_media_files(
    work_dir: &Path,
    rule: &RemoveMediaRule,
) -> Result<(), std::io::Error> {
    let mut remove_pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut removed_files: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    let mut read_dir = tokio::fs::read_dir(work_dir).await?;
    let mut entries = Vec::new();
    while let Some(entry) = read_dir.next_entry().await? {
        entries.push(entry);
    }

    for entry in &entries {
        let check_file_path = entry.path();
        if !check_file_path.is_file() {
            continue;
        }

        let file_ext = check_file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        for (upper_exts, lower_exts) in rule {
            if !upper_exts.contains(&file_ext) {
                continue;
            }

            // File is empty?
            if tokio::fs::metadata(&check_file_path)
                .await
                .is_ok_and(|m| m.len() == 0)
            {
                println!(" - !x!: File {check_file_path:?} is Empty! Skipping...");
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
        println!(
            "- Remove file {:?}, because {:?} exists.",
            replacing_file_path.file_name(),
            check_file_path.file_name()
        );
        let _ = tokio::fs::remove_file(replacing_file_path).await;
    }

    // Remove zero-sized media files
    crate::options::bms_folder::remove_zero_sized_media_files(work_dir, false).await?;

    // Count extensions for mp4 warning
    let mut ext_count: HashMap<String, Vec<String>> = HashMap::new();
    let mut count_read_dir = tokio::fs::read_dir(work_dir).await?;
    while let Some(entry) = count_read_dir.next_entry().await? {
        let count_file_path = entry.path();
        if !count_file_path.is_file() {
            continue;
        }
        let file_ext = count_file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let file_name = count_file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        ext_count
            .entry(file_ext.to_string())
            .or_default()
            .push(file_name);
    }

    if let Some(mp4_files) = ext_count.get("mp4")
        && mp4_files.len() > 1
    {
        println!(" - Tips: {work_dir:?} has more than 1 mp4 files!");
    }

    Ok(())
}

/// Remove unneeded media files from all works in `root_dir`
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn remove_unneed_media_files(
    root_dir: &Path,
    rule: Option<RemoveMediaRule>,
) -> Result<(), std::io::Error> {
    let rule = match rule {
        Some(r) if !r.is_empty() => r,
        _ => {
            for (i, r) in REMOVE_MEDIA_FILE_RULES.iter().enumerate() {
                println!("- {i}: {r:?}");
            }
            let selection_str = crate::options::input::input_string("Select Preset (Default: 0):");
            let selection = if selection_str.is_empty() {
                0
            } else {
                selection_str.parse::<usize>().unwrap_or(0)
            };
            REMOVE_MEDIA_FILE_RULES[selection].clone()
        }
    };

    println!("Selected: {rule:?}");

    let mut read_dir = tokio::fs::read_dir(root_dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let bms_dir_path = entry.path();
        if !bms_dir_path.is_dir() {
            continue;
        }
        workdir_remove_unneed_media_files(&bms_dir_path, &rule).await?;
    }

    Ok(())
}

/// Move works out one level (un-nest subdirectories)
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn move_out_works(target_root_dir: &Path) -> Result<(), std::io::Error> {
    let mut read_dir = tokio::fs::read_dir(target_root_dir).await?;
    let mut entries = Vec::new();
    while let Some(entry) = read_dir.next_entry().await? {
        entries.push(entry);
    }

    for entry in entries {
        let root_dir_path = entry.path();
        if !root_dir_path.is_dir() {
            continue;
        }

        let mut work_read_dir = tokio::fs::read_dir(&root_dir_path).await?;
        while let Some(work_entry) = work_read_dir.next_entry().await? {
            let work_dir_path = work_entry.path();

            let work_dir_name = work_dir_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let target_work_dir_path = target_root_dir.join(work_dir_name);

            move_elements_across_dir(
                &work_dir_path,
                &target_work_dir_path,
                MoveOptions::default(),
                &REPLACE_OPTION_UPDATE_PACK,
            )
            .await?;
        }

        // Remove empty root_dir if not having files
        if !is_dir_having_file(&root_dir_path).await {
            let _ = tokio::fs::remove_dir(&root_dir_path).await;
        }
    }

    Ok(())
}

/// Move works with same name from one dir to another
///
/// This matches Python's `move_works_with_same_name` behavior:
/// - Shows pairs of directories to merge
/// - Asks for user confirmation before proceeding
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn move_works_with_same_name(
    root_dir_from: &Path,
    root_dir_to: &Path,
) -> Result<(), std::io::Error> {
    if !root_dir_from.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("源路径不存在或不是目录: {}", root_dir_from.display()),
        ));
    }
    if !root_dir_to.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("目标路径不存在或不是目录: {}", root_dir_to.display()),
        ));
    }

    // Get source subdirectories
    let mut from_subdirs: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(mut read_dir) = tokio::fs::read_dir(root_dir_from).await {
        while let Some(entry) = read_dir.next_entry().await? {
            if !entry.path().is_dir() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(String::from) else {
                continue;
            };
            from_subdirs.push((name, entry.path()));
        }
    }

    // Get target subdirectories
    let mut to_subdirs: Vec<String> = Vec::new();
    if let Ok(mut read_dir) = tokio::fs::read_dir(root_dir_to).await {
        while let Some(entry) = read_dir.next_entry().await? {
            if !entry.path().is_dir() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(String::from) else {
                continue;
            };
            to_subdirs.push(name);
        }
    }

    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();

    // Find matching pairs
    for (from_name, from_path) in &from_subdirs {
        for to_name in &to_subdirs {
            if to_name.starts_with(from_name) {
                let to_path = root_dir_to.join(to_name);
                println!(" -> {from_name} => {to_name}");
                pairs.push((from_path.clone(), to_path));
                break;
            }
        }
    }

    if pairs.is_empty() {
        return Ok(());
    }

    // Confirm with user (matches Python behavior)
    if !crate::options::input::input_confirm("是否合并？", false) {
        return Ok(());
    }

    // Execute moves
    for (from_path, to_path) in &pairs {
        println!("合并: {} -> {}", from_path.display(), to_path.display());
        move_elements_across_dir(
            from_path,
            to_path,
            MoveOptions::default(),
            &REPLACE_OPTION_UPDATE_PACK,
        )
        .await?;
    }

    Ok(())
}

/// Move works with same name to sibling directories
///
/// This matches Python's `move_works_with_same_name_to_siblings` behavior:
/// - Shows pairs of directories to merge
/// - Asks for user confirmation before proceeding
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory operations fail.
pub async fn move_works_with_same_name_to_siblings(
    root_dir_from: &Path,
) -> Result<(), std::io::Error> {
    if !root_dir_from.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("源路径不存在或不是目录: {}", root_dir_from.display()),
        ));
    }

    let Some(parent_dir) = root_dir_from.parent() else {
        return Ok(());
    };

    let root_base_name = root_dir_from
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Get source subdirectories
    let mut from_subdirs: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(mut read_dir) = tokio::fs::read_dir(root_dir_from).await {
        while let Some(entry) = read_dir.next_entry().await? {
            if !entry.path().is_dir() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(String::from) else {
                continue;
            };
            from_subdirs.push((name, entry.path()));
        }
    }

    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();

    // Iterate sibling directories
    if let Ok(mut siblings) = tokio::fs::read_dir(parent_dir).await {
        while let Some(sibling) = siblings.next_entry().await? {
            let sibling_path = sibling.path();
            if !sibling_path.is_dir() {
                continue;
            }

            let sibling_name = sibling_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if sibling_name == root_base_name {
                continue;
            }

            // Get sibling's subdirectories
            let mut to_subdirs: Vec<String> = Vec::new();
            if let Ok(mut read_dir) = tokio::fs::read_dir(&sibling_path).await {
                while let Some(entry) = read_dir.next_entry().await? {
                    if !entry.path().is_dir() {
                        continue;
                    }
                    let Some(name) = entry.file_name().to_str().map(String::from) else {
                        continue;
                    };
                    to_subdirs.push(name);
                }
            }

            // Find matching pairs
            for (from_name, from_path) in &from_subdirs {
                for to_name in &to_subdirs {
                    if to_name.starts_with(from_name) {
                        let target_path = sibling_path.join(to_name);
                        println!(" -> {from_name} => {}", target_path.display());
                        pairs.push((from_path.clone(), target_path));
                        break;
                    }
                }
            }
        }
    }

    if pairs.is_empty() {
        return Ok(());
    }

    // Confirm with user (matches Python behavior)
    if !crate::options::input::input_confirm("是否合并到各平级目录？", false) {
        return Ok(());
    }

    // Execute moves
    for (from_path, target_path) in &pairs {
        println!("合并: {} -> {}", from_path.display(), target_path.display());
        move_elements_across_dir(
            from_path,
            target_path,
            MoveOptions::default(),
            &REPLACE_OPTION_UPDATE_PACK,
        )
        .await?;
    }

    Ok(())
}

/// Merge split folders back together.
///
/// This reverses the `split_folders_with_first_char` operation by moving
/// contents from single-character-named subdirectories back to the parent.
///
/// # Errors
///
/// Returns [`anyhow::Error`] if directory operations fail.
pub async fn merge_split_folders(root_dir: &Path) -> Result<(), anyhow::Error> {
    let mut read_dir = tokio::fs::read_dir(root_dir).await?;
    let mut entries = Vec::new();
    while let Some(entry) = read_dir.next_entry().await? {
        if entry.path().is_dir() {
            entries.push(entry);
        }
    }

    let dir_names: Vec<String> = entries
        .iter()
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    let mut pairs: Vec<(String, String)> = Vec::new();

    for dir_name in &dir_names {
        let dir_path = root_dir.join(dir_name);
        if !dir_path.is_dir() {
            continue;
        }

        if dir_name.ends_with(']') {
            let Some(dir_name_mps_i) = dir_name.rfind('[') else {
                continue;
            };
            let dir_name_without_artist = &dir_name[..dir_name_mps_i - 1];
            if dir_name_without_artist.is_empty() {
                continue;
            }

            let dir_path_without_artist = root_dir.join(dir_name_without_artist);
            if !dir_path_without_artist.is_dir() {
                continue;
            }

            let dir_names_with_starter: Vec<&String> = dir_names
                .iter()
                .filter(|d| d.starts_with(&format!("{dir_name_without_artist} [")))
                .collect();

            if dir_names_with_starter.len() > 2 {
                println!(
                    " !_! {dir_name_without_artist} have more then 2 folders! {dir_names_with_starter:?}"
                );
                continue;
            }

            pairs.push((dir_name.clone(), dir_name_without_artist.to_string()));
        }
    }

    let mut last_from_dir_name = String::new();
    let mut duplicate_list: Vec<String> = Vec::new();
    for (_target_dir_name, from_dir_name) in &pairs {
        if last_from_dir_name == *from_dir_name {
            duplicate_list.push(from_dir_name.clone());
        }
        last_from_dir_name.clone_from(from_dir_name);
    }

    if !duplicate_list.is_empty() {
        println!("Duplicate!");
        for name in &duplicate_list {
            println!(" -> {name}");
        }
        anyhow::bail!("Found duplicate target directories: {duplicate_list:?}");
    }

    for (target_dir_name, from_dir_name) in &pairs {
        println!("- Find Dir pair: {target_dir_name} <- {from_dir_name}");
    }

    let selection_str = crate::options::input::input_string(&format!(
        "There are {} actions. Do transferring? [y/N]:",
        pairs.len()
    ));
    if !selection_str.to_lowercase().starts_with('y') {
        println!("Aborted.");
        return Ok(());
    }

    for (target_dir_name, from_dir_name) in &pairs {
        let from_dir_path = root_dir.join(from_dir_name);
        let target_dir_path = root_dir.join(target_dir_name);
        println!(" - Moving: {target_dir_name} <- {from_dir_name}");
        move_elements_across_dir(
            &from_dir_path,
            &target_dir_path,
            MoveOptions::default(),
            &ReplaceOptions::default(),
        )
        .await?;
    }

    Ok(())
}
