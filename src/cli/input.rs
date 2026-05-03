//! Interactive input utilities.
//!
//! This module provides functions for handling user input
//! via stdin/stdout.

use std::io::{self, Write};

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
