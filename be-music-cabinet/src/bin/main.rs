#![recursion_limit = "1024"]

use iced::{Application, Command, Element, Length, Theme, executor, theme, widget};
use std::process::Command as StdCommand;

#[derive(Debug, Clone, PartialEq)]
pub enum CommandCategory {
    Work,
    Root,
    Pack,
    Bms,
    Fs,
    RootEvent,
    Rawpack,
}

impl std::fmt::Display for CommandCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CommandCategory::Work => "Work",
            CommandCategory::Root => "Root",
            CommandCategory::Pack => "Pack",
            CommandCategory::Bms => "Bms",
            CommandCategory::Fs => "Fs",
            CommandCategory::RootEvent => "RootEvent",
            CommandCategory::Rawpack => "Rawpack",
        };
        write!(f, "{}", s)
    }
}

impl CommandCategory {
    pub fn all() -> Vec<CommandCategory> {
        vec![
            CommandCategory::Work,
            CommandCategory::Root,
            CommandCategory::Pack,
            CommandCategory::Bms,
            CommandCategory::Fs,
            CommandCategory::RootEvent,
            CommandCategory::Rawpack,
        ]
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    CategoryChanged(CommandCategory),
    InputChanged(String),
    SetTypeChanged(String),
    SecondDirectoryChanged(String),
    SimilarityChanged(String),
    CountChanged(String),
    ExecuteCommand,
    CommandCompleted(Result<String, String>),
}

#[derive(Debug, Clone)]
pub struct App {
    current_category: CommandCategory,
    directory_path: String,
    set_type: String,
    second_directory: String, // 用于需要第二个目录的命令
    similarity: String,       // 用于相似度设置
    count: String,            // 用于数量设置
    is_executing: bool,
    log_messages: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_category: CommandCategory::Work,
            directory_path: String::new(),
            set_type: "append_title_artist".to_string(),
            second_directory: String::new(),
            similarity: "0.7".to_string(),
            count: "10".to_string(),
            is_executing: false,
            log_messages: Vec::new(),
        }
    }
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (App::default(), Command::none())
    }

    fn title(&self) -> String {
        "BE Music Cabinet GUI".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CategoryChanged(category) => {
                self.current_category = category;
                Command::none()
            }
            Message::InputChanged(value) => {
                self.directory_path = value;
                Command::none()
            }
            Message::SetTypeChanged(value) => {
                self.set_type = value;
                Command::none()
            }
            Message::SecondDirectoryChanged(value) => {
                self.second_directory = value;
                Command::none()
            }
            Message::SimilarityChanged(value) => {
                self.similarity = value;
                Command::none()
            }
            Message::CountChanged(value) => {
                self.count = value;
                Command::none()
            }
            Message::ExecuteCommand => {
                if self.directory_path.is_empty() {
                    self.log_messages.push("错误: 目录路径不能为空".to_string());
                    return Command::none();
                }

                self.is_executing = true;
                self.log_messages.push(format!(
                    "开始执行: {} 命令...",
                    self.current_category
                ));

                // 使用CLI命令避免异步类型问题
                let directory = self.directory_path.clone();
                let set_type = self.set_type.clone();
                let second_directory = self.second_directory.clone();
                let similarity = self.similarity.clone();
                let count = self.count.clone();
                let category = self.current_category.clone();

                Command::perform(
                    async move {
                        
                        execute_cli_command(
                            category,
                            directory,
                            set_type,
                            second_directory,
                            similarity,
                            count,
                        )
                        .await
                    },
                    Message::CommandCompleted,
                )
            }
            Message::CommandCompleted(result) => {
                self.is_executing = false;
                match result {
                    Ok(output) => {
                        self.log_messages.push("执行成功!".to_string());
                        if !output.is_empty() {
                            self.log_messages.push(output);
                        }
                    }
                    Err(error) => {
                        self.log_messages.push(format!("执行失败: {}", error));
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let title = widget::text("BE Music Cabinet").size(24);

        // 命令类别选择
        let category_options = CommandCategory::all();

        let category_picker = widget::pick_list(
            category_options,
            Some(self.current_category.clone()),
            Message::CategoryChanged,
        )
        .placeholder("选择命令类别");

        // 目录输入
        let directory_input = widget::text_input("输入目录路径", &self.directory_path)
            .on_input(Message::InputChanged)
            .padding(10);

        // 动态参数输入区域
        let mut dynamic_inputs: Vec<Element<Message>> = Vec::new();

        match self.current_category {
            CommandCategory::Work => {
                dynamic_inputs.push(widget::text("Set Type:").size(14).into());
                dynamic_inputs.push(
                    widget::text_input("Set Type (e.g., append_title_artist)", &self.set_type)
                        .on_input(Message::SetTypeChanged)
                        .padding(10)
                        .into(),
                );
            }
            CommandCategory::Root => {
                dynamic_inputs.push(widget::text("相似度阈值:").size(14).into());
                dynamic_inputs.push(
                    widget::text_input("相似度 (0.0-1.0)", &self.similarity)
                        .on_input(Message::SimilarityChanged)
                        .padding(10)
                        .into(),
                );
            }
            CommandCategory::Pack => {
                dynamic_inputs.push(widget::text("目标目录:").size(14).into());
                dynamic_inputs.push(
                    widget::text_input("目标打包目录", &self.second_directory)
                        .on_input(Message::SecondDirectoryChanged)
                        .padding(10)
                        .into(),
                );
            }
            CommandCategory::Fs => {
                dynamic_inputs.push(widget::text("同步目标目录:").size(14).into());
                dynamic_inputs.push(
                    widget::text_input("同步目标目录路径", &self.second_directory)
                        .on_input(Message::SecondDirectoryChanged)
                        .padding(10)
                        .into(),
                );
            }
            CommandCategory::Bms | CommandCategory::RootEvent | CommandCategory::Rawpack => {
                // 这些命令通常只需要目录路径
                dynamic_inputs.push(
                    widget::text("").into(), // 占位符
                );
            }
        }

        let dynamic_content = widget::column(dynamic_inputs).spacing(10);

        // 执行按钮
        let execute_button = if self.is_executing {
            widget::button("执行中...").style(theme::Button::Secondary)
        } else {
            widget::button("执行命令").on_press(Message::ExecuteCommand)
        };

        // 日志显示
        let log_content: Element<_> = if self.log_messages.is_empty() {
            widget::text("执行日志将在这里显示").into()
        } else {
            widget::scrollable(widget::column(
                self.log_messages
                    .iter()
                    .map(|msg| widget::text(msg).size(12).into())
                    .collect::<Vec<_>>(),
            ))
            .height(Length::Fixed(200.0))
            .into()
        };

        let content = widget::column![
            title,
            widget::vertical_space(),
            widget::text("命令类别:").size(16),
            category_picker,
            widget::vertical_space(),
            widget::text("目录路径:").size(16),
            directory_input,
            widget::vertical_space(),
            dynamic_content,
            widget::vertical_space(),
            execute_button,
            widget::vertical_space(),
            widget::text("执行日志:").size(16),
            log_content,
        ]
        .padding(20)
        .spacing(10);

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}

// 通过CLI调用避免异步类型问题
async fn execute_cli_command(
    category: CommandCategory,
    directory: String,
    set_type: String,
    second_directory: String,
    similarity: String,
    count: String,
) -> Result<String, String> {
    let mut cmd = StdCommand::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("be-music-cabinet-cli")
        .arg("--");

    match category {
        CommandCategory::Work => {
            cmd.arg("work")
                .arg("set-name")
                .arg("--dir")
                .arg(&directory)
                .arg("--set-type")
                .arg(&set_type);
        }
        CommandCategory::Root => {
            cmd.arg("root")
                .arg("scan-folder-similar-folders")
                .arg(&directory);

            // 如果提供了相似度，添加参数
            if !similarity.is_empty() && similarity != "0.7" {
                cmd.arg("--similarity").arg(&similarity);
            }
        }
        CommandCategory::Pack => {
            cmd.arg("pack").arg("pack-dir").arg(&directory);

            // 如果提供了第二个目录，可能需要其他参数
            if !second_directory.is_empty() {
                cmd.arg("--output").arg(&second_directory);
            }
        }
        CommandCategory::Bms => {
            cmd.arg("bms").arg("scan-dir").arg(&directory);
        }
        CommandCategory::Fs => {
            cmd.arg("fs").arg("sync-folder").arg(&directory);

            // 添加目标目录
            if !second_directory.is_empty() {
                cmd.arg(&second_directory);
            }
        }
        CommandCategory::RootEvent => {
            cmd.arg("root-event").arg("scan-folder").arg(&directory);
        }
        CommandCategory::Rawpack => {
            cmd.arg("rawpack").arg("pack-dir").arg(&directory);
        }
    }

    // 使用smol执行命令
    smol::unblock(move || {
        let output = cmd.output().map_err(|e| format!("执行命令失败: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("命令执行失败: {}", stderr))
        }
    })
    .await
}

fn main() -> iced::Result {
    App::run(iced::Settings::default())
}
