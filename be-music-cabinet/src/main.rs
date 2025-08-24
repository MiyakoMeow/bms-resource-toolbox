// GUI 模式不直接使用库入口
use iced::multi_window::Application as MwApplication;
use iced::widget::{button, checkbox, column, pick_list, row, scrollable, text, text_input};
use iced::window;
use iced::{Alignment, Command, Element, Font, Length, Settings, Theme, executor};
use iced::{Subscription, time};
use log::info;
use quote::ToTokens;
use std::collections::{BTreeMap, HashMap};
use std::sync::{
    Arc, Mutex as StdMutex,
    atomic::{AtomicU64, Ordering},
};
use syn::{Attribute, Fields, Item, ItemEnum, Type, parse_file};

// 调用库侧 CLI 与命令执行
use be_music_cabinet::{Cli, run_command};
use clap::Parser;

use futures::future::{AbortHandle, AbortRegistration, Abortable};
use log::{LevelFilter, Log, Metadata, Record};
use once_cell::sync::OnceCell;
use std::cell::RefCell;

// 解析目标：从 lib.rs 抽取 Commands 与其子枚举结构，动态生成 UI
const LIB_RS_SRC: &str = include_str!("lib.rs");

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum UiFieldType {
    PathBuf,
    String,
    Usize,
    F64,
    Bool,
    Enum(Vec<String>), // 枚举变体列表
    Other(String),
}

#[derive(Debug, Clone)]
struct UiFieldSpec {
    name: String,
    ty: UiFieldType,
    has_long: bool,
    long_name: Option<String>,
    default: Option<String>,
    value_name: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UiVariantSpec {
    name: String,
    fields: Vec<UiFieldSpec>,
    doc: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UiSubEnumSpec {
    name: String,
    variants: Vec<UiVariantSpec>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UiTopSpec {
    variant_ident: String,
    sub_enum_ident: String,
    doc: Option<String>,
}

#[derive(Debug, Clone)]
struct UiTree {
    top_commands: Vec<UiTopSpec>,
    sub_enums: HashMap<String, UiSubEnumSpec>,
}

fn ident_to_string_path(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        _ => format!("{}", ty.to_token_stream()),
    }
}

fn type_to_ui_type(ty: &Type) -> UiFieldType {
    let ts = ident_to_string_path(ty);
    match ts.as_str() {
        "PathBuf" | "std::path::PathBuf" => UiFieldType::PathBuf,
        "String" | "std::string::String" => UiFieldType::String,
        "usize" => UiFieldType::Usize,
        "f64" => UiFieldType::F64,
        "bool" => UiFieldType::Bool,
        "BmsFolderSetNameType" => UiFieldType::Enum(vec![
            "replace_title_artist".to_string(),
            "append_title_artist".to_string(),
            "append_artist".to_string(),
        ]),
        _ => UiFieldType::Other(ts),
    }
}

fn attr_tokens_contains_long(attr: &Attribute) -> bool {
    // 尽量不写死解析，宽松判断 "long" 字样
    let s = attr.to_token_stream().to_string();
    s.contains(" long") || s.contains("long ") || s.contains("long,") || s.contains("long=")
}

fn extract_arg_value_name(attr: &Attribute) -> Option<String> {
    // 查找 value_name = "..."
    let s = attr.to_token_stream().to_string();
    let key = "value_name = \"";
    if let Some(pos) = s.find(key) {
        let rest = &s[pos + key.len()..];
        if let Some(end) = rest.find('\"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn extract_default_value(attr: &Attribute) -> Option<String> {
    // 查找 default_value = "..."
    let s = attr.to_token_stream().to_string();
    let key = "default_value = \"";
    if let Some(pos) = s.find(key) {
        let rest = &s[pos + key.len()..];
        if let Some(end) = rest.find('\"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn get_doc_attr_text(attrs: &[Attribute]) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    for a in attrs {
        if a.path().is_ident("doc") {
            let ts = a.to_token_stream().to_string();
            // 形如: #[doc = "..."]
            let key = "= \"";
            if let Some(pos) = ts.find(key) {
                let rest = &ts[pos + key.len()..];
                if let Some(end) = rest.find('\"') {
                    lines.push(rest[..end].to_string());
                }
            }
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn to_kebab_case(name: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for ch in name.chars() {
        if ch == '_' {
            out.push('-');
            prev_lower = false;
            continue;
        }
        if ch.is_ascii_uppercase() {
            if prev_lower {
                out.push('-');
            }
            for c in ch.to_lowercase() {
                out.push(c);
            }
            prev_lower = false;
        } else {
            out.push(ch);
            prev_lower = ch.is_ascii_lowercase();
        }
    }
    out
}

fn to_long_flag(name: &str) -> String {
    name.replace('_', "-")
}

fn build_ui_tree() -> UiTree {
    let file = parse_file(LIB_RS_SRC).expect("parse lib.rs failed");
    let mut enums: HashMap<String, ItemEnum> = HashMap::new();
    for item in file.items {
        if let Item::Enum(e) = item {
            enums.insert(e.ident.to_string(), e);
        }
    }

    let commands = enums
        .get("Commands")
        .expect("Commands enum not found in lib.rs");

    let mut top_commands: Vec<UiTopSpec> = Vec::new();
    for var in &commands.variants {
        // 变体如: Work { command: WorkCommands }
        let mut sub_enum_ident = None;
        if let Fields::Named(named) = &var.fields {
            for f in &named.named {
                if let Some(ident) = &f.ident
                    && ident == "command"
                {
                    sub_enum_ident = Some(ident_to_string_path(&f.ty));
                    break;
                }
            }
        }
        if let Some(sub) = sub_enum_ident {
            top_commands.push(UiTopSpec {
                variant_ident: var.ident.to_string(),
                sub_enum_ident: sub,
                doc: get_doc_attr_text(&var.attrs),
            });
        }
    }

    let mut sub_enums: HashMap<String, UiSubEnumSpec> = HashMap::new();
    for top in &top_commands {
        if let Some(sub_enum) = enums.get(&top.sub_enum_ident) {
            let mut variants: Vec<UiVariantSpec> = Vec::new();
            for v in &sub_enum.variants {
                let mut fields_spec: Vec<UiFieldSpec> = Vec::new();
                match &v.fields {
                    Fields::Named(named) => {
                        for f in &named.named {
                            let name = f
                                .ident
                                .as_ref()
                                .map(|i| i.to_string())
                                .unwrap_or_else(|| "arg".to_string());

                            let ty = type_to_ui_type(&f.ty);
                            let mut has_long = false;
                            let mut value_name = None;
                            let mut default = None;
                            for attr in &f.attrs {
                                if attr.path().is_ident("arg") {
                                    if attr_tokens_contains_long(attr) {
                                        has_long = true;
                                    }
                                    if value_name.is_none() {
                                        value_name = extract_arg_value_name(attr);
                                    }
                                    if default.is_none() {
                                        default = extract_default_value(attr);
                                    }
                                }
                            }
                            let long_name = if has_long {
                                Some(to_long_flag(&name))
                            } else {
                                None
                            };
                            fields_spec.push(UiFieldSpec {
                                name,
                                ty,
                                has_long,
                                long_name,
                                default,
                                value_name,
                            });
                        }
                    }
                    Fields::Unnamed(_) | Fields::Unit => {}
                }
                variants.push(UiVariantSpec {
                    name: v.ident.to_string(),
                    fields: fields_spec,
                    doc: get_doc_attr_text(&v.attrs),
                });
            }
            sub_enums.insert(
                top.sub_enum_ident.clone(),
                UiSubEnumSpec {
                    name: top.sub_enum_ident.clone(),
                    variants,
                },
            );
        }
    }

    UiTree {
        top_commands,
        sub_enums,
    }
}

#[derive(Debug, Clone)]
enum Msg {
    TopChanged(usize),
    SubChanged(usize),
    FieldTextChanged(String, String),
    FieldBoolChanged(String, bool),
    Run,
    TickAll,
    LogTerminate(window::Id),
}

struct App {
    tree: UiTree,
    top_idx: usize,
    sub_idx: usize,
    inputs: BTreeMap<String, String>,
    bools: BTreeMap<String, bool>,
    status: String,
    #[allow(dead_code)]
    running: bool,
    windows: BTreeMap<window::Id, TaskWindow>,
}

struct TaskWindow {
    task_id: TaskId,
    args: Vec<String>,
    logs: String,
    running: bool,
    abort_handle: Option<AbortHandle>,
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
}

impl MwApplication for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Msg;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut app = App {
            tree: build_ui_tree(),
            top_idx: 0,
            sub_idx: 0,
            inputs: BTreeMap::new(),
            bools: BTreeMap::new(),
            status: String::new(),
            running: false,
            windows: BTreeMap::new(),
        };
        app.ensure_defaults();
        (app, Command::none())
    }

    fn title(&self, id: window::Id) -> String {
        if id == window::Id::MAIN {
            "Be-Music Cabinet GUI".to_string()
        } else if let Some(w) = self.windows.get(&id) {
            let args_text = w.args.join(" ");
            format!("任务日志窗口 - {}", args_text)
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
            Msg::Run => {
                let args = self.build_cli_args();
                let args_for_view = args.clone().join(" ");
                self.status = format!("已启动: {}", args_for_view);
                let (win_id, open_cmd) = window::spawn(window::Settings { size: [800u16, 400u16].into(), ..window::Settings::default() });
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
                        }
                    }
                }
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
        }
    }

    fn view(&self, id: window::Id) -> Element<'_, Self::Message> {
        if id != window::Id::MAIN
            && let Some(w) = self.windows.get(&id)
        {
            let content = column![
                scrollable(text(w.logs.clone())).height(Length::Fill),
                row![
                    button(text(if w.running { "终止" } else { "已停止" })).on_press(
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

        let mut fields_col = column![].spacing(8).push(text("参数").size(18));
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
            fields_col = fields_col.push(row_widget);
        }

        let content = column![
            row![
                text("命令").size(18),
                pick_list(tops.clone(), Some(tops[self.top_idx].clone()), move |v| {
                    let idx = tops.iter().position(|t| t == &v).unwrap_or(0);
                    Msg::TopChanged(idx)
                })
            ]
            .spacing(10),
            row![
                text("子命令").size(18),
                pick_list(subs.clone(), Some(subs[self.sub_idx].clone()), move |v| {
                    let idx = subs.iter().position(|t| t == &v).unwrap_or(0);
                    Msg::SubChanged(idx)
                })
            ]
            .spacing(10),
            fields_col,
            row![button(text("执行")).on_press(Msg::Run),].spacing(10),
            scrollable(text(self.status.clone())).height(Length::Fill),
        ]
        .padding(12)
        .spacing(12)
        .align_items(Alignment::Start);

        content.into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_millis(200)).map(|_| Msg::TickAll)
    }
}

fn pick_chinese_font() -> Font {
    // 优先尝试系统常见中文字体名称；若失败，再使用 fontdb 扫描
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
        // 仅依赖名称，若系统存在将由后端解析
        return Font::with_name(fam);
    }
    // 回退：仍返回一个名称字体，交由系统选择
    Font::with_name("Microsoft YaHei")
}

fn main() -> iced::Result {
    // 简单的测试来验证枚举类型识别
    info!("Testing enum type detection...");

    let tree = build_ui_tree();

    // 查找 RootCommands::SetName
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
    // 设置主窗口默认尺寸 800x400
    settings.window.size = [800u16, 400u16].into();
    <App as MwApplication>::run(settings)
}

// ========================= 多任务日志：全局 Logger 与日志窗口 =========================

// 任务 ID 分配器
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

// 线程局部：标记当前线程属于哪个任务，用于日志路由
thread_local! {
    static CURRENT_TASK_ID: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// 全局缓冲区：每个任务一个待拉取的日志行队列
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

#[allow(dead_code)]
fn drain_lines(id: TaskId) -> Vec<String> {
    let m = buffers();
    let mut guard = m.lock().unwrap();
    guard.remove(&id).unwrap_or_default()
}

// 全局 GUI Logger：将 log::info 等输出路由到各任务的缓冲区
#[allow(dead_code)]
struct GuiLogger {
    _buf: Arc<StdMutex<HashMap<u64, Vec<String>>>>,
}

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
                // 无任务上下文：忽略或输出到主控台。这里忽略以避免串扰。
            }
        });
    }

    fn flush(&self) {}
}

#[allow(dead_code)]
fn init_gui_logger() {
    // 确保缓冲区存在
    let _ = buffers();
    // 注册全局 logger（只需一次）
    let _ = log::set_boxed_logger(Box::new(GuiLogger {
        _buf: buffers().clone(),
    }));
    log::set_max_level(LevelFilter::Info);
}

// 已移除独立日志窗口实现，统一在主窗口中展示任务日志

#[allow(dead_code)]
fn start_task(args: Vec<String>) {
    let task_id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
    init_gui_logger();
    let (_abort_handle, abort_reg) = AbortHandle::new_pair();
    let cmd_line = args.clone().join(" ");
    let cmd_line_for_log = cmd_line.clone();
    blocking::unblock(move || {
        CURRENT_TASK_ID.with(|c| *c.borrow_mut() = Some(task_id));
        let result = smol::block_on(async move {
            // 在终端打印生成的命令
            println!("[GUI] 命令: {}", cmd_line);
            // 解析完整 argv（包含程序名）
            let cli = match Cli::try_parse_from(args.clone()) {
                Ok(cli) => cli,
                Err(e) => {
                    return Err(format!("参数解析失败: {}", e));
                }
            };
            let fut = run_command(&cli.command);
            match Abortable::new(fut, abort_reg).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(e.to_string()),
                Err(_aborted) => Err("任务已终止".to_string()),
            }
        });
        match result {
            Ok(()) => push_line(task_id, "[完成]".to_string()),
            Err(e) => push_line(task_id, format!("[错误] {}", e)),
        }
        CURRENT_TASK_ID.with(|c| *c.borrow_mut() = None);
    })
    .detach();
    // 在主窗口日志中输出命令
    push_line(task_id, format!("[已启动] {}", cmd_line_for_log));
}

fn start_task_for_window(task_id: TaskId, abort_reg: AbortRegistration, args: Vec<String>) {
    let cmd_line = args.clone().join(" ");
    let cmd_line_for_log = cmd_line.clone();
    blocking::unblock(move || {
        CURRENT_TASK_ID.with(|c| *c.borrow_mut() = Some(task_id));
        let result = smol::block_on(async move {
            println!("[GUI] 命令: {}", cmd_line);
            let cli = match Cli::try_parse_from(args.clone()) {
                Ok(cli) => cli,
                Err(e) => {
                    return Err(format!("参数解析失败: {}", e));
                }
            };
            let fut = run_command(&cli.command);
            match Abortable::new(fut, abort_reg).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(e.to_string()),
                Err(_aborted) => Err("任务已终止".to_string()),
            }
        });
        match result {
            Ok(()) => push_line(task_id, "[完成]".to_string()),
            Err(e) => push_line(task_id, format!("[错误] {}", e)),
        }
        CURRENT_TASK_ID.with(|c| *c.borrow_mut() = None);
    })
    .detach();
    push_line(task_id, format!("[已启动] {}", cmd_line_for_log));
}
