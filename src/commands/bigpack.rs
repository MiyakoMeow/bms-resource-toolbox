use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::cli::BigpackCommand;
use crate::fs::move_ops::{
    is_dir_having_file, move_elements_across_dir, MoveOptions, REPLACE_OPTION_UPDATE_PACK,
};


type RuleList = Vec<(Vec<String>, Vec<String>)>;

struct FirstCharRule {
    name: &'static str,
    matches: fn(&str) -> bool,
}

fn get_first_char_rules() -> Vec<FirstCharRule> {
    fn match_digit(name: &str) -> bool {
        let Some(ch) = name.chars().next() else {
            return false;
        };
        ch.is_ascii_digit()
    }
    fn match_abcd(name: &str) -> bool {
        let Some(ch) = name.chars().next() else {
            return false;
        };
        let u = ch.to_ascii_uppercase();
        ('A'..='D').contains(&u)
    }
    fn match_efghijk(name: &str) -> bool {
        let Some(ch) = name.chars().next() else {
            return false;
        };
        let u = ch.to_ascii_uppercase();
        ('E'..='K').contains(&u)
    }
    fn match_lmnopq(name: &str) -> bool {
        let Some(ch) = name.chars().next() else {
            return false;
        };
        let u = ch.to_ascii_uppercase();
        ('L'..='Q').contains(&u)
    }
    fn match_rst(name: &str) -> bool {
        let Some(ch) = name.chars().next() else {
            return false;
        };
        let u = ch.to_ascii_uppercase();
        ('R'..='T').contains(&u)
    }
    fn match_uvwxyz(name: &str) -> bool {
        let Some(ch) = name.chars().next() else {
            return false;
        };
        let u = ch.to_ascii_uppercase();
        ('U'..='Z').contains(&u)
    }
    fn match_hiragana(name: &str) -> bool {
        let Some(ch) = name.chars().next() else {
            return false;
        };
        ('\u{3040}'..='\u{309f}').contains(&ch)
    }
    fn match_katakana(name: &str) -> bool {
        let Some(ch) = name.chars().next() else {
            return false;
        };
        ('\u{30a0}'..='\u{30ff}').contains(&ch)
    }
    fn match_cjk(name: &str) -> bool {
        let Some(ch) = name.chars().next() else {
            return false;
        };
        ('\u{4e00}'..='\u{9fa5}').contains(&ch)
    }
    fn match_fallback(_name: &str) -> bool {
        true
    }

    vec![
        FirstCharRule { name: "0-9", matches: match_digit },
        FirstCharRule { name: "ABCD", matches: match_abcd },
        FirstCharRule { name: "EFGHIJK", matches: match_efghijk },
        FirstCharRule { name: "LMNOPQ", matches: match_lmnopq },
        FirstCharRule { name: "RST", matches: match_rst },
        FirstCharRule { name: "UVWXYZ", matches: match_uvwxyz },
        FirstCharRule { name: "平假名", matches: match_hiragana },
        FirstCharRule { name: "片假名", matches: match_katakana },
        FirstCharRule { name: "汉字", matches: match_cjk },
        FirstCharRule { name: "+", matches: match_fallback },
    ]
}

fn first_char_rule_find(name: &str) -> &'static str {
    for rule in get_first_char_rules() {
        if (rule.matches)(name) {
            return rule.name;
        }
    }
    "未分类"
}

fn split_folders_with_first_char(root_dir: &Path) {
    let root_folder_name = match root_dir.file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => return,
    };
    if !root_dir.is_dir() {
        println!("{} is not a dir! Aborting...", root_dir.display());
        return;
    }
    if root_folder_name.ends_with(']') {
        println!("{} endswith ']'. Aborting...", root_dir.display());
        return;
    }
    let parent_dir = root_dir.parent().unwrap_or(root_dir);

    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return;
    };
    let element_names: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    for element_name in &element_names {
        let element_path = root_dir.join(element_name);
        let rule = first_char_rule_find(element_name);
        let target_dir = parent_dir.join(format!("{root_folder_name} [{rule}]"));
        if !target_dir.is_dir() {
            let _ = std::fs::create_dir(&target_dir);
        }
        let target_path = target_dir.join(element_name);
        if let Err(e) = std::fs::rename(&element_path, &target_path) {
            eprintln!(" !_! Move error: {e}");
        }
    }

    if !is_dir_having_file(root_dir) {
        let _ = std::fs::remove_dir(root_dir);
    }
}

fn undo_split_pack(dir_name: &Path) {
    let root_folder_name = match dir_name.file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => return,
    };
    let parent_dir = dir_name.parent().unwrap_or(dir_name);

    let Ok(entries) = std::fs::read_dir(parent_dir) else {
        return;
    };
    let mut pairs: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let folder_name = entry.file_name().to_string_lossy().to_string();
        let folder_path = entry.path();
        if folder_name.starts_with(&format!("{root_folder_name} [")) && folder_name.ends_with(']') {
            println!(" - {} <- {}", dir_name.display(), folder_path.display());
            pairs.push((folder_path, dir_name.to_path_buf()));
        }
    }

    if !dialoguer::Confirm::new()
        .with_prompt("Confirm?")
        .default(false)
        .interact()
        .unwrap_or(false)
    {
        return;
    }

    let move_opts = MoveOptions { print_info: false };
    for (from_dir, to_dir) in &pairs {
        move_elements_across_dir(from_dir, to_dir, &move_opts, &REPLACE_OPTION_UPDATE_PACK);
    }
}

fn move_works_in_pack(from_dir: &Path, to_dir: &Path) {
    if from_dir == to_dir {
        return;
    }

    let Ok(entries) = std::fs::read_dir(from_dir) else {
        return;
    };
    let subdirs: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let move_opts = MoveOptions { print_info: false };
    let mut move_count = 0usize;
    for bms_dir_name in &subdirs {
        let bms_dir = from_dir.join(bms_dir_name);
        if !bms_dir.is_dir() {
            continue;
        }
        println!("Moving: {bms_dir_name}");
        let dst_bms_dir = to_dir.join(bms_dir_name);
        move_elements_across_dir(&bms_dir, &dst_bms_dir, &move_opts, &REPLACE_OPTION_UPDATE_PACK);
        move_count += 1;
    }
    if move_count > 0 {
        println!("Move {move_count} songs.");
        return;
    }

    move_elements_across_dir(from_dir, to_dir, &move_opts, &REPLACE_OPTION_UPDATE_PACK);
}

fn move_out_works(root_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return;
    };
    let subdirs: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let move_opts = MoveOptions { print_info: false };
    for root_dir_name in &subdirs {
        let root_dir_path = root_dir.join(root_dir_name);
        if !root_dir_path.is_dir() {
            continue;
        }
        let Ok(inner) = std::fs::read_dir(&root_dir_path) else {
            continue;
        };
        let inner_names: Vec<String> = inner
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        for work_dir_name in &inner_names {
            let work_dir_path = root_dir_path.join(work_dir_name);
            let target_work_dir_path = root_dir.join(work_dir_name);
            move_elements_across_dir(
                &work_dir_path,
                &target_work_dir_path,
                &move_opts,
                &REPLACE_OPTION_UPDATE_PACK,
            );
        }
        if !is_dir_having_file(&root_dir_path) {
            let _ = std::fs::remove_dir(&root_dir_path);
        }
    }
}

fn move_works_with_same_name(from_dir: &Path, to_dir: &Path) {
    if !from_dir.is_dir() {
        println!("{} is not a dir!", from_dir.display());
        return;
    }
    if !to_dir.is_dir() {
        println!("{} is not a dir!", to_dir.display());
        return;
    }

    let from_subdirs = get_subdir_names(from_dir);
    let to_subdirs = get_subdir_names(to_dir);

    let mut pairs: Vec<(&str, &std::path::PathBuf, &str, &std::path::PathBuf)> = Vec::new();

    for from_name in &from_subdirs {
        for to_name in &to_subdirs {
            if to_name.0.starts_with(&from_name.0) {
                pairs.push((&from_name.0, &from_name.1, &to_name.0, &to_name.1));
                break;
            }
        }
    }

    for (from_name, _, to_name, _) in &pairs {
        println!(" -> {from_name} => {to_name}");
    }
    if !dialoguer::Confirm::new()
        .with_prompt("是否合并？")
        .default(false)
        .interact()
        .unwrap_or(false)
    {
        return;
    }

    let move_opts = MoveOptions { print_info: false };
    for (_, from_path, _, to_path) in &pairs {
        println!("合并: '{}' -> '{}'", from_path.display(), to_path.display());
        move_elements_across_dir(from_path, to_path, &move_opts, &REPLACE_OPTION_UPDATE_PACK);
    }
}

fn move_works_with_same_name_to_siblings(dir: &Path) {
    if !dir.is_dir() {
        println!("{} is not a dir!", dir.display());
        return;
    }

    let parent_dir = dir.parent().unwrap_or(dir);
    let root_base_name = dir.file_name().unwrap_or_default().to_string_lossy().to_string();
    let from_subdirs = get_subdir_names(dir);

    let Ok(sibling_entries) = std::fs::read_dir(parent_dir) else {
        return;
    };
    let mut pairs: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();

    for sibling_entry in sibling_entries.flatten() {
        let sibling_name = sibling_entry.file_name().to_string_lossy().to_string();
        let sibling_path = sibling_entry.path();
        if sibling_name == root_base_name || !sibling_path.is_dir() {
            continue;
        }
        let to_subdirs = get_subdir_names(&sibling_path);
        for from_name in &from_subdirs {
            for to_name in &to_subdirs {
                if to_name.0.starts_with(&from_name.0) {
                    pairs.push((from_name.1.clone(), to_name.1.clone()));
                    break;
                }
            }
        }
    }

    for (from_path, target_path) in &pairs {
        println!(" -> {} => {}", from_path.display(), target_path.display());
    }
    if !dialoguer::Confirm::new()
        .with_prompt("是否合并到各平级目录？")
        .default(false)
        .interact()
        .unwrap_or(false)
    {
        return;
    }

    let move_opts = MoveOptions { print_info: false };
    for (from_path, target_path) in &pairs {
        println!("合并: '{}' -> '{}'", from_path.display(), target_path.display());
        move_elements_across_dir(from_path, target_path, &move_opts, &REPLACE_OPTION_UPDATE_PACK);
    }
}

fn get_subdir_names(dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let path = dir.join(&name);
            (name, path)
        })
        .collect()
}

const REMOVE_MEDIA_RULE_ORAJA: &[(&[&str], &[&str])] = &[
    (&["mp4"], &["avi", "wmv", "mpg", "mpeg"]),
    (&["avi"], &["wmv", "mpg", "mpeg"]),
    (&["flac", "wav"], &["ogg"]),
    (&["flac"], &["wav"]),
    (&["mpg"], &["wmv"]),
];

const REMOVE_MEDIA_RULE_WAV_FILL_FLAC: &[(&[&str], &[&str])] = &[
    (&["wav"], &["flac"]),
];

const REMOVE_MEDIA_RULE_MPG_FILL_WMV: &[(&[&str], &[&str])] = &[
    (&["mpg"], &["wmv"]),
];

fn get_remove_media_rules(preset: usize) -> Vec<(Vec<String>, Vec<String>)> {
    let raw: &[(&[&str], &[&str])] = match preset {
        0 => REMOVE_MEDIA_RULE_ORAJA,
        1 => REMOVE_MEDIA_RULE_WAV_FILL_FLAC,
        2 => REMOVE_MEDIA_RULE_MPG_FILL_WMV,
        _ => REMOVE_MEDIA_RULE_ORAJA,
    };
    raw.iter()
        .map(|(upper, lower)| {
            (
                upper.iter().map(|s| (*s).to_string()).collect(),
                lower.iter().map(|s| (*s).to_string()).collect(),
            )
        })
        .collect()
}

pub fn remove_unneed_media_files(root_dir: &Path, preset: usize) {
    let rules = get_remove_media_rules(preset);
    println!("Selected preset {preset}");

    let Ok(entries) = std::fs::read_dir(root_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let bms_dir_path = entry.path();
        if !bms_dir_path.is_dir() {
            continue;
        }
        workdir_remove_unneed_media_files(&bms_dir_path, &rules);
    }
}

fn workdir_remove_unneed_media_files(work_dir: &Path, rules: &RuleList) {
    let Ok(entries) = std::fs::read_dir(work_dir) else {
        return;
    };
    let files: Vec<(String, std::path::PathBuf)> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            (name, e.path())
        })
        .collect();

    let mut remove_pairs: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();
    let mut removed_files: HashSet<std::path::PathBuf> = HashSet::new();

    for (file_name, check_file_path) in &files {
        let file_ext = Path::new(file_name)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        for (upper_exts, lower_exts) in rules {
            if !upper_exts.contains(&file_ext) {
                continue;
            }
            let Ok(meta) = std::fs::metadata(check_file_path) else {
                continue;
            };
            if meta.len() == 0 {
                println!(" - !x!: File {} is Empty! Skipping...", check_file_path.display());
                continue;
            }
            for lower_ext in lower_exts {
                let stem = Path::new(file_name).file_stem().unwrap_or_default().to_string_lossy();
                let replacing_file_path = work_dir.join(format!("{stem}.{lower_ext}"));
                if !replacing_file_path.is_file() {
                    continue;
                }
                if removed_files.contains(&replacing_file_path) {
                    continue;
                }
                remove_pairs.push((check_file_path.clone(), replacing_file_path.clone()));
                removed_files.insert(replacing_file_path.clone());
            }
        }
    }

    if !remove_pairs.is_empty() {
        println!("Entering: {}", work_dir.display());
    }

    for (check_file_path, replacing_file_path) in &remove_pairs {
        println!(
            "- Remove file {}, because {} exists.",
            replacing_file_path.display(),
            check_file_path.display()
        );
        let _ = std::fs::remove_file(replacing_file_path);
    }

    remove_zero_sized_media_files(work_dir);

    let mut ext_count: HashMap<String, Vec<String>> = HashMap::new();
    for (file_name, file_path) in &files {
        if !file_path.is_file() {
            continue;
        }
        let ext = Path::new(file_name)
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        ext_count.entry(ext).or_default().push(file_name.clone());
    }
    if let Some(mp4_list) = ext_count.get("mp4")
        && mp4_list.len() > 1 {
            println!(" - Tips: {} has more than 1 mp4 files! {mp4_list:?}", work_dir.display());
        }
}

fn remove_zero_sized_media_files(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let items: Vec<_> = entries.flatten().collect();
    let mut next_dirs: Vec<String> = Vec::new();

    for entry in &items {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_file() {
            let lower = name.to_lowercase();
            let is_media = lower.ends_with(".ogg")
                || lower.ends_with(".wav")
                || lower.ends_with(".flac")
                || lower.ends_with(".mp4")
                || lower.ends_with(".wmv")
                || lower.ends_with(".avi")
                || lower.ends_with(".mpg")
                || lower.ends_with(".mpeg")
                || lower.ends_with(".bmp")
                || lower.ends_with(".jpg")
                || lower.ends_with(".png");
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
        remove_zero_sized_media_files(&dir.join(sub));
    }
}

pub async fn handle(cmd: BigpackCommand) {
    match cmd {
        BigpackCommand::Split { root_dir } => split_folders_with_first_char(&root_dir),
        BigpackCommand::UndoSplit { dir_name } => undo_split_pack(&dir_name),
        BigpackCommand::MoveWorks { from_dir, to_dir } => move_works_in_pack(&from_dir, &to_dir),
        BigpackCommand::MoveOut { root_dir } => move_out_works(&root_dir),
        BigpackCommand::MoveSameName { from_dir, to_dir } => {
            move_works_with_same_name(&from_dir, &to_dir);
        }
        BigpackCommand::MoveSameNameSiblings { dir } => {
            move_works_with_same_name_to_siblings(&dir);
        }
        BigpackCommand::RemoveMedia { root_dir, preset } => {
            remove_unneed_media_files(&root_dir, preset);
        }
    }
}
