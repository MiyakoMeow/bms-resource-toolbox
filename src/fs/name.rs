use std::path::Path;

/// Replace characters invalid in filenames with fullwidth alternatives.
///
/// Returns a sanitized string safe for use as a file or directory name.
#[must_use]
pub fn get_valid_fs_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            ':' => '：',
            '\\' => '＼',
            '/' => '／',
            '*' => '＊',
            '?' => '？',
            '!' => '！',
            '"' => '＂',
            '<' => '＜',
            '>' => '＞',
            '|' => '｜',
            _ => c,
        })
        .collect()
}

/// Compute similarity between two BMS directories based on shared media files.
///
/// Returns a ratio `[0.0, 1.0]` indicating how similar the media content is.
#[must_use]
pub async fn bms_dir_similarity(dir_path_a: &Path, dir_path_b: &Path) -> f64 {
    use std::collections::HashSet;

    const MEDIA_EXTS: &[&str] = &[
        ".ogg", ".wav", ".flac", ".mp4", ".wmv", ".avi", ".mpg", ".mpeg", ".bmp", ".jpg", ".png",
    ];

    async fn fetch_dir_elements(
        dir_path: &Path,
    ) -> (HashSet<String>, HashSet<String>, HashSet<String>) {
        let mut file_set = HashSet::new();
        let mut media_set = HashSet::new();
        let mut non_media_set = HashSet::new();

        let Ok(mut entries) = tokio::fs::read_dir(dir_path).await else {
            return (file_set, media_set, non_media_set);
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            file_set.insert(name.to_string());

            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let ext = ext.to_lowercase();
            let with_dot = format!(".{ext}");

            if MEDIA_EXTS.contains(&with_dot.as_str()) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    media_set.insert(stem.to_string());
                }
            } else {
                non_media_set.insert(name.to_string());
            }
        }

        (file_set, media_set, non_media_set)
    }

    let (file_set_a, media_set_a, non_media_set_a) = fetch_dir_elements(dir_path_a).await;
    let (file_set_b, media_set_b, non_media_set_b) = fetch_dir_elements(dir_path_b).await;

    if file_set_a.is_empty()
        || file_set_b.is_empty()
        || media_set_a.is_empty()
        || media_set_b.is_empty()
        || non_media_set_a.is_empty()
        || non_media_set_b.is_empty()
    {
        return 0.0;
    }

    let intersection: HashSet<_> = media_set_a.intersection(&media_set_b).collect();
    let min_media_count = media_set_a.len().min(media_set_b.len());

    if min_media_count == 0 {
        return 0.0;
    }

    #[expect(clippy::cast_precision_loss)]
    let intersection_ratio = intersection.len() as f64 / min_media_count as f64;
    intersection_ratio
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_valid_fs_name() {
        assert_eq!(get_valid_fs_name("Artist - Title"), "Artist - Title");
        assert_eq!(get_valid_fs_name("Artist: Title"), "Artist： Title");
        assert_eq!(get_valid_fs_name("Test/File"), "Test／File");
        assert_eq!(get_valid_fs_name("Test\\File"), "Test＼File");
        assert_eq!(get_valid_fs_name("Test*File"), "Test＊File");
        assert_eq!(get_valid_fs_name("Test?File"), "Test？File");
        assert_eq!(get_valid_fs_name("Test!File"), "Test！File");
        assert_eq!(get_valid_fs_name("Test\"File"), "Test＂File");
        assert_eq!(get_valid_fs_name("Test<File"), "Test＜File");
        assert_eq!(get_valid_fs_name("Test>File"), "Test＞File");
        assert_eq!(get_valid_fs_name("Test|File"), "Test｜File");
    }
}
