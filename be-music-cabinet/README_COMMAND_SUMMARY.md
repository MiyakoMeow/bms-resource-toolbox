# be-music-cabinet Command Summary

This document summarizes all available commands for the be-music-cabinet command line application.

## Command Categories

### 1. Work Directory Operations (work)
Operations for handling individual BMS work directories.

| Command | Function | Parameters |
|------|------|------|
| `set-name` | Set directory name based on BMS file | `DIR` + `--set-type` |
| `undo-set-name` | Undo directory name setting | `DIR` + `--set-type` |
| `remove-empty-media` | Remove zero-byte media files | `DIR` |

### 2. Root Directory Operations (root)
Batch operations for handling BMS root directories.

| Command | Function | Parameters |
|------|------|------|
| `copy-numbered-names` | Copy numbered work directory names | `FROM` `TO` |
| `split-by-first-char` | Split folders by first character | `DIR` |
| `undo-split` | Undo split operation | `DIR` |
| `merge-split` | Merge split folders | `DIR` |
| `move-works` | Move works | `FROM` `TO` |
| `move-out-works` | Move out one directory level | `DIR` |
| `move-same-name` | Move works with same name | `FROM` `TO` |
| `remove-unneed-media` | Remove unnecessary media files | `DIR` + `--rule` |
| `scan-similar-folders` | Scan similar folders | `DIR` + `--similarity` |

### 3. Large Package Processing Operations (pack)
Processing BMS large package conversion and generation.

| Command | Function | Parameters |
|------|------|------|
| `raw-to-hq` | Convert raw package to HQ version | `DIR` |
| `hq-to-lq` | Convert HQ version to LQ version | `DIR` |
| `setup-rawpack-to-hq` | Large package generation script | `PACK_DIR` `ROOT_DIR` |
| `update-rawpack-to-hq` | Large package update script | `PACK_DIR` `ROOT_DIR` `SYNC_DIR` |

### 4. BMS File Related Operations (bms)
Parsing and checking operations for BMS files.

| Command | Function | Parameters |
|------|------|------|
| `parse-bms` | Parse BMS file | `FILE` |
| `parse-bmson` | Parse BMSON file | `FILE` |
| `get-bms-list` | Get BMS file list | `DIR` |
| `get-bms-info` | Get BMS information | `DIR` |
| `is-work-dir` | Check if it's a work directory | `DIR` |
| `is-root-dir` | Check if it's a root directory | `DIR` |

### 5. File System Related Operations (fs)
Various operations for handling the file system.

| Command | Function | Parameters |
|------|------|------|
| `is-file-same` | Check if file contents are the same | `FILE1` `FILE2` |
| `is-dir-having-file` | Check if directory contains files | `DIR` |
| `remove-empty-folders` | Remove empty folders | `DIR` |
| `bms-dir-similarity` | Calculate BMS directory similarity | `DIR1` `DIR2` |

### 6. Root Directory Event Related Operations (root-event)
Handling events and batch operations for root directories.

| Command | Function | Parameters |
|------|------|------|
| `check-num-folder` | Check numbered folders | `DIR` `MAX` |
| `create-num-folders` | Create numbered folders | `DIR` `COUNT` |
| `generate-work-info-table` | Generate work information table | `DIR` |

## Common Parameter Descriptions

### Setting Type (--set-type)
- `replace_title_artist`: Replace with title + artist
- `append_title_artist`: Append title + artist (default)
- `append_artist`: Append artist only

### Rule Type (--rule)
- `oraja`: beatoraja rule (default)
- `wav_fill_flac`: WAV fill FLAC rule
- `mpg_fill_wmv`: MPG fill WMV rule

### Similarity Threshold (--similarity)
- Range: 0.0 - 1.0
- Default value: 0.7

## Quick Reference

### Basic Operations
```bash
# Set directory name
be-music-cabinet work set-name ./MyBMSFolder

# Remove unnecessary files
be-music-cabinet root remove-unneed-media ./BMSRoot

# Convert large package
be-music-cabinet pack raw-to-hq ./BMSRoot
```

### File Checking
```bash
# Check directory type
be-music-cabinet bms is-work-dir ./MyBMSFolder

# Check if files are the same
be-music-cabinet fs is-file-same ./file1.txt ./file2.txt

# Remove empty folders
be-music-cabinet fs remove-empty-folders ./BMSRoot
```

### Batch Operations
```bash
# Split folders
be-music-cabinet root split-by-first-char ./BMSRoot

# Create numbered folders
be-music-cabinet root-event create-num-folders ./BMSRoot 100

# Generate information table
be-music-cabinet root-event generate-work-info-table ./BMSRoot
```

## Notes

1. All paths support both relative and absolute paths
2. All operations are asynchronous, large file processing may take time
3. Some functions require external tools (ffmpeg, flac, etc.)
4. Please backup important files before execution
5. Ensure you have read/write permissions for target directories

## Getting Help

```bash
# Main help
be-music-cabinet --help

# Subcommand help
be-music-cabinet work --help
be-music-cabinet root --help
be-music-cabinet pack --help
be-music-cabinet bms --help
be-music-cabinet fs --help
be-music-cabinet root-event --help

# Specific command help
be-music-cabinet work set-name --help
```
