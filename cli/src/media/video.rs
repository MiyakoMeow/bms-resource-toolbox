use std::{
    cell::LazyCell,
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use futures::stream::{self, StreamExt as FuturesStreamExt, TryStreamExt};
use serde::Deserialize;
use smol::{
    fs::{self, remove_file},
    io::{self},
    process::Command,
};
use which::which;

/// Video stream information
#[derive(Debug, Deserialize)]
struct Stream {
    codec_type: String,
    width: Option<i32>,
    height: Option<i32>,
    bit_rate: Option<String>,
}

/// Media file probe result
#[derive(Debug, Deserialize)]
struct MediaProbe {
    streams: Vec<Stream>,
}

/// Video information
#[derive(Debug, Clone)]
#[allow(unused)]
pub struct VideoInfo {
    width: i32,
    height: i32,
    bit_rate: i32,
}

/// Video processing preset configuration
#[derive(Debug, Clone)]
pub struct VideoPreset {
    /// Executor name (e.g., "ffmpeg")
    executor: String,
    /// Input arguments (split tokens)
    input_args: Vec<String>,
    /// Filter arguments (split tokens)
    filter_args: Vec<String>,
    /// Output file extension
    output_ext: String,
    /// Output video codec
    output_codec: String,
    /// Extra arguments (split tokens)
    extra_args: Vec<String>,
}

impl VideoPreset {
    /// Create new video preset
    pub fn new(
        executor: &str,
        input_args: &[&str],
        filter_args: &[&str],
        output_ext: &str,
        output_codec: &str,
        extra_args: &[&str],
    ) -> Self {
        Self {
            executor: executor.to_string(),
            input_args: input_args.iter().map(|s| s.to_string()).collect(),
            filter_args: filter_args.iter().map(|s| s.to_string()).collect(),
            output_ext: output_ext.to_string(),
            output_codec: output_codec.to_string(),
            extra_args: extra_args.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Get output file path
    fn output_path(&self, input_path: &Path) -> PathBuf {
        input_path.with_extension(&self.output_ext)
    }

    /// Build argv for processing video
    fn argv(&self, input_path: &Path, output_path: &Path) -> (String, Vec<String>) {
        let mut argv: Vec<String> = Vec::new();
        argv.extend(self.input_args.clone());
        argv.push(input_path.display().to_string());
        argv.extend(self.filter_args.clone());
        argv.push("-map_metadata".to_string());
        argv.push("0".to_string());
        argv.push("-c:v".to_string());
        argv.push(self.output_codec.clone());
        argv.extend(self.extra_args.clone());
        argv.push(output_path.display().to_string());
        (self.executor.clone(), argv)
    }
}

/// Predefined video processing preset collection
#[allow(clippy::declare_interior_mutable_const)]
pub const VIDEO_PRESETS: LazyCell<HashMap<&'static str, VideoPreset>> = LazyCell::new(|| {
    let mut map = HashMap::new();
    // 512x512 preset
    let filter_complex_512 = "[0:v]scale=512:512:force_original_aspect_ratio=increase,crop=512:512:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=512:512:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]";
    map.insert(
        "AVI_512X512",
        VideoPreset::new(
            "ffmpeg",
            &["-hide_banner", "-i"],
            &["-filter_complex", filter_complex_512, "-map", "[vid]"],
            "avi",
            "mpeg4",
            &["-an", "-q:v", "8"],
        ),
    );
    map.insert(
        "WMV2_512X512",
        VideoPreset::new(
            "ffmpeg",
            &["-hide_banner", "-i"],
            &["-filter_complex", filter_complex_512, "-map", "[vid]"],
            "wmv",
            "wmv2",
            &["-an", "-q:v", "8"],
        ),
    );
    map.insert(
        "MPEG1VIDEO_512X512",
        VideoPreset::new(
            "ffmpeg",
            &["-hide_banner", "-i"],
            &["-filter_complex", filter_complex_512, "-map", "[vid]"],
            "mpg",
            "mpeg1video",
            &["-an", "-b:v", "1500k"],
        ),
    );

    // 480p preset
    let filter_complex_480 = "[0:v]scale=640:480:force_original_aspect_ratio=increase,crop=640:480:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=640:480:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]";
    map.insert(
        "AVI_480P",
        VideoPreset::new(
            "ffmpeg",
            &["-hide_banner", "-i"],
            &["-filter_complex", filter_complex_480, "-map", "[vid]"],
            "avi",
            "mpeg4",
            &["-an", "-q:v", "8"],
        ),
    );
    map.insert(
        "WMV2_480P",
        VideoPreset::new(
            "ffmpeg",
            &["-hide_banner", "-i"],
            &["-filter_complex", filter_complex_480, "-map", "[vid]"],
            "wmv",
            "wmv2",
            &["-an", "-q:v", "8"],
        ),
    );
    map.insert(
        "MPEG1VIDEO_480P",
        VideoPreset::new(
            "ffmpeg",
            &["-hide_banner", "-i"],
            &["-filter_complex", filter_complex_480, "-map", "[vid]"],
            "mpg",
            "mpeg1video",
            &["-an", "-b:v", "1500k"],
        ),
    );

    map
});

/// Get media file information (using ffprobe)
///
/// # Parameters
/// - `file_path`: file path to probe
///
/// # Returns
/// Structure containing media information
async fn get_media_file_probe(file_path: &Path) -> io::Result<MediaProbe> {
    which("ffprobe").map_err(|_| io::Error::other("Executable not found: ffprobe"))?;

    let output = Command::new("ffprobe")
        .args([
            "-show_format",
            "-show_streams",
            "-print_format",
            "json",
            "-v",
            "quiet",
            &file_path.display().to_string(),
        ])
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

/// Get video information
///
/// # Parameters
/// - `file_path`: video file path
///
/// # Returns
/// Video information structure
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

            // Parse bitrate (may be string or number)
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

/// Get video dimensions
///
/// # Parameters
/// - `file_path`: video file path
///
/// # Returns
/// Video width and height tuple
async fn get_video_size(file_path: &Path) -> io::Result<(i32, i32)> {
    let info = get_video_info(file_path).await?;
    Ok((info.width, info.height))
}

/// Get recommended preset list (based on video aspect ratio)
///
/// # Parameters
/// - `file_path`: video file path
///
/// # Returns
/// List of recommended preset names
async fn get_preferred_presets(file_path: &Path) -> io::Result<Vec<&'static str>> {
    let (width, height) = get_video_size(file_path).await?;
    let aspect_ratio = width as f32 / height as f32;
    let target_aspect = 640.0 / 480.0; // Standard aspect ratio for 480p

    if aspect_ratio > target_aspect {
        // Use 480p preset for widescreen videos
        Ok(vec!["MPEG1VIDEO_480P", "WMV2_480P", "AVI_480P"])
    } else {
        // Use 512x512 preset for others
        Ok(vec!["MPEG1VIDEO_512X512", "WMV2_512X512", "AVI_512X512"])
    }
}

/// Process video files in directory
///
/// # Parameters
/// - `dir_path`: target directory path
/// - `input_extensions`: list of input file extensions
/// - `preset_names`: list of preset names
/// - `remove_original`: remove original file on success
/// - `remove_existing`: remove existing output files
/// - `use_preferred`: whether to use recommended presets
///
/// # Returns
/// Whether processing was successful
async fn process_videos_in_directory(
    dir_path: &Path,
    input_extensions: &[&str],
    preset_names: &[&str],
    remove_original: bool,
    remove_existing: bool,
    use_preferred: bool,
) -> io::Result<bool> {
    // Pre-check executors existence for provided presets
    {
        use std::collections::HashSet;
        let mut executors: HashSet<String> = HashSet::new();
        for name in preset_names {
            #[allow(clippy::borrow_interior_mutable_const)]
            if let Some(p) = VIDEO_PRESETS.get(*name) {
                executors.insert(p.executor.clone());
            }
        }
        for exec in executors {
            which(&exec).map_err(|_| io::Error::other(format!("Executable not found: {exec}")))?;
        }
    }
    // Collect tasks
    let mut entries = fs::read_dir(dir_path).await?;
    let mut tasks: Vec<PathBuf> = Vec::new();
    while let Some(entry) = smol::stream::StreamExt::next(&mut entries).await {
        let entry = entry?;
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }
        if let Some(ext) = file_path.extension().and_then(OsStr::to_str)
            && input_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
        {
            tasks.push(file_path);
        }
    }

    let had_error = Arc::new(AtomicBool::new(false));

    // Process concurrently using disk locks（流式）
    stream::iter(tasks)
        .map(|file_path| {
            let had_error = had_error.clone();
            let preset_names = preset_names.to_vec();
            async move {
                log::info!("Processing video: {}", file_path.display());

                // Choose preset
                let mut presets_to_try = preset_names;
                if use_preferred && let Ok(preferred) = get_preferred_presets(&file_path).await {
                    presets_to_try = preferred;
                    presets_to_try.extend(presets_to_try.clone());
                }

                let mut success = false;
                for preset_name in &presets_to_try {
                    #[allow(clippy::borrow_interior_mutable_const)]
                    let Some(preset) = VIDEO_PRESETS.get(*preset_name).cloned() else {
                        continue;
                    };

                    let output_path = preset.output_path(&file_path);
                    if file_path == output_path {
                        continue;
                    }

                    if output_path.exists() {
                        if remove_existing {
                            if let Err(e) = remove_file(&output_path).await {
                                eprintln!("Failed to remove existing file: {e}");
                            }
                        } else {
                            log::info!("Output file exists, skipping: {}", output_path.display());
                            continue;
                        }
                    }

                    let (program, argv) = preset.argv(&file_path, &output_path);
                    log::info!("Executing: {} {:?}", program, argv);

                    let output = Command::new(&program).args(&argv).output().await;

                    match output {
                        Ok(output) if output.status.success() => {
                            log::info!("Successfully converted: {}", output_path.display());
                            success = true;
                            if remove_original && let Err(e) = { remove_file(&file_path).await } {
                                eprintln!("Failed to remove original file: {e}");
                            }
                            break;
                        }
                        Ok(output) => {
                            eprintln!(
                                "Conversion failed for preset {}: {}",
                                preset_name,
                                String::from_utf8_lossy(&output.stderr)
                            );
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
                    had_error.store(true, Ordering::Relaxed);
                    eprintln!("All presets failed for: {}", file_path.display());
                }

                Ok::<(), io::Error>(())
            }
        })
        .buffer_unordered(64)
        .try_for_each(|_| async { Ok(()) })
        .await?;

    Ok(!had_error.load(Ordering::Relaxed))
}

/// Process all BMS folders under root directory
///
/// # Parameters
/// - `root_dir`: root directory path
/// - `input_extensions`: list of input file extensions
/// - `preset_names`: list of preset names
/// - `remove_original`: remove original file on success
/// - `remove_existing`: remove existing output files
/// - `use_preferred`: whether to use recommended presets
pub async fn process_bms_video_folders(
    root_dir: &Path,
    input_extensions: &[&str],
    preset_names: &[&str],
    remove_original: bool,
    remove_existing: bool,
    use_preferred: bool,
) -> io::Result<()> {
    // Validate preset names
    for name in preset_names {
        #[allow(clippy::borrow_interior_mutable_const)]
        if !VIDEO_PRESETS.contains_key(*name) {
            return Err(io::Error::other(format!("Invalid preset name: {name}")));
        }
    }

    let mut entries = fs::read_dir(root_dir).await?;
    while let Some(entry) = smol::stream::StreamExt::next(&mut entries).await {
        let entry = entry?;
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        log::info!("Processing BMS folder: {}", dir_path.display());

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
            Ok(true) => log::info!("Successfully processed {}", dir_path.display()),
            Ok(false) => eprintln!("Errors occurred in {}", dir_path.display()),
            Err(e) => eprintln!("Error processing {}: {}", dir_path.display(), e),
        }
    }

    Ok(())
}

// compute_parallelism_for_dir has been moved to crate::fs module
