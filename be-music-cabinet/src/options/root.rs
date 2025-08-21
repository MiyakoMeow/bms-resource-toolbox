use std::path::Path;

use futures::StreamExt;
use smol::{fs, io};
use strsim::jaro_winkler;

/// 该脚本使用于以下情况：
/// 已经有一个文件夹A，它的子文件夹名为“”等带有编号+小数点的形式。
/// 现在有另一个文件夹B，它的子文件夹名都只有编号。
/// 将A中的子文件夹名，同步给B的对应的子文件夹。
pub async fn copy_numbered_workdir_names(
    root_dir_from: impl AsRef<Path>,
    root_dir_to: impl AsRef<Path>,
) -> io::Result<()> {
    let root_from = root_dir_from.as_ref();
    let root_to = root_dir_to.as_ref();

    // 收集 root_from 下所有目录名
    let mut src_names = Vec::new();
    let mut entries = fs::read_dir(root_from).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && let Some(name) = path.file_name()
        {
            src_names.push(name.to_string_lossy().into_owned());
        }
    }

    // 处理 root_to 下的目录
    let mut dst_entries = fs::read_dir(root_to).await?;
    while let Some(entry) = dst_entries.next().await {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();

        // 取第一段数字（空格或点号之前）
        let dir_num = dir_name_str
            .split_whitespace()
            .next()
            .and_then(|s| s.split('.').next())
            .unwrap_or_default();

        if dir_num.chars().all(|c| c.is_ascii_digit()) {
            // 在 src_names 中找以 dir_num 开头的目录
            if let Some(src_name) = src_names.iter().find(|n| n.starts_with(dir_num)) {
                let target_path = root_to.join(src_name);
                println!("Rename {:?} -> {}", path.display(), src_name);
                fs::rename(&path, &target_path).await?;
            }
        }
    }

    Ok(())
}

/// 异步扫描 `root_dir` 下的子目录，并按字典序两两比对相似度。
/// 当相似度 ≥ `similarity_trigger` 时，打印这一对目录。
///
/// # Example
/// ```
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     scan_folder_similar_folders("./", 0.7).await?;
///     Ok(())
/// }
/// ```
pub async fn scan_folder_similar_folders(
    root_dir: impl AsRef<Path>,
    similarity_trigger: f64,
) -> io::Result<Vec<(String, String, f64)>> {
    // 读目录 -> 收集所有子目录的名字（相对名）
    let mut entries = fs::read_dir(root_dir.as_ref()).await?;
    let mut dir_names = Vec::new();

    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let file_type = entry.file_type().await?;
        if file_type.is_dir() {
            dir_names.push(entry.file_name().into_string().unwrap());
        }
    }

    // 按字典序排序
    dir_names.sort_unstable();

    // 顺序扫描相邻两项
    let print_tasks = dir_names
        .windows(2)
        .filter_map(|w| {
            let (former, current) = (&w[0], &w[1]);
            let similarity = jaro_winkler(former, current); // ← 改动在这里
            (similarity >= similarity_trigger).then_some((
                former.to_string(),
                current.to_string(),
                similarity,
            ))
        })
        .collect::<Vec<_>>();

    Ok(print_tasks)
}
