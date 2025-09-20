use std::fmt;
use std::io::{self, Write};
use std::str::FromStr;

use smol::process::Command;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[repr(u32)]
pub enum BMSEvent {
    BOFNT = 19,
    BOFTT = 20,
    LetsBMSEdit = 101,
    LetsBMSEdit2 = 102,
    LetsBMSEdit3 = 103,
    LetsBMSEdit4 = 104,
}

impl BMSEvent {
    /// Event list page
    pub fn list_url(&self) -> &'static str {
        match self {
            BMSEvent::BOFNT => "https://manbow.nothing.sh/event/event.cgi?action=sp&event=142",
            BMSEvent::BOFTT => "https://manbow.nothing.sh/event/event.cgi?action=sp&event=146",
            BMSEvent::LetsBMSEdit => "https://venue.bmssearch.net/letsbmsedit",
            BMSEvent::LetsBMSEdit2 => "https://venue.bmssearch.net/letsbmsedit2",
            BMSEvent::LetsBMSEdit3 => "https://venue.bmssearch.net/letsbmsedit3",
            BMSEvent::LetsBMSEdit4 => "https://venue.bmssearch.net/letsbmsedit4",
        }
    }

    /// Event work details page
    pub fn work_info_url(&self, work_num: u32) -> String {
        match self {
            BMSEvent::BOFNT => format!(
                "https://manbow.nothing.sh/event/event.cgi?action=More_def&num={work_num}&event=142",
            ),
            BMSEvent::BOFTT => format!(
                "https://manbow.nothing.sh/event/event.cgi?action=More_def&num={work_num}&event=146",
            ),
            BMSEvent::LetsBMSEdit => {
                format!("https://venue.bmssearch.net/letsbmsedit/{work_num}")
            }
            BMSEvent::LetsBMSEdit2 => {
                format!("https://venue.bmssearch.net/letsbmsedit2/{work_num}")
            }
            BMSEvent::LetsBMSEdit3 => {
                format!("https://venue.bmssearch.net/letsbmsedit3/{work_num}")
            }
            BMSEvent::LetsBMSEdit4 => {
                format!("https://venue.bmssearch.net/letsbmsedit4/{work_num}")
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
                BMSEvent::BOFNT => "BOFNT",
                BMSEvent::BOFTT => "BOFTT",
                BMSEvent::LetsBMSEdit => "LetsBMSEdit",
                BMSEvent::LetsBMSEdit2 => "LetsBMSEdit2",
                BMSEvent::LetsBMSEdit3 => "LetsBMSEdit3",
                BMSEvent::LetsBMSEdit4 => "LetsBMSEdit4",
            }
        )
    }
}

impl FromStr for BMSEvent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "19" | "BOFNT" => Ok(BMSEvent::BOFNT),
            "20" | "BOFTT" => Ok(BMSEvent::BOFTT),
            "101" | "LetsBMSEdit" => Ok(BMSEvent::LetsBMSEdit),
            "102" | "LetsBMSEdit2" => Ok(BMSEvent::LetsBMSEdit2),
            "103" | "LetsBMSEdit3" => Ok(BMSEvent::LetsBMSEdit3),
            "104" | "LetsBMSEdit4" => Ok(BMSEvent::LetsBMSEdit4),
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

/// Open BMS event list page
pub async fn open_event_list(event: BMSEvent) -> io::Result<()> {
    log::info!("Opening BMS event list: {}", event);
    open_browser(event.list_url()).await
}

/// Open multiple BMS event work details pages
pub async fn open_event_works(event: BMSEvent, work_ids: &[u32]) -> io::Result<()> {
    log::info!("Opening BMS event works: {} (IDs: {:?})", event, work_ids);
    for &work_id in work_ids {
        let url = event.work_info_url(work_id);
        open_browser(&url).await?;
    }
    Ok(())
}
