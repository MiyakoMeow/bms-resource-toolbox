//! Interactive input utilities.
//!
//! This module provides functions for handling user input
//! with history tracking for paths.

use std::any::Any;
use std::io::{self, Write};
use std::path::PathBuf;

use tokio::io::AsyncWriteExt;
use tokio::runtime::Handle;

static HISTORY_FILE: &str = "history.log";

/// Input type for interactive prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
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
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Input {
    /// The type of input to receive.
    pub input_type: InputType,
    /// Description of what the input represents.
    pub description: String,
}

impl Input {
    /// Execute the input prompt
    #[must_use]
    #[allow(dead_code)]
    pub fn exec_input(&self) -> Box<dyn Any> {
        match self.input_type {
            InputType::Any => {
                let result = input_string("Input:");
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
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

/// Read a string from stdin with a prompt.
///
/// # Panics
///
/// Panics if flushing stdout fails.
#[must_use]
pub fn input_string(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Read a path from stdin with history support.
///
/// # Panics
///
/// Panics if flushing stdout fails.
#[must_use]
pub fn input_path(_prompt: &str) -> PathBuf {
    let history = Handle::current().block_on(load_path_history());
    let mut paths = history;

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

    let mut input_str =
        input_string("直接输入路径，或输入上面的数字（索引）进行选择，输入？查看所有选项：");

    if input_str == "?" || input_str == "？" {
        if paths.is_empty() {
            println!("暂无历史路径记录");
        } else {
            println!("所有可选选项：");
            for (i, path) in paths.iter().enumerate() {
                println!("  {}: {}", i, path.display());
            }
        }
        input_str = input_string("请输入选择：");
    }

    let selected: PathBuf = if input_str.chars().all(|c| c.is_ascii_digit()) {
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
        let p = PathBuf::from(&input_str);
        paths.retain(|x| x != &p);
        paths.insert(0, p.clone());
        p
    };

    Handle::current().block_on(save_path_history(&paths));
    selected
}

/// Ask for confirmation with a prompt.
///
/// Returns `true` if the user confirms (y/Y/empty when `default_yes`),
/// `false` otherwise.
#[must_use]
pub fn input_confirm(prompt: &str, default_yes: bool) -> bool {
    let default_str = if default_yes { "[Y/n]" } else { "[y/N]" };
    let result = input_string(&format!("{prompt} {default_str}"));
    if default_yes {
        result.is_empty() || result.to_lowercase().starts_with('y')
    } else {
        result.to_lowercase().starts_with('y')
    }
}

async fn load_path_history() -> Vec<PathBuf> {
    let history_path = PathBuf::from(HISTORY_FILE);
    if !history_path.exists() {
        return Vec::new();
    }

    match tokio::fs::read_to_string(&history_path).await {
        Ok(content) => {
            let lines: Vec<PathBuf> = content
                .lines()
                .map(|s| PathBuf::from(s.trim()))
                .filter(|p| !p.as_os_str().is_empty())
                .collect();
            lines
        }
        Err(_) => Vec::new(),
    }
}

async fn save_path_history(paths: &[PathBuf]) {
    if let Ok(mut file) = tokio::fs::File::create(HISTORY_FILE).await {
        for path in paths {
            // Intentionally ignored: history file write is best-effort
            let _ = file
                .write_all(format!("{}\n", path.display()).as_bytes())
                .await;
        }
    }
}
