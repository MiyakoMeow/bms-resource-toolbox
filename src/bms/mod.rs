pub mod encoding;
pub mod parse;
pub mod work;

pub use parse::{BmsDifficulty, BmsInfo};

use std::path::Path;

pub const BMS_FILE_EXTS: &[&str] = &[".bms", ".bme", ".bml", ".pms"];
pub const BMSON_FILE_EXTS: &[&str] = &[".bmson"];
pub const CHART_FILE_EXTS: &[&str] = &[".bms", ".bme", ".bml", ".pms", ".bmson"];
pub const AUDIO_FILE_EXTS: &[&str] = &[".flac", ".ogg", ".wav"];
pub const VIDEO_FILE_EXTS: &[&str] = &[".mp4", ".mkv", ".avi", ".wmv", ".mpg", ".mpeg"];
pub const IMAGE_FILE_EXTS: &[&str] = &[".jpg", ".png", ".bmp", ".svg"];
pub const MEDIA_FILE_EXTS: &[&str] = &[
    ".flac",
    ".ogg",
    ".wav",
    ".mp4",
    ".mkv",
    ".avi",
    ".wmv",
    ".mpg",
    ".mpeg",
    ".jpg",
    ".png",
    ".bmp",
    ".svg",
];

fn ext_matches(filename: &str, exts: &[&str]) -> bool {
    let lower = filename.to_lowercase();
    exts.iter().any(|ext| lower.ends_with(ext))
}

#[must_use]
pub fn get_dir_bms_list(dir: &Path) -> Vec<BmsInfo> {
    let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
    let id = dir_name.split('.').next().unwrap_or(dir_name.as_ref());
    let encoding = crate::bms::encoding::BOFTT_ID_SPECIFIC_ENCODING_TABLE
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| *v);

    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };

    entries
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            let path = entry.path();
            let filename = path.file_name()?.to_string_lossy();
            if ext_matches(&filename, BMS_FILE_EXTS) {
                parse::parse_bms_file(&path, encoding).ok()
            } else if ext_matches(&filename, BMSON_FILE_EXTS) {
                parse::parse_bmson_file(&path, encoding).ok()
            } else {
                None
            }
        })
        .collect()
}

#[must_use]
pub fn get_dir_bms_info(dir: &Path) -> Option<BmsInfo> {
    let bms_list = get_dir_bms_list(dir);
    if bms_list.is_empty() {
        return None;
    }

    let titles: Vec<String> = bms_list.iter().map(|b| b.title.clone()).collect();
    let artists: Vec<String> = bms_list.iter().map(|b| b.artist.clone()).collect();
    let genres: Vec<String> = bms_list.iter().map(|b| b.genre.clone()).collect();

    let default_opts = work::ExtractOpts::default();
    let mut title = work::extract_work_name(&titles, &default_opts);

    if title.ends_with('-') {
        let dash_count = title.chars().filter(|&c| c == '-').count();
        if dash_count % 2 != 0 {
            let chars: Vec<char> = title.chars().collect();
            if chars.len() >= 2 && chars[chars.len() - 2].is_whitespace() {
                let trimmed: String = chars[..chars.len() - 1].iter().collect();
                trimmed.trim_end().clone_into(&mut title);
            }
        }
    }

    let artist_opts = work::ExtractOpts {
        remove_tailing_sign_list: vec![
            "/".into(),
            ":".into(),
            "：".into(),
            "-".into(),
            "obj".into(),
            "obj.".into(),
            "Obj".into(),
            "Obj.".into(),
            "OBJ".into(),
            "OBJ.".into(),
        ],
        ..default_opts
    };
    let artist = work::extract_work_name(&artists, &artist_opts);
    let genre = work::extract_work_name(&genres, &default_opts);

    Some(BmsInfo {
        title,
        artist,
        genre,
        difficulty: BmsDifficulty::Unknown,
        playlevel: 0,
        bmp_formats: vec![],
    })
}
