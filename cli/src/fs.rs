pub mod moving;
pub mod rawpack;
pub mod sync;

use std::{collections::HashSet, path::Path};

use sha3::{Digest, Sha3_512, digest::Output};
use smol::{
    fs,
    io::{self, AsyncReadExt},
    stream::StreamExt,
};

/// Signs:
///  ：＼／＊？＂＜＞｜
pub fn get_vaild_fs_name(ori_name: &str) -> String {
    ori_name
        .replace(":", "：")
        .replace("\\", "＼")
        .replace("/", "／")
        .replace("*", "＊")
        .replace("?", "？")
        .replace("!", "！")
        .replace('"', "＂")
        .replace("<", "＜")
        .replace(">", "＞")
        .replace("|", "｜")
}

/// Quick check if two files have the same content (SHA256)
pub async fn is_file_same_content(a: &Path, b: &Path) -> io::Result<bool> {
    async fn sha256(path: &Path) -> io::Result<Output<Sha3_512>> {
        let mut file = fs::File::open(path).await?;
        let mut hasher = Sha3_512::new();
        let mut buf = vec![0; 64 * 1024];
        loop {
            let n = file.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(hasher.finalize())
    }
    let a_md = fs::metadata(a).await?;
    let b_md = fs::metadata(b).await?;
    if a_md.len() != b_md.len() || a_md.is_dir() || b_md.is_dir() {
        return Ok(false);
    }
    let a = sha256(a).await?;
    let b = sha256(b).await?;
    Ok(a == b)
}

/// Check if directory "contains files"
pub async fn is_dir_having_file(dir: &Path) -> io::Result<bool> {
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let ft = entry.file_type().await?;
        if ft.is_file() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Common media extensions
pub const MEDIA_EXT_LIST: &[&str] = {
    &[
        ".ogg", ".wav", ".flac", ".mp4", ".wmv", ".avi", ".mpg", ".mpeg", ".bmp", ".jpg", ".png",
    ]
};

/// Remove all empty directories under parent_dir
pub async fn remove_empty_folders(parent_dir: impl AsRef<Path>, dry_run: bool) -> io::Result<()> {
    let parent = parent_dir.as_ref();
    let mut entries = fs::read_dir(parent).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let path = entry.path();
        let ft = entry.file_type().await?;
        if !ft.is_dir() {
            continue;
        }
        if !is_dir_having_file(&path).await? {
            log::info!("Remove empty dir: {}", path.display());
            if dry_run {
                log::info!("[dry-run] Skipped removing {}", path.display());
            } else if let Err(e) = fs::remove_dir_all(&path).await {
                log::info!(" x {e}!");
            }
        }
    }
    Ok(())
}

/// Directory triple: (all files, media file stems, non-media files)
#[derive(Debug, Default)]
#[allow(unused)]
struct DirElements {
    files: Vec<String>,
    media_stems: HashSet<String>,
    non_media_stems: HashSet<String>,
}

async fn fetch_dir_elements(dir: impl AsRef<Path>) -> io::Result<DirElements> {
    let dir = dir.as_ref();
    let mut entries = fs::read_dir(dir).await?;
    let mut files = Vec::new();
    let mut media_stems: HashSet<String> = HashSet::new();
    let mut non_media_stems: HashSet<String> = HashSet::new();

    while let Some(entry) = entries.next().await {
        let file_path = entry?.path();
        let file_stem = file_path
            .file_stem()
            .and_then(|path| path.to_str())
            .unwrap_or("");
        let file_ext = file_path
            .extension()
            .and_then(|path| path.to_str())
            .unwrap_or("bms");
        if MEDIA_EXT_LIST.contains(&file_ext) {
            media_stems.insert(file_stem.to_string());
        } else {
            non_media_stems.insert(file_stem.to_string());
        }
        files.push(file_stem.to_string());
    }

    Ok(DirElements {
        files,
        media_stems,
        non_media_stems,
    })
}

/// Calculate similarity between two directories (intersection of media file stems / smaller set)
pub async fn bms_dir_similarity(
    dir_a: impl AsRef<Path>,
    dir_b: impl AsRef<Path>,
) -> io::Result<f64> {
    let a = fetch_dir_elements(dir_a).await?;
    let b = fetch_dir_elements(dir_b).await?;

    if a.files.is_empty()
        || a.media_stems.is_empty()
        || b.files.is_empty()
        || b.media_stems.is_empty()
    {
        return Ok(0.0);
    }

    let intersect = a.media_stems.intersection(&b.media_stems).count();
    let min = a.media_stems.len().min(b.media_stems.len());
    Ok(intersect as f64 / min as f64)
}
