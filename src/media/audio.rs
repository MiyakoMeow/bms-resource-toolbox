//! Audio conversion presets and processing.
//!
//! This module provides audio conversion presets for formats
//! like FLAC, OGG, and WAV using external tools.

use std::path::Path;
use std::process::Stdio;
use std::sync::LazyLock;
use tokio::process::Command;

/// Audio conversion preset.
#[allow(dead_code)]
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
#[allow(dead_code)]
#[must_use]
pub fn audio_preset_ogg_q10() -> AudioPreset {
    AudioPreset::new("oggenc", "ogg", Some("-q10"))
}

/// Audio preset for OGG encoding using `FFmpeg`.
#[allow(dead_code)]
#[must_use]
pub fn audio_preset_ogg_ffmpeg() -> AudioPreset {
    AudioPreset::new("ffmpeg", "ogg", None)
}

/// Audio preset for WAV encoding using `FFmpeg`.
#[allow(dead_code)]
#[must_use]
pub fn audio_preset_wav_ffmpeg() -> AudioPreset {
    AudioPreset::new("ffmpeg", "wav", None)
}

/// Audio preset for extracting WAV from FLAC.
#[allow(dead_code)]
#[must_use]
pub fn audio_preset_wav_from_flac() -> AudioPreset {
    AudioPreset::new("flac", "wav", Some("-d --keep-foreign-metadata-if-present -f"))
}

/// Audio preset for FLAC encoding.
#[allow(dead_code)]
#[must_use]
pub fn audio_preset_flac() -> AudioPreset {
    AudioPreset::new("flac", "flac", Some("--keep-foreign-metadata-if-present --best -f"))
}

/// Audio preset for FLAC encoding using `FFmpeg`.
#[allow(dead_code)]
#[must_use]
pub fn audio_preset_flac_ffmpeg() -> AudioPreset {
    AudioPreset::new("ffmpeg", "flac", None)
}

/// Backward compatibility constants (deprecated - use functions instead)
/// Deprecated OGG Q10 audio preset constant.
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use functions instead")]
pub const AUDIO_PRESET_OGG_Q10: () = ();
/// Deprecated OGG `FFmpeg` audio preset constant.
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use functions instead")]
pub const AUDIO_PRESET_OGG_FFMPEG: () = ();
/// Deprecated WAV `FFmpeg` audio preset constant.
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use functions instead")]
pub const AUDIO_PRESET_WAV_FFMPEG: () = ();
/// Deprecated WAV from FLAC audio preset constant.
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use functions instead")]
pub const AUDIO_PRESET_WAV_FROM_FLAC: () = ();
/// Deprecated FLAC audio preset constant.
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use functions instead")]
pub const AUDIO_PRESET_FLAC: () = ();
/// Deprecated FLAC `FFmpeg` audio preset constant.
#[allow(dead_code)]
#[deprecated(since = "0.1.0", note = "Use functions instead")]
pub const AUDIO_PRESET_FLAC_FFMPEG: () = ();

/// Lazy static for OGG Q10 audio preset.
#[allow(dead_code)]
pub static AUDIO_PRESET_OGG_Q10_VAL: LazyLock<AudioPreset> = LazyLock::new(audio_preset_ogg_q10);
/// Lazy static for OGG `FFmpeg` audio preset.
#[allow(dead_code)]
pub static AUDIO_PRESET_OGG_FFMPEG_VAL: LazyLock<AudioPreset> = LazyLock::new(audio_preset_ogg_ffmpeg);
/// Lazy static for WAV `FFmpeg` audio preset.
#[allow(dead_code)]
pub static AUDIO_PRESET_WAV_FFMPEG_VAL: LazyLock<AudioPreset> = LazyLock::new(audio_preset_wav_ffmpeg);
/// Lazy static for WAV from FLAC audio preset.
#[allow(dead_code)]
pub static AUDIO_PRESET_WAV_FROM_FLAC_VAL: LazyLock<AudioPreset> = LazyLock::new(audio_preset_wav_from_flac);
/// Lazy static for FLAC audio preset.
#[allow(dead_code)]
pub static AUDIO_PRESET_FLAC_VAL: LazyLock<AudioPreset> = LazyLock::new(audio_preset_flac);
/// Lazy static for FLAC `FFmpeg` audio preset.
#[allow(dead_code)]
pub static AUDIO_PRESET_FLAC_FFMPEG_VAL: LazyLock<AudioPreset> = LazyLock::new(audio_preset_flac_ffmpeg);

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

/// Check if external audio tool is available
#[allow(dead_code)]
pub async fn check_audio_tool(exec: &str) -> bool {
    let output = Command::new(exec)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
    output.map(|s| s.success()).unwrap_or(false)
}

/// Check ffmpeg availability
#[allow(dead_code)]
pub async fn check_ffmpeg() -> bool {
    check_audio_tool("ffmpeg").await
}

/// Check flac availability
#[allow(dead_code)]
pub async fn check_flac() -> bool {
    check_audio_tool("flac").await
}

/// Check oggenc availability
#[allow(dead_code)]
pub async fn check_oggenc() -> bool {
    check_audio_tool("oggenc").await
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
        let preset = audio_preset_flac();
        let cmd = get_audio_process_cmd(&input, &output, &preset);
        assert!(cmd.contains("flac"));
        assert!(cmd.contains("input.wav"));
        assert!(cmd.contains("output.flac"));
    }
}
