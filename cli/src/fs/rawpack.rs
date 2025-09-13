use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

use smol::stream::StreamExt;
use smol::{fs, io};

use crate::fs::moving::{ReplacePreset, move_elements_across_dir, replace_options_from_preset};

/// Extract supported archives to specified cache directory
pub async fn unzip_file_to_cache_dir(
    file_path: impl AsRef<Path>,
    cache_dir_path: impl AsRef<Path>,
) -> io::Result<()> {
    let file_path = file_path.as_ref();
    let cache_dir_path = cache_dir_path.as_ref();

    let file_name = file_path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("unknown");

    let ext = file_path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "zip" => extract_zip(file_path, cache_dir_path).await?,
        "7z" => extract_7z(file_path, cache_dir_path).await?,
        "rar" => extract_rar(file_path, cache_dir_path).await?,
        _ => {
            // Not an archive => copy after space
            let target_name = file_name
                .split_once(' ')
                .map(|(_, s)| s)
                .unwrap_or(file_name);
            let target = cache_dir_path.join(target_name);
            fs::copy(file_path, target).await?;
        }
    }
    Ok(())
}

/* ---------- ZIP ---------- */
async fn extract_zip(src: &Path, dst: &Path) -> io::Result<()> {
    log::info!("Extracting {} to {} (zip)", src.display(), dst.display());
    let file = std::fs::File::open(src)?;
    let mut archive = zip::ZipArchive::new(file)?;
    smol::block_on(async move { archive.extract(dst) }).map_err(io::Error::other)
}

/* ---------- 7z ---------- */
async fn extract_7z(src: &Path, dst: &Path) -> io::Result<()> {
    log::info!("Extracting {} to {} (7z)", src.display(), dst.display());
    // sevenz-rust is a synchronous library, spawn_blocking
    let src = src.to_path_buf();
    let dst = dst.to_path_buf();
    smol::block_on(async move { sevenz_rust::decompress_file(&src, &dst) })
        .map_err(io::Error::other)
}

/* ---------- RAR ---------- */
async fn extract_rar(src: &Path, dst: &Path) -> io::Result<()> {
    log::info!("Extracting {} to {} (RAR)", src.display(), dst.display());
    // unrar is a synchronous library
    let src = src.to_path_buf();
    let dst = dst.to_path_buf();
    let mut archive =
        smol::block_on(async move { unrar::Archive::new(&src).open_for_processing() })
            .map_err(io::Error::other)?;
    smol::block_on(async move {
        while let Some(header) = archive.read_header().map_err(io::Error::other)? {
            log::info!(
                "{} bytes: {}",
                header.entry().unpacked_size,
                header.entry().filename.to_string_lossy(),
            );
            let dst_path = dst.join(header.entry().filename.as_path());
            archive = if header.entry().is_file() {
                header.extract_to(dst_path).map_err(io::Error::other)?
            } else {
                header.skip().map_err(io::Error::other)?
            };
        }
        Ok(())
    })
}

/// Extract "numeric prefix" file name list from pack directory
pub fn get_num_set_file_names(pack_dir: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let mut res = Vec::new();
    for entry in std::fs::read_dir(pack_dir.as_ref())? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let id_str = name.split(' ').next().unwrap_or("");
        if id_str.chars().all(|c| c.is_ascii_digit()) && entry.file_type()?.is_file() {
            res.push(name);
        }
    }
    Ok(res)
}

/// Flatten next level files/directories in cache directory to current level
pub async fn move_out_files_in_folder_in_cache_dir(
    cache_dir_path: impl AsRef<Path>,
    replace_preset: ReplacePreset,
) -> io::Result<bool> {
    let cache_dir_path = cache_dir_path.as_ref();

    let mut done = false;
    let mut error = false;

    let mut cache_folder_count = 0;
    let mut cache_file_count = 0;
    let mut file_ext_count: HashMap<String, Vec<String>> = HashMap::new();
    loop {
        let mut inner_dir_name = None;

        // Rescan directory
        let mut entries = fs::read_dir(cache_dir_path).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if entry.file_type().await?.is_dir() {
                if name == "__MACOSX" {
                    fs::remove_dir_all(&path).await?;
                    continue;
                }
                cache_folder_count += 1;
                inner_dir_name = Some(name);
            } else {
                cache_file_count += 1;
                let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
                file_ext_count.entry(ext).or_default().push(name);
            }
        }

        if cache_folder_count == 0 {
            done = true;
        }
        if cache_folder_count == 1 && cache_file_count >= 10 {
            done = true;
        }
        if cache_folder_count > 1 {
            log::info!(
                " !_! {}: has more than 1 folders, please do it manually.",
                cache_dir_path.display()
            );
            error = true;
        }
        if done || error {
            break;
        }

        // Move inner directory contents to current level
        if let Some(name) = inner_dir_name {
            let inner_path = cache_dir_path.join(&name);
            let inner_inner = inner_path.join(&name);
            if fs::metadata(&inner_inner).await.is_ok_and(|m| m.is_dir()) {
                log::info!(
                    " - Renaming inner inner dir name: {}",
                    inner_inner.display()
                );
                fs::rename(&inner_inner, format!("{}-rep", inner_inner.display())).await?;
            }
            log::info!(
                " - Moving inner files in {} to {}",
                inner_path.display(),
                cache_dir_path.display()
            );
            move_elements_across_dir(
                &inner_path,
                cache_dir_path,
                replace_options_from_preset(replace_preset),
            )
            .await?;
            fs::remove_dir(&inner_path).await.ok();
        }
    }

    if error {
        return Ok(false);
    }

    if cache_folder_count == 0 && cache_file_count == 0 {
        log::info!(" !_! {}: Cache is Empty!", cache_dir_path.display());
        fs::remove_dir(cache_dir_path).await?;
        return Ok(false);
    }

    // Warn about multiple mp4 files
    if let Some(mp4) = file_ext_count.get("mp4")
        && mp4.len() > 1
    {
        log::info!(
            " - Tips: {} has more than 1 mp4 files! {:?}",
            cache_dir_path.display(),
            mp4
        );
    }
    Ok(true)
}
