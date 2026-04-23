use std::collections::HashMap;
use std::path::Path;

use crate::bms::{get_dir_bms_info, MEDIA_FILE_EXTS};
use crate::cli::FolderCommand;
use crate::fs::bms_dir_similarity;
use crate::fs::move_ops::{
    move_elements_across_dir, MoveOptions, ReplaceAction, ReplaceOptions,
    REPLACE_OPTION_UPDATE_PACK,
};
use crate::fs::name::get_valid_fs_name;
use crate::util::str_similarity;

pub async fn handle(cmd: FolderCommand) -> crate::Result<()> {
    match cmd {
        FolderCommand::SetName { root_dir } => set_name_by_bms(&root_dir),
        FolderCommand::AppendName { root_dir } => append_name_by_bms(&root_dir),
        FolderCommand::AppendArtist { root_dir } => append_artist_name_by_bms(&root_dir),
        FolderCommand::CopyNames { src_dir, dst_dir } => {
            copy_numbered_workdir_names(&src_dir, &dst_dir);
        }
        FolderCommand::ScanSimilar {
            root_dir,
            threshold,
        } => scan_folder_similar_folders(&root_dir, threshold),
        FolderCommand::UndoRename { root_dir } => undo_set_name(&root_dir),
        FolderCommand::CleanMedia { root_dir } => clean_media(&root_dir),
    }
    Ok(())
}

fn set_name_by_bms(root_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return;
    };
    let fail_list: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let dir_name = e.file_name().to_string_lossy().to_string();
            if workdir_set_name_by_bms(&e.path()) {
                None
            } else {
                Some(dir_name)
            }
        })
        .collect();

    if !fail_list.is_empty() {
        println!("Fail Count: {}", fail_list.len());
        println!("{fail_list:?}");
    }
}

fn workdir_set_name_by_bms(work_dir: &Path) -> bool {
    let mut info = get_dir_bms_info(work_dir);

    while info.is_none() {
        println!(
            "{} has no bms/bmson files! Trying to move out.",
            work_dir.display()
        );
        let Ok(entries) = std::fs::read_dir(work_dir) else {
            return false;
        };
        let elements: Vec<_> = entries
            .flatten()
            .map(|e| e.file_name())
            .collect();

        if elements.is_empty() {
            println!(" - Empty dir! Deleting...");
            let _ = std::fs::remove_dir(work_dir);
            return false;
        }
        if elements.len() != 1 {
            println!(" - Element count: {}", elements.len());
            return false;
        }
        let inner_dir = work_dir.join(&elements[0]);
        if !inner_dir.is_dir() {
            println!(
                " - Folder has only a file: {}",
                elements[0].to_string_lossy()
            );
            return false;
        }
        println!(" - Moving out files...");
        move_elements_across_dir(
            &inner_dir,
            work_dir,
            &MoveOptions { print_info: false },
            &ReplaceOptions {
                ext: HashMap::new(),
                default: ReplaceAction::Replace,
            },
        );
        info = get_dir_bms_info(work_dir);
    }

    let info = info.unwrap();
    let Some(parent_dir) = work_dir.parent() else {
        return false;
    };

    if info.title.is_empty() && info.artist.is_empty() {
        println!("{}: Info title and artist is EMPTY!", work_dir.display());
        return false;
    }

    let new_dir_path = parent_dir.join(format!(
        "{} [{}]",
        get_valid_fs_name(&info.title),
        get_valid_fs_name(&info.artist)
    ));

    if work_dir == new_dir_path {
        return true;
    }

    println!(
        "{}: Rename! Title: {}; Artist: {}",
        work_dir.display(),
        info.title,
        info.artist
    );

    if !new_dir_path.is_dir() {
        if let Err(e) = std::fs::rename(work_dir, &new_dir_path) {
            eprintln!(" !_! Rename error: {e}");
            return false;
        }
        return true;
    }

    let similarity = bms_dir_similarity(work_dir, &new_dir_path);
    println!(
        " - Directory {} exists! Similarity: {similarity}",
        new_dir_path.display()
    );
    if similarity < 0.8 {
        println!(" - Merge canceled.");
        return false;
    }

    println!(" - Merge start!");
    move_elements_across_dir(
        work_dir,
        &new_dir_path,
        &MoveOptions { print_info: false },
        &REPLACE_OPTION_UPDATE_PACK,
    );
    true
}

pub fn append_name_by_bms(root_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return;
    };
    let fail_list: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let dir_name = e.file_name().to_string_lossy().to_string();
            if workdir_append_name_by_bms(&e.path()) {
                None
            } else {
                Some(dir_name)
            }
        })
        .collect();

    if !fail_list.is_empty() {
        println!("Fail Count: {}", fail_list.len());
        println!("{fail_list:?}");
    }
}

fn workdir_append_name_by_bms(work_dir: &Path) -> bool {
    let dir_name = work_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    if !dir_name.trim().chars().all(|c| c.is_ascii_digit()) {
        println!("{} has been renamed! Skipping...", work_dir.display());
        return false;
    }

    let Some(info) = get_dir_bms_info(work_dir) else {
        println!("{} has no bms/bmson files!", work_dir.display());
        return false;
    };

    println!(
        "{} found bms title: {} artist: {}",
        work_dir.display(),
        info.title,
        info.artist
    );

    let Some(parent) = work_dir.parent() else {
        return false;
    };
    let new_dir_path = parent.join(format!(
        "{}. {} [{}]",
        dir_name,
        get_valid_fs_name(&info.title),
        get_valid_fs_name(&info.artist)
    ));

    if let Err(e) = std::fs::rename(work_dir, &new_dir_path) {
        eprintln!(" !_! Rename error: {e}");
        return false;
    }
    true
}

fn append_artist_name_by_bms(root_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return;
    };
    let dir_names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let mut pairs: Vec<(String, String)> = Vec::new();

    for dir_name in &dir_names {
        if dir_name.ends_with(']') {
            continue;
        }
        let dir_path = root_dir.join(dir_name);
        let Some(info) = get_dir_bms_info(&dir_path) else {
            println!("Dir {} has no bms files!", dir_path.display());
            continue;
        };
        let new_dir_name = format!("{} [{}]", dir_name, get_valid_fs_name(&info.artist));
        println!("- Ready to rename: {dir_name} -> {new_dir_name}");
        pairs.push((dir_name.clone(), new_dir_name));
    }

    if !dialoguer::Confirm::new()
        .with_prompt("Do transferring?")
        .default(false)
        .interact()
        .unwrap_or(false)
    {
        println!("Aborted.");
        return;
    }

    for (from_name, target_name) in &pairs {
        let from_path = root_dir.join(from_name);
        let target_path = root_dir.join(target_name);
        if let Err(e) = std::fs::rename(&from_path, &target_path) {
            eprintln!(" !_! Rename error: {e}");
        }
    }
}

pub fn copy_numbered_workdir_names(src_dir: &Path, dst_dir: &Path) {
    let Ok(src_entries) = std::fs::read_dir(src_dir) else {
        return;
    };
    let src_dir_names: Vec<String> = src_entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let Ok(dst_entries) = std::fs::read_dir(dst_dir) else {
        return;
    };
    for entry in dst_entries.flatten() {
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }
        let dir_num = dir_name
            .split(' ')
            .next()
            .unwrap_or(&dir_name)
            .split('.')
            .next()
            .unwrap_or(&dir_name);
        if !dir_num.chars().all(|c| c.is_ascii_digit()) {
            println!("Skipping non-numbered folder: {dir_name}");
            continue;
        }
        for src_name in &src_dir_names {
            if !src_name.starts_with(dir_num) {
                continue;
            }
            let target_path = dst_dir.join(src_name);
            println!("Rename {dir_name} to {src_name}");
            if let Err(e) = std::fs::rename(&dir_path, &target_path) {
                eprintln!(" !_! Rename error: {e}");
            }
            break;
        }
    }
}

fn scan_folder_similar_folders(root_dir: &Path, threshold: f64) {
    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return;
    };
    let mut dir_names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    println!("当前目录下有{}个文件夹。", dir_names.len());
    dir_names.sort();

    for i in 1..dir_names.len() {
        let similarity = str_similarity(&dir_names[i - 1], &dir_names[i]);
        if similarity < threshold {
            continue;
        }
        println!(
            "发现相似项：{} <=> {}",
            dir_names[i - 1],
            dir_names[i]
        );
    }
}

fn undo_set_name(root_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }
        let new_dir_name = dir_name.split(' ').next().unwrap_or(&dir_name);
        if dir_name == new_dir_name {
            continue;
        }
        let new_dir_path = root_dir.join(new_dir_name);
        if new_dir_path.is_dir() {
            println!(
                "Warning: Target {} already exists! Skipping {dir_name}",
                new_dir_path.display()
            );
            continue;
        }
        println!("Rename {dir_name} to {new_dir_name}");
        if let Err(e) = std::fs::rename(&dir_path, &new_dir_path) {
            eprintln!(" !_! Rename error: {e}");
        }
    }
}

fn clean_media(root_dir: &Path) {
    clean_media_recursive(root_dir);
}

fn clean_media_recursive(current_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(current_dir) else {
        println!("Not a valid dir! Aborting...");
        return;
    };
    let items: Vec<_> = entries.flatten().collect();
    let mut next_dirs: Vec<String> = Vec::new();

    for entry in &items {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_file() {
            let lower = name.to_lowercase();
            let is_temp = lower == "desktop.ini"
                || lower == "thumbs.db"
                || lower == ".ds_store"
                || name.starts_with(".trash-")
                || name.starts_with("._");

            if is_temp {
                println!(" - Remove temp file: {}", path.display());
                let _ = std::fs::remove_file(&path);
                continue;
            }

            let is_media = MEDIA_FILE_EXTS.iter().any(|ext| lower.ends_with(ext));
            if !is_media {
                continue;
            }
            let Ok(meta) = std::fs::metadata(&path) else {
                continue;
            };
            if meta.len() > 0 {
                continue;
            }
            println!(" - Remove empty file: {}", path.display());
            let _ = std::fs::remove_file(&path);
        } else if path.is_dir() {
            next_dirs.push(name);
        }
    }

    for sub in next_dirs {
        clean_media_recursive(&current_dir.join(sub));
    }
}
