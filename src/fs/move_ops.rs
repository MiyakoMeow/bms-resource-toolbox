use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplaceAction {
    Skip,
    Replace,
    Rename,
    CheckReplace,
}

#[derive(Debug, Clone)]
pub struct ReplaceOptions {
    pub ext: HashMap<String, ReplaceAction>,
    pub default: ReplaceAction,
}

#[derive(Debug, Clone)]
pub struct MoveOptions {
    pub print_info: bool,
}

pub static REPLACE_OPTION_UPDATE_PACK: LazyLock<ReplaceOptions> = LazyLock::new(|| ReplaceOptions {
    ext: HashMap::from([
        ("bms".into(), ReplaceAction::CheckReplace),
        ("bml".into(), ReplaceAction::CheckReplace),
        ("bme".into(), ReplaceAction::CheckReplace),
        ("pms".into(), ReplaceAction::CheckReplace),
        ("txt".into(), ReplaceAction::CheckReplace),
        ("bmson".into(), ReplaceAction::CheckReplace),
    ]),
    default: ReplaceAction::Replace,
});

#[must_use]
pub fn is_dir_having_file(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if std::fs::metadata(&path).is_ok_and(|m| m.len() > 0) {
                return true;
            }
        } else if path.is_dir() && is_dir_having_file(&path) {
            return true;
        }
    }
    false
}

#[must_use]
pub fn is_same_content(a: &Path, b: &Path) -> bool {
    let Ok(bytes_a) = std::fs::read(a) else {
        return false;
    };
    let Ok(bytes_b) = std::fs::read(b) else {
        return false;
    };
    bytes_a == bytes_b
}

fn get_ext(path: &Path) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn get_action(ext: &str, opts: &ReplaceOptions) -> ReplaceAction {
    opts.ext.get(ext).copied().unwrap_or(opts.default)
}

#[allow(clippy::missing_panics_doc)]
pub fn move_elements_across_dir(
    src_dir: &Path,
    dst_dir: &Path,
    move_opts: &MoveOptions,
    replace_opts: &ReplaceOptions,
) {
    if src_dir == dst_dir {
        return;
    }
    if !src_dir.is_dir() {
        return;
    }
    if !dst_dir.is_dir() {
        if let Err(e) = std::fs::rename(src_dir, dst_dir) {
            eprintln!(" !_! Rename error: {e}");
        }
        return;
    }

    let Ok(entries) = std::fs::read_dir(src_dir) else {
        return;
    };
    let items: Vec<(PathBuf, PathBuf)> = entries
        .flatten()
        .map(|e| {
            let src = e.path();
            let dst = dst_dir.join(e.file_name());
            (src, dst)
        })
        .collect();

    let next_folders: Mutex<Vec<(PathBuf, PathBuf)>> = Mutex::new(Vec::new());
    let reserved: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());

    let plan_tasks: Vec<_> = items
        .into_iter()
        .filter_map(|(src, dst)| plan_action(&src, &dst, replace_opts, &next_folders, &reserved))
        .collect();

    let write_ops: Vec<(PathBuf, PathBuf)> = plan_tasks;
    let print_info = move_opts.print_info;

    std::thread::scope(|s| {
        for (src, dst) in write_ops {
            s.spawn(move || {
                if print_info {
                    println!(" - Moving from {} to {}", src.display(), dst.display());
                }
                if let Err(e) = std::fs::rename(&src, &dst) {
                    eprintln!(" !_! Move error: {e}");
                }
            });
        }
    });

    let next = next_folders.into_inner().unwrap();
    for (src, dst) in next {
        move_elements_across_dir(&src, &dst, move_opts, replace_opts);
    }

    if replace_opts.default != ReplaceAction::Skip || !is_dir_having_file(src_dir) {
        let _ = std::fs::remove_dir_all(src_dir);
    }
}

#[allow(clippy::type_complexity)]
fn plan_action(
    src: &Path,
    dst: &Path,
    replace_opts: &ReplaceOptions,
    next_folders: &Mutex<Vec<(PathBuf, PathBuf)>>,
    reserved: &Mutex<HashSet<PathBuf>>,
) -> Option<(PathBuf, PathBuf)> {
    if src.is_file() {
        plan_move_file(src, dst, replace_opts, reserved)
    } else if src.is_dir() {
        plan_move_dir(src, dst, next_folders)
    } else {
        None
    }
}

fn plan_move_file(
    src: &Path,
    dst: &Path,
    replace_opts: &ReplaceOptions,
    reserved: &Mutex<HashSet<PathBuf>>,
) -> Option<(PathBuf, PathBuf)> {
    let ext = get_ext(src);
    let action = get_action(&ext, replace_opts);

    match action {
        ReplaceAction::Replace => Some((src.to_path_buf(), dst.to_path_buf())),
        ReplaceAction::Skip => {
            if dst.is_file() {
                None
            } else {
                Some((src.to_path_buf(), dst.to_path_buf()))
            }
        }
        ReplaceAction::Rename => plan_move_rename(src, dst, reserved),
        ReplaceAction::CheckReplace => {
            if !dst.is_file() || is_same_content(src, dst) {
                Some((src.to_path_buf(), dst.to_path_buf()))
            } else {
                plan_move_rename(src, dst, reserved)
            }
        }
    }
}

fn plan_move_rename(
    src: &Path,
    dst: &Path,
    reserved: &Mutex<HashSet<PathBuf>>,
) -> Option<(PathBuf, PathBuf)> {
    let stem = dst.file_stem()?.to_string_lossy().to_string();
    let ext = dst.extension().map(|e| e.to_string_lossy().to_string());
    let parent = dst.parent()?;

    for i in 0..100 {
        let new_name = match &ext {
            Some(e) => format!("{stem}.{i}.{e}"),
            None => format!("{stem}.{i}"),
        };
        let new_dst = parent.join(&new_name);

        let mut reserved_guard = reserved.lock().unwrap();
        if reserved_guard.contains(&new_dst) {
            continue;
        }
        if new_dst.is_file() {
            if is_same_content(src, &new_dst) {
                return None;
            }
            continue;
        }
        reserved_guard.insert(new_dst.clone());
        drop(reserved_guard);
        return Some((src.to_path_buf(), new_dst));
    }
    None
}

fn plan_move_dir(
    src: &Path,
    dst: &Path,
    next_folders: &Mutex<Vec<(PathBuf, PathBuf)>>,
) -> Option<(PathBuf, PathBuf)> {
    if dst.is_dir() {
        next_folders
            .lock()
            .unwrap()
            .push((src.to_path_buf(), dst.to_path_buf()));
        None
    } else {
        Some((src.to_path_buf(), dst.to_path_buf()))
    }
}
