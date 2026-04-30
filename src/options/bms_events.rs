//! BMS event utilities.
//!
//! This module provides utilities for BMS events like BOFTT.

use std::any::Any;

use super::input::input_string;
use webbrowser;

/// BMS event types for work information pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BMSEvent {
    /// BOF Team Festival.
    #[allow(clippy::upper_case_acronyms)]
    BOFTT = 20,
    /// BOF 2021.
    BOF21 = 21,
    /// `LetsBMS` Edit 3.
    LetsBMSEdit3 = 103,
}

impl BMSEvent {
    fn from_value(val: i32) -> Self {
        match val {
            21 => BMSEvent::BOF21,
            103 => BMSEvent::LetsBMSEdit3,
            _ => BMSEvent::BOFTT,
        }
    }

    fn list_url(self) -> &'static str {
        match self {
            BMSEvent::BOFTT => "https://manbow.nothing.sh/event/event.cgi?action=sp&event=146",
            BMSEvent::BOF21 => "https://manbow.nothing.sh/event/event.cgi?action=sp&event=149",
            BMSEvent::LetsBMSEdit3 => "https://venue.bmssearch.net/letsbmsedit3",
        }
    }

    fn work_info_url(self, work_num: i32) -> String {
        match self {
            BMSEvent::BOFTT => format!(
                "https://manbow.nothing.sh/event/event.cgi?action=More_def&num={work_num}&event=146"
            ),
            BMSEvent::BOF21 => format!(
                "https://manbow.nothing.sh/event/event.cgi?action=More_def&num={work_num}&event=149"
            ),
            BMSEvent::LetsBMSEdit3 => {
                format!("https://venue.bmssearch.net/letsbmsedit3/{work_num}")
            }
        }
    }
}

/// Jump to work info page for a BMS event.
pub fn jump_to_work_info(_args: &[Box<dyn Any>]) {
    println!("Select BMS Event:");
    for event in &[BMSEvent::BOFTT, BMSEvent::BOF21, BMSEvent::LetsBMSEdit3] {
        println!(" {} -> {:?}", *event as i32, event);
    }

    let default_event = BMSEvent::BOFTT as i32;
    let event_val_str = input_string(&format!(
        "Input event value (Default: BOFTT {default_event}):"
    ));
    let event_val = if event_val_str.is_empty() {
        default_event
    } else {
        event_val_str.parse::<i32>().unwrap_or(default_event)
    };
    let event = BMSEvent::from_value(event_val);
    println!(" -> Selected Event: {event:?}");

    println!(" !: Input \"1\": jump to work id 1. (Normal)");
    println!(" !: Input \"2 5\": jump to work id 2, 3, 4 and 5. (Special: Range)");
    println!(" !: Input \"2 5 6\": jump to work id 2, 5 and 6. (Normal)");
    println!(" !: Press Ctrl+C to Quit.");
    let tips = "Input id (default: Jump to List):";

    loop {
        let num_str = input_string(tips).trim().replace(['[', ']'], "");

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

/// Open URL in browser.
pub fn open_url(url: &str) {
    let _ = webbrowser::open(url);
}
