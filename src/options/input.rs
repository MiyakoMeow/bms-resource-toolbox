//! Interactive input utilities.
//!
//! This module provides functions for handling user input
//! with history tracking for paths.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

#[allow(dead_code)]
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

#[allow(dead_code)]
const HISTORY_FILE: &str = "input_history.log";

/// Read input with path history support
#[must_use]
#[allow(dead_code)]
pub fn input_path(prompt: &str) -> PathBuf {
    // Load history
    let history = load_path_history();
    let mut paths = history;

    // Show history if available
    if !paths.is_empty() {
        println!("输入路径开始。以下是之前使用过的路径：");
        let display: Vec<_> = paths.iter().take(5).collect();
        for (i, path) in display.iter().enumerate() {
            println!(" -> {}: {}", i, path.display());
        }
        if paths.len() > 5 {
            println!("（还有 {} 个历史路径，输入？查看全部）", paths.len() - 5);
        }
    }

    // Get user input
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let mut input_str = input_line.trim();

    // Handle help command
    if input_str == "?" || input_str == "？" {
        if paths.is_empty() {
            println!("暂无历史路径记录");
        } else {
            println!("所有可选选项：");
            for (i, path) in paths.iter().enumerate() {
                println!("  {}: {}", i, path.display());
            }
        }
        print!("请输入选择：");
        io::stdout().flush().unwrap();
        input_line.clear();
        io::stdin().read_line(&mut input_line).unwrap();
        input_str = input_line.trim();
    }

    let selected: PathBuf = if input_str.chars().all(|c| c.is_ascii_digit()) {
        // Numeric selection
        if let Ok(idx) = input_str.parse::<usize>() {
            if idx < paths.len() {
                let selected = paths.remove(idx);
                paths.insert(0, selected.clone());
                selected
            } else {
                PathBuf::from(input_str)
            }
        } else {
            PathBuf::from(input_str)
        }
    } else {
        // Direct path input
        let p = PathBuf::from(input_str);
        // Remove if exists and add to front
        paths.retain(|x| x != &p);
        paths.insert(0, p.clone());
        p
    };

    // Save history
    save_path_history(&paths);

    selected
}

/// Load path history from file
#[allow(dead_code)]
fn load_path_history() -> Vec<PathBuf> {
    let history_path = PathBuf::from(HISTORY_FILE);
    if !history_path.exists() {
        return Vec::new();
    }

    let file = File::open(&history_path).ok();
    match file {
        Some(f) => {
            let reader = BufReader::new(f);
            reader
                .lines()
                .map_while(std::result::Result::ok)
                .map(|s| PathBuf::from(s.trim()))
                .filter(|p| !p.as_os_str().is_empty())
                .collect()
        }
        None => Vec::new(),
    }
}

/// Save path history to file
#[allow(dead_code)]
fn save_path_history(paths: &[PathBuf]) {
    if let Ok(mut file) = File::create(HISTORY_FILE) {
        for path in paths {
            let _ = writeln!(file, "{}", path.display());
        }
    }
}

/// Get string input
#[must_use]
#[allow(dead_code)]
pub fn input_string(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Get yes/no confirmation
#[must_use]
#[allow(dead_code)]
pub fn input_confirm(prompt: &str, default: bool) -> bool {
    let default_str = if default { "[Y/n]" } else { "[y/N]" };
    print!("{prompt} {default_str}: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();
    input.is_empty() || input == "y" || input == "yes"
}
