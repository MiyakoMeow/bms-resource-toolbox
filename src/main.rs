//! BMS Resource Toolbox - Interactive menu interface
//!
//! This module provides an interactive CLI that replicates the behavior
//! of bms-resource-scripts main.py exactly.

#![expect(clippy::too_many_lines)]

use std::any::Any;
use std::path::PathBuf;

use tokio::runtime::Handle;

use options::bms_events::jump_to_work_info;
use options::input::{input_confirm, input_path, input_string};

mod bms;
mod error;
mod fs;
mod media;
mod options;
mod scripts;

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
                && bms_exts
                    .iter()
                    .any(|ext| name.to_lowercase().ends_with(ext))
            {
                return false;
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
            remove_zero_sized_media_files, scan_folder_similar_folders, set_name_by_bms,
            undo_set_name,
        };

        let mut bms_folder_opts: Vec<MenuOption> = Vec::new();

        bms_folder_opts.push(MenuOption {
            name: "BMS根目录：按照BMS设置文件夹名".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = Handle::current().block_on(set_name_by_bms(path));
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
                let _ = Handle::current().block_on(append_name_by_bms(path));
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
                let _ = Handle::current().block_on(append_artist_name_by_bms(path));
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
                let _ = remove_zero_sized_media_files(path, false);
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
            merge_split_folders, move_out_works, move_works_in_pack, move_works_with_same_name,
            move_works_with_same_name_to_siblings, remove_unneed_media_files,
            split_folders_with_first_char, undo_split_pack,
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

        bigpack_opts.push(MenuOption {
            name: "BMS大包目录：合并已分割的文件夹（撤销按首字符分割）".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = merge_split_folders(path);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
            }],
            check_func: Some(is_root_dir),
            confirm: ConfirmType::DefaultYes,
        });

        bigpack_opts.push(MenuOption {
            name: "BMS大包目录：移除冗余媒体文件（按预设规则）".to_string(),
            exec_func: |args| {
                let path = args[0].downcast_ref::<PathBuf>().unwrap();
                let _ = remove_unneed_media_files(path, None);
            },
            inputs: vec![Input {
                input_type: InputType::Path,
                description: "Root Dir".to_string(),
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
                let _ = Handle::current().block_on(generate_work_info_table(path));
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
                let _ = unzip_numeric_to_bms_folder(pack, cache, root, false);
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
                let _ = unzip_with_name_to_bms_folder(pack, cache, root, false);
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
                Handle::current()
                    .block_on(pack_setup_rawpack_to_hq(pack, root))
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
                Handle::current()
                    .block_on(pack_update_rawpack_to_hq(pack, root, sync))
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
