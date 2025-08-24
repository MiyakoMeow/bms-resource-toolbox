use std::fmt;
use std::io::{self, Write};
use std::str::FromStr;

use smol::process::Command;

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum BMSEvent {
    BOFTT = 20,
    LetsBMSEdit3 = 103,
}

impl BMSEvent {
    /// Event list page
    pub fn list_url(&self) -> &'static str {
        match self {
            BMSEvent::BOFTT => "https://manbow.nothing.sh/event/event.cgi?action=sp&event=146",
            BMSEvent::LetsBMSEdit3 => "https://venue.bmssearch.net/letsbmsedit3",
        }
    }

    /// Event work details page
    pub fn work_info_url(&self, work_num: u32) -> String {
        match self {
            BMSEvent::BOFTT => format!(
                "https://manbow.nothing.sh/event/event.cgi?action=More_def&num={work_num}&event=146",
            ),
            BMSEvent::LetsBMSEdit3 => {
                format!("https://venue.bmssearch.net/letsbmsedit3/{work_num}")
            }
        }
    }
}

impl fmt::Display for BMSEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BMSEvent::BOFTT => "BOFTT",
                BMSEvent::LetsBMSEdit3 => "LetsBMSEdit3",
            }
        )
    }
}

impl FromStr for BMSEvent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "20" | "BOFTT" => Ok(BMSEvent::BOFTT),
            "103" | "LetsBMSEdit3" => Ok(BMSEvent::LetsBMSEdit3),
            _ => Err(()),
        }
    }
}

/// Open default browser
pub async fn open_browser(url: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let _cmd = Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(url)
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }
    Ok(())
}

pub async fn activate() {
    log::info!("Select BMS Event:");
    for event in [BMSEvent::BOFTT, BMSEvent::LetsBMSEdit3] {
        log::info!(" {} -> {}", event as u32, event);
    }

    log::info!("Input event value (Default: 20): ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let event = if buf.trim().is_empty() {
        BMSEvent::BOFTT
    } else {
        BMSEvent::from_str(&buf).unwrap_or(BMSEvent::BOFTT)
    };
    log::info!(" -> Selected Event: {event}");

    log::info!(" !: Input \"1\": jump to work id 1. (Normal)");
    log::info!(" !: Input \"2 5\": jump to work id 2, 3, 4 and 5. (Special: Range)");
    log::info!(" !: Input \"2 5 6\": jump to work id 2, 5 and 6. (Normal)");
    log::info!(" !: Press Ctrl+C to Quit.");
    log::info!("Input id (default: Jump to List):");

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        let line = line.trim();
        if line.is_empty() {
            log::info!("Open BMS List.");
            open_browser(event.list_url()).await.ok();
            continue;
        }

        // Parse input
        let parts: Vec<&str> = line
            .split_whitespace()
            .flat_map(|s| s.split(','))
            .filter(|s| !s.is_empty())
            .collect();

        let nums: Result<Vec<u32>, _> = parts.iter().map(|s| s.parse::<u32>()).collect();

        let Ok(nums) = nums else {
            log::info!("Please input valid number.");
            return;
        };
        match nums.len() {
            0 => continue,
            1 => {
                let id = nums[0];
                log::info!("Open no.{id}");
                open_browser(&event.work_info_url(id)).await.ok();
            }
            2 => {
                let (mut start, mut end) = (nums[0], nums[1]);
                if start > end {
                    std::mem::swap(&mut start, &mut end);
                }
                for id in start..=end {
                    open_browser(&event.work_info_url(id)).await.ok();
                }
            }
            _ => {
                for &id in &nums {
                    open_browser(&event.work_info_url(id)).await.ok();
                }
            }
        }
    }
}
