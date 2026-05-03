use std::path::Path;

#[must_use]
pub fn get_ext(path: &Path) -> &str {
    path.file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.rsplit('.').next())
        .unwrap_or("")
}

pub async fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), std::io::Error> {
    if !source.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Source is not a directory",
        ));
    }

    tokio::fs::create_dir_all(target).await?;

    let mut entries = tokio::fs::read_dir(source).await?;
    while let Some(entry) = entries.next_entry().await? {
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            Box::pin(copy_dir_recursive(&source_path, &target_path)).await?;
        } else {
            tokio::fs::copy(&source_path, &target_path).await?;
        }
    }

    Ok(())
}
