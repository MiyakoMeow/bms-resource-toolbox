use std::{collections::HashMap, path::Path};

use smol::{fs, io, stream::StreamExt};

use super::is_file_same_content;

/// 与 Python SoftSyncExec 等价
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftSyncExec {
    None,
    Copy,
    Move,
}

impl std::fmt::Display for SoftSyncExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SoftSyncExec::None => write!(f, "无操作"),
            SoftSyncExec::Copy => write!(f, "使用复制命令"),
            SoftSyncExec::Move => write!(f, "使用移动命令"),
        }
    }
}

/// 同步预设
#[derive(Debug, Clone)]
pub struct SoftSyncPreset {
    pub name: String,
    pub allow_src_exts: Vec<String>,
    pub disallow_src_exts: Vec<String>,
    pub allow_other_exts: bool,
    /// (from_exts, to_exts)
    pub no_activate_ext_bound_pairs: Vec<(Vec<String>, Vec<String>)>,
    pub remove_dst_extra_files: bool,
    pub check_file_size: bool,
    pub check_file_mtime: bool,
    pub check_file_sha512: bool,
    pub remove_src_same_files: bool,
    pub exec: SoftSyncExec,
}

impl Default for SoftSyncPreset {
    fn default() -> Self {
        Self {
            name: "本地文件同步预设".into(),
            allow_src_exts: Vec::new(),
            disallow_src_exts: Vec::new(),
            allow_other_exts: true,
            no_activate_ext_bound_pairs: Vec::new(),
            remove_dst_extra_files: true,
            check_file_size: true,
            check_file_mtime: true,
            check_file_sha512: false,
            remove_src_same_files: false,
            exec: SoftSyncExec::Copy,
        }
    }
}

impl std::fmt::Display for SoftSyncPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}：{}", self.name, self.exec)?;
        if self.allow_other_exts {
            write!(f, " 允许同步未定义扩展名")?;
        }
        if !self.allow_src_exts.is_empty() {
            write!(f, " 允许扩展名：{:?}", self.allow_src_exts)?;
        }
        if !self.disallow_src_exts.is_empty() {
            write!(f, " 拒绝扩展名：{:?}", self.disallow_src_exts)?;
        }
        if self.remove_src_same_files {
            write!(f, " 移除源中相对于目标，不需要同步的文件")?;
        }
        if self.remove_dst_extra_files {
            write!(f, " 移除目标文件夹相对源文件夹的多余文件")?;
        }
        if self.check_file_mtime {
            write!(f, " 检查修改时间")?;
        }
        if self.check_file_size {
            write!(f, " 检查大小")?;
        }
        if self.check_file_sha512 {
            write!(f, " 检查SHA-512")?;
        }
        Ok(())
    }
}

/* ---------- 预设 ---------- */
pub fn preset_default() -> SoftSyncPreset {
    SoftSyncPreset::default()
}

pub fn preset_for_append() -> SoftSyncPreset {
    SoftSyncPreset {
        name: "同步预设（用于更新包）".into(),
        check_file_size: true,
        check_file_mtime: false,
        check_file_sha512: true,
        remove_src_same_files: true,
        remove_dst_extra_files: false,
        exec: SoftSyncExec::None,
        ..Default::default()
    }
}

pub fn preset_flac() -> SoftSyncPreset {
    SoftSyncPreset {
        allow_src_exts: vec!["flac".into()],
        allow_other_exts: false,
        remove_dst_extra_files: false,
        ..Default::default()
    }
}

pub fn preset_mp4_avi() -> SoftSyncPreset {
    SoftSyncPreset {
        allow_src_exts: vec!["mp4".into(), "avi".into()],
        allow_other_exts: false,
        remove_dst_extra_files: false,
        ..Default::default()
    }
}

pub fn preset_cache() -> SoftSyncPreset {
    SoftSyncPreset {
        allow_src_exts: vec!["mp4".into(), "avi".into(), "flac".into()],
        allow_other_exts: false,
        remove_dst_extra_files: false,
        exec: SoftSyncExec::None,
        ..Default::default()
    }
}

/// 递归同步
pub async fn sync_folder(
    src_dir: impl AsRef<Path>,
    dst_dir: impl AsRef<Path>,
    preset: &SoftSyncPreset,
) -> io::Result<()> {
    let src_dir = src_dir.as_ref();
    let dst_dir = dst_dir.as_ref();

    let mut src_copy_files = Vec::new();
    let mut src_move_files = Vec::new();
    let mut src_remove_files = Vec::new();
    let mut dst_remove_files = Vec::new();
    let mut dst_remove_dirs = Vec::new();

    // 收集目录条目
    let mut src_entries = fs::read_dir(src_dir).await?;
    let mut dst_entries = fs::read_dir(dst_dir).await?;
    let mut src_map = HashMap::new();
    let mut dst_map = HashMap::new();

    while let Some(entry) = src_entries.next().await {
        let e = entry?;
        src_map.insert(e.file_name(), e);
    }
    while let Some(entry) = dst_entries.next().await {
        let e = entry?;
        dst_map.insert(e.file_name(), e);
    }

    // 1. 处理源
    for (name, entry) in src_map {
        let src_path = entry.path();
        let dst_path = dst_dir.join(&name);

        if entry.file_type().await?.is_dir() {
            if !dst_path.exists() {
                fs::create_dir_all(&dst_path).await?;
            }
            sync_folder(&src_path, &dst_path, preset).await?;
            continue;
        }

        // 处理文件
        let Some(ext) = name
            .to_str()
            .and_then(|s| s.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()))
        else {
            continue;
        };

        // 扩展名校验
        let mut ext_ok = preset.allow_other_exts;
        if preset.allow_src_exts.iter().any(|e| e == &ext) {
            ext_ok = true;
        }
        if preset.disallow_src_exts.iter().any(|e| e == &ext) {
            ext_ok = false;
        }
        if !ext_ok {
            continue;
        }

        // 扩展名绑定检查
        let mut bound = false;
        for (from, to) in &preset.no_activate_ext_bound_pairs {
            if from.iter().any(|e| e == &ext) {
                for to_ext in to {
                    let bound_path = dst_path.with_extension(to_ext);
                    if bound_path.exists() {
                        bound = true;
                        break;
                    }
                }
            }
            if bound {
                break;
            }
        }
        if bound {
            continue;
        }

        // 检查目标文件
        let dst_file_exists = dst_path.exists();
        let mut same = dst_file_exists;
        if dst_file_exists {
            let src_md = fs::metadata(&src_path).await?;
            let dst_md = fs::metadata(&dst_path).await?;

            if preset.check_file_size && same {
                same &= src_md.len() == dst_md.len();
            }
            if preset.check_file_mtime && same {
                // 比较 mtime 秒级即可
                let src_mtime = src_md.modified()?;
                let dst_mtime = dst_md.modified()?;
                same &= src_mtime == dst_mtime;
            }
            if preset.check_file_sha512 && same {
                same &= is_file_same_content(&src_path, &dst_path).await?;
            }
        }

        // 执行
        if !dst_file_exists || !same {
            match preset.exec {
                SoftSyncExec::None => {}
                SoftSyncExec::Copy => {
                    fs::copy(&src_path, &dst_path).await?;
                    src_copy_files.push(name.to_string_lossy().into_owned());
                }
                SoftSyncExec::Move => {
                    fs::rename(&src_path, &dst_path).await?;
                    src_move_files.push(name.to_string_lossy().into_owned());
                }
            }
        }

        if preset.remove_src_same_files && dst_file_exists && same {
            fs::remove_file(&src_path).await?;
            src_remove_files.push(name.to_string_lossy().into_owned());
        }
    }

    // 2. 清理目标多余条目
    if preset.remove_dst_extra_files {
        for (name, entry) in dst_map {
            let src_path = src_dir.join(&name);
            let dst_path = entry.path();

            if !smol::block_on(async { src_path.exists() }) {
                if entry.file_type().await?.is_dir() {
                    fs::remove_dir_all(&dst_path).await?;
                    dst_remove_dirs.push(name.to_string_lossy().into_owned());
                } else {
                    fs::remove_file(&dst_path).await?;
                    dst_remove_files.push(name.to_string_lossy().into_owned());
                }
            }
        }
    }

    // 打印
    let has_any = !src_copy_files.is_empty()
        || !src_move_files.is_empty()
        || !src_remove_files.is_empty()
        || !dst_remove_files.is_empty()
        || !dst_remove_dirs.is_empty();
    if has_any {
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

    Ok(())
}
