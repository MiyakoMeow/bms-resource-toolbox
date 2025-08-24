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

/// Audio processing preset configuration
#[derive(Debug, Clone)]
pub struct AudioPreset {
    /// Executor name (e.g., "ffmpeg", "oggenc")
    executor: String,
    /// Output format (e.g., "ogg", "flac")
    output_format: String,
    /// Additional arguments (can be empty)
    arguments: Option<String>,
}

impl AudioPreset {
    /// Create new audio preset
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

/// Get command string for processing audio files
///
/// # Parameters
/// - `input_path`: input file path
/// - `output_path`: output file path
/// - `preset`: audio preset to use
///
/// # Returns
/// Generated command line string
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
    let mut fallback_files = Vec::new();
    let mut has_error = false;
    let mut last_error = None;

    // Determine concurrency limit based on whether it's on C drive (use more threads for non-C drives)
    let is_hdd = !dir_path.starts_with("C:");
    let max_workers = if is_hdd {
        num_cpus::get().min(24)
    } else {
        num_cpus::get()
    };
    let semaphore = Arc::new(Semaphore::new(max_workers));

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

    // Process each file
    for file_path in tasks {
        let mut current_preset_index = 0;
        let mut success = false;

        while current_preset_index < presets.len() {
            let preset = &presets[current_preset_index];
            let output_path = file_path.with_extension(&preset.output_format);

            // Check if output file exists
            if output_path.exists() {
                if remove_existing {
                    if let Ok(metadata) = fs::metadata(&output_path).await
                        && metadata.len() > 0
                    {
                        log::info!("Removing existing file: {}", output_path.display());
                        let _ = remove_file(&output_path).await;
                    }
                } else {
                    log::info!("Skipping existing file: {}", output_path.display());
                    current_preset_index += 1;
                    continue;
                }
            }

            // Get and execute command
            if let Some(cmd) = get_audio_command(&file_path, &output_path, preset) {
                let permit = semaphore.clone().acquire_arc().await;

                // Execute conversion command
                #[cfg(target_family = "windows")]
                let program = "powershell";
                #[cfg(not(target_family = "windows"))]
                let program = "sh";
                let output = Command::new(program).arg("-c").arg(&cmd).output().await;

                drop(permit); // Release semaphore

                match output {
                    Ok(output) if output.status.success() => {
                        // Conversion successful
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
                        // Conversion failed
                        log::info!(
                            "Preset failed [{}]: {} -> {}",
                            preset.executor,
                            file_path.display(),
                            output_path.display()
                        );
                        last_error = Some((file_path.clone(), output));
                    }
                    Err(e) => {
                        // Command execution error
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

            // Remove original file after all preset attempts fail
            if remove_on_fail && let Err(e) = remove_file(&file_path).await {
                eprintln!(
                    "Error deleting failed file: {} - {}",
                    file_path.display(),
                    e
                );
            }
        }
    }

    // Output processing results
    if total_files > 0 {
        log::info!("Processed {} files in {}", total_files, dir_path.display());
    }
    if !fallback_files.is_empty() {
        log::info!(
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
            log::info!("Original files for failed conversions were removed");
        }
    }

    Ok(!has_error)
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
