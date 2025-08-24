# BMS Large Package Processing Module

This module provides various functions for processing BMS large packages, synchronized from the Python version.

## Function List

### 1. Raw Package -> HQ Version Large Package
```rust
use be_music_cabinet::options::pack::pack_raw_to_hq;

// Convert raw package to HQ version large package (for beatoraja/Qwilight players)
pack_raw_to_hq("path/to/root/dir").await?;
```

### 2. HQ Version Large Package -> LQ Version Large Package
```rust
use be_music_cabinet::options::pack::pack_hq_to_lq;

// Convert HQ version large package to LQ version large package (for LR2 players)
pack_hq_to_lq("path/to/root/dir").await?;
```

### 3. Large Package Generation Script: Raw Package -> HQ Version Large Package
```rust
use be_music_cabinet::options::pack::pack_setup_rawpack_to_hq;

// Quickly create HQ version large package from raw package
pack_setup_rawpack_to_hq("path/to/pack/dir", "path/to/root/dir").await?;
```

### 4. Large Package Update Script: Raw Package -> HQ Version Large Package
```rust
use be_music_cabinet::options::pack::pack_update_rawpack_to_hq;

// Quickly update HQ version large package from raw package
pack_update_rawpack_to_hq("path/to/pack/dir", "path/to/root/dir", "path/to/sync/dir").await?;
```

## Processing Flow

### Raw Package -> HQ Version Large Package Flow
1. **Audio Processing**: WAV -> FLAC conversion
2. **File Cleanup**: Remove unnecessary media files

### HQ Version Large Package -> LQ Version Large Package Flow
1. **Audio Processing**: FLAC -> OGG conversion
2. **Video Processing**: MP4 -> Multiple format conversion (MPEG1, WMV2, AVI)

### Large Package Generation Script Flow
1. **Extraction**: Extract numerically named files from package directory to target directory
2. **Naming**: Set directory names based on BMS files
3. **Audio Processing**: WAV -> FLAC conversion
4. **File Cleanup**: Remove unnecessary media files

### Large Package Update Script Flow
1. **Extraction**: Extract numerically named files from package directory to target directory
2. **Sync Naming**: Sync directory names from existing directories
3. **Audio Processing**: WAV -> FLAC conversion
4. **File Cleanup**: Remove unnecessary media files
5. **Soft Sync**: Sync directory files
6. **Empty Folder Cleanup**: Remove empty folders

## Audio Presets

The system supports the following audio processing presets:

- **FLAC**: Use flac tool for lossless compression
- **FLAC_FFMPEG**: Use ffmpeg for FLAC conversion
- **OGG_Q10**: Use oggenc for OGG conversion (quality 10)

## Video Presets

The system supports the following video processing presets:

- **MPEG1VIDEO_512X512**: MPEG1 format, 512x512 resolution
- **WMV2_512X512**: WMV2 format, 512x512 resolution
- **AVI_512X512**: AVI format, 512x512 resolution

## Extraction Features

The system supports the following extraction features:

- **Numeric Named File Recognition**: Automatically recognize package files starting with numbers
- **Multi-format Support**: Automatic extraction of ZIP, 7Z, RAR formats
- **Smart Directory Organization**: Automatically organize extracted directory structure
- **Target Directory Creation**: Automatically create target directories by numeric ID

## Notes

- All operations are asynchronous and require using `.await`
- Audio and video processing require corresponding external tools (ffmpeg, flac, oggenc, etc.)
- Extraction functionality is fully implemented, supporting ZIP, 7Z, RAR formats
- File synchronization uses smart strategies to avoid overwriting important files
- Supports recursive cleanup of empty folders
- Package files must start with numbers to be processed

## Required External Tools

- **ffmpeg**: For audio and video conversion
- **flac**: For FLAC format processing
- **oggenc**: For OGG format processing
- **zip/7z/rar**: For extraction functionality

## Error Handling

All functions return `io::Result<()>`, containing detailed error information. Common errors include:

- File not found or insufficient permissions
- External tools not installed or execution failed
- Insufficient disk space
- Unsupported file format
