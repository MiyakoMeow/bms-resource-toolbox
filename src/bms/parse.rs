use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BmsDifficulty {
    Unknown = 0,
    Beginner = 1,
    Normal = 2,
    Hyper = 3,
    Another = 4,
    Insane = 5,
}

#[allow(clippy::match_same_arms)]
impl BmsDifficulty {
    #[must_use]
    pub fn from_int(v: i32) -> Self {
        match v {
            1 => Self::Beginner,
            2 => Self::Normal,
            3 => Self::Hyper,
            4 => Self::Another,
            5 => Self::Insane,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BmsInfo {
    pub title: String,
    pub artist: String,
    pub genre: String,
    pub difficulty: BmsDifficulty,
    pub playlevel: i32,
    pub bmp_formats: Vec<String>,
}

#[allow(clippy::missing_errors_doc)]
pub fn parse_bms_file(path: &Path, encoding: Option<&str>) -> crate::Result<BmsInfo> {
    let bytes = std::fs::read(path)?;
    let file_str = crate::bms::encoding::get_bms_file_str(&bytes, encoding);

    let mut title = String::new();
    let mut artist = String::new();
    let mut genre = String::new();
    let mut difficulty = BmsDifficulty::Unknown;
    let mut playlevel: i32 = 0;
    let mut ext_list: Vec<String> = Vec::new();

    for line in file_str.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("#ARTIST") {
            rest.trim().clone_into(&mut artist);
        } else if let Some(rest) = line.strip_prefix("#TITLE") {
            rest.trim().clone_into(&mut title);
        } else if let Some(rest) = line.strip_prefix("#GENRE") {
            rest.trim().clone_into(&mut genre);
        } else if let Some(rest) = line.strip_prefix("#PLAYLEVEL") {
            let value_str = rest.trim();
            if !value_str.is_empty()
                && value_str.bytes().all(|b| b.is_ascii_digit())
                && let Ok(val) = value_str.parse::<i32>()
            {
                playlevel = if (0..=99).contains(&val) { val } else { -1 };
            }
        } else if let Some(rest) = line.strip_prefix("#DIFFICULTY") {
            let value_str = rest.trim();
            if !value_str.is_empty()
                && value_str.bytes().all(|b| b.is_ascii_digit())
                && let Ok(val) = value_str.parse::<i32>()
            {
                difficulty = BmsDifficulty::from_int(val);
            }
        } else if let Some(rest) = line.strip_prefix("#BMP") {
            let value_str = rest.trim();
            if let Some(ext) = Path::new(value_str).extension() {
                ext_list.push(format!(".{}", ext.to_string_lossy()));
            }
        }
    }

    Ok(BmsInfo {
        title,
        artist,
        genre,
        difficulty,
        playlevel,
        bmp_formats: ext_list,
    })
}

fn json_get<'a>(value: &'a serde_json::Value, keys: &[&str]) -> &'a serde_json::Value {
    let mut current = value;
    for key in keys {
        current = current.get(key).unwrap_or(&serde_json::Value::Null);
    }
    current
}

#[allow(clippy::missing_errors_doc)]
pub fn parse_bmson_file(path: &Path, encoding: Option<&str>) -> crate::Result<BmsInfo> {
    let bytes = std::fs::read(path)?;
    let file_str = crate::bms::encoding::get_bms_file_str(&bytes, encoding);

    let bmson: serde_json::Value = serde_json::from_str(&file_str).map_err(|e| {
        crate::AppError::Parse(format!("JSON decode error for {}: {e}", path.display()))
    })?;

    let title = json_get(&bmson, &["info", "title"])
        .as_str()
        .unwrap_or("")
        .to_owned();
    let artist = json_get(&bmson, &["info", "artist"])
        .as_str()
        .unwrap_or("")
        .to_owned();
    let genre = json_get(&bmson, &["info", "genre"])
        .as_str()
        .unwrap_or("")
        .to_owned();
    #[allow(clippy::cast_possible_truncation)]
    let playlevel = json_get(&bmson, &["info", "level"])
        .as_i64()
        .unwrap_or(0) as i32;

    let mut ext_list: Vec<String> = Vec::new();
    if let Some(headers) = json_get(&bmson, &["bga", "bga_header"]).as_array() {
        for header in headers {
            if let Some(name) = header.get("name").and_then(|v| v.as_str())
                && let Some(ext) = Path::new(name).extension()
            {
                ext_list.push(format!(".{}", ext.to_string_lossy()));
            }
        }
    }

    Ok(BmsInfo {
        title,
        artist,
        genre,
        difficulty: BmsDifficulty::Unknown,
        playlevel,
        bmp_formats: ext_list,
    })
}
