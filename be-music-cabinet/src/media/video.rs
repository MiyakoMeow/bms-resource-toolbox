use std::{
    cell::LazyCell,
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;
use smol::{
    fs::{self, remove_file},
    io::{self},
    lock::Semaphore,
    process::Command,
    stream::StreamExt,
};

/// 视频流信息
#[derive(Debug, Deserialize)]
struct Stream {
    codec_type: String,
    width: Option<i32>,
    height: Option<i32>,
    bit_rate: Option<String>,
}

/// 媒体文件探测结果
#[derive(Debug, Deserialize)]
struct MediaProbe {
    streams: Vec<Stream>,
}

/// 视频信息
#[derive(Debug, Clone)]
#[allow(unused)]
pub struct VideoInfo {
    width: i32,
    height: i32,
    bit_rate: i32,
}

/// 视频处理预设配置
#[derive(Debug, Clone)]
pub struct VideoPreset {
    /// 执行器名称 (如 "ffmpeg")
    executor: String,
    /// 输入参数
    input_args: String,
    /// 滤镜参数
    filter_args: String,
    /// 输出文件扩展名
    output_ext: String,
    /// 输出视频编码
    output_codec: String,
    /// 附加参数
    extra_args: String,
}

impl VideoPreset {
    /// 创建新的视频预设
    pub fn new(
        executor: &str,
        input_args: &str,
        filter_args: &str,
        output_ext: &str,
        output_codec: &str,
        extra_args: &str,
    ) -> Self {
        Self {
            executor: executor.to_string(),
            input_args: input_args.to_string(),
            filter_args: filter_args.to_string(),
            output_ext: output_ext.to_string(),
            output_codec: output_codec.to_string(),
            extra_args: extra_args.to_string(),
        }
    }

    /// 获取输出文件路径
    fn output_path(&self, input_path: &Path) -> PathBuf {
        input_path.with_extension(&self.output_ext)
    }

    /// 获取处理视频的命令
    fn command(&self, input_path: &Path, output_path: &Path) -> String {
        format!(
            "{} {} \"{}\" {} -map_metadata 0 -c:v {} {} \"{}\"",
            self.executor,
            self.input_args,
            input_path.display(),
            self.filter_args,
            self.output_codec,
            self.extra_args,
            output_path.display()
        )
    }
}

/// 预定义的视频处理预设集合
#[allow(clippy::declare_interior_mutable_const)]
pub const VIDEO_PRESETS: LazyCell<HashMap<&'static str, VideoPreset>> = LazyCell::new(|| {
    let mut map = HashMap::new();
    // 512x512 预设
    let filter_512 = r#"-filter_complex "[0:v]scale=512:512:force_original_aspect_ratio=increase,crop=512:512:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=512:512:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]" -map [vid]"#;
    map.insert(
        "AVI_512X512",
        VideoPreset::new(
            "ffmpeg",
            "-hide_banner -i",
            filter_512,
            "avi",
            "mpeg4",
            "-an -q:v 8",
        ),
    );
    map.insert(
        "WMV2_512X512",
        VideoPreset::new(
            "ffmpeg",
            "-hide_banner -i",
            filter_512,
            "wmv",
            "wmv2",
            "-an -q:v 8",
        ),
    );
    map.insert(
        "MPEG1VIDEO_512X512",
        VideoPreset::new(
            "ffmpeg",
            "-hide_banner -i",
            filter_512,
            "mpg",
            "mpeg1video",
            "-an -b:v 1500k",
        ),
    );

    // 480p 预设
    let filter_480 = r#"-filter_complex "[0:v]scale=640:480:force_original_aspect_ratio=increase,crop=640:480:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=640:480:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]" -map [vid]"#;
    map.insert(
        "AVI_480P",
        VideoPreset::new(
            "ffmpeg",
            "-hide_banner -i",
            filter_480,
            "avi",
            "mpeg4",
            "-an -q:v 8",
        ),
    );
    map.insert(
        "WMV2_480P",
        VideoPreset::new(
            "ffmpeg",
            "-hide_banner -i",
            filter_480,
            "wmv",
            "wmv2",
            "-an -q:v 8",
        ),
    );
    map.insert(
        "MPEG1VIDEO_480P",
        VideoPreset::new(
            "ffmpeg",
            "-hide_banner -i",
            filter_480,
            "mpg",
            "mpeg1video",
            "-an -b:v 1500k",
        ),
    );

    map
});

/// 获取媒体文件信息 (使用 ffprobe)
///
/// # 参数
/// - `file_path`: 要探测的文件路径
///
/// # 返回值
/// 包含媒体信息的结构体
async fn get_media_file_probe(file_path: &Path) -> io::Result<MediaProbe> {
    let cmd = format!(
        "ffprobe -show_format -show_streams -print_format json -v quiet \"{}\"",
        file_path.display()
    );

    #[cfg(target_family = "windows")]
    let program = "powershell";
    #[cfg(not(target_family = "windows"))]
    let program = "sh";
    let output = Command::new(program)
        .arg("-c")
        .arg(&cmd)
        .output()
        .await
        .map_err(|_| io::Error::other("Failed to execute ffprobe command"))?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "ffprobe failed with status: {}\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let probe: MediaProbe = serde_json::from_str(&json_str)
        .map_err(|_| io::Error::other("Failed to parse ffprobe JSON"))?;

    Ok(probe)
}

/// 获取视频信息
///
/// # 参数
/// - `file_path`: 视频文件路径
///
/// # 返回值
/// 视频信息结构体
async fn get_video_info(file_path: &Path) -> io::Result<VideoInfo> {
    let probe = get_media_file_probe(file_path).await?;

    for stream in probe.streams {
        if stream.codec_type == "video" {
            let width = stream
                .width
                .ok_or(io::Error::other("Missing width in video stream"))?;
            let height = stream
                .height
                .ok_or(io::Error::other("Missing height in video stream"))?;

            // 解析比特率 (可能为字符串或数字)
            let bit_rate = stream
                .bit_rate
                .as_ref()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            return Ok(VideoInfo {
                width,
                height,
                bit_rate,
            });
        }
    }

    Err(io::Error::other("No video stream found in file"))
}

/// 获取视频尺寸
///
/// # 参数
/// - `file_path`: 视频文件路径
///
/// # 返回值
/// 视频宽高元组
async fn get_video_size(file_path: &Path) -> io::Result<(i32, i32)> {
    let info = get_video_info(file_path).await?;
    Ok((info.width, info.height))
}

/// 获取推荐预设列表 (基于视频宽高比)
///
/// # 参数
/// - `file_path`: 视频文件路径
///
/// # 返回值
/// 推荐的预设名称列表
async fn get_preferred_presets(file_path: &Path) -> io::Result<Vec<&'static str>> {
    let (width, height) = get_video_size(file_path).await?;
    let aspect_ratio = width as f32 / height as f32;
    let target_aspect = 640.0 / 480.0; // 480p的标准宽高比

    if aspect_ratio > target_aspect {
        // 宽屏视频使用480p预设
        Ok(vec!["MPEG1VIDEO_480P", "WMV2_480P", "AVI_480P"])
    } else {
        // 其他使用512x512预设
        Ok(vec!["MPEG1VIDEO_512X512", "WMV2_512X512", "AVI_512X512"])
    }
}

/// 处理目录中的视频文件
///
/// # 参数
/// - `dir_path`: 目标目录路径
/// - `input_extensions`: 输入文件扩展名列表
/// - `preset_names`: 预设名称列表
/// - `remove_original`: 成功后删除原文件
/// - `remove_existing`: 删除已存在的输出文件
/// - `use_preferred`: 是否使用推荐预设
///
/// # 返回值
/// 处理是否成功
async fn process_videos_in_directory(
    dir_path: &Path,
    input_extensions: &[&str],
    preset_names: &[&str],
    remove_original: bool,
    remove_existing: bool,
    use_preferred: bool,
) -> io::Result<bool> {
    let mut entries = fs::read_dir(dir_path).await?;
    let mut has_error = false;

    // 使用信号量限制并发量 (避免过多视频同时转换)
    let semaphore = Arc::new(Semaphore::new(2)); // 同时最多处理2个视频

    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        // 检查文件扩展名
        if let Some(ext) = file_path.extension().and_then(OsStr::to_str) {
            if !input_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                continue;
            }
        } else {
            continue;
        }

        println!("Processing video: {}", file_path.display());

        // 根据配置确定使用的预设
        let mut presets_to_try = preset_names.to_vec();
        if use_preferred {
            if let Ok(preferred) = get_preferred_presets(&file_path).await {
                presets_to_try = preferred;
                presets_to_try.extend(preset_names); // 添加原始预设作为备选
            }
        }

        // 尝试每个预设直到成功
        let mut success = false;
        for preset_name in &presets_to_try {
            #[allow(clippy::borrow_interior_mutable_const)]
            let Some(preset) = VIDEO_PRESETS.get(*preset_name).cloned() else {
                continue;
            };

            let output_path = preset.output_path(&file_path);
            if file_path == output_path {
                continue; // 跳过相同的输入输出
            }

            // 检查输出文件是否存在
            if output_path.exists() {
                if remove_existing {
                    if let Err(e) = remove_file(&output_path).await {
                        eprintln!("Failed to remove existing file: {e}");
                    }
                } else {
                    println!("Output file exists, skipping: {}", output_path.display());
                    continue;
                }
            }

            // 获取信号量许可
            let permit = semaphore.clone().acquire_arc().await;

            // 执行转换命令
            let cmd = preset.command(&file_path, &output_path);
            println!("Executing: {cmd}");

            let output = Command::new("sh").arg("-c").arg(&cmd).output().await;

            drop(permit); // 释放信号量

            match output {
                Ok(output) if output.status.success() => {
                    println!("Successfully converted: {}", output_path.display());
                    success = true;

                    // 删除原文件
                    if remove_original {
                        if let Err(e) = remove_file(&file_path).await {
                            eprintln!("Failed to remove original file: {e}");
                        }
                    }
                    break; // 成功，跳出预设循环
                }
                Ok(output) => {
                    // 转换失败
                    eprintln!(
                        "Conversion failed for preset {}: {}",
                        preset_name,
                        String::from_utf8_lossy(&output.stderr)
                    );

                    // 清理失败的输出文件
                    if output_path.exists() {
                        let _ = remove_file(&output_path).await;
                    }
                }
                Err(e) => {
                    eprintln!("Command execution error: {e}");
                }
            }
        }

        if !success {
            has_error = true;
            eprintln!("All presets failed for: {}", file_path.display());
        }
    }

    Ok(!has_error)
}

/// 处理根目录下的所有BMS文件夹
///
/// # 参数
/// - `root_dir`: 根目录路径
/// - `input_extensions`: 输入文件扩展名列表
/// - `preset_names`: 预设名称列表
/// - `remove_original`: 成功后删除原文件
/// - `remove_existing`: 删除已存在的输出文件
/// - `use_preferred`: 是否使用推荐预设
pub async fn process_bms_video_folders(
    root_dir: &Path,
    input_extensions: &[&str],
    preset_names: &[&str],
    remove_original: bool,
    remove_existing: bool,
    use_preferred: bool,
) -> io::Result<()> {
    // 验证预设名称
    for name in preset_names {
        #[allow(clippy::borrow_interior_mutable_const)]
        if !VIDEO_PRESETS.contains_key(*name) {
            return Err(io::Error::other(format!("Invalid preset name: {name}")));
        }
    }

    let mut entries = fs::read_dir(root_dir).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        println!("Processing BMS folder: {}", dir_path.display());

        match process_videos_in_directory(
            &dir_path,
            input_extensions,
            preset_names,
            remove_original,
            remove_existing,
            use_preferred,
        )
        .await
        {
            Ok(true) => println!("Successfully processed {}", dir_path.display()),
            Ok(false) => eprintln!("Errors occurred in {}", dir_path.display()),
            Err(e) => eprintln!("Error processing {}: {}", dir_path.display(), e),
        }
    }

    Ok(())
}
