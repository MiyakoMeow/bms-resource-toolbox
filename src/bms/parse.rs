//! BMS file parsing.
//!
//! This module handles parsing of BMS and BMSON chart files
//! and extracting metadata like title, artist, and difficulty.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::bms::encoding::read_bms_file;
use crate::bms::types::{BMSDifficulty, BMSInfo};
use std::collections::HashMap;
use std::path::Path;

/// Parse a BMS file and extract metadata
///
/// # Errors
#[allow(dead_code)]
pub fn parse_bms_file<P: AsRef<Path>>(path: P) -> Result<BMSInfo, std::io::Error> {
    let content = read_bms_file(path)?;
    Ok(parse_bms_content(&content))
}

/// Parse BMS content string and extract metadata
#[allow(dead_code)]
pub fn parse_bms_content(content: &str) -> BMSInfo {
    let mut info = BMSInfo::default();
    let mut header_map: HashMap<String, String> = HashMap::new();

    // Parse all #KEY VALUE lines
    for line in content.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix('#') {
            if let Some(space_idx) = stripped.find(' ') {
                let key = stripped[..space_idx].to_uppercase();
                let value = stripped[space_idx + 1..].trim().to_string();
                header_map.insert(key, value);
            } else if let Some(tab_idx) = stripped.find('\t') {
                let key = stripped[..tab_idx].to_uppercase();
                let value = stripped[tab_idx + 1..].trim().to_string();
                header_map.insert(key, value);
            }
        }
    }

    // Extract known fields
    info.title = header_map
        .get("TITLE")
        .cloned()
        .unwrap_or_default();
    info.artist = header_map
        .get("ARTIST")
        .cloned()
        .unwrap_or_default();
    info.genre = header_map
        .get("GENRE")
        .cloned()
        .unwrap_or_default();

    // Parse playlevel
    if let Some(pl) = header_map.get("PLAYLEVEL") {
        info.playlevel = pl.parse().unwrap_or(0);
    }

    // Parse difficulty
    if let Some(diff) = header_map.get("DIFFICULTY") {
        info.difficulty = diff.parse::<i32>().map(BMSDifficulty::from).unwrap_or(BMSDifficulty::Unknown);
    }

    // Parse total
    if let Some(total) = header_map.get("TOTAL") {
        info.total = total.parse().ok();
    }

    // Parse stage file
    info.stage_file = header_map.get("STAGEFILE").cloned();

    // Collect BMP formats referenced
    let mut bmp_formats: Vec<String> = Vec::new();
    for key in header_map.keys() {
        if key.starts_with("BMP") && key.len() > 3 {
            let num = &key[3..];
            // Skip numeric keys that are likely channel data
            if num.parse::<f64>().is_err()
                && let Some(value) = header_map.get(&format!("BMP{num}"))
                    && !bmp_formats.contains(value) {
                        bmp_formats.push(value.clone());
                    }
        }
    }
    info.bmp_formats = bmp_formats;

    info
}

/// Parse a BMSON (JSON) file
///
/// # Errors
#[allow(dead_code)]
pub fn parse_bmson_file<P: AsRef<Path>>(path: P) -> Result<BMSInfo, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    parse_bmson_content(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Parse BMSON JSON content
#[allow(dead_code)]
pub fn parse_bmson_content(content: &str) -> Result<BMSInfo, serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct BmsonInfo {
        #[serde(rename = "title")]
        title: Option<String>,
        #[serde(rename = "artist")]
        artist: Option<String>,
        #[serde(rename = "genre")]
        genre: Option<String>,
        #[serde(rename = "playlevel")]
        playlevel: Option<i32>,
        #[serde(rename = "difficulty")]
        difficulty: Option<i32>,
        #[serde(rename = "total")]
        total: Option<f64>,
        #[serde(rename = "stagefile")]
        stagefile: Option<String>,
    }

    let info: BmsonInfo = serde_json::from_str(content)?;
    Ok(BMSInfo {
        title: info.title.unwrap_or_default(),
        artist: info.artist.unwrap_or_default(),
        genre: info.genre.unwrap_or_default(),
        difficulty: info.difficulty.map_or(BMSDifficulty::Unknown, BMSDifficulty::from),
        playlevel: info.playlevel.unwrap_or(0),
        bmp_formats: Vec::new(),
        total: info.total,
        stage_file: info.stagefile,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bms_content() {
        let content = r#"
#TITLE Test Song
#ARTIST Test Artist
#GENRE Test Genre
#PLAYLEVEL 5
#DIFFICULTY 3
#TOTAL 180.5
#STAGEFILE stage.png
"#;
        let info = parse_bms_content(content);
        assert_eq!(info.title, "Test Song");
        assert_eq!(info.artist, "Test Artist");
        assert_eq!(info.genre, "Test Genre");
        assert_eq!(info.playlevel, 5);
        assert_eq!(info.difficulty, BMSDifficulty::Hyper);
        assert_eq!(info.total, Some(180.5));
    }
}
