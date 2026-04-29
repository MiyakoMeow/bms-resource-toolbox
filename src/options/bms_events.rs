//! BMS event utilities.
//!
//! This module provides utilities for BMS events like BOFTT.

use tracing::info;

/// BMS event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code, clippy::upper_case_acronyms)]
pub enum BMSEvent {
    /// BOFTT event (BOF2020)
    BOFTT = 20,
    /// BOF21 event
    BOF21 = 21,
    /// Let's BMS Edit 3
    LetsBMSEdit3 = 103,
}

#[allow(dead_code)]
impl BMSEvent {
    /// Get the list URL for this event
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[must_use]
    pub fn list_url(&self) -> String {
        match self {
            BMSEvent::BOFTT => "https://vibrato.cn/bms/list-20.html".to_string(),
            BMSEvent::BOF21 => "https://vibrato.cn/bms/list-21.html".to_string(),
            BMSEvent::LetsBMSEdit3 => "https://vibrato.cn/bms/list-103.html".to_string(),
        }
    }

    /// Get the work info URL for a specific work number
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[must_use]
    pub fn work_info_url(&self, work_num: i32) -> String {
        match self {
            BMSEvent::BOFTT => format!("https://vibrato.cn/bms/view-20-{work_num}.html"),
            BMSEvent::BOF21 => format!("https://vibrato.cn/bms/view-21-{work_num}.html"),
            BMSEvent::LetsBMSEdit3 => format!("https://vibrato.cn/bms/view-103-{work_num}.html"),
        }
    }
}

/// Jump to work info page for a BMS event
///
/// This opens the web browser to the event's list page
#[allow(dead_code)]
#[allow(clippy::missing_panics_doc)]
pub fn jump_to_work_info() {
    use std::io::{self, Write};

    info!("BMS Event List:");
    info!("1. BOFTT (BOF2020)");
    info!("2. BOF21");
    info!("3. Let's BMS Edit 3");

    print!("Select event (1-3): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    let event = match input {
        "1" => BMSEvent::BOFTT,
        "2" => BMSEvent::BOF21,
        "3" => BMSEvent::LetsBMSEdit3,
        _ => {
            info!("Invalid selection, defaulting to BOFTT");
            BMSEvent::BOFTT
        }
    };

    // Open list page in browser
    let url = event.list_url();
    info!("Opening: {}", url);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .ok();
    }

    // Prompt for work number
    print!("Input work number: ");
    io::stdout().flush().unwrap();

    let mut work_input = String::new();
    io::stdin().read_line(&mut work_input).unwrap();
    let work_input = work_input.trim();

    if let Ok(work_num) = work_input.parse::<i32>() {
        let work_url = event.work_info_url(work_num);
        info!("Opening work: {}", work_url);

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", &work_url])
                .spawn()
                .ok();
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&work_url)
                .spawn()
                .ok();
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&work_url)
                .spawn()
                .ok();
        }
    } else {
        info!("Invalid work number");
    }
}
