pub mod move_ops;
pub mod name;
pub mod rawpack;
pub mod sync;

use std::collections::HashSet;
use std::path::Path;

use move_ops::is_dir_having_file;

pub fn remove_empty_folder(parent_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(parent_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !is_dir_having_file(&path) {
            println!("Remove empty dir: {}", path.display());
            if let Err(e) = std::fs::remove_dir_all(&path) {
                eprintln!(" x {e}!");
            }
        }
    }
}

const MEDIA_EXTS: &[&str] = &[
    ".ogg", ".wav", ".flac", ".mp4", ".wmv", ".avi", ".mpg", ".mpeg", ".bmp", ".jpg", ".png",
];

#[allow(clippy::struct_field_names)]
struct DirElements {
    file_set: HashSet<String>,
    media_stem_set: HashSet<String>,
    non_media_set: HashSet<String>,
}

fn fetch_dir_elements(dir: &Path) -> Option<DirElements> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };
    let paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();

    let mut file_set = HashSet::new();
    let mut media_stem_set = HashSet::new();
    let mut non_media_set = HashSet::new();

    for p in &paths {
        if !p.is_file() {
            continue;
        }
        let name = p.file_name()?.to_string_lossy().to_string();
        file_set.insert(name.clone());

        let lower = name.to_lowercase();
        let is_media = MEDIA_EXTS.iter().any(|ext| lower.ends_with(ext));
        if is_media {
            media_stem_set.insert(p.file_stem()?.to_string_lossy().to_string());
        } else {
            non_media_set.insert(name);
        }
    }

    Some(DirElements {
        file_set,
        media_stem_set,
        non_media_set,
    })
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn bms_dir_similarity(dir_a: &Path, dir_b: &Path) -> f64 {
    let Some(elems_a) = fetch_dir_elements(dir_a) else {
        return 0.0;
    };
    if elems_a.file_set.is_empty() || elems_a.media_stem_set.is_empty() || elems_a.non_media_set.is_empty() {
        return 0.0;
    }
    let Some(elems_b) = fetch_dir_elements(dir_b) else {
        return 0.0;
    };
    if elems_b.file_set.is_empty() || elems_b.media_stem_set.is_empty() || elems_b.non_media_set.is_empty() {
        return 0.0;
    }

    let intersection: HashSet<_> = elems_a
        .media_stem_set
        .intersection(&elems_b.media_stem_set)
        .cloned()
        .collect();
    let min_len = elems_a
        .media_stem_set
        .len()
        .min(elems_b.media_stem_set.len());

    intersection.len() as f64 / min_len as f64
}
