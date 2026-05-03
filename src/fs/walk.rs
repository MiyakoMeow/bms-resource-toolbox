use std::path::Path;

pub async fn remove_empty_dirs(dir: &Path) -> Result<(), std::io::Error> {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return Ok(());
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !super::pack_move::is_dir_having_file(&path).await {
            println!("Remove empty dir: {path:?}");
            match tokio::fs::remove_dir_all(&path).await {
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    println!(" x PermissionError!");
                }
                Err(e) => return Err(e),
                Ok(()) => {}
            }
        }
    }

    Ok(())
}
