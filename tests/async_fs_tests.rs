//! Async tests for fs::walk module.

use bms_resource_toolbox::fs::{has_chart_file, remove_empty_dirs};
use std::path::PathBuf;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("bms_toolbox_tests")
        .join(format!("{prefix}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn test_has_chart_file_returns_false_for_empty_dir() {
    let dir = unique_temp_dir("no_chart");
    assert!(!has_chart_file(&dir).await);
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_has_chart_file_returns_false_for_nonexistent_path() {
    let path = std::env::temp_dir().join("bms_toolbox_tests").join("nonexistent_48291");
    assert!(!has_chart_file(&path).await);
}

#[tokio::test]
async fn test_has_chart_file_returns_true_for_bms_dir() {
    let dir = unique_temp_dir("with_chart");
    tokio::fs::write(dir.join("test.bms"), "#TITLE Test\n").await.unwrap();

    assert!(has_chart_file(&dir).await);
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_has_chart_file_detects_all_extensions() {
    let dir = unique_temp_dir("ext_check");
    for ext in &["bms", "bme", "bml", "pms", "bmson"] {
        let sub = dir.join(ext);
        tokio::fs::create_dir_all(&sub).await.unwrap();
        tokio::fs::write(sub.join(format!("chart.{ext}")), "").await.unwrap();
        assert!(has_chart_file(&sub).await, "should detect .{ext}");
    }
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_has_chart_file_ignores_non_chart_files() {
    let dir = unique_temp_dir("no_ext_match");
    tokio::fs::write(dir.join("readme.txt"), "hello").await.unwrap();
    tokio::fs::write(dir.join("song.mp3"), "data").await.unwrap();
    tokio::fs::write(dir.join("image.png"), "data").await.unwrap();

    assert!(!has_chart_file(&dir).await);
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_remove_empty_dirs_removes_leaf() {
    let dir = unique_temp_dir("rm_empty");
    let empty = dir.join("empty_sub");
    tokio::fs::create_dir_all(&empty).await.unwrap();

    let result = remove_empty_dirs(&dir).await;
    assert!(result.is_ok());
    assert!(!empty.exists());
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_remove_empty_dirs_removes_nested_empty() {
    let dir = unique_temp_dir("rm_nested");
    let nested = dir.join("a").join("b").join("c");
    tokio::fs::create_dir_all(&nested).await.unwrap();

    let result = remove_empty_dirs(&dir).await;
    assert!(result.is_ok());
    assert!(!dir.join("a").exists());
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_remove_empty_dirs_preserves_non_empty() {
    let dir = unique_temp_dir("rm_preserve");
    let sub = dir.join("has_file");
    tokio::fs::create_dir_all(&sub).await.unwrap();
    tokio::fs::write(sub.join("data.txt"), "content").await.unwrap();

    let result = remove_empty_dirs(&dir).await;
    assert!(result.is_ok());
    assert!(sub.exists());
    assert!(sub.join("data.txt").exists());
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn test_remove_empty_dirs_on_nonexistent_path() {
    let path = std::env::temp_dir()
        .join("bms_toolbox_tests")
        .join("nonexistent_rm_7291");
    let result = remove_empty_dirs(&path).await;
    assert!(result.is_ok());
}
