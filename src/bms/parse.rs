//! BMS file parsing.
//!
//! This module handles parsing of BMS and BMSON chart files
//! and extracting metadata like title, artist, and difficulty.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::bms::encoding::read_bms_file;
use crate::bms::types::{BMSDifficulty, BMSInfo};
use std::path::Path;

/// Parse a BMS file and extract metadata - 异步版本
///
/// # Errors
#[allow(dead_code)]
pub async fn parse_bms_file<P: AsRef<Path>>(path: P) -> Result<BMSInfo, std::io::Error> {
    let content = read_bms_file(path).await?;
    Ok(parse_bms_content(&content))
}

/// Parse BMS text content and extract metadata.
#[must_use]
pub fn parse_bms_content(content: &str) -> BMSInfo {
    let mut title = String::new();
    let mut artist = String::new();
    let mut genre = String::new();
    let mut difficulty = BMSDifficulty::Unknown;
    let mut playlevel: i32 = 0;
    let mut bmp_formats: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("#ARTIST") {
            artist = line.replace("#ARTIST", "").trim().to_string();
        } else if line.starts_with("#TITLE") {
            title = line.replace("#TITLE", "").trim().to_string();
        } else if line.starts_with("#GENRE") {
            genre = line.replace("#GENRE", "").trim().to_string();
        } else if line.starts_with("#PLAYLEVEL") {
            let value_str = line.replace("#PLAYLEVEL", "").trim().to_string();
            if !value_str.is_empty()
                && value_str.chars().all(|c| c.is_ascii_digit())
                && let Ok(val) = value_str.parse::<f64>()
            {
                #[allow(clippy::cast_possible_truncation)]
                let int_val = val as i32;
                playlevel = if (0..=99).contains(&int_val) {
                    int_val
                } else {
                    -1
                };
            }
        } else if line.starts_with("#DIFFICULTY") {
            let value_str = line.replace("#DIFFICULTY", "").trim().to_string();
            if !value_str.is_empty()
                && value_str.chars().all(|c| c.is_ascii_digit())
                && let Ok(val) = value_str.parse::<f64>()
            {
                #[allow(clippy::cast_possible_truncation)]
                let int_val = val as i32;
                if (0..=5).contains(&int_val) {
                    difficulty = BMSDifficulty::from(int_val);
                }
            }
        } else if line.starts_with("#BMP") {
            let value_str = line.replace("#BMP", "").trim().to_string();
            if let Some(dot_pos) = value_str.rfind('.') {
                let ext = &value_str[dot_pos..];
                if !ext.is_empty() {
                    bmp_formats.push(ext.to_string());
                }
            }
        }
    }

    BMSInfo {
        title,
        artist,
        genre,
        difficulty,
        playlevel,
        bmp_formats,
        total: None,
        stage_file: None,
    }
}

/// Parse a BMSON (JSON) file - 异步版本
///
/// # Errors
pub async fn parse_bmson_file<P: AsRef<Path>>(
    path: P,
    encoding: Option<&str>,
) -> Result<BMSInfo, std::io::Error> {
    let bytes = tokio::fs::read(path.as_ref()).await?;
    let content = crate::bms::encoding::get_bms_file_str(&bytes, encoding);
    match parse_bmson_content(&content) {
        Ok(info) => Ok(info),
        Err(e) => {
            tracing::info!(" !_!: Json Decode Error! {:?}", e);
            Ok(BMSInfo::new(
                "Error".to_string(),
                "Error".to_string(),
                "Error".to_string(),
            ))
        }
    }
}

/// Parse BMSON JSON content
///
/// BMSON format stores metadata in an `info` nested object, not at the top level.
/// This matches Python's `parse_bmson_file` behavior.
#[allow(clippy::cast_possible_truncation)]
pub fn parse_bmson_content(content: &str) -> Result<BMSInfo, serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct BmsonInfo {
        title: Option<String>,
        artist: Option<String>,
        genre: Option<String>,
        level: Option<f64>,
    }

    #[derive(serde::Deserialize)]
    struct BgaHeader {
        name: String,
    }

    #[derive(serde::Deserialize)]
    struct Bga {
        bga_header: Option<Vec<BgaHeader>>,
    }

    #[derive(serde::Deserialize)]
    struct BmsonRoot {
        info: Option<BmsonInfo>,
        bga: Option<Bga>,
    }

    let root: BmsonRoot = serde_json::from_str(content)?;

    let info = root.info.unwrap_or(BmsonInfo {
        title: None,
        artist: None,
        genre: None,
        level: None,
    });

    let mut bmp_formats: Vec<String> = Vec::new();
    if let Some(bga) = root.bga
        && let Some(bga_headers) = bga.bga_header
    {
        for header in bga_headers {
            if let Some(dot_pos) = header.name.rfind('.') {
                let ext = &header.name[dot_pos..];
                if !ext.is_empty() {
                    bmp_formats.push(ext.to_string());
                }
            }
        }
    }

    Ok(BMSInfo {
        title: info.title.unwrap_or_default(),
        artist: info.artist.unwrap_or_default(),
        genre: info.genre.unwrap_or_default(),
        difficulty: BMSDifficulty::Unknown,
        playlevel: info.level.unwrap_or(0.0) as i32,
        bmp_formats,
        total: None,
        stage_file: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bms_content_basic() {
        let content = r"
#TITLE Test Song
#ARTIST Test Artist
#GENRE Test Genre
#PLAYLEVEL 5
#DIFFICULTY 3
#TOTAL 180.5
#STAGEFILE stage.png
";
        let info = parse_bms_content(content);
        assert_eq!(info.title, "Test Song");
        assert_eq!(info.artist, "Test Artist");
        assert_eq!(info.genre, "Test Genre");
        assert_eq!(info.playlevel, 5);
        assert_eq!(info.difficulty, BMSDifficulty::Hyper);
    }

    #[test]
    fn test_parse_bms_content() {
        let content = r"#TITLE Test Title
#ARTIST Test Artist
#GENRE Test Genre
#PLAYLEVEL 5
#DIFFICULTY 3
";
        let info = parse_bms_content(content);
        assert_eq!(info.title, "Test Title");
        assert_eq!(info.artist, "Test Artist");
        assert_eq!(info.genre, "Test Genre");
        assert_eq!(info.playlevel, 5);
        assert_eq!(info.difficulty, BMSDifficulty::Hyper);
    }

    #[test]
    fn test_parse_bmson_content() {
        let content = r#"{
            "info": {
                "title": "Test Title",
                "artist": "Test Artist",
                "genre": "Test Genre",
                "level": 5
            }
        }"#;
        let info = parse_bmson_content(content).unwrap();
        assert_eq!(info.title, "Test Title");
        assert_eq!(info.artist, "Test Artist");
        assert_eq!(info.genre, "Test Genre");
        assert_eq!(info.playlevel, 5);
    }

    #[test]
    fn test_parse_bmp_formats() {
        let content = r"
#BMP bg.bmp
#BMP01 image01.png
#BMP02 image02.jpg
#BMPAA other.gif
";
        let info = parse_bms_content(content);
        assert_eq!(info.bmp_formats, vec![".bmp", ".png", ".jpg", ".gif"]);
    }

    #[test]
    fn test_playlevel_out_of_range() {
        let content = "#PLAYLEVEL 100";
        let info = parse_bms_content(content);
        assert_eq!(info.playlevel, -1);

        let content = "#PLAYLEVEL 50";
        let info = parse_bms_content(content);
        assert_eq!(info.playlevel, 50);
    }

    #[test]
    fn test_playlevel_non_decimal() {
        let content = "#PLAYLEVEL -5";
        let info = parse_bms_content(content);
        assert_eq!(info.playlevel, 0);

        let content = "#PLAYLEVEL abc";
        let info = parse_bms_content(content);
        assert_eq!(info.playlevel, 0);
    }

    #[test]
    fn test_bmson_float_level() {
        let content = r#"{"info":{"title":"T","artist":"A","genre":"G","level":7.5}}"#;
        let info = parse_bmson_content(content).unwrap();
        assert_eq!(info.playlevel, 7);
    }
}
