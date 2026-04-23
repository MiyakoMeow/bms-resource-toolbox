use std::path::Path;

use crate::cli::RawpackCommand;
use crate::fs::move_ops::{self, MoveOptions, REPLACE_OPTION_UPDATE_PACK};
use crate::fs::rawpack;

pub async fn handle(cmd: RawpackCommand) -> crate::Result<()> {
    match cmd {
        RawpackCommand::UnzipNumeric {
            pack_dir,
            cache_dir,
            root_dir,
        } => {
            unzip_numeric_to_bms_folder(&pack_dir, &cache_dir, &root_dir);
            Ok(())
        }
        RawpackCommand::UnzipWithName {
            pack_dir,
            cache_dir,
            root_dir,
        } => {
            unzip_with_name_to_bms_folder(&pack_dir, &cache_dir, &root_dir);
            Ok(())
        }
        RawpackCommand::SetNum { dir } => {
            set_num(&dir);
            Ok(())
        }
    }
}

pub fn unzip_numeric_to_bms_folder(pack_dir: &Path, cache_dir: &Path, root_dir: &Path) {
    if !cache_dir.is_dir() {
        let _ = std::fs::create_dir_all(cache_dir);
    }
    if !root_dir.is_dir() {
        let _ = std::fs::create_dir_all(root_dir);
    }

    let file_names = rawpack::get_num_set_file_names(pack_dir);

    for file_name in &file_names {
        let id = file_name.split(' ').next().unwrap_or(file_name);
        let id_cache_dir = cache_dir.join(id);
        let file_path = pack_dir.join(file_name);

        prepare_cache_dir(&id_cache_dir);

        if let Err(e) = rawpack::unzip_file_to_cache_dir(&file_path, &id_cache_dir) {
            eprintln!("Error extracting {}: {e}", file_path.display());
            continue;
        }
        let _ = rawpack::move_out_files_in_folder_in_cache_dir(&id_cache_dir);

        let target = find_matching_dir(root_dir, id).unwrap_or_else(|| root_dir.join(id));
        if !target.is_dir() {
            let _ = std::fs::create_dir(&target);
        }
        move_ops::move_elements_across_dir(
            &id_cache_dir,
            &target,
            &MoveOptions { print_info: true },
            &REPLACE_OPTION_UPDATE_PACK,
        );

        move_pack_to_boftt_packs(pack_dir, file_name);
    }
}

pub fn unzip_with_name_to_bms_folder(pack_dir: &Path, cache_dir: &Path, root_dir: &Path) {
    if !cache_dir.is_dir() {
        let _ = std::fs::create_dir_all(cache_dir);
    }
    if !root_dir.is_dir() {
        let _ = std::fs::create_dir_all(root_dir);
    }

    let file_names = get_all_file_names(pack_dir);

    for file_name in &file_names {
        let stem = Path::new(file_name)
            .file_stem()
            .map_or_else(|| file_name.clone(), |s| s.to_string_lossy().to_string());

        let id_cache_dir = cache_dir.join(&stem);
        let file_path = pack_dir.join(file_name);

        prepare_cache_dir(&id_cache_dir);

        if let Err(e) = rawpack::unzip_file_to_cache_dir(&file_path, &id_cache_dir) {
            eprintln!("Error extracting {}: {e}", file_path.display());
            continue;
        }
        let _ = rawpack::move_out_files_in_folder_in_cache_dir(&id_cache_dir);

        let target = root_dir.join(&stem);
        if !target.is_dir() {
            let _ = std::fs::create_dir(&target);
        }
        move_ops::move_elements_across_dir(
            &id_cache_dir,
            &target,
            &MoveOptions { print_info: true },
            &REPLACE_OPTION_UPDATE_PACK,
        );

        move_pack_to_boftt_packs(pack_dir, file_name);
    }
}

fn set_num(dir: &Path) {
    const ALLOWED_EXTS: &[&str] = &["zip", "7z", "rar", "mp4", "bms", "bme", "bml", "pms"];

    loop {
        let files = get_unnumbered_files(dir, ALLOWED_EXTS);
        if files.is_empty() {
            println!("All files are numbered!");
            return;
        }

        println!("Unnumbered files:");
        for (i, name) in files.iter().enumerate() {
            println!("  {}: {}", i + 1, name);
        }

        let input: String = dialoguer::Input::new()
            .with_prompt("Enter number (file_index number or just number, q to quit)")
            .interact()
            .unwrap_or_default();

        let input = input.trim();
        if input == "q" || input.is_empty() {
            return;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.len() {
            1 => {
                let Ok(num) = parts[0].parse::<u32>() else {
                    println!("Invalid number");
                    continue;
                };
                let old_name = &files[0];
                let new_name = format!("{num} {old_name}");
                rename_file_in_dir(dir, old_name, &new_name);
            }
            2 => {
                let Ok(file_index) = parts[0].parse::<usize>() else {
                    println!("Invalid file index");
                    continue;
                };
                let Ok(num) = parts[1].parse::<u32>() else {
                    println!("Invalid number");
                    continue;
                };
                if file_index == 0 || file_index > files.len() {
                    println!("File index out of range");
                    continue;
                }
                let old_name = &files[file_index - 1];
                let new_name = format!("{num} {old_name}");
                rename_file_in_dir(dir, old_name, &new_name);
            }
            _ => {
                println!("Invalid input");
                continue;
            }
        }

        if !dialoguer::Confirm::new()
            .with_prompt("Continue?")
            .default(true)
            .interact()
            .unwrap_or(false)
        {
            return;
        }
    }
}

fn prepare_cache_dir(cache_dir: &Path) {
    if cache_dir.is_dir() {
        let _ = std::fs::remove_dir_all(cache_dir);
    }
    let _ = std::fs::create_dir_all(cache_dir);
}

fn find_matching_dir(root_dir: &Path, id: &str) -> Option<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == id
            || name.starts_with(&format!("{id}."))
            || name.starts_with(&format!("{id} "))
        {
            return Some(entry.path());
        }
    }
    None
}

fn get_all_file_names(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect()
}

fn move_pack_to_boftt_packs(pack_dir: &Path, file_name: &str) {
    let boftt_dir = pack_dir.join("BOFTTPacks");
    if !boftt_dir.is_dir() {
        let _ = std::fs::create_dir(&boftt_dir);
    }
    let src = pack_dir.join(file_name);
    let dst = boftt_dir.join(file_name);
    if let Err(e) = std::fs::rename(&src, &dst) {
        eprintln!("Error moving {}: {e}", src.display());
    }
}

fn get_unnumbered_files(dir: &Path, allowed_exts: &[&str]) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();

            let ext = Path::new(&name)
                .extension()
                .map(|ex| ex.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if !allowed_exts.contains(&ext.as_str()) {
                return None;
            }

            let lower = name.to_lowercase();
            if lower.contains(".part") {
                return None;
            }

            let Ok(meta) = std::fs::metadata(e.path()) else {
                return None;
            };
            if meta.len() == 0 {
                return None;
            }

            let first_part = name.split(' ').next().unwrap_or(&name);
            if first_part.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }

            Some(name)
        })
        .collect()
}

fn rename_file_in_dir(dir: &Path, old_name: &str, new_name: &str) {
    let old_path = dir.join(old_name);
    let new_path = dir.join(new_name);
    println!("Rename: {old_name} -> {new_name}");
    if let Err(e) = std::fs::rename(&old_path, &new_path) {
        eprintln!("Error renaming: {e}");
    }
}
