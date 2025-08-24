use std::path::{Path, PathBuf};

use bms_rs::parse::model::Bms;
use futures::future::try_join_all;
use smol::{fs, io, stream::StreamExt};
use xlsxwriter::{Workbook, XlsxError};

use crate::bms::get_dir_bms_info;

/// 1. Check if pure numeric folders from 1..=max are missing
pub async fn check_num_folder(root: &Path, max: usize) -> io::Result<Vec<PathBuf>> {
    let mut missing = Vec::new();
    for id in 1..=max {
        let path = root.join(id.to_string());
        if !fs::metadata(&path).await.is_ok_and(|m| m.is_dir()) {
            missing.push(path);
        }
    }
    Ok(missing)
}

/// 2. Create new folders if pure numeric folders don't exist
pub async fn create_num_folders(root: &Path, count: usize) -> io::Result<()> {
    let mut futs = Vec::new();
    for id in 1..=count {
        let path = root.join(id.to_string());
        futs.push(smol::spawn(async move { fs::create_dir_all(&path).await }));
    }
    try_join_all(futs).await?;
    Ok(())
}

/// 3. Scan all numeric folders under root directory and write to bms_list.xlsx
pub async fn generate_work_info_table(root: &Path) -> io::Result<()> {
    // First collect all numeric folders
    let mut dir_ids = Vec::new();
    let mut entries = fs::read_dir(root).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        if entry.file_type().await?.is_dir()
            && let Ok(id) = entry.file_name().into_string().unwrap().parse::<u32>()
        {
            dir_ids.push((id, entry.path()));
        }
    }
    dir_ids.sort_unstable_by_key(|(id, _)| *id);

    // Read info.toml in parallel
    let info_futs: Vec<_> = dir_ids
        .iter()
        .map(|(id, path)| {
            let id = *id;
            let path = path.clone();
            smol::spawn(async move {
                let info = get_dir_bms_info(&path).await?;
                Ok::<(u32, Option<Bms>), io::Error>((id, info))
            })
        })
        .collect();
    let infos: Vec<_> = try_join_all(info_futs).await?;

    // Write Excel
    async {
        let xlsx_path = root.join("bms_list.xlsx");
        let workbook = Workbook::new(&xlsx_path.to_string_lossy())?;
        let mut sheet = workbook.add_worksheet(Some("BMS List"))?;

        // Write headers
        sheet.write_string(0, 0, "ID", None)?;
        sheet.write_string(0, 1, "Title", None)?;
        sheet.write_string(0, 2, "Artist", None)?;
        sheet.write_string(0, 3, "Genre", None)?;

        for (row, (id, info)) in infos.into_iter().enumerate() {
            let Some(info) = info else { continue };
            let row = (row + 1) as u32;
            sheet.write_number(row, 0, id as f64, None)?;
            sheet.write_string(row, 1, &info.header.title.unwrap_or_default(), None)?;
            sheet.write_string(row, 2, &info.header.artist.unwrap_or_default(), None)?;
            sheet.write_string(row, 3, &info.header.genre.unwrap_or_default(), None)?;
        }

        workbook.close()?;
        log::info!("Saved {}", xlsx_path.display());
        Ok::<(), XlsxError>(())
    }
    .await
    .map_err(|e| io::Error::other(e.to_string()))?;
    Ok(())
}
