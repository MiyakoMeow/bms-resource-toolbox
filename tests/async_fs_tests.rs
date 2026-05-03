//! Async tests for `fs::walk` module.

use bms_resource_toolbox::fs::remove_empty_dirs;
use std::path::PathBuf;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("bms_toolbox_tests")
        .join(format!("{prefix}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
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
    tokio::fs::write(sub.join("data.txt"), "content")
        .await
        .unwrap();

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
