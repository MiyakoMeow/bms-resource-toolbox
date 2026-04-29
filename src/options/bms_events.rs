//! BMS event utilities.
//!
//! This module provides utilities for BMS events like BOFTT.

/// BMS event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BMSEvent {
    /// BOFTT event (BOF2020)
    Boftt,
    /// BOF21 event
    Bo21,
    /// Let's BMS Edit 3
    LetsBmsEdit3,
}

impl BMSEvent {
    /// Get the list URL for this event
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn list_url(self) -> &'static str {
        match self {
            BMSEvent::Boftt => "https://manbow.nothing.sh/event/event.cgi?action=sp&event=146",
            BMSEvent::Bo21 => "https://manbow.nothing.sh/event/event.cgi?action=sp&event=149",
            BMSEvent::LetsBmsEdit3 => "https://venue.bmssearch.net/letsbmsedit3",
        }
    }

    /// Get the work info URL for a specific work number
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn work_info_url(self, work_num: i32) -> String {
        match self {
            BMSEvent::Boftt => {
                format!(
                    "https://manbow.nothing.sh/event/event.cgi?action=More_def&num={work_num}&event=146"
                )
            }
            BMSEvent::Bo21 => {
                format!(
                    "https://manbow.nothing.sh/event/event.cgi?action=More_def&num={work_num}&event=149"
                )
            }
            BMSEvent::LetsBmsEdit3 => {
                format!("https://venue.bmssearch.net/letsbmsedit3/{work_num}")
            }
        }
    }

    /// Create `BMSEvent` from value, defaulting to BOFTT
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn from_value_or_default(val: i32) -> Self {
        match val {
            21 => BMSEvent::Bo21,
            103 => BMSEvent::LetsBmsEdit3,
            _ => BMSEvent::Boftt,
        }
    }
}

/// Jump to work info page for a BMS event
///
/// This replicates Python's `jump_to_work_info()`:
/// - Displays available events with their values
/// - Prompts for event value (default: BOFTT = 20)
/// - Prompts for work ID(s) or range, or empty to open list page
/// - Opens URLs in browser using xdg-open on Linux
///
/// # Panics
///
/// Panics if stdout flush or stdin read fails.
#[allow(dead_code)]
pub fn jump_to_work_info() {
    use std::io::{self, Write};

    println!("Select BMS Event:");
    for event in &[BMSEvent::Boftt, BMSEvent::Bo21, BMSEvent::LetsBmsEdit3] {
        println!(" {} -> {:?}", *event as i32, event);
    }

    let default_event_val = BMSEvent::Boftt as i32;
    print!("Input event value (Default: BOFTT): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    let event_val = if input.is_empty() {
        default_event_val
    } else {
        input.parse().unwrap_or(default_event_val)
    };
    let event = BMSEvent::from_value_or_default(event_val);
    println!(" -> Selected Event: {event:?}");

    println!(" !: Input \"1\": jump to work id 1. (Normal)");
    println!(" !: Input \"2 5\": jump to work id 2, 3, 4 and 5. (Special: Range)");
    println!(" !: Input \"2 5 6\": jump to work id 2, 5 and 6. (Normal)");
    println!(" !: Press Ctrl+C to Quit.");
    let tips = "Input id (default: Jump to List):";

    loop {
        print!("{tips} ");
        io::stdout().flush().unwrap();

        let mut num_str = String::new();
        io::stdin().read_line(&mut num_str).unwrap();
        let num_str = num_str.trim().replace(['[', ']'], "");

        let mut nums: Vec<i32> = Vec::new();
        for token in num_str.replace(',', " ").split_whitespace() {
            if !token.is_empty()
                && let Ok(n) = token.parse::<i32>()
            {
                nums.push(n);
            }
        }

        if nums.len() > 2 {
            for num_val in &nums {
                open_url(&event.work_info_url(*num_val));
            }
        } else if nums.len() == 2 {
            let (start, end) = (nums[0], nums[1]);
            let (start, end) = if start > end {
                (end, start)
            } else {
                (start, end)
            };
            for id in start..=end {
                open_url(&event.work_info_url(id));
            }
        } else if !num_str.is_empty() && num_str.chars().all(|c| c.is_ascii_digit()) {
            println!("Open no.{num_str}");
            let id = num_str.parse::<i32>().unwrap_or(1);
            open_url(&event.work_info_url(id));
        } else if !num_str.is_empty() {
            println!("Please input vaild number.");
        } else {
            println!("Open BMS List.");
            open_url(event.list_url());
        }
    }
}

/// Open URL in browser
#[allow(dead_code)]
fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn().ok();
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn().ok();
    }
}
