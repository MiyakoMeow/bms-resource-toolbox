//! BMS file parsing.
//!
//! This module handles parsing of BMS and BMSON chart files
//! and extracting metadata like title, artist, and difficulty.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::bms::encoding::read_bms_file;
use crate::bms::types::{BMSDifficulty, BMSInfo};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

/// Parse a BMS file and extract metadata - 异步版本
///
/// # Errors
pub async fn parse_bms_file<P: AsRef<Path>>(path: P) -> Result<BMSInfo, std::io::Error> {
    let content = read_bms_file(path).await?;
    Ok(parse_bms_content(&content))
}

/// Parse BMS content string and extract metadata
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
    info.title = header_map.get("TITLE").cloned().unwrap_or_default();
    info.artist = header_map.get("ARTIST").cloned().unwrap_or_default();
    info.genre = header_map.get("GENRE").cloned().unwrap_or_default();

    // Parse playlevel
    if let Some(pl) = header_map.get("PLAYLEVEL") {
        info.playlevel = pl.parse().unwrap_or(0);
    }

    // Parse difficulty
    if let Some(diff) = header_map.get("DIFFICULTY") {
        info.difficulty = diff
            .parse::<i32>()
            .map(BMSDifficulty::from)
            .unwrap_or(BMSDifficulty::Unknown);
    }

    // Parse total
    if let Some(total) = header_map.get("TOTAL") {
        info.total = total.parse().ok();
    }

    // Parse stage file
    info.stage_file = header_map.get("STAGEFILE").cloned();

    // Collect BMP formats referenced
    // Python parses all #BMP* lines and extracts suffix from the value
    let mut bmp_formats: Vec<String> = Vec::new();

    // Handle #BMP line itself (no numeric suffix)
    if let Some(value) = header_map.get("BMP")
        && let Some(ext) = Path::new(value).extension()
    {
        let ext_str = format!(".{}", ext.to_string_lossy());
        if !ext_str.is_empty() && !bmp_formats.contains(&ext_str) {
            bmp_formats.push(ext_str);
        }
    }

    // Handle #BMP01, #BMP02, etc. (channel keys - skip if value looks like a number)
    for key in header_map.keys() {
        if key.starts_with("BMP") && key.len() > 3 {
            let num = &key[3..];
            // Skip numeric keys that are likely channel data (e.g., BMP01)
            if num.parse::<f64>().is_err()
                && let Some(value) = header_map.get(&format!("BMP{num}"))
                && !bmp_formats.contains(value)
            {
                bmp_formats.push(value.clone());
            }
        }
    }
    info.bmp_formats = bmp_formats;

    info
}

/// Parse a BMSON (JSON) file - 异步版本
///
/// # Errors
pub async fn parse_bmson_file<P: AsRef<Path>>(path: P) -> Result<BMSInfo, std::io::Error> {
    let content = fs::read_to_string(path).await?;
    parse_bmson_content(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Parse BMSON JSON content
///
/// BMSON format stores metadata in an `info` nested object, not at the top level.
/// This matches Python's `parse_bmson_file` behavior.
pub fn parse_bmson_content(content: &str) -> Result<BMSInfo, serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct BmsonInfo {
        title: Option<String>,
        artist: Option<String>,
        genre: Option<String>,
        #[serde(rename = "level")]
        level: Option<i32>,
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
            if let Some(ext) = Path::new(&header.name).extension() {
                let ext_str = format!(".{}", ext.to_string_lossy());
                if !bmp_formats.contains(&ext_str) {
                    bmp_formats.push(ext_str);
                }
            }
        }
    }

    Ok(BMSInfo {
        title: info.title.unwrap_or_default(),
        artist: info.artist.unwrap_or_default(),
        genre: info.genre.unwrap_or_default(),
        difficulty: BMSDifficulty::Unknown,
        playlevel: info.level.unwrap_or(0),
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
        assert_eq!(info.total, Some(180.5));
    }

    #[test]
    fn test_parse_bms_content() {
        let content = r#"#TITLE Test Title
#ARTIST Test Artist
#GENRE Test Genre
#PLAYLEVEL 5
#DIFFICULTY 3
"#;
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
}
