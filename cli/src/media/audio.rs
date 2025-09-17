use std::{
    cell::LazyCell,
    collections::HashMap,
    ffi::OsStr,
    path::Path,
    sync::{Arc, Mutex},
};

use futures::stream::StreamExt as _;
use futures::stream::{self, TryStreamExt};
use smol::{
    fs::{self, remove_file},
    io,
    process::Command,
};
use std::sync::atomic::{AtomicBool, Ordering};
use which::which;

/// Audio processing preset configuration
#[derive(Debug, Clone)]
pub struct AudioPreset {
    /// Executor name (e.g., "ffmpeg", "oggenc")
    executor: String,
    /// Output format (e.g., "ogg", "flac")
    output_format: String,
    /// Additional arguments (split, no shell quoting needed)
    arguments: Option<Vec<String>>,
}

impl AudioPreset {
    /// Create new audio preset
    fn new(executor: &str, output_format: &str, arguments: Option<&[&str]>) -> Self {
        Self {
            executor: executor.to_string(),
            output_format: output_format.to_string(),
            arguments: arguments.map(|arr| arr.iter().map(|s| s.to_string()).collect()),
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
            Some(&["--keep-foreign-metadata-if-present", "--best", "-f"]),
        ),
    );
    map.insert("FLAC_FFMPEG", AudioPreset::new("ffmpeg", "flac", None));
    map.insert(
        "WAV_FROM_FLAC",
        AudioPreset::new(
            "flac",
            "wav",
            Some(&["-d", "--keep-foreign-metadata-if-present", "-f"]),
        ),
    );
    map.insert("WAV_FFMPEG", AudioPreset::new("ffmpeg", "wav", None));
    map.insert(
        "OGG_Q10",
        AudioPreset::new("oggenc", "ogg", Some(&["-q10"])),
    );
    map.insert("OGG_FFMPEG", AudioPreset::new("ffmpeg", "ogg", None));
    map
});

/// Build executable and argv for processing audio files
///
/// # Parameters
/// - `input_path`: input file path
/// - `output_path`: output file path
/// - `preset`: audio preset to use
///
/// # Returns
/// Program name and argv vector for execution
fn build_audio_command(
    input_path: &Path,
    output_path: &Path,
    preset: &AudioPreset,
) -> Option<(String, Vec<String>)> {
    match preset.executor.as_str() {
        "ffmpeg" => {
            let mut argv: Vec<String> = vec![
                "-hide_banner".to_string(),
                "-loglevel".to_string(),
                "panic".to_string(),
                "-i".to_string(),
                input_path.display().to_string(),
                "-f".to_string(),
                preset.output_format.clone(),
                "-map_metadata".to_string(),
                "0".to_string(),
            ];
            if let Some(extra) = &preset.arguments {
                argv.extend(extra.clone());
            }
            argv.push(output_path.display().to_string());
            Some(("ffmpeg".to_string(), argv))
        }
        "oggenc" => {
            let mut argv: Vec<String> = Vec::new();
            if let Some(extra) = &preset.arguments {
                argv.extend(extra.clone());
            }
            argv.push(input_path.display().to_string());
            argv.push("-o".to_string());
            argv.push(output_path.display().to_string());
            Some(("oggenc".to_string(), argv))
        }
        "flac" => {
            let mut argv: Vec<String> = Vec::new();
            if let Some(extra) = &preset.arguments {
                argv.extend(extra.clone());
            }
            argv.push(input_path.display().to_string());
            argv.push("-o".to_string());
            argv.push(output_path.display().to_string());
            Some(("flac".to_string(), argv))
        }
        _ => None,
    }
}

/// Convert audio files in specified directory
///
/// # Parameters
/// - `dir_path`: target directory path
/// - `input_extensions`: list of input file extensions to process
/// - `presets`: list of presets to try in order
/// - `remove_on_success`: remove original file after successful conversion
/// - `remove_on_fail`: remove original file after all attempts fail
/// - `remove_existing`: whether to overwrite existing output files
///
/// # Returns
/// Whether the conversion operation was completely successful
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
    let failures: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let had_error = Arc::new(AtomicBool::new(false));

    // Collect files to process
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
        log::info!(
            "Entering dir: {}, Input extensions: {:?}",
            dir_path.display(),
            input_extensions
        );
        log::info!("Using presets: {presets:?}");
    }

    // Pre-check executors existence
    {
        use std::collections::HashSet;
        let mut executors: HashSet<&str> = HashSet::new();
        for p in presets {
            executors.insert(p.executor.as_str());
        }
        for exec in executors {
            which(exec).map_err(|_| io::Error::other(format!("Executable not found: {exec}")))?;
        }
    }

    // Process each file concurrently using disk locks（流式）
    let failures_cloned = failures.clone();
    let had_error_cloned = had_error.clone();
    stream::iter(tasks)
        .map(|file_path| {
            let failures = failures_cloned.clone();
            let had_error = had_error_cloned.clone();
            let presets = presets.to_vec();
            async move {
                let mut current_preset_index = 0;
                let mut success = false;

                while current_preset_index < presets.len() {
                    let preset = &presets[current_preset_index];
                    let output_path = file_path.with_extension(&preset.output_format);

                    // If target file already exists
                    if output_path.exists() {
                        if remove_existing {
                            if let Ok(metadata) = fs::metadata(&output_path).await
                                && metadata.len() > 0
                            {
                                log::info!("Removing existing file: {}", output_path.display());
                                // Lock only when deleting target
                                let _ = remove_file(&output_path).await;
                            }
                        } else {
                            log::info!("Skipping existing file: {}", output_path.display());
                            current_preset_index += 1;
                            continue;
                        }
                    }

                    // Execute command directly without shell
                    if let Some((program, argv)) =
                        build_audio_command(&file_path, &output_path, preset)
                    {
                        let output = Command::new(&program).args(&argv).output().await;

                        match output {
                            Ok(output) if output.status.success() => {
                                if remove_on_success
                                    && let Err(e) = {
                                        // Lock only when deleting source
                                        remove_file(&file_path).await
                                    }
                                {
                                    eprintln!(
                                        "Error deleting original file: {} - {}",
                                        file_path.display(),
                                        e
                                    );
                                }
                                success = true;
                                break;
                            }
                            Ok(_) => {
                                log::info!(
                                    "Preset failed [{}]: {} -> {}",
                                    preset.executor,
                                    file_path.display(),
                                    output_path.display()
                                );
                            }
                            Err(e) => {
                                eprintln!("Command execution error: {e}");
                            }
                        }
                    }

                    current_preset_index += 1;
                }

                if !success {
                    had_error.store(true, Ordering::Relaxed);
                    if let Some(name) = file_path.file_name().and_then(|s| s.to_str()) {
                        let mut guard = failures.lock().unwrap();
                        guard.push(name.to_string());
                    }
                    if remove_on_fail && let Err(e) = { remove_file(&file_path).await } {
                        eprintln!(
                            "Error deleting failed file: {} - {}",
                            file_path.display(),
                            e
                        );
                    }
                }

                Ok::<(), io::Error>(())
            }
        })
        .buffer_unordered(64)
        .try_for_each(|_| async { Ok(()) })
        .await?;

    // Output processing results
    if total_files > 0 {
        log::info!("Processed {} files in {}", total_files, dir_path.display());
    }
    let failures = failures.lock().unwrap();
    if !failures.is_empty() {
        log::info!(
            "{} files failed all presets: {:?}",
            failures.len(),
            *failures
        );
    }
    if had_error.load(Ordering::Relaxed) && remove_on_fail {
        log::info!("Original files for failed conversions were removed");
    }

    Ok(!had_error.load(Ordering::Relaxed))
}

/// Process all BMS folders under root directory
///
/// # Parameters
/// - `root_dir`: root directory path
/// - `input_extensions`: list of input file extensions
/// - `preset_names`: list of preset names to use
/// - `remove_on_success`: remove original file on success
/// - `remove_on_fail`: remove original file on failure
/// - `skip_on_fail`: skip subsequent processing on error
pub async fn process_bms_folders(
    root_dir: &Path,
    input_extensions: &[&str],
    preset_names: &[&str],
    remove_on_success: bool,
    remove_on_fail: bool,
    skip_on_fail: bool,
) -> io::Result<()> {
    // Parse preset names into preset objects
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

    // Iterate through all subdirectories under root directory
    let mut entries = fs::read_dir(root_dir).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        log::info!("Processing directory: {}", dir_path.display());
        match transfer_audio_in_directory(
            &dir_path,
            input_extensions,
            &presets,
            remove_on_success,
            remove_on_fail,
            true, // Always overwrite existing files
        )
        .await
        {
            Ok(true) => log::info!("Successfully processed {}", dir_path.display()),
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

// compute_parallelism_for_dir has been moved to crate::fs module
