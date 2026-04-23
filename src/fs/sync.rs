use std::path::Path;
use std::sync::LazyLock;

use sha2::{Digest, Sha512};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncExec {
    None,
    Copy,
    Move,
}

impl std::fmt::Display for SyncExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncExec::None => write!(f, "无操作"),
            SyncExec::Copy => write!(f, "使用复制命令"),
            SyncExec::Move => write!(f, "使用移动命令"),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SoftSyncPreset {
    pub name: String,
    pub allow_src_exts: Vec<String>,
    pub disallow_src_exts: Vec<String>,
    pub allow_other_exts: bool,
    pub no_activate_ext_bound_pairs: Vec<(Vec<String>, Vec<String>)>,
    pub remove_dst_extra_files: bool,
    pub check_file_size: bool,
    pub check_file_mtime: bool,
    pub check_file_sha512: bool,
    pub remove_src_same_files: bool,
    pub exec: SyncExec,
}

impl std::fmt::Display for SoftSyncPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}：", self.name)?;
        write!(f, "{} ", self.exec)?;
        if self.allow_other_exts {
            write!(f, "允许同步未定义扩展名 ")?;
        }
        if !self.allow_src_exts.is_empty() {
            write!(f, "允许扩展名：{:?} ", self.allow_src_exts)?;
        }
        if !self.disallow_src_exts.is_empty() {
            write!(f, "拒绝扩展名：{:?} ", self.disallow_src_exts)?;
        }
        if self.remove_src_same_files {
            write!(f, "移除源中相对于目标，不需要同步的文件 ")?;
        }
        if self.remove_dst_extra_files {
            write!(f, "移除目标文件夹相对源文件夹的多余文件 ")?;
        }
        if self.check_file_mtime {
            write!(f, "检查修改时间 ")?;
        }
        if self.check_file_size {
            write!(f, "检查大小 ")?;
        }
        if self.check_file_sha512 {
            write!(f, "检查SHA-512 ")?;
        }
        Ok(())
    }
}

pub static SYNC_PRESET_DEFAULT: LazyLock<SoftSyncPreset> = LazyLock::new(|| SoftSyncPreset {
    name: "本地文件同步预设".into(),
    allow_src_exts: vec![],
    disallow_src_exts: vec![],
    allow_other_exts: true,
    no_activate_ext_bound_pairs: vec![],
    remove_dst_extra_files: true,
    check_file_size: true,
    check_file_mtime: true,
    check_file_sha512: false,
    remove_src_same_files: false,
    exec: SyncExec::Copy,
});

pub static SYNC_PRESET_FOR_APPEND: LazyLock<SoftSyncPreset> = LazyLock::new(|| SoftSyncPreset {
    name: "同步预设（用于更新包）".into(),
    allow_src_exts: vec![],
    disallow_src_exts: vec![],
    allow_other_exts: true,
    no_activate_ext_bound_pairs: vec![],
    remove_dst_extra_files: false,
    check_file_size: true,
    check_file_mtime: false,
    check_file_sha512: true,
    remove_src_same_files: true,
    exec: SyncExec::None,
});

pub static SYNC_PRESET_FLAC: LazyLock<SoftSyncPreset> = LazyLock::new(|| SoftSyncPreset {
    name: "本地文件同步预设".into(),
    allow_src_exts: vec!["flac".into()],
    disallow_src_exts: vec![],
    allow_other_exts: false,
    no_activate_ext_bound_pairs: vec![],
    remove_dst_extra_files: false,
    check_file_size: true,
    check_file_mtime: true,
    check_file_sha512: false,
    remove_src_same_files: false,
    exec: SyncExec::Copy,
});

pub static SYNC_PRESET_MP4_AVI: LazyLock<SoftSyncPreset> = LazyLock::new(|| SoftSyncPreset {
    name: "本地文件同步预设".into(),
    allow_src_exts: vec!["mp4".into(), "avi".into()],
    disallow_src_exts: vec![],
    allow_other_exts: false,
    no_activate_ext_bound_pairs: vec![],
    remove_dst_extra_files: false,
    check_file_size: true,
    check_file_mtime: true,
    check_file_sha512: false,
    remove_src_same_files: false,
    exec: SyncExec::Copy,
});

pub static SYNC_PRESET_CACHE: LazyLock<SoftSyncPreset> = LazyLock::new(|| SoftSyncPreset {
    name: "本地文件同步预设".into(),
    allow_src_exts: vec!["mp4".into(), "avi".into(), "flac".into()],
    disallow_src_exts: vec![],
    allow_other_exts: false,
    no_activate_ext_bound_pairs: vec![],
    remove_dst_extra_files: false,
    check_file_size: true,
    check_file_mtime: true,
    check_file_sha512: false,
    remove_src_same_files: false,
    exec: SyncExec::None,
});

pub static SYNC_PRESETS: LazyLock<Vec<&'static SoftSyncPreset>> = LazyLock::new(|| {
    vec![
        &SYNC_PRESET_DEFAULT,
        &SYNC_PRESET_FOR_APPEND,
        &SYNC_PRESET_FLAC,
        &SYNC_PRESET_MP4_AVI,
        &SYNC_PRESET_CACHE,
    ]
});

#[must_use]
pub fn get_file_sha512(file_path: &Path) -> String {
    let Ok(bytes) = std::fs::read(file_path) else {
        return String::new();
    };
    let mut hasher = Sha512::new();
    hasher.update(&bytes);
    format!("{:x}", hasher.finalize())
}

#[allow(clippy::too_many_lines)]
pub fn sync_folder(src_dir: &Path, dst_dir: &Path, preset: &SoftSyncPreset) {
    let Ok(src_entries) = std::fs::read_dir(src_dir) else {
        return;
    };
    let Ok(dst_entries) = std::fs::read_dir(dst_dir) else {
        return;
    };

    let src_list: Vec<String> = src_entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    let dst_list: Vec<String> = dst_entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let mut src_copy_files: Vec<String> = vec![];
    let mut src_move_files: Vec<String> = vec![];
    let mut src_remove_files: Vec<String> = vec![];
    let mut dst_remove_files: Vec<String> = vec![];
    let mut dst_remove_dirs: Vec<String> = vec![];

    for src_element in &src_list {
        let src_path = src_dir.join(src_element);
        let dst_path = dst_dir.join(src_element);

        if src_path.is_dir() {
            if dst_path.is_dir() {
                sync_folder(&src_path, &dst_path, preset);
            } else {
                let _ = std::fs::create_dir(&dst_path);
                sync_folder(&src_path, &dst_path, preset);
            }
            continue;
        }

        if !src_path.is_file() {
            continue;
        }

        let ext = src_element.rsplit('.').next().unwrap_or("").to_lowercase();
        let mut ext_check_passed = preset.allow_other_exts;
        if preset.allow_src_exts.contains(&ext) {
            ext_check_passed = true;
        }
        if preset.disallow_src_exts.contains(&ext) {
            ext_check_passed = false;
        }
        if !ext_check_passed {
            continue;
        }

        let mut ext_in_bound = false;
        for (bound_from, bound_to) in &preset.no_activate_ext_bound_pairs {
            if !bound_from.contains(&ext) {
                continue;
            }
            for bound_ext in bound_to {
                let suffix = if bound_ext.starts_with('.') {
                    bound_ext.clone()
                } else {
                    format!(".{bound_ext}")
                };
                let bound_file_path = dst_path.with_extension(&suffix[1..]);
                if bound_file_path.is_file() {
                    ext_in_bound = true;
                    break;
                }
            }
            if ext_in_bound {
                break;
            }
        }
        if ext_in_bound {
            continue;
        }

        let dst_file_exists = dst_path.is_file();
        let mut is_same_file = dst_file_exists;

        if preset.check_file_size && is_same_file && dst_file_exists {
            let src_size = std::fs::metadata(&src_path).map(|m| m.len()).unwrap_or(0);
            let dst_size = std::fs::metadata(&dst_path).map(|m| m.len()).unwrap_or(0);
            is_same_file = is_same_file && src_size == dst_size;
        }
        if preset.check_file_mtime && is_same_file && dst_file_exists {
            let src_mtime = std::fs::metadata(&src_path)
                .and_then(|m| m.modified())
                .ok();
            let dst_mtime = std::fs::metadata(&dst_path)
                .and_then(|m| m.modified())
                .ok();
            is_same_file = is_same_file && src_mtime.is_some_and(|s| dst_mtime.is_some_and(|d| s == d));
        }
        if preset.check_file_sha512 && is_same_file && dst_file_exists {
            let src_value = get_file_sha512(&src_path);
            let dst_value = get_file_sha512(&dst_path);
            is_same_file = is_same_file && src_value == dst_value;
        }

        if !dst_file_exists || !is_same_file {
            let src_mtime = std::fs::metadata(&src_path)
                .and_then(|m| m.modified())
                .ok();
            match preset.exec {
                SyncExec::None => {}
                SyncExec::Copy => {
                    src_copy_files.push(src_element.clone());
                    if let Err(e) = std::fs::copy(&src_path, &dst_path) {
                        eprintln!("Copy error: {e}");
                    }
                    if let Some(mtime) = src_mtime {
                        let ft = filetime::FileTime::from_system_time(mtime);
                        let _ = filetime::set_file_mtime(&dst_path, ft);
                    }
                }
                SyncExec::Move => {
                    src_move_files.push(src_element.clone());
                    if let Err(e) = std::fs::rename(&src_path, &dst_path) {
                        eprintln!("Move error: {e}");
                    }
                    if let Some(mtime) = src_mtime {
                        let ft = filetime::FileTime::from_system_time(mtime);
                        let _ = filetime::set_file_mtime(&dst_path, ft);
                    }
                }
            }
        }

        if preset.remove_src_same_files && dst_file_exists && is_same_file && src_path.is_file() {
            src_remove_files.push(src_element.clone());
            let _ = std::fs::remove_file(&src_path);
        }
    }

    if preset.remove_dst_extra_files {
        for dst_element in &dst_list {
            let src_path = src_dir.join(dst_element);
            let dst_path = dst_dir.join(dst_element);

            if dst_path.is_dir() {
                if !src_path.is_dir() {
                    dst_remove_dirs.push(dst_element.clone());
                    let _ = std::fs::remove_dir_all(&dst_path);
                }
            } else if dst_path.is_file() && !src_path.is_file() {
                dst_remove_files.push(dst_element.clone());
                let _ = std::fs::remove_file(&dst_path);
            }
        }
    }

    if !src_copy_files.is_empty()
        || !src_move_files.is_empty()
        || !src_remove_files.is_empty()
        || !dst_remove_files.is_empty()
        || !dst_remove_dirs.is_empty()
    {
        println!("{} -> {}:", src_dir.display(), dst_dir.display());
        if !src_copy_files.is_empty() {
            println!("Src copy: {src_copy_files:?}");
        }
        if !src_move_files.is_empty() {
            println!("Src move: {src_move_files:?}");
        }
        if !src_remove_files.is_empty() {
            println!("Src remove: {src_remove_files:?}");
        }
        if !dst_remove_files.is_empty() {
            println!("Dst remove: {dst_remove_files:?}");
        }
        if !dst_remove_dirs.is_empty() {
            println!("Dst remove dir: {dst_remove_dirs:?}");
        }
    }
}
