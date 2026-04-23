use crate::bms::parse::BmsInfo;

#[must_use]
pub fn get_valid_fs_name(ori_name: &str) -> String {
    ori_name
        .replace(':', "\u{ff1a}")
        .replace('\\', "\u{ff3c}")
        .replace('/', "\u{ff0f}")
        .replace('*', "\u{ff0a}")
        .replace('?', "\u{ff1f}")
        .replace('!', "\u{ff01}")
        .replace('"', "\u{ff02}")
        .replace('<', "\u{ff1c}")
        .replace('>', "\u{ff1e}")
        .replace('|', "\u{ff5c}")
}

#[must_use]
pub fn get_work_folder_name(id: &str, info: &BmsInfo) -> String {
    format!(
        "{}. {} [{}]",
        id,
        get_valid_fs_name(&info.title),
        get_valid_fs_name(&info.artist)
    )
}
