# be-music-cabinet Command Line Application

be-music-cabinet is a command line tool for managing BMS music files, providing rich functionality to organize and process BMS files.

## Installation

```bash
cargo build --release
```

The compiled executable is located at `target/release/be-music-cabinet.exe`

## Basic Usage

```bash
be-music-cabinet <COMMAND>
```

## Command Categories

### 1. Work Directory Operations (work)

Operations for processing individual BMS work directories.

#### Set Directory Name
```bash
# Set directory name based on BMS file
be-music-cabinet work set-name <DIR> [--set-type <TYPE>]

# 示例
be-music-cabinet work set-name ./MyBMSFolder
be-music-cabinet work set-name ./MyBMSFolder --set-type append_title_artist
```

Set type options:
- `replace_title_artist`: Replace with title+artist
- `append_title_artist`: Append title+artist (default)
- `append_artist`: Append artist only

#### Undo Set Directory Name
```bash
# Undo set directory name
be-music-cabinet work undo-set-name <DIR> [--set-type <TYPE>]

# Examples
be-music-cabinet work undo-set-name ./MyBMSFolder
```

#### Remove Zero-byte Media Files
```bash
# Remove zero-byte media files
be-music-cabinet work remove-empty-media <DIR>

# Examples
be-music-cabinet work remove-empty-media ./MyBMSFolder
```

### 2. Root Directory Operations (root)

Batch operations for processing BMS root directories.

#### Copy Numbered Work Directory Names
```bash
# Copy numbered work directory names
be-music-cabinet root copy-numbered-names <FROM> <TO>

# Examples
be-music-cabinet root copy-numbered-names ./source ./target
```

#### Split Folders by First Character
```bash
# Split folders by first character
be-music-cabinet root split-by-first-char <DIR>

# Examples
be-music-cabinet root split-by-first-char ./BMSRoot
```

#### Undo Split Operation
```bash
# Undo split operation
be-music-cabinet root undo-split <DIR>

# Examples
be-music-cabinet root undo-split ./BMSRoot
```

#### Merge Split Folders
```bash
# Merge split folders
be-music-cabinet root merge-split <DIR>

# Examples
be-music-cabinet root merge-split ./BMSRoot
```

#### Move Works
```bash
# Move works
be-music-cabinet root move-works <FROM> <TO>

# Examples
be-music-cabinet root move-works ./source ./target
```

#### Move Out One Directory Level
```bash
# Move out one directory level
be-music-cabinet root move-out-works <DIR>

# Examples
be-music-cabinet root move-out-works ./BMSRoot
```

#### Move Works with Same Name
```bash
# Move works with same name
be-music-cabinet root move-same-name <FROM> <TO>

# Examples
be-music-cabinet root move-same-name ./source ./target
```

#### Remove Unnecessary Media Files
```bash
# Remove unnecessary media files
be-music-cabinet root remove-unneed-media <DIR> [--rule <RULE>]

# Examples
be-music-cabinet root remove-unneed-media ./BMSRoot
be-music-cabinet root remove-unneed-media ./BMSRoot --rule oraja
```

Rule type options:
- `oraja`: beatoraja rule (default)
- `wav_fill_flac`: WAV fill FLAC rule
- `mpg_fill_wmv`: MPG fill WMV rule

### 3. Large Package Processing Operations (pack)

Processing BMS large package conversion and generation.

#### Raw Package -> HQ Version Large Package
```bash
# Convert raw package to HQ version large package (for beatoraja/Qwilight players)
be-music-cabinet pack raw-to-hq <DIR>

# Examples
be-music-cabinet pack raw-to-hq ./BMSRoot
```

#### HQ Version Large Package -> LQ Version Large Package
```bash
# Convert HQ version large package to LQ version large package (for LR2 players)
be-music-cabinet pack hq-to-lq <DIR>

# Examples
be-music-cabinet pack hq-to-lq ./BMSRoot
```

#### Large Package Generation Script
```bash
# Quickly create HQ version large package from raw package
be-music-cabinet pack setup-rawpack-to-hq <PACK_DIR> <ROOT_DIR>

# Examples
be-music-cabinet pack setup-rawpack-to-hq ./packs ./BMSRoot
```

#### Large Package Update Script
```bash
# Quickly update HQ version large package from raw package
be-music-cabinet pack update-rawpack-to-hq <PACK_DIR> <ROOT_DIR> <SYNC_DIR>

# Examples
be-music-cabinet pack update-rawpack-to-hq ./packs ./BMSRoot ./SyncDir
```

### 4. BMS File Related Operations (bms)

Parsing and checking operations for BMS files.

#### Parse BMS File
```bash
# Parse BMS file
be-music-cabinet bms parse-bms <FILE>

# Examples
be-music-cabinet bms parse-bms ./song.bms
```

#### Parse BMSON File
```bash
# Parse BMSON file
be-music-cabinet bms parse-bmson <FILE>

# Examples
be-music-cabinet bms parse-bmson ./song.bmson
```

#### Get BMS File List
```bash
# Get BMS file list in directory
be-music-cabinet bms get-bms-list <DIR>

# Examples
be-music-cabinet bms get-bms-list ./BMSFolder
```

#### Get BMS Information
```bash
# Get BMS information in directory
be-music-cabinet bms get-bms-info <DIR>

# Examples
be-music-cabinet bms get-bms-info ./BMSFolder
```

#### Check Directory Type
```bash
# Check if it's a work directory
be-music-cabinet bms is-work-dir <DIR>

# Check if it's a root directory
be-music-cabinet bms is-root-dir <DIR>

# Examples
be-music-cabinet bms is-work-dir ./MyBMSFolder
be-music-cabinet bms is-root-dir ./BMSRoot
```

### 5. File System Related Operations (fs)

Various operations for handling the file system.

#### File Comparison
```bash
# Check if two files have the same content
be-music-cabinet fs is-file-same <FILE1> <FILE2>

# Examples
be-music-cabinet fs is-file-same ./file1.txt ./file2.txt
```

#### Directory Check
```bash
# Check if directory contains files
be-music-cabinet fs is-dir-having-file <DIR>

# Examples
be-music-cabinet fs is-dir-having-file ./MyFolder
```

#### Cleanup Operations
```bash
# Remove empty folders
be-music-cabinet fs remove-empty-folders <DIR>

# Examples
be-music-cabinet fs remove-empty-folders ./BMSRoot
```

#### Similarity Calculation
```bash
# Calculate BMS directory similarity
be-music-cabinet fs bms-dir-similarity <DIR1> <DIR2>

# Examples
be-music-cabinet fs bms-dir-similarity ./folder1 ./folder2
```

### 6. Root Directory Event Related Operations (root-event)

Handling events and batch operations for root directories.

#### Numbered Folder Management
```bash
# Check numbered folders
be-music-cabinet root-event check-num-folder <DIR> <MAX>

# Create numbered folders
be-music-cabinet root-event create-num-folders <DIR> <COUNT>

# Examples
be-music-cabinet root-event check-num-folder ./BMSRoot 1000
be-music-cabinet root-event create-num-folders ./BMSRoot 100
```

#### Information Table Generation
```bash
# Generate work information table
be-music-cabinet root-event generate-work-info-table <DIR>

# Examples
be-music-cabinet root-event generate-work-info-table ./BMSRoot
```

#### Raw Package -> HQ Version Large Package
```bash
# Convert raw package to HQ version large package (for beatoraja/Qwilight players)
be-music-cabinet pack raw-to-hq <DIR>

# Examples
be-music-cabinet pack raw-to-hq ./BMSRoot
```

#### HQ Version Large Package -> LQ Version Large Package
```bash
# Convert HQ version large package to LQ version large package (for LR2 players)
be-music-cabinet pack hq-to-lq <DIR>

# Examples
be-music-cabinet pack hq-to-lq ./BMSRoot
```

#### Large Package Generation Script
```bash
# Quickly create HQ version large package from raw package
be-music-cabinet pack setup-rawpack-to-hq <PACK_DIR> <ROOT_DIR>

# Examples
be-music-cabinet pack setup-rawpack-to-hq ./packs ./BMSRoot
```

#### Large Package Update Script
```bash
# Quickly update HQ version large package from raw package
be-music-cabinet pack update-rawpack-to-hq <PACK_DIR> <ROOT_DIR> <SYNC_DIR>

# Examples
be-music-cabinet pack update-rawpack-to-hq ./packs ./BMSRoot ./SyncDir
```

## Common Workflows

### 1. Organize Individual BMS Folder
```bash
# Set directory name
be-music-cabinet work set-name ./MyBMSFolder

# Remove zero-byte files
be-music-cabinet work remove-empty-media ./MyBMSFolder
```

### 2. Organize Entire BMS Root Directory
```bash
# Split folders by first character
be-music-cabinet root split-by-first-char ./BMSRoot

# Remove unnecessary media files
be-music-cabinet root remove-unneed-media ./BMSRoot

# Merge split folders
be-music-cabinet root merge-split ./BMSRoot
```

### 3. Generate HQ Version Large Package
```bash
# Generate HQ version large package from raw package
be-music-cabinet pack setup-rawpack-to-hq ./packs ./BMSRoot

# Or directly convert existing directory
be-music-cabinet pack raw-to-hq ./BMSRoot
```

### 4. Generate LQ Version Large Package
```bash
# Generate LQ version large package from HQ version
be-music-cabinet pack hq-to-lq ./BMSRoot
```

### 5. BMS File Analysis
```bash
# Check directory type
be-music-cabinet bms is-work-dir ./MyBMSFolder
be-music-cabinet bms is-root-dir ./BMSRoot

# Get BMS information
be-music-cabinet bms get-bms-info ./MyBMSFolder

# Parse BMS file
be-music-cabinet bms parse-bms ./song.bms
```

### 6. File System Maintenance
```bash
# Check if files are the same
be-music-cabinet fs is-file-same ./file1.txt ./file2.txt

# Remove empty folders
be-music-cabinet fs remove-empty-folders ./BMSRoot

# Calculate directory similarity
be-music-cabinet fs bms-dir-similarity ./folder1 ./folder2
```

### 7. Batch Operations
```bash
# Create numbered folders
be-music-cabinet root-event create-num-folders ./BMSRoot 100

# Generate work information table
be-music-cabinet root-event generate-work-info-table ./BMSRoot
```

## Notes

1. **Backup Important Files**: Please backup important BMS files before executing any operations
2. **Path Format**: Supports both relative and absolute paths
3. **Asynchronous Operations**: All operations are asynchronous, large file processing may take some time
4. **External Dependencies**: Some functions require external tools (such as ffmpeg, flac, etc.)
5. **Permission Requirements**: Ensure you have read/write permissions for target directories

## Error Handling

If you encounter errors, the program will display detailed error information. Common errors include:
- File not found or insufficient permissions
- External tools not installed
- Insufficient disk space
- Unsupported file format

## Help Information

Get help information:
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
be-music-cabinet bms parse-bms --help
be-music-cabinet fs is-file-same --help
```
