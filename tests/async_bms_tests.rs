//! Async tests for `bms::encoding` and `bms::parse` modules.

use bms_resource_toolbox::bms::encoding::read_bms_file;
use bms_resource_toolbox::bms::parse::parse_bms_file;
use std::path::PathBuf;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("bms_toolbox_tests")
        .join(format!("{prefix}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn test_read_bms_file_utf8() {
    let dir = unique_temp_dir("read_utf8");
    let path = dir.join("test.bms");
    tokio::fs::write(&path, "#TITLE テスト曲\n#ARTIST テスト\n")
        .await
        .unwrap();

    let content = read_bms_file(&path).await.unwrap();
    assert!(content.contains("#TITLE"));
    assert!(content.contains("テスト曲"));
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_read_bms_file_ascii() {
    let dir = unique_temp_dir("read_ascii");
    let path = dir.join("test.bms");
    tokio::fs::write(&path, "#TITLE Test Song\n#ARTIST Artist\n")
        .await
        .unwrap();

    let content = read_bms_file(&path).await.unwrap();
    assert!(content.contains("#TITLE Test Song"));
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_read_bms_file_nonexistent() {
    let path = std::env::temp_dir()
        .join("bms_toolbox_tests")
        .join("nonexistent_read_48291.bms");
    let result = read_bms_file(&path).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parse_bms_file_full_metadata() {
    let dir = unique_temp_dir("parse_full");
    let path = dir.join("test.bms");
    let content = "\
#TITLE Test Song
#ARTIST Test Artist
#GENRE Genre
#PLAYLEVEL 7
#DIFFICULTY 4
#TOTAL 200.5
#STAGEFILE bg.png
";
    tokio::fs::write(&path, content).await.unwrap();

    let info = parse_bms_file(&path, None).await.unwrap();
    assert_eq!(info.title, "Test Song");
    assert_eq!(info.artist, "Test Artist");
    assert_eq!(info.genre, "Genre");
    assert_eq!(info.playlevel, 7);
    assert_eq!(
        info.difficulty,
        bms_resource_toolbox::bms::BMSDifficulty::Another
    );
    assert_eq!(info.total, None);
    assert_eq!(info.stage_file, None);
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_parse_bms_file_minimal() {
    let dir = unique_temp_dir("parse_min");
    let path = dir.join("test.bms");
    tokio::fs::write(&path, "#TITLE Only Title\n")
        .await
        .unwrap();

    let info = parse_bms_file(&path, None).await.unwrap();
    assert_eq!(info.title, "Only Title");
    assert_eq!(info.artist, "");
    assert_eq!(info.playlevel, 0);
    assert_eq!(info.total, None);
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_parse_bms_file_nonexistent() {
    let path = std::env::temp_dir()
        .join("bms_toolbox_tests")
        .join("nonexistent_parse_48291.bms");
    let result = parse_bms_file(&path, None).await;
    assert!(result.is_err());
}
