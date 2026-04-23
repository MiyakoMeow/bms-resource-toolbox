use std::path::Path;

use crate::bms::CHART_FILE_EXTS;

pub fn check_exec(cmd: &str, args: &str, name: &str) -> bool {
    if let Ok(output) = std::process::Command::new(cmd).args(args.split_whitespace()).output() { output.status.success() } else {
        tracing::warn!("{name} not found");
        false
    }
}

#[must_use] 
pub fn check_ffmpeg_exec() -> bool {
    check_exec("ffmpeg", "-version", "ffmpeg")
}

#[must_use] 
pub fn check_flac_exec() -> bool {
    check_exec("flac", "--version", "flac")
}

#[must_use] 
pub fn check_oggenc_exec() -> bool {
    check_exec("oggenc", "-v", "oggenc")
}

#[must_use] 
pub fn is_root_dir(dirs: &[&Path]) -> bool {
    dirs.iter().all(|dir| {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type()
                    && file_type.is_file()
                        && let Some(ext) = entry.path().extension().and_then(|e| e.to_str())
                            && CHART_FILE_EXTS.contains(&ext.to_lowercase().as_str()) {
                                return false;
                            }
            }
        }
        true
    })
}

#[must_use] 
pub fn is_work_dir(dirs: &[&Path]) -> bool {
    !is_root_dir(dirs)
}

#[must_use] 
pub fn is_not_a_dir(dir: &Path) -> bool {
    !dir.is_dir()
}

#[must_use] 
pub fn str_similarity(a: &str, b: &str) -> f64 {
    strsim::normalized_levenshtein(a, b)
}
