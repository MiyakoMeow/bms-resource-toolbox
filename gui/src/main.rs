// GUI mode does not directly use library entry
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf as StdPathBuf,
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicU64, Ordering},
    },
};

use clap::Parser;
use futures::future::{AbortHandle, AbortRegistration, Abortable};
use iced::{
    Alignment, Command, Element, Font, Length, Settings, Subscription, Theme, executor,
    multi_window::Application as MwApplication,
    time,
    widget::{
        button, checkbox, column, pick_list, row, scrollable, text, text_input,
        tooltip::{self, Position},
    },
    window,
};
use log::info;
use log::{LevelFilter, Log, Metadata, Record};
use once_cell::sync::OnceCell;

// Call library side CLI and command execution
use be_music_cabinet_cli::{Cli, run_command};

// Import parser module
mod parser;
use parser::{
    UiFieldType, UiSubEnumSpec, UiTopSpec, UiTree, UiVariantSpec, build_ui_tree, to_kebab_case,
};

#[derive(Debug, Clone)]
enum Msg {
    TopChanged(usize),
    SubChanged(usize),
    FieldTextChanged(String, String),
    FieldBoolChanged(String, bool),
    PickDir(String), // key for the PathBuf field
    Run,
    TickAll,
    LogTerminate(window::Id),
    UpdateMaxLines(window::Id, usize),
    ToggleAutoScroll(window::Id, bool),
}

struct App {
    tree: UiTree,
    top_idx: usize,
    sub_idx: usize,
    inputs: BTreeMap<String, String>,
    bools: BTreeMap<String, bool>,
    status: String,
    windows: BTreeMap<window::Id, TaskWindow>,
    path_history: Vec<String>,
}

struct TaskWindow {
    task_id: TaskId,
    args: Vec<String>,
    logs: String,
    running: bool,
    abort_handle: Option<AbortHandle>,
    max_lines: usize,
    auto_scroll: bool,
}

impl App {
    fn current_top(&self) -> &UiTopSpec {
        &self.tree.top_commands[self.top_idx]
    }

    fn current_sub_enum(&self) -> &UiSubEnumSpec {
        let name = &self.current_top().sub_enum_ident;
        self.tree.sub_enums.get(name).expect("sub enum not found")
    }

    fn current_variant(&self) -> &UiVariantSpec {
        &self.current_sub_enum().variants[self.sub_idx]
    }

    fn field_key(&self, field_name: &str) -> String {
        format!(
            "{}::{}::{}",
            self.current_top().sub_enum_ident,
            self.current_variant().name,
            field_name
        )
    }

    fn ensure_defaults(&mut self) {
        let fields = self.current_variant().fields.clone();
        for f in &fields {
            let key = self.field_key(&f.name);
            match f.ty {
                UiFieldType::Bool => {
                    self.bools.entry(key).or_insert(false);
                }
                _ => {
                    let default_val = f.default.clone().unwrap_or_else(String::new);
                    self.inputs.entry(key).or_insert(default_val);
                }
            }
        }
    }

    fn cleanup_closed_windows(&mut self) {
        // 检查任务是否已完成，如果已完成且窗口已关闭，则清理内存
        let mut to_remove = Vec::new();
        for (wid, w) in &mut self.windows {
            if !w.running {
                // 检查任务是否真的已完成（通过检查日志缓冲区是否为空）
                let m = buffers();
                let guard = m.lock().unwrap();
                if !guard.contains_key(&w.task_id)
                    || guard.get(&w.task_id).map(|v| v.is_empty()).unwrap_or(true)
                {
                    to_remove.push(*wid);
                }
            }
        }

        for wid in to_remove {
            if let Some(w) = self.windows.remove(&wid) {
                // 清理日志缓冲区
                let m = buffers();
                let mut guard = m.lock().unwrap();
                guard.remove(&w.task_id);
                // 中止任务
                if let Some(h) = w.abort_handle {
                    h.abort();
                }
                // 清理日志字符串，释放内存
                drop(w.logs);
                // 清理参数向量，释放内存
                drop(w.args);
            }
        }
    }

    fn build_cli_args(&self) -> Vec<String> {
        let top = self.current_top();
        let var = self.current_variant();
        let mut args = Vec::new();
        args.push("be-music-cabinet".to_string());
        args.push(to_kebab_case(&top.variant_ident));
        args.push(to_kebab_case(&var.name));
        for f in &var.fields {
            let key = format!("{}::{}::{}", top.sub_enum_ident, var.name, f.name);
            match f.ty {
                UiFieldType::Bool => {
                    if *self.bools.get(&key).unwrap_or(&false)
                        && let Some(long) = &f.long_name
                    {
                        args.push(format!("--{}", long));
                    }
                }
                _ => {
                    let val = self.inputs.get(&key).cloned().unwrap_or_default();
                    if f.has_long
                        && let Some(long) = &f.long_name
                    {
                        args.push(format!("--{}", long));
                    }
                    if !val.is_empty() {
                        args.push(val);
                    }
                }
            }
        }
        args
    }

    fn capture_and_save_paths(&mut self) {
        let mut any_new = false;
        for (_key, val) in self.inputs.iter() {
            if val.is_empty() {
                continue;
            }
            // Simple judgment: consider it a path if it contains path separators or looks like a drive letter
            let looks_like_path = val.contains('/')
                || val.contains('\\')
                || (val.len() >= 2
                    && val.as_bytes()[1] == b':'
                    && val.as_bytes()[0].is_ascii_alphabetic());
            if looks_like_path && !self.path_history.iter().any(|p| p == val) {
                self.path_history.insert(0, val.clone());
                // Limit to keep at most 50 entries
                if self.path_history.len() > 50 {
                    self.path_history.truncate(50);
                }
                any_new = true;
            }
        }
        if any_new {
            let _ = save_path_history(&self.path_history);
        }
    }
}

impl MwApplication for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Msg;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // Initialize GUI logger
        init_gui_logger();
        let path_history = load_path_history();
        let mut app = App {
            tree: build_ui_tree(),
            top_idx: 0,
            sub_idx: 0,
            inputs: BTreeMap::new(),
            bools: BTreeMap::new(),
            status: String::new(),
            windows: BTreeMap::new(),
            path_history,
        };
        app.ensure_defaults();
        (app, Command::none())
    }

    fn title(&self, id: window::Id) -> String {
        if id == window::Id::MAIN {
            "Be-Music Cabinet GUI".to_string()
        } else if let Some(w) = self.windows.get(&id) {
            let args_text = w.args.join(" ");
            format!("Task Log Window - {}", args_text)
        } else {
            "Be-Music Cabinet".to_string()
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Msg::TopChanged(i) => {
                self.top_idx = i;
                self.sub_idx = 0;
                self.inputs.clear();
                self.bools.clear();
                self.ensure_defaults();
                Command::none()
            }
            Msg::SubChanged(i) => {
                self.sub_idx = i;
                self.inputs.clear();
                self.bools.clear();
                self.ensure_defaults();
                Command::none()
            }
            Msg::FieldTextChanged(key, value) => {
                self.inputs.insert(key, value);
                Command::none()
            }
            Msg::FieldBoolChanged(key, value) => {
                self.bools.insert(key, value);
                Command::none()
            }
            Msg::PickDir(key) => {
                let pick = async move {
                    let dlg = rfd::FileDialog::new();
                    dlg.pick_folder()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default()
                };
                Command::perform(pick, move |path| Msg::FieldTextChanged(key.clone(), path))
            }
            Msg::Run => {
                let args = self.build_cli_args();
                let args_for_view = args.clone().join(" ");
                self.status = format!("Started: {}", args_for_view);
                // Record the filled paths to history
                self.capture_and_save_paths();
                let (win_id, open_cmd) = window::spawn(window::Settings {
                    size: [800u16, 400u16].into(),
                    ..window::Settings::default()
                });
                let (abort_handle, abort_reg) = AbortHandle::new_pair();
                let task_id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
                self.windows.insert(
                    win_id,
                    TaskWindow {
                        task_id,
                        args: args.clone(),
                        logs: String::new(),
                        running: true,
                        abort_handle: Some(abort_handle),
                        max_lines: 1000,
                        auto_scroll: true,
                    },
                );
                start_task_for_window(task_id, abort_reg, args);
                open_cmd
            }
            Msg::TickAll => {
                let ids: Vec<window::Id> = self.windows.keys().cloned().collect();
                for wid in ids {
                    if let Some(w) = self.windows.get_mut(&wid) {
                        let lines = drain_lines(w.task_id);
                        if !lines.is_empty() {
                            for l in lines {
                                w.logs.push_str(&l);
                                if !w.logs.ends_with('\n') {
                                    w.logs.push('\n');
                                }
                            }

                            // 限制日志行数
                            let log_lines: Vec<&str> = w.logs.lines().collect();
                            if log_lines.len() > w.max_lines {
                                w.logs = log_lines[log_lines.len() - w.max_lines..].join("\n");
                                if !w.logs.ends_with('\n') {
                                    w.logs.push('\n');
                                }
                            }

                            // 如果启用了自动滚动，确保能看到最新内容
                            if w.auto_scroll {
                                // 这里不需要额外处理，因为iced会自动滚动到最新内容
                            }
                        }
                    }
                }

                // 清理已关闭的窗口
                self.cleanup_closed_windows();

                Command::none()
            }
            Msg::LogTerminate(wid) => {
                if let Some(w) = self.windows.get_mut(&wid) {
                    if let Some(h) = w.abort_handle.take() {
                        h.abort();
                    }
                    w.running = false;
                }
                Command::none()
            }

            Msg::UpdateMaxLines(wid, max_lines) => {
                if let Some(w) = self.windows.get_mut(&wid) {
                    // 验证输入范围
                    if max_lines > 0 && max_lines <= 100000 {
                        w.max_lines = max_lines;
                        // 限制日志行数
                        let lines: Vec<&str> = w.logs.lines().collect();
                        if lines.len() > max_lines {
                            w.logs = lines[lines.len() - max_lines..].join("\n");
                            if !w.logs.ends_with('\n') {
                                w.logs.push('\n');
                            }
                        }
                    }
                }
                Command::none()
            }
            Msg::ToggleAutoScroll(wid, auto_scroll) => {
                if let Some(w) = self.windows.get_mut(&wid) {
                    w.auto_scroll = auto_scroll;
                }
                Command::none()
            }
        }
    }

    fn view(&self, id: window::Id) -> Element<'_, Self::Message> {
        if id != window::Id::MAIN
            && let Some(w) = self.windows.get(&id)
        {
            let content = column![
                scrollable(text(w.logs.clone())).height(Length::Fill),
                row![
                    text("Max Lines:"),
                    text_input("1000", &w.max_lines.to_string())
                        .on_input(move |input| {
                            if let Ok(value) = input.parse::<usize>() {
                                if value > 0 && value <= 100000 {
                                    Msg::UpdateMaxLines(id, value)
                                } else {
                                    Msg::TickAll
                                }
                            } else {
                                Msg::TickAll
                            }
                        })
                        .width(Length::Fixed(80.0)),
                    checkbox("Auto Scroll", w.auto_scroll)
                        .on_toggle(move |checked| Msg::ToggleAutoScroll(id, checked)),
                    button(text(if w.running { "Terminate" } else { "Stopped" })).on_press(
                        if w.running {
                            Msg::LogTerminate(id)
                        } else {
                            Msg::TickAll
                        }
                    )
                ]
                .spacing(10),
            ]
            .padding(12)
            .spacing(12)
            .align_items(Alignment::Start);
            return content.into();
        }

        let tops: Vec<String> = self
            .tree
            .top_commands
            .iter()
            .map(|t| t.variant_ident.clone())
            .collect();

        let subs: Vec<String> = self
            .current_sub_enum()
            .variants
            .iter()
            .map(|v| v.name.clone())
            .collect();

        let mut fields_col = column![].spacing(8).push(text("Parameters").size(18));
        for f in &self.current_variant().fields {
            let key = self.field_key(&f.name);
            let label = f.value_name.as_deref().unwrap_or(&f.name);
            let row_widget: Element<_> = match &f.ty {
                UiFieldType::Bool => {
                    let checked = *self.bools.get(&key).unwrap_or(&false);
                    row![
                        text(label),
                        checkbox("", checked).on_toggle({
                            let k = key.clone();
                            move |v| Msg::FieldBoolChanged(k.clone(), v)
                        })
                    ]
                    .spacing(10)
                    .into()
                }
                UiFieldType::Enum(variants) => {
                    let current_val = self.inputs.get(&key).cloned().unwrap_or_default();
                    let selected_idx = variants.iter().position(|v| v == &current_val).unwrap_or(0);
                    let selected_variant = variants.get(selected_idx).cloned().unwrap_or_default();
                    row![
                        text(label),
                        pick_list(variants.clone(), Some(selected_variant), {
                            let k = key.clone();
                            let vs = variants.clone();
                            move |v| {
                                let idx = vs.iter().position(|variant| variant == &v).unwrap_or(0);
                                Msg::FieldTextChanged(k.clone(), vs[idx].clone())
                            }
                        })
                        .width(Length::Fill)
                    ]
                    .spacing(10)
                    .into()
                }
                UiFieldType::PathBuf => {
                    let val = self.inputs.get(&key).cloned().unwrap_or_default();
                    let history = self.path_history.clone();
                    let selected = if history.iter().any(|h| h == &val) {
                        Some(val.clone())
                    } else {
                        None
                    };
                    let row_input = row![
                        text(label),
                        text_input("", &val)
                            .on_input({
                                let k = key.clone();
                                move |v| Msg::FieldTextChanged(k.clone(), v)
                            })
                            .width(Length::Fill)
                    ]
                    .spacing(10);
                    let row_helpers = row![
                        pick_list(history, selected, {
                            let k = key.clone();
                            move |v: String| Msg::FieldTextChanged(k.clone(), v)
                        })
                        .width(Length::Fill),
                        button(text("Select...")).on_press({
                            let k = key.clone();
                            Msg::PickDir(k)
                        })
                    ]
                    .spacing(10);
                    column![row_input, row_helpers].spacing(6).into()
                }
                _ => {
                    let val = self.inputs.get(&key).cloned().unwrap_or_default();
                    row![
                        text(label),
                        text_input("", &val)
                            .on_input({
                                let k = key.clone();
                                move |v| Msg::FieldTextChanged(k.clone(), v)
                            })
                            .width(Length::Fill)
                    ]
                    .spacing(10)
                    .into()
                }
            };
            let row_widget: Element<_> = if let Some(doc) = f.doc.as_ref() {
                tooltip::Tooltip::new(row_widget, text(doc.clone()), Position::FollowCursor).into()
            } else {
                row_widget
            };
            fields_col = fields_col.push(row_widget);
        }

        let top_pick: Element<_> = {
            let pl = pick_list(tops.clone(), Some(tops[self.top_idx].clone()), move |v| {
                let idx = tops.iter().position(|t| t == &v).unwrap_or(0);
                Msg::TopChanged(idx)
            });
            if let Some(doc) = self.current_top().doc.as_ref() {
                tooltip::Tooltip::new(pl, text(doc.clone()), Position::FollowCursor).into()
            } else {
                pl.into()
            }
        };

        let sub_pick: Element<_> = {
            let pl = pick_list(subs.clone(), Some(subs[self.sub_idx].clone()), move |v| {
                let idx = subs.iter().position(|t| t == &v).unwrap_or(0);
                Msg::SubChanged(idx)
            });
            if let Some(doc) = self.current_variant().doc.as_ref() {
                tooltip::Tooltip::new(pl, text(doc.clone()), Position::FollowCursor).into()
            } else {
                pl.into()
            }
        };

        let content = column![
            row![text("Command").size(18), top_pick,].spacing(10),
            row![text("Subcommand").size(18), sub_pick,].spacing(10),
            fields_col,
            row![button(text("Execute")).on_press(Msg::Run),].spacing(10),
            scrollable(text(self.status.clone())).height(Length::Fill),
        ]
        .padding(12)
        .spacing(12)
        .align_items(Alignment::Start);

        content.into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // 处理定时器
        time::every(std::time::Duration::from_millis(200)).map(|_| Msg::TickAll)
    }
}

fn pick_chinese_font() -> Font {
    // Prioritize trying common Chinese font names in the system; if failed, use fontdb scanning
    if let Some(fam) = [
        "Microsoft YaHei",
        "Microsoft JhengHei",
        "SimSun",
        "SimHei",
        "Noto Sans CJK SC",
        "Noto Sans CJK TC",
        "WenQuanYi Micro Hei",
    ]
    .into_iter()
    .next()
    {
        // Only depend on name, if system exists it will be resolved by backend
        return Font::with_name(fam);
    }
    // Fallback: still return a named font, leave it to system to choose
    Font::with_name("Microsoft YaHei")
}

fn main() -> iced::Result {
    // Simple test to verify enum type recognition
    info!("Testing enum type detection...");

    let tree = build_ui_tree();

    // Find RootCommands::SetName
    if let Some(root_cmd) = tree.top_commands.iter().find(|t| t.variant_ident == "Root")
        && let Some(sub_enum) = tree.sub_enums.get(&root_cmd.sub_enum_ident)
        && let Some(set_name_variant) = sub_enum.variants.iter().find(|v| v.name == "SetName")
    {
        for field in &set_name_variant.fields {
            if field.name == "set_type" {
                match &field.ty {
                    UiFieldType::Enum(variants) => {
                        info!(
                            "✓ Found enum field 'set_type' with variants: {:?}",
                            variants
                        );
                    }
                    other => {
                        info!("✗ Expected enum type, got: {:?}", other);
                    }
                }
            }
        }
    }

    info!("Test completed. Starting GUI...");
    info!(
        "Note: The GUI should now show a dropdown menu for the 'set_type' field in Root > SetName command"
    );
    info!("Available options: replace_title_artist, append_title_artist, append_artist");

    let mut settings: Settings<()> = Settings {
        id: None,
        ..Settings::default()
    };
    settings.default_font = pick_chinese_font();
    settings.default_text_size = 16.into();
    settings.antialiasing = true;
    // Set main window default size 800x400
    settings.window.size = [800u16, 400u16].into();
    <App as MwApplication>::run(settings)
}

// ========================= Multi-task Logging: Global Logger and Log Window =========================

// Task ID allocator
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

// Thread local: mark which task current thread belongs to, used for log routing
thread_local! {
    static CURRENT_TASK_ID: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// Global buffer: one log line queue to be pulled for each task
type TaskId = u64;
type LogBufferMap = HashMap<TaskId, Vec<String>>;
type SharedLogBuffers = Arc<StdMutex<LogBufferMap>>;
static LOG_BUFFERS: OnceCell<SharedLogBuffers> = OnceCell::new();

fn buffers() -> &'static SharedLogBuffers {
    LOG_BUFFERS.get_or_init(|| Arc::new(StdMutex::new(HashMap::new())))
}

fn push_line(id: TaskId, line: String) {
    let m = buffers();
    let mut guard = m.lock().unwrap();
    guard.entry(id).or_default().push(line);
}

fn drain_lines(id: TaskId) -> Vec<String> {
    let m = buffers();
    let mut guard = m.lock().unwrap();
    guard.remove(&id).unwrap_or_default()
}

// Global GUI Logger: route log::info etc. output to each task's buffer
struct GuiLogger;

impl Log for GuiLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!("{}", record.args());
        CURRENT_TASK_ID.with(|cell| {
            if let Some(id) = *cell.borrow() {
                push_line(id, line.clone());
            } else {
                // No task context: ignore or output to main console. Ignore here to avoid interference.
            }
        });
    }

    fn flush(&self) {}
}

fn init_gui_logger() {
    // Ensure buffer exists
    let _ = buffers();
    // Register global logger (only once)
    let _ = log::set_boxed_logger(Box::new(GuiLogger));
    log::set_max_level(LevelFilter::Info);
}

// Removed independent log window implementation, unified task log display in main window

// Removed single window old startup entry

fn start_task_for_window(task_id: TaskId, abort_reg: AbortRegistration, args: Vec<String>) {
    let cmd_line = args.clone().join(" ");
    let cmd_line_for_log = cmd_line.clone();
    blocking::unblock(move || {
        CURRENT_TASK_ID.with(|c| *c.borrow_mut() = Some(task_id));
        let result = smol::block_on(async move {
            println!("[GUI] Command: {}", cmd_line);
            let cli = match Cli::try_parse_from(args.clone()) {
                Ok(cli) => cli,
                Err(e) => {
                    return Err(format!("Parameter parsing failed: {}", e));
                }
            };
            let fut = run_command(&cli.command);
            match Abortable::new(fut, abort_reg).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(e.to_string()),
                Err(_aborted) => Err("Task terminated".to_string()),
            }
        });
        match result {
            Ok(()) => push_line(task_id, "[Completed]".to_string()),
            Err(e) => push_line(task_id, format!("[Error] {}", e)),
        }
        CURRENT_TASK_ID.with(|c| *c.borrow_mut() = None);
    })
    .detach();
    push_line(task_id, format!("[Started] {}", cmd_line_for_log));
}

// ========================= Path History: Save and Load =========================
const HISTORY_FILE: &str = "path_history.json";

fn history_file_path() -> StdPathBuf {
    // Under run directory
    StdPathBuf::from(HISTORY_FILE)
}

fn load_path_history() -> Vec<String> {
    let path = history_file_path();
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str::<Vec<String>>(&s).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_path_history(list: &Vec<String>) -> Result<(), String> {
    let path = history_file_path();
    let content = serde_json::to_string_pretty(list).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}
