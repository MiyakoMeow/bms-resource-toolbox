//! BMS file parsing.
//!
//! This module handles parsing of BMS and BMSON chart files
//! and extracting metadata like title, artist, and difficulty.

#![allow(clippy::cast_possible_truncation)]

use crate::bms::types::{BMSDifficulty, BMSInfo};
use std::path::Path;

fn is_decimal_digit(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_digit() || ('\u{FF10}'..='\u{FF19}').contains(&c))
}

fn parse_fullwidth_number(s: &str) -> Option<f64> {
    let normalized: String = s
        .chars()
        .map(|c| {
            if ('\u{FF10}'..='\u{FF19}').contains(&c) {
                char::from_digit(c as u32 - 0xFF10, 10).unwrap()
            } else {
                c
            }
        })
        .collect();
    normalized.parse::<f64>().ok()
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
                && is_decimal_digit(&value_str)
                && let Some(val) = parse_fullwidth_number(&value_str)
            {
                playlevel = if (0.0..=99.0).contains(&val) {
                    val as i32
                } else {
                    -1
                };
            }
        } else if line.starts_with("#DIFFICULTY") {
            let value_str = line.replace("#DIFFICULTY", "").trim().to_string();
            if !value_str.is_empty()
                && is_decimal_digit(&value_str)
                && let Some(val) = parse_fullwidth_number(&value_str)
                && (0.0..=5.0).contains(&val)
            {
                let diff_val = val as i32;
                difficulty = BMSDifficulty::from(diff_val);
            }
        } else if line.starts_with("#BMP") {
            let value_str = line.replace("#BMP", "").trim().to_string();
            let ext = std::path::Path::new(&value_str)
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            bmp_formats.push(ext);
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
///
/// Returns `std::io::Error` if the file cannot be read,
/// or if the parsed content cannot be decoded.
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
///
/// # Errors
///
/// Returns `serde_json::Error` if the content is not valid JSON or has an unexpected structure.
#[allow(clippy::cast_possible_truncation)]
pub fn parse_bmson_content(content: &str) -> Result<BMSInfo, serde_json::Error> {
    let root: serde_json::Value = serde_json::from_str(content)?;

    let info = root.get("info").cloned().unwrap_or(serde_json::Value::Null);

    let title = info
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let artist = info
        .get("artist")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let genre = info
        .get("genre")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let level = info
        .get("level")
        .and_then(|v| match v {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse::<f64>().ok(),
            _ => None,
        })
        .unwrap_or(0.0);

    let mut bmp_formats: Vec<String> = Vec::new();
    if let Some(bga) = root.get("bga")
        && let Some(bga_headers) = bga.get("bga_header")
        && let Some(headers) = bga_headers.as_array()
    {
        for header in headers {
            if let Some(name) = header.get("name").and_then(|v| v.as_str()) {
                let ext = std::path::Path::new(name)
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                bmp_formats.push(ext);
            }
        }
    }

    Ok(BMSInfo {
        title,
        artist,
        genre,
        difficulty: BMSDifficulty::Unknown,
        playlevel: level as i32,
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
