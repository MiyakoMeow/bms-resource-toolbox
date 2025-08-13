use std::{
    cell::LazyCell,
    collections::HashMap,
    ffi::OsStr,
    path::Path,
    process::{ExitStatus, Output},
    sync::Arc,
};

use smol::{
    fs::{self, remove_file},
    io,
    lock::Semaphore,
    process::Command,
    stream::StreamExt,
};

/// 音频处理预设配置
#[derive(Debug, Clone)]
pub struct AudioPreset {
    /// 执行器名称 (如 "ffmpeg", "oggenc")
    executor: String,
    /// 输出格式 (如 "ogg", "flac")
    output_format: String,
    /// 附加参数 (可为空)
    arguments: Option<String>,
}

impl AudioPreset {
    /// 创建新的音频预设
    fn new(executor: &str, output_format: &str, arguments: Option<&str>) -> Self {
        Self {
            executor: executor.to_string(),
            output_format: output_format.to_string(),
            arguments: arguments.map(|s| s.to_string()),
        }
    }
}

#[allow(clippy::declare_interior_mutable_const)]
pub const AUDIO_PRESETS: LazyCell<HashMap<&'static str, AudioPreset>> = LazyCell::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "FLAC",
        AudioPreset::new(
            "flac",
            "flac",
            Some("--keep-foreign-metadata-if-present --best -f"),
        ),
    );
    map.insert("FLAC_FFMPEG", AudioPreset::new("ffmpeg", "flac", None));
    map.insert(
        "WAV_FROM_FLAC",
        AudioPreset::new(
            "flac",
            "wav",
            Some("-d --keep-foreign-metadata-if-present -f"),
        ),
    );
    map.insert("WAV_FFMPEG", AudioPreset::new("ffmpeg", "wav", None));
    map.insert("OGG_Q10", AudioPreset::new("oggenc", "ogg", Some("-q10")));
    map.insert("OGG_FFMPEG", AudioPreset::new("ffmpeg", "ogg", None));
    map
});

/// 获取处理音频文件的命令字符串
///
/// # 参数
/// - `input_path`: 输入文件路径
/// - `output_path`: 输出文件路径
/// - `preset`: 使用的音频预设
///
/// # 返回值
/// 生成的命令行字符串
fn get_audio_command(
    input_path: &Path,
    output_path: &Path,
    preset: &AudioPreset,
) -> Option<String> {
    match preset.executor.as_str() {
        "ffmpeg" => {
            let args = preset.arguments.as_deref().unwrap_or("");
            Some(format!(
                "ffmpeg -hide_banner -loglevel panic -i \"{}\" -f {} -map_metadata 0 {} \"{}\"",
                input_path.display(),
                preset.output_format,
                args,
                output_path.display()
            ))
        }
        "oggenc" => {
            let args = preset.arguments.as_deref().unwrap_or("");
            Some(format!(
                "oggenc {} \"{}\" -o \"{}\"",
                args,
                input_path.display(),
                output_path.display()
            ))
        }
        "flac" => {
            let args = preset.arguments.as_deref().unwrap_or("");
            Some(format!(
                "flac {} \"{}\" -o \"{}\"",
                args,
                input_path.display(),
                output_path.display()
            ))
        }
        _ => None,
    }
}

/// 在指定目录中转换音频文件
///
/// # 参数
/// - `dir_path`: 目标目录路径
/// - `input_extensions`: 要处理的输入文件扩展名列表
/// - `presets`: 按顺序尝试的预设列表
/// - `remove_on_success`: 转换成功后删除原文件
/// - `remove_on_fail`: 所有尝试失败后删除原文件
/// - `remove_existing`: 是否覆盖已存在的输出文件
///
/// # 返回值
/// 转换操作是否完全成功
async fn transfer_audio_in_directory(
    dir_path: &Path,
    input_extensions: &[&str],
    presets: &[AudioPreset],
    remove_on_success: bool,
    remove_on_fail: bool,
    remove_existing: bool,
) -> io::Result<bool> {
    let mut tasks = Vec::new();
    let mut total_files = 0;
    let mut fallback_files = Vec::new();
    let mut has_error = false;
    let mut last_error = None;

    // 根据是否在C盘确定并发限制（非C盘使用更多线程）
    let is_hdd = !dir_path.starts_with("C:");
    let max_workers = if is_hdd {
        num_cpus::get().min(24)
    } else {
        num_cpus::get()
    };
    let semaphore = Arc::new(Semaphore::new(max_workers));

    // 收集需要处理的文件
    let mut entries = fs::read_dir(dir_path).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension().and_then(OsStr::to_str)
            && input_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
        {
            total_files += 1;
            tasks.push(path.clone());
        }
    }

    if total_files > 0 {
        println!(
            "Entering dir: {}, Input extensions: {:?}",
            dir_path.display(),
            input_extensions
        );
        println!("Using presets: {presets:?}");
    }

    // 处理每个文件
    for file_path in tasks {
        let mut current_preset_index = 0;
        let mut success = false;

        while current_preset_index < presets.len() {
            let preset = &presets[current_preset_index];
            let output_path = file_path.with_extension(&preset.output_format);

            // 检查输出文件是否存在
            if output_path.exists() {
                if remove_existing {
                    if let Ok(metadata) = fs::metadata(&output_path).await
                        && metadata.len() > 0
                    {
                        println!("Removing existing file: {}", output_path.display());
                        let _ = remove_file(&output_path).await;
                    }
                } else {
                    println!("Skipping existing file: {}", output_path.display());
                    current_preset_index += 1;
                    continue;
                }
            }

            // 获取并执行命令
            if let Some(cmd) = get_audio_command(&file_path, &output_path, preset) {
                let permit = semaphore.clone().acquire_arc().await;

                // 执行转换命令
                #[cfg(target_family = "windows")]
                let program = "powershell";
                #[cfg(not(target_family = "windows"))]
                let program = "sh";
                let output = Command::new(program).arg("-c").arg(&cmd).output().await;

                drop(permit); // 释放信号量

                match output {
                    Ok(output) if output.status.success() => {
                        // 转换成功
                        if remove_on_success && let Err(e) = remove_file(&file_path).await {
                            eprintln!(
                                "Error deleting original file: {} - {}",
                                file_path.display(),
                                e
                            );
                        }
                        success = true;
                        break;
                    }
                    Ok(output) => {
                        // 转换失败
                        println!(
                            "Preset failed [{}]: {} -> {}",
                            preset.executor,
                            file_path.display(),
                            output_path.display()
                        );
                        last_error = Some((file_path.clone(), output));
                    }
                    Err(e) => {
                        // 命令执行错误
                        eprintln!("Command execution error: {e}");
                        last_error = Some((
                            file_path.clone(),
                            Output {
                                status: ExitStatus::default(),
                                stdout: Vec::new(),
                                stderr: e.to_string().into_bytes(),
                            },
                        ));
                    }
                }
            }

            current_preset_index += 1;
        }

        if !success {
            has_error = true;
            fallback_files.push(file_path.file_name().unwrap().to_string_lossy().to_string());

            // 所有预设尝试失败后删除原文件
            if remove_on_fail && let Err(e) = remove_file(&file_path).await {
                eprintln!(
                    "Error deleting failed file: {} - {}",
                    file_path.display(),
                    e
                );
            }
        }
    }

    // 输出处理结果
    if total_files > 0 {
        println!("Processed {} files in {}", total_files, dir_path.display());
    }
    if !fallback_files.is_empty() {
        println!(
            "{} files failed all presets: {:?}",
            fallback_files.len(),
            fallback_files
        );
    }
    if has_error {
        if let Some((err_path, output)) = last_error {
            eprintln!("Last error on file: {}", err_path.display());
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        if remove_on_fail {
            println!("Original files for failed conversions were removed");
        }
    }

    Ok(!has_error)
}

/// 处理根目录下的所有BMS文件夹
///
/// # 参数
/// - `root_dir`: 根目录路径
/// - `input_extensions`: 输入文件扩展名列表
/// - `preset_names`: 要使用的预设名称列表
/// - `remove_on_success`: 成功时删除原文件
/// - `remove_on_fail`: 失败时删除原文件
/// - `skip_on_fail`: 遇到错误时跳过后续处理
pub async fn process_bms_folders(
    root_dir: &Path,
    input_extensions: &[&str],
    preset_names: &[&str],
    remove_on_success: bool,
    remove_on_fail: bool,
    skip_on_fail: bool,
) -> io::Result<()> {
    // 将预设名称解析为预设对象
    let presets: Vec<AudioPreset> = preset_names
        .iter()
        .filter_map(|name| {
            let binding = AUDIO_PRESETS;
            let preset = binding.get(name);
            preset.cloned()
        })
        .collect();

    if presets.is_empty() {
        io::Error::other("No valid presets provided");
    }

    // 遍历根目录下的所有子目录
    let mut entries = fs::read_dir(root_dir).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        println!("Processing directory: {}", dir_path.display());
        match transfer_audio_in_directory(
            &dir_path,
            input_extensions,
            &presets,
            remove_on_success,
            remove_on_fail,
            true, // 总是覆盖已存在文件
        )
        .await
        {
            Ok(true) => println!("Successfully processed {}", dir_path.display()),
            Ok(false) => {
                eprintln!("Errors occurred in {}", dir_path.display());
                if skip_on_fail {
                    eprintln!("Skipping remaining folders due to error");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", dir_path.display(), e);
                if skip_on_fail {
                    break;
                }
            }
        }
    }

    Ok(())
}
