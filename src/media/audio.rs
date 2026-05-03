//! Audio conversion presets and processing.
//!
//! This module provides audio conversion presets for formats
//! like FLAC, OGG, and WAV using external tools.

use std::path::Path;
use std::sync::LazyLock;

/// Audio conversion preset.
#[derive(Debug, Clone)]
pub struct AudioPreset {
    /// Executable name (ffmpeg, flac, oggenc)
    pub exec: String,
    /// Output format (flac, ogg, wav)
    pub output_format: String,
    /// Additional arguments for the executable
    pub arg: Option<String>,
}

impl AudioPreset {
    /// Create a new audio preset.
    #[must_use]
    pub fn new(exec: &str, output_format: &str, arg: Option<&str>) -> Self {
        Self {
            exec: exec.to_string(),
            output_format: output_format.to_string(),
            arg: arg.map(std::string::ToString::to_string),
        }
    }
}

/// Audio preset for OGG encoding at quality 10.
pub static AUDIO_PRESET_OGG_Q10: LazyLock<AudioPreset> =
    LazyLock::new(|| AudioPreset::new("oggenc", "ogg", Some("-q10")));

/// Audio preset for WAV encoding using `FFmpeg`.
pub static AUDIO_PRESET_WAV_FFMPEG: LazyLock<AudioPreset> =
    LazyLock::new(|| AudioPreset::new("ffmpeg", "wav", None));

/// Audio preset for extracting WAV from FLAC.
pub static AUDIO_PRESET_WAV_FROM_FLAC: LazyLock<AudioPreset> = LazyLock::new(|| {
    AudioPreset::new(
        "flac",
        "wav",
        Some("-d --keep-foreign-metadata-if-present -f"),
    )
});

/// Audio preset for FLAC encoding.
pub static AUDIO_PRESET_FLAC: LazyLock<AudioPreset> = LazyLock::new(|| {
    AudioPreset::new(
        "flac",
        "flac",
        Some("--keep-foreign-metadata-if-present --best -f"),
    )
});

/// Audio preset for FLAC encoding using `FFmpeg`.
pub static AUDIO_PRESET_FLAC_FFMPEG: LazyLock<AudioPreset> =
    LazyLock::new(|| AudioPreset::new("ffmpeg", "flac", None));

/// Get audio process command
#[must_use]
pub fn get_audio_process_cmd(
    file_path: &Path,
    output_file_path: &Path,
    preset: &AudioPreset,
) -> String {
    let input = file_path.to_string_lossy();
    let output = output_file_path.to_string_lossy();

    if preset.exec == "ffmpeg" {
        let args = preset.arg.as_deref().unwrap_or("");
        format!(
            "ffmpeg -hide_banner -loglevel panic -i \"{}\" -f {} -map_metadata 0 {} \"{}\"",
            input, preset.output_format, args, output
        )
    } else if preset.exec == "oggenc" {
        let args = preset.arg.as_deref().unwrap_or("");
        format!("oggenc {args} \"{input}\" -o \"{output}\"")
    } else if preset.exec == "flac" {
        let args = preset.arg.as_deref().unwrap_or("");
        format!("flac {args} \"{input}\" -o \"{output}\"")
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_audio_preset_new() {
        let preset = AudioPreset::new("ffmpeg", "mp3", Some("-b:a 320k"));
        assert_eq!(preset.exec, "ffmpeg");
        assert_eq!(preset.output_format, "mp3");
        assert_eq!(preset.arg, Some("-b:a 320k".to_string()));
    }

    #[test]
    fn test_get_audio_process_cmd() {
        let input = PathBuf::from("/path/to/input.wav");
        let output = PathBuf::from("/path/to/output.flac");
        let preset = AUDIO_PRESET_FLAC.clone();
        let cmd = get_audio_process_cmd(&input, &output, &preset);
        assert!(cmd.contains("flac"));
        assert!(cmd.contains("input.wav"));
        assert!(cmd.contains("output.flac"));
    }

    #[test]
    fn test_audio_preset_flac() {
        let preset = AUDIO_PRESET_FLAC.clone();
        assert_eq!(preset.output_format, "flac");
        assert!(preset.exec.contains("flac"));
    }

    #[test]
    fn test_audio_preset_ogg() {
        let preset = AUDIO_PRESET_OGG_Q10.clone();
        assert_eq!(preset.output_format, "ogg");
        assert!(preset.exec.contains("oggenc"));
    }
}
