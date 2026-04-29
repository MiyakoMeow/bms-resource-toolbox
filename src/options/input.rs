//! Interactive input utilities.
//!
//! This module provides functions for handling user input
//! with history tracking for paths.

use std::any::Any;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

const HISTORY_FILE: &str = "input_history.log";

/// Input type for interactive prompts.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputType {
    /// Any string input.
    #[default]
    Any,
    /// A single word without spaces.
    Word,
    /// An integer input.
    Int,
    /// A file system path with history.
    Path,
}

/// Confirmation type for interactive prompts.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfirmType {
    /// No confirmation required.
    NoConfirm,
    /// Default to yes (confirm unless explicitly declined).
    #[default]
    DefaultYes,
    /// Default to no (decline unless explicitly confirmed).
    DefaultNo,
}

/// Input specification for interactive prompts.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Input {
    /// The type of input to receive.
    pub input_type: InputType,
    /// Description of what the input represents.
    pub description: String,
}

impl Input {
    /// Execute the input prompt
    #[allow(dead_code)]
    #[must_use]
    pub fn exec_input(&self) -> Box<dyn Any> {
        match self.input_type {
            InputType::Any => {
                let result = input_string("Input: ");
                Box::new(result)
            }
            InputType::Word => {
                let tips = "Input a word: ";
                let mut w_str = input_string(tips);
                while w_str.contains(' ') {
                    println!("Requires a word. Re-input.");
                    w_str = input_string(tips);
                }
                Box::new(w_str)
            }
            InputType::Int => {
                let tips = "Input a number: ";
                let mut w_str = input_string(tips);
                while !w_str.chars().all(|c| c.is_ascii_digit()) {
                    println!("Requires a number. Re-input.");
                    w_str = input_string(tips);
                }
                Box::new(w_str.parse::<i32>().unwrap_or(0))
            }
            InputType::Path => {
                let result = input_path("");
                Box::new(result)
            }
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            input_type: InputType::Any,
            description: String::new(),
        }
    }
}

/// Option for interactive menu.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CliOption<T: FnMut(Box<dyn Any>)> {
    /// Function to execute when option is selected.
    pub func: T,
    /// Display name of the option.
    pub name: String,
    /// Input specifications for this option.
    pub inputs: Vec<Input>,
    /// Confirmation type before execution.
    pub confirm: ConfirmType,
}









/// Read input with path history support.
///
/// # Panics
///
/// Panics if flushing stdout fails.
#[allow(dead_code)]
#[must_use]
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

/// Load path history from file.
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

/// Save path history to file.
#[allow(dead_code)]
fn save_path_history(paths: &[PathBuf]) {
    if let Ok(mut file) = File::create(HISTORY_FILE) {
        for path in paths {
            let _ = writeln!(file, "{}", path.display());
        }
    }
}

/// Get string input.
///
/// # Panics
///
/// Panics if flushing stdout fails.
#[allow(dead_code)]
#[must_use]
pub fn input_string(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}


