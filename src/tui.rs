//! TUI interface using ratatui.
//!
//! Interactive terminal UI that acts as a shell over the CLI subcommands.
//! User selects a menu item → constructs the matching [`Commands`] → calls [`dispatch`].

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

use crate::cli::{self, Commands};

// ---------------------------------------------------------------------------
// Menu definition — one item per CLI subcommand, grouped with 10-gap numbering
// ---------------------------------------------------------------------------

/// Single entry in the TUI menu.
struct MenuItem {
    number: usize,
    name: &'static str,
    make_cmd: fn() -> Commands,
}

/// Build the full menu list.
#[allow(clippy::too_many_lines)]
fn build_menu() -> Vec<MenuItem> {
    // Helper macros to reduce repetition
    macro_rules! p {
        ($num:expr, $name:expr, $variant:ident { $($field:ident),* $(,)? }) => {
            MenuItem {
                number: $num,
                name: $name,
                make_cmd: || Commands::$variant { $($field: Default::default()),* },
            }
        };
        ($num:expr, $name:expr, $variant:ident) => {
            MenuItem {
                number: $num,
                name: $name,
                make_cmd: || Commands::$variant,
            }
        };
    }

    let mut items = Vec::new();
    let mut n = 1usize;

    // ── BMS活动 ────────────────────────────────────────────
    items.push(p!(n, "BMS活动：跳转至作品信息页", JumpToWorkInfo));
    n = 11;

    // ── BMS根目录 ──────────────────────────────────────────
    items.push(p!(
        n,
        "BMS根目录：按照BMS设置文件夹名",
        SetNameByBms { path }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS根目录：按照BMS追加文件夹名",
        AppendNameByBms { path }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS根目录：按照BMS追加文件夹艺术家名",
        AppendArtistNameByBms { path }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS根目录：克隆带编号的文件夹名",
        CopyNumberedWorkdirNames { from, to }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS根目录：扫描相似文件夹名",
        ScanFolderSimilarFolders { path }
    ));
    n += 1;
    items.push(p!(n, "BMS根目录：撤销重命名", UndoSetName { path }));
    n += 1;
    items.push(p!(
        n,
        "BMS根目录：移除大小为0的媒体文件和临时文件",
        RemoveZeroSizedMediaFiles { path }
    ));
    n = 21;

    // ── BMS大包目录 ────────────────────────────────────────
    items.push(p!(
        n,
        "BMS大包目录：按照首字符分成多个文件夹",
        SplitFoldersWithFirstChar { path }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS大包目录：（撤销）按照首字符分成多个文件夹",
        UndoSplitPack { path }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS大包目录：将目录A下的作品移动到目录B（自动合并）",
        MoveWorksInPack { from, to }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS大包父目录：移出一层目录（自动合并）",
        MoveOutWorks { path }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS大包目录：合并文件名相似的子文件夹到目标",
        MoveWorksWithSameName { from, to }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS大包目录：将文件名相似的子文件夹合并到各平级目录",
        MoveWorksWithSameNameToSiblings { path }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS大包目录：合并被拆分的文件夹",
        MergeSplitFolders { path }
    ));
    n = 31;

    // ── BMS活动目录 ────────────────────────────────────────
    items.push(p!(
        n,
        "BMS活动目录：检查编号对应文件夹是否存在",
        CheckNumFolder { path, count }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS活动目录：创建只带有编号的空文件夹",
        CreateNumFolders { path, count }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS活动目录：生成活动作品xlsx表格",
        GenerateWorkInfoTable { path }
    ));
    n = 41;

    // ── BMS媒体 ────────────────────────────────────────────
    items.push(p!(n, "BMS根目录：音频文件转换", TransferAudio { path }));
    n += 1;
    items.push(p!(n, "BMS根目录：视频文件转换", TransferVideo { path }));
    n = 51;

    // ── BMS原文件 ──────────────────────────────────────────
    items.push(p!(
        n,
        "BMS原文件：解压编号文件至根目录（自动处理文件夹嵌套）",
        UnzipNumericToBmsFolder { pack, cache, root }
    ));
    n += 1;
    items.push(p!(
        n,
        "BMS原文件：解压文件至根目录（按原名）",
        UnzipWithNameToBmsFolder { pack, cache, root }
    ));
    n += 1;
    items.push(p!(n, "BMS原文件：赋予编号", SetFileNum { path }));
    n = 61;

    // ── 大包脚本 ───────────────────────────────────────────
    items.push(p!(
        n,
        "大包生成脚本：原包 -> HQ版大包",
        PackSetupRawpackToHq { pack, root }
    ));
    n += 1;
    items.push(p!(
        n,
        "大包更新脚本：原包 -> HQ版大包",
        PackUpdateRawpackToHq { pack, root, sync }
    ));
    n += 1;
    items.push(p!(n, "BMS大包脚本：原包 -> HQ版大包", PackRawToHq { path }));
    n += 1;
    items.push(p!(
        n,
        "BMS大包脚本：HQ版大包 -> LQ版大包",
        PackHqToLq { path }
    ));

    items
}

// ---------------------------------------------------------------------------
// Interactive parameter input (inside TUI)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Main TUI loop
// ---------------------------------------------------------------------------

/// Launch the TUI.
///
/// Returns `Ok(())` on normal exit (Esc or after executing a command).
///
/// # Errors
///
/// Returns terminal/crossterm errors if alternate screen, raw mode, or rendering fails.
#[allow(clippy::too_many_lines)]
pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let _ = crossterm::execute!(io::stdout(), EnterAlternateScreen);

    let menu = build_menu();
    let mut state = ListState::default();
    state.select(Some(0));
    let mut input_buf = String::new();
    let mut status = String::new();

    let result = 'outer: loop {
        let items: Vec<ListItem> = menu
            .iter()
            .map(|m| {
                ListItem::new(Line::from(Span::styled(
                    format!(" {}: {}", m.number, m.name),
                    Style::default(),
                )))
            })
            .collect();

        let list = List::new(items)
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

            f.render_stateful_widget(list, chunks[0], &mut state);

            let title = if input_buf.is_empty() {
                " ↑↓ 选择 · Enter 确认 · 数字跳转 · Esc 退出 "
            } else {
                " 输入编号 (Enter确认 · Esc取消) "
            };
            f.render_widget(
                Paragraph::new(input_buf.as_str())
                    .block(Block::default().borders(Borders::ALL).title(title)),
                chunks[1],
            );
            f.render_widget(
                Paragraph::new(status.as_str())
                    .block(Block::default().borders(Borders::ALL).title(" 状态 ")),
                chunks[2],
            );
        })?;

        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Esc => break 'outer Ok(()),

            KeyCode::Up => {
                if let Some(s) = state.selected()
                    && s > 0
                {
                    state.select(Some(s - 1));
                }
            }

            KeyCode::Down => {
                if let Some(s) = state.selected()
                    && s < menu.len() - 1
                {
                    state.select(Some(s + 1));
                }
            }

            KeyCode::Enter => {
                // If typing a number, jump to that item
                if !input_buf.is_empty() {
                    if let Ok(num) = input_buf.parse::<usize>()
                        && let Some(idx) = menu.iter().position(|m| m.number == num)
                    {
                        state.select(Some(idx));
                        status.clear();
                    } else {
                        status = format!("无效编号: {input_buf}");
                    }
                    input_buf.clear();
                    continue;
                }

                // Execute selected command
                let Some(selected) = state.selected() else {
                    continue;
                };
                let item = &menu[selected];
                let cmd = (item.make_cmd)();

                // Tear down TUI before running the command (it may need stdin/stdout)
                disable_raw_mode()?;
                let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen);
                terminal.show_cursor()?;

                println!("\n▶ {}\n", item.name);
                cli::dispatch(&cmd);
                println!("\n✓ 完成。按 Enter 返回菜单…");
                {
                    let mut dummy = String::new();
                    let _ = io::stdin().read_line(&mut dummy);
                }

                // Re-establish TUI
                enable_raw_mode()?;
                let _ = crossterm::execute!(io::stdout(), EnterAlternateScreen);
                // hide cursor again
                let _ = terminal.draw(|f| {
                    f.render_widget(Paragraph::new("").block(Block::default()), f.area());
                });
                status = format!("上次执行: {}", item.name);
            }

            KeyCode::Char(c) if c.is_ascii_digit() => {
                input_buf.push(c);
            }

            KeyCode::Backspace => {
                input_buf.pop();
            }

            _ => {}
        }
    };

    // Ensure terminal is restored on every exit path
    disable_raw_mode()?;
    let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen);
    terminal.show_cursor()?;

    result
}
