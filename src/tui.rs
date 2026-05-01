//! TUI interface using ratatui.
//!
//! Provides an interactive terminal UI with menu navigation
//! and parameter input for all toolbox functions.

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

/// Menu item for TUI display.
struct TuiMenuItem {
    /// Display number (1, 11, 21, etc. with gaps).
    number: usize,
    /// Display name.
    name: String,
}

/// Generate the menu items with Python-compatible numbering.
fn generate_menu_items() -> Vec<TuiMenuItem> {
    let mut items = Vec::new();
    let mut num = 1usize;

    items.push(TuiMenuItem {
        number: num,
        name: "BMS活动：跳转至作品信息页".into(),
    });
    num = 11;

    let bms_folder = [
        "BMS根目录：按照BMS设置文件夹名",
        "BMS根目录：按照BMS追加文件夹名",
        "BMS根目录：按照BMS追加文件夹艺术家名",
        "BMS根目录：克隆带编号的文件夹名",
        "BMS根目录：扫描相似文件夹名",
        "BMS根目录：撤销重命名",
        "BMS根目录：移除大小为0的媒体文件和临时文件",
    ];
    for name in bms_folder {
        items.push(TuiMenuItem {
            number: num,
            name: name.into(),
        });
        num += 1;
    }
    num = 21;

    let bigpack = [
        "BMS大包目录：按照首字符分成多个文件夹",
        "BMS大包目录：（撤销）按照首字符分成多个文件夹",
        "BMS大包目录：将目录A下的作品移动到目录B（自动合并）",
        "BMS大包父目录：移出一层目录（自动合并）",
        "BMS大包目录：合并文件名相似的子文件夹到目标",
        "BMS大包目录：将文件名相似的子文件夹合并到各平级目录",
        "BMS大包目录：合并被拆分的文件夹",
    ];
    for name in bigpack {
        items.push(TuiMenuItem {
            number: num,
            name: name.into(),
        });
        num += 1;
    }
    num = 31;

    let event = [
        "BMS活动目录：检查编号对应文件夹是否存在",
        "BMS活动目录：创建只带有编号的空文件夹",
        "BMS活动目录：生成活动作品xlsx表格",
    ];
    for name in event {
        items.push(TuiMenuItem {
            number: num,
            name: name.into(),
        });
        num += 1;
    }
    num = 41;

    let media = ["BMS根目录：音频文件转换", "BMS根目录：视频文件转换"];
    for name in media {
        items.push(TuiMenuItem {
            number: num,
            name: name.into(),
        });
        num += 1;
    }
    num = 51;

    let rawpack = [
        "BMS原文件：解压编号文件至根目录（自动处理文件夹嵌套）",
        "BMS原文件：解压文件至根目录（按原名）",
        "BMS原文件：赋予编号",
    ];
    for name in rawpack {
        items.push(TuiMenuItem {
            number: num,
            name: name.into(),
        });
        num += 1;
    }
    num = 61;

    let scripts = [
        "大包生成脚本：原包 -> HQ版大包",
        "大包更新脚本：原包 -> HQ版大包",
        "BMS大包脚本：原包 -> HQ版大包",
        "BMS大包脚本：HQ版大包 -> LQ版大包",
    ];
    for name in scripts {
        items.push(TuiMenuItem {
            number: num,
            name: name.into(),
        });
        num += 1;
    }

    items
}

/// Run the TUI application.
///
/// Returns `Ok(Some(number))` when user selects a menu item,
/// `Ok(None)` when user presses Esc to exit,
/// or `Err` on terminal setup failure.
pub fn run_tui() -> Result<Option<usize>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    {
        let _ = crossterm::execute!(io::stdout(), EnterAlternateScreen);
    }

    let items = generate_menu_items();
    let mut state = ListState::default();
    state.select(Some(0));
    let mut input_buffer = String::new();
    let mut status_message = String::new();
    let mut result = None;

    loop {
        let menu_items: Vec<ListItem> = items
            .iter()
            .map(|item| {
                ListItem::new(Line::from(Span::styled(
                    format!(" {}: {}", item.number, item.name),
                    Style::default(),
                )))
            })
            .collect();

        let menu = List::new(menu_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" BMS Resource Toolbox "),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">>");

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(5),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(f.area());

            f.render_stateful_widget(menu, chunks[0], &mut state);

            let input_title = if input_buffer.is_empty() {
                " 按上下键选择, Enter确认, 输入数字跳转, Esc退出 "
            } else {
                " 输入选项编号 (Enter确认, Esc退出) "
            };
            let input_widget = Paragraph::new(input_buffer.as_str())
                .block(Block::default().borders(Borders::ALL).title(input_title));
            f.render_widget(input_widget, chunks[1]);

            let status_widget = Paragraph::new(status_message.as_str())
                .block(Block::default().borders(Borders::ALL).title(" 状态 "));
            f.render_widget(status_widget, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Up => {
                        if let Some(selected) = state.selected()
                            && selected > 0 {
                                state.select(Some(selected - 1));
                            }
                    }
                    KeyCode::Down => {
                        if let Some(selected) = state.selected()
                            && selected < items.len() - 1 {
                                state.select(Some(selected + 1));
                            }
                    }
                    KeyCode::Enter => {
                        if !input_buffer.is_empty() {
                            if let Ok(num) = input_buffer.parse::<usize>()
                                && let Some(idx) = items.iter().position(|i| i.number == num) {
                                    state.select(Some(idx));
                                    input_buffer.clear();
                                    status_message = format!("选中: {}", items[idx].name);
                                    continue;
                                }
                            status_message = format!("无效编号: {input_buffer}");
                            input_buffer.clear();
                            continue;
                        }

                        if let Some(selected) = state.selected() {
                            result = Some(items[selected].number);
                            break;
                        }
                    }
                    KeyCode::Char(c) => {
                        if c.is_ascii_digit() {
                            input_buffer.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        input_buffer.pop();
                    }
                    _ => {}
                }
            }
    }

    disable_raw_mode()?;
    {
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen);
    }
    terminal.show_cursor()?;

    Ok(result)
}
