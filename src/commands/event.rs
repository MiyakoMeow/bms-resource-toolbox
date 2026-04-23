use std::path::Path;

use crate::cli::EventCommand;

pub async fn handle(cmd: EventCommand) -> crate::Result<()> {
    match cmd {
        EventCommand::CheckNum { root_dir, count } => check_num_folder(&root_dir, count),
        EventCommand::CreateNum { root_dir, count } => create_num_folders(&root_dir, count),
    }
    Ok(())
}

fn check_num_folder(root_dir: &Path, count: u32) {
    for n in 1..=count {
        let dir = root_dir.join(n.to_string());
        if !dir.is_dir() {
            println!("Not found: {}", dir.display());
        }
    }
}

fn create_num_folders(root_dir: &Path, count: u32) {
    let existing = get_existing_subdir_names(root_dir);
    for n in 1..=count {
        let n_str = n.to_string();
        let exists = existing.iter().any(|name| {
            name == &n_str
                || name.starts_with(&format!("{n_str}."))
                || name.starts_with(&format!("{n_str} "))
        });
        if !exists {
            let dir = root_dir.join(&n_str);
            println!("Creating: {}", dir.display());
            if let Err(e) = std::fs::create_dir(&dir) {
                eprintln!("Error creating {}: {e}", dir.display());
            }
        }
    }
}

fn get_existing_subdir_names(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect()
}
