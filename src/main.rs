//! BMS Resource Toolbox - Interactive menu interface
//!
//! This module provides an interactive CLI that replicates the behavior
//! of bms-resource-scripts main.py exactly.

#![expect(clippy::too_many_lines)]

use std::any::Any;
use std::io::{self, Write};
use std::path::PathBuf;

use tokio::io::AsyncWriteExt;
use tokio::runtime::Handle;

mod bms;
mod error;
mod fs;
mod media;
mod options;
mod scripts;

static HISTORY_FILE: &str = "input_history.log";

fn input_string(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn input_path(_prompt: &str) -> PathBuf {
    let history = Handle::current().block_on(load_path_history());
    let mut paths = history;

    if !paths.is_empty() {
        println!("输入路径开始。以下是之前使用过的路径：");
        let display: Vec<_> = paths.iter().take(5).collect();
        for (i, path) in display.iter().enumerate() {
            println!(" -> {}: {}", i, path.display());
        }
        if paths.len() > 5 {
            println!("（还有 {} 个历史路径，输入？查看全部）", paths.len() - 5);
        }
    }

    let mut input_str =
        input_string("直接输入路径，或输入上面的数字（索引）进行选择，输入？查看所有选项：");

    if input_str == "?" || input_str == "？" {
        if paths.is_empty() {
            println!("暂无历史路径记录");
        } else {
            println!("所有可选选项：");
            for (i, path) in paths.iter().enumerate() {
                println!("  {}: {}", i, path.display());
            }
        }
        input_str = input_string("请输入选择：");
    }

    let selected: PathBuf = if input_str.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(idx) = input_str.parse::<usize>() {
            if idx < paths.len() {
                let selected = paths.remove(idx);
                paths.insert(0, selected.clone());
                selected
            } else {
                PathBuf::from(input_str)
            }
        } else {
            PathBuf::from(input_str)
        }
    } else {
        let p = PathBuf::from(&input_str);
        paths.retain(|x| x != &p);
        paths.insert(0, p.clone());
        p
    };

    Handle::current().block_on(save_path_history(&paths));
    selected
}

async fn load_path_history() -> Vec<PathBuf> {
    let history_path = PathBuf::from(HISTORY_FILE);
    if !history_path.exists() {
        return Vec::new();
    }

    match tokio::fs::read_to_string(&history_path).await {
        Ok(content) => {
            let lines: Vec<PathBuf> = content
                .lines()
                .map(|s| PathBuf::from(s.trim()))
                .filter(|p| !p.as_os_str().is_empty())
                .collect();
            lines
        }
        Err(_) => Vec::new(),
    }
}

async fn save_path_history(paths: &[PathBuf]) {
    if let Ok(mut file) = tokio::fs::File::create(HISTORY_FILE).await {
        for path in paths {
            let _ = file.write_all(format!("{}\n", path.display()).as_bytes()).await;
        }
    }
}

fn input_confirm(prompt: &str, default_yes: bool) -> bool {
    let default_str = if default_yes { "[Y/n]" } else { "[y/N]" };
    let result = input_string(&format!("{prompt} {default_str}"));
    result.is_empty() || result.to_lowercase().starts_with('y')
}

/// Input type for interactive prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    /// Any string input.
    Any,
    /// A single word without spaces.
    Word,
    /// An integer input.
    Int,
    /// A file system path with history.
    Path,
}

/// Confirmation type for interactive prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmType {
    /// No confirmation required.
    NoConfirm,
    /// Default to yes (confirm unless explicitly declined).
    DefaultYes,
    /// Default to no (decline unless explicitly confirmed).
    DefaultNo,
}

/// Input specification for interactive prompts.
#[derive(Debug, Clone)]
pub struct Input {
    /// The type of input to receive.
    pub input_type: InputType,
    /// Description of what the input represents.
    pub description: String,
}

impl Input {
    fn exec_input(&self) -> Box<dyn Any> {
        match self.input_type {
            InputType::Any => Box::new(input_string("Input:")),
            InputType::Word => {
                let tips = "Input a word:";
                let mut w_str = input_string(tips);
                while w_str.contains(' ') {
                    println!("Requires a word. Re-input.");
                    w_str = input_string(tips);
                }
                Box::new(w_str)
            }
            InputType::Int => {
                let tips = "Input a number:";
                let mut w_str = input_string(tips);
                while !w_str.chars().all(|c| c.is_ascii_digit()) {
                    println!("Requires a number. Re-input.");
                    w_str = input_string(tips);
                }
                Box::new(w_str.parse::<i32>().unwrap_or(0))
            }
            InputType::Path => Box::new(input_path("")),
        }
    }
}

type CheckFunc = fn(&[Box<dyn Any>]) -> bool;
type ExecFunc = fn(&[Box<dyn Any>]);

/// Menu option for interactive CLI.
#[derive(Debug, Clone)]
pub struct MenuOption {
    /// Display name of the option.
    pub name: String,
    /// Function to execute when option is selected.
    pub exec_func: ExecFunc,
    /// Input specifications for this option.
    pub inputs: Vec<Input>,
    /// Optional validation function to run before execution.
    pub check_func: Option<CheckFunc>,
    /// Confirmation type before execution.
    pub confirm: ConfirmType,
}

impl MenuOption {
    fn exec(&self) {
        println!("{}", self.name);

        let mut args: Vec<Box<dyn Any>> = Vec::new();
        for (i, input_arg) in self.inputs.iter().enumerate() {
            println!(
                "参数编号： {}/{}, 类型：{:?}, 描述：{}",
                i + 1,
                self.inputs.len(),
                input_arg.input_type,
                input_arg.description
            );
            let res = input_arg.exec_input();
            println!(" - 输入：\"{res:?}\"");
            args.push(res);
        }

        if let Some(check) = self.check_func {
            let passed = check(&args);
            if !passed {
                println!(" - 检查未通过。");
                return;
            }
        }

        match self.confirm {
            ConfirmType::NoConfirm => {}
            ConfirmType::DefaultYes => {
                if !self.inputs.is_empty() {
                    println!("确认以下输入：");
                    for (i, input_arg) in self.inputs.iter().enumerate() {
                        let val = args.get(i).map(|v| format!("{v:?}")).unwrap_or_default();
                        println!(" - 参数{}: {} = {}", i + 1, input_arg.description, val);
                    }
                }
                if !input_confirm("确认？", true) {
                    return;
                }
            }
            ConfirmType::DefaultNo => {
                if !self.inputs.is_empty() {
                    println!("确认以下输入：");
                    for (i, input_arg) in self.inputs.iter().enumerate() {
                        let val = args.get(i).map(|v| format!("{v:?}")).unwrap_or_default();
                        println!(" - 参数{}: {} = {}", i + 1, input_arg.description, val);
                    }
                }
                if !input_confirm("确认？", false) {
                    return;
                }
            }
        }

        (self.exec_func)(&args);
    }
}

fn is_root_dir(paths: &[Box<dyn Any>]) -> bool {
    let path = paths
        .first()
        .and_then(|p| p.downcast_ref::<PathBuf>())
        .unwrap();

    if !path.is_dir() {
        return false;
    }

    let bms_exts = [".bms", ".bme", ".bml", ".pms", ".bmson"];

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file()
                && let Some(name) = p.file_name().and_then(|n| n.to_str())
            {
                let lower = name.to_lowercase();
                if bms_exts.iter().any(|ext| lower.ends_with(ext)) {
                    return false;
                }
            }
        }
    }
    true
}

fn check_ffmpeg(_paths: &[Box<dyn Any>]) -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn check_flac(_paths: &[Box<dyn Any>]) -> bool {
    std::process::Command::new("flac")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn check_oggenc(_paths: &[Box<dyn Any>]) -> bool {
    std::process::Command::new("oggenc")
        .arg("-v")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

#[tokio::main]
async fn main() {
    let mut options: Vec<(String, Vec<MenuOption>)> = Vec::new();

    options.push((
        "BMS活动".to_string(),
        vec![MenuOption {
            name: "BMS活动：跳转至作品信息页".to_string(),
            exec_func: jump_to_work_info,
            inputs: vec![],
            check_func: None,
            confirm: ConfirmType::NoConfirm,
        }],
    ));

    {
        use options::bms_folder::{
            append_artist_name_by_bms, append_name_by_bms, copy_numbered_workdir_names,
            scan_folder_similar_folders, set_name_by_bms, undo_set_name,
        };

        let mut bms_folder_opts: Vec<MenuOption> = Vec::new();

        bms_folder_opts.push(MenuOption {
            name: "BMS根目录：按照BMS设置文件夹名".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = set_name_by_bms(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bms_folder_opts.push(MenuOption {
            name: "BMS根目录：按照BMS追加文件夹名".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = append_name_by_bms(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bms_folder_opts.push(MenuOption {
            name: "BMS根目录：按照BMS追加文件夹艺术家名".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = append_artist_name_by_bms(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bms_folder_opts.push(MenuOption {
            name: "BMS根目录：克隆带编号的文件夹名".to_string(),
            exec_func: |args| {
                let from = args[0].downcast_ref::<PathBuf>().unwrap();
                let to = args[1].downcast_ref::<PathBuf>().unwrap();
                let _ = copy_numbered_workdir_names(from, to);
            },
            inputs: vec![
                Input {
                    input_type: InputType::Path,
                    description: "Src Root Dir".to_string(),
                },
                Input {
                    input_type: InputType::Path,
                    description: "Dst Root Dir".to_string(),
                },
            ],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bms_folder_opts.push(MenuOption {
            name: "BMS根目录：扫描相似文件夹名".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = scan_folder_similar_folders(path, 0.7);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bms_folder_opts.push(MenuOption {
            name: "BMS根目录：撤销重命名".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = undo_set_name(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bms_folder_opts.push(MenuOption {
            name: "BMS根目录：移除大小为0的媒体文件和临时文件".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = options::bms_folder_bigpack::remove_zero_sized_media_files(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        options.push(("BMS根目录".to_string(), bms_folder_opts));
    }

    {
        use options::bms_folder_bigpack::{
            move_out_works, move_works_in_pack, move_works_with_same_name,
            move_works_with_same_name_to_siblings, split_folders_with_first_char, undo_split_pack,
        };

        let mut bigpack_opts: Vec<MenuOption> = Vec::new();

        bigpack_opts.push(MenuOption {
            name: "BMS大包目录：将该目录下的作品，按照首字符分成多个文件夹".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = split_folders_with_first_char(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bigpack_opts.push(MenuOption {
            name: "BMS大包目录：（撤销操作）将该目录下的作品，按照首字符分成多个文件夹".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = undo_split_pack(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "The target folder path.".to_string(),
            }],
            check_func: Some(|args| !is_root_dir(args)),
            confirm: ConfirmType::DefaultYes,
        });

        bigpack_opts.push(MenuOption {
            name: "BMS大包目录：将目录A下的作品，移动到目录B（自动合并）".to_string(),
            exec_func: |args| {
                let from = args[0].downcast_ref::<PathBuf>().unwrap();
                let to = args[1].downcast_ref::<PathBuf>().unwrap();
                let _ = move_works_in_pack(from, to);
            },
            inputs: vec![
                Input {
                    input_type: InputType::Path,
                    description: "From".to_string(),
                },
                Input {
                    input_type: InputType::Path,
                    description: "To".to_string(),
                },
            ],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bigpack_opts.push(MenuOption {
            name: "BMS大包父目录：移出一层目录（自动合并）".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = move_out_works(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Target Root Dir".to_string(),
            }],
            check_func: None,
            confirm: ConfirmType::DefaultYes,
        });

        bigpack_opts.push(MenuOption {
            name: "BMS大包目录：将源文件夹(dir_from)中，文件名相似的子文件夹，合并到目标文件夹(dir_to)中的对应子文件夹".to_string(),
            exec_func: |args| {
                let from = args[0].downcast_ref::<PathBuf>().unwrap();
                let to = args[1].downcast_ref::<PathBuf>().unwrap();
                let _ = move_works_with_same_name(from, to);
            },
            inputs: vec![
                Input { input_type: InputType::Path, description: "From".to_string() },
                Input { input_type: InputType::Path, description: "To".to_string() },
            ],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bigpack_opts.push(MenuOption {
            name: "BMS大包目录：将该目录中文件名相似的子文件夹，合并到各平级目录中".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = move_works_with_same_name_to_siblings(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Dir".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        options.push(("BMS大包目录".to_string(), bigpack_opts));
    }

    {
        use options::bms_folder_event::{
            check_num_folder, create_num_folders, generate_work_info_table,
        };

        let mut event_opts: Vec<MenuOption> = Vec::new();

        event_opts.push(MenuOption {
            name: "BMS活动目录：检查各个的编号对应的文件夹是否存在".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let count = *args[1].downcast_ref::<i32>().unwrap();
                check_num_folder(path, count);
            },
            inputs: vec![
                Input {
                    input_type: InputType::Path,
                    description: "Root Dir:".to_string(),
                },
                Input {
                    input_type: InputType::Int,
                    description: "Create Number:".to_string(),
                },
            ],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        event_opts.push(MenuOption {
            name: "BMS活动目录：创建只带有编号的空文件夹".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let count = *args[1].downcast_ref::<i32>().unwrap();
                let _ = create_num_folders(path, count);
            },
            inputs: vec![
                Input {
                    input_type: InputType::Path,
                    description: "Root Dir:".to_string(),
                },
                Input {
                    input_type: InputType::Int,
                    description: "Create Number:".to_string(),
                },
            ],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        event_opts.push(MenuOption {
            name: "BMS活动目录：生成活动作品的xlsx表格".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = generate_work_info_table(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir:".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        options.push(("BMS活动目录".to_string(), event_opts));
    }

    {
        use options::bms_folder_media::{transfer_audio, transfer_video};

        let mut media_opts: Vec<MenuOption> = Vec::new();

        media_opts.push(MenuOption {
            name: "BMS根目录：音频文件转换".to_string(),
            exec_func: |_args| {
                let path = input_path("");
                Handle::current().block_on(transfer_audio(&path)).ok();
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(|args| {
                is_root_dir(args) && check_ffmpeg(args) && check_flac(args) && check_oggenc(args)
            }),
            confirm: ConfirmType::DefaultYes,
        });

        media_opts.push(MenuOption {
            name: "BMS根目录：视频文件转换".to_string(),
            exec_func: |_args| {
                let path = input_path("");
                Handle::current().block_on(transfer_video(&path)).ok();
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(|args| is_root_dir(args) && check_ffmpeg(args)),
            confirm: ConfirmType::DefaultYes,
        });

        options.push(("BMS媒体".to_string(), media_opts));
    }

    {
        use options::rawpack::{
            set_file_num, unzip_numeric_to_bms_folder, unzip_with_name_to_bms_folder,
        };

        let mut rawpack_opts: Vec<MenuOption> = Vec::new();

        rawpack_opts.push(MenuOption {
            name: "BMS原文件：将赋予编号的文件，解压或放置至指定根目录下，带对应编号的作品目录（自动处理文件夹嵌套）".to_string(),
            exec_func: |args| {
                let pack = args[0].downcast_ref::<PathBuf>().unwrap();
                let cache = args[1].downcast_ref::<PathBuf>().unwrap();
                let root = args[2].downcast_ref::<PathBuf>().unwrap();
                let _ = unzip_numeric_to_bms_folder(pack, cache, root);
            },
            inputs: vec![
                Input { input_type: InputType::Path, description: "Pack Dir".to_string() },
                Input { input_type: InputType::Path, description: "Cache Dir".to_string() },
                Input { input_type: InputType::Path, description: "Root Dir".to_string() },
            ],
            check_func: None,
            confirm: ConfirmType::DefaultYes,
        });

        rawpack_opts.push(MenuOption {
            name: "BMS原文件：将文件，解压或放置至指定根目录下，对应原文件名的作品目录（自动处理文件夹嵌套）".to_string(),
            exec_func: |args| {
                let pack = args[0].downcast_ref::<PathBuf>().unwrap();
                let cache = args[1].downcast_ref::<PathBuf>().unwrap();
                let root = args[2].downcast_ref::<PathBuf>().unwrap();
                let _ = unzip_with_name_to_bms_folder(pack, cache, root);
            },
            inputs: vec![
                Input { input_type: InputType::Path, description: "Pack Dir".to_string() },
                Input { input_type: InputType::Path, description: "Cache Dir".to_string() },
                Input { input_type: InputType::Path, description: "Root Dir".to_string() },
            ],
            check_func: None,
            confirm: ConfirmType::DefaultYes,
        });

        rawpack_opts.push(MenuOption {
            name: "BMS原文件：赋予编号".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = set_file_num(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "RawFile Dir".to_string(),
            }],
            check_func: None,
            confirm: ConfirmType::DefaultYes,
        });

        options.push(("BMS原文件".to_string(), rawpack_opts));
    }

    {
        use scripts::pack::{
            pack_hq_to_lq, pack_raw_to_hq, pack_setup_rawpack_to_hq, pack_update_rawpack_to_hq,
        };

        let mut scripts_opts: Vec<MenuOption> = Vec::new();

        scripts_opts.push(MenuOption {
            name: "大包生成脚本：原包 -> HQ版大包".to_string(),
            exec_func: |args| {
                let pack = args[0].downcast_ref::<PathBuf>().unwrap();
                let root = args[1].downcast_ref::<PathBuf>().unwrap();
                Handle::current().block_on(pack_setup_rawpack_to_hq(pack, root)).ok();
            },
            inputs: vec![
                Input {
                    input_type: InputType::Path,
                    description: "Pack Dir".to_string(),
                },
                Input {
                    input_type: InputType::Path,
                    description: "Root Dir".to_string(),
                },
            ],
            check_func: Some(|args| check_flac(args) && check_ffmpeg(args)),
            confirm: ConfirmType::DefaultYes,
        });

        scripts_opts.push(MenuOption {
            name: "大包更新脚本：原包 -> HQ版大包".to_string(),
            exec_func: |args| {
                let pack = args[0].downcast_ref::<PathBuf>().unwrap();
                let root = args[1].downcast_ref::<PathBuf>().unwrap();
                let sync = args[2].downcast_ref::<PathBuf>().unwrap();
                Handle::current().block_on(pack_update_rawpack_to_hq(pack, root, sync))
                    .ok();
            },
            inputs: vec![
                Input {
                    input_type: InputType::Path,
                    description: "Pack Dir".to_string(),
                },
                Input {
                    input_type: InputType::Path,
                    description: "Root Dir".to_string(),
                },
                Input {
                    input_type: InputType::Path,
                    description: "Sync Dir".to_string(),
                },
            ],
            check_func: Some(|args| check_flac(args) && check_ffmpeg(args)),
            confirm: ConfirmType::DefaultYes,
        });

        scripts_opts.push(MenuOption {
            name: "BMS大包脚本：原包 -> HQ版大包".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                Handle::current().block_on(pack_raw_to_hq(path)).ok();
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(|args| check_flac(args) && check_ffmpeg(args)),
            confirm: ConfirmType::DefaultYes,
        });

        scripts_opts.push(MenuOption {
            name: "BMS大包脚本：HQ版大包 -> LQ版大包".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                Handle::current().block_on(pack_hq_to_lq(path)).ok();
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(|args| check_oggenc(args) && check_ffmpeg(args)),
            confirm: ConfirmType::DefaultYes,
        });

        options.push(("大包脚本".to_string(), scripts_opts));
    }

    let mut option_map: std::collections::HashMap<usize, (&str, usize)> =
        std::collections::HashMap::new();
    let mut current_number = 1usize;

    println!("功能列表如下：");
    for (module_name, module_options) in &options {
        println!("\n【{module_name}】");
        for (i, opt) in module_options.iter().enumerate() {
            option_map.insert(current_number, (module_name.as_str(), i));
            println!(" - {}: {}", current_number, opt.name);
            current_number += 1;
        }
        current_number = ((current_number - 1) / 10 + 1) * 10 + 1;
    }

    // Loop until valid selection (matches Python behavior)
    let selection = loop {
        let selection_str = input_string("\n输入要启用的功能的下标：");
        if let Ok(num) = selection_str.parse::<usize>()
            && option_map.contains_key(&num)
        {
            break num;
        }
        println!("请重新输入");
    };

    if let Some((module_name, opt_idx)) = option_map.get(&selection) {
        let module_idx = options
            .iter()
            .position(|(name, _)| name == module_name)
            .unwrap();
        options[module_idx].1[*opt_idx].exec();
    }
}

/// BMS event types for work information pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BMSEvent {
    /// BOF Team Festival.
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

fn jump_to_work_info(_args: &[Box<dyn Any>]) {
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
