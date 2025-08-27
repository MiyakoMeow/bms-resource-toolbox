# bms-resource-toolbox Command Line Application

bms-resource-toolbox is a command line tool for managing BMS music files, providing rich functionality to organize and process BMS files.

## Installation

```bash
cargo build --release
```

The compiled executable is located at `target/release/bms-resource-toolbox.exe`

## Basic Usage

```bash
bms-resource-toolbox <COMMAND>
```

## Command Categories

### 1. Work Directory Operations (work)

Operations for processing individual BMS work directories.

#### Set Directory Name
```bash
# Set directory name based on BMS file
bms-resource-toolbox work set-name <DIR> [--set-type <TYPE>]

# 示例
bms-resource-toolbox work set-name ./MyBMSFolder
bms-resource-toolbox work set-name ./MyBMSFolder --set-type append_title_artist
```

Set type options:
- `replace_title_artist`: Replace with title+artist
- `append_title_artist`: Append title+artist (default)
- `append_artist`: Append artist only

#### Undo Set Directory Name
```bash
# Undo set directory name
bms-resource-toolbox work undo-set-name <DIR> [--set-type <TYPE>]

# Examples
bms-resource-toolbox work undo-set-name ./MyBMSFolder
```

#### Remove Zero-byte Media Files
```bash
# Remove zero-byte media files
bms-resource-toolbox work remove-empty-media <DIR>

# Examples
bms-resource-toolbox work remove-empty-media ./MyBMSFolder
```

### 2. Root Directory Operations (root)

Batch operations for processing BMS root directories.

#### Copy Numbered Work Directory Names
```bash
# Copy numbered work directory names
bms-resource-toolbox root copy-numbered-names <FROM> <TO>

# Examples
bms-resource-toolbox root copy-numbered-names ./source ./target
```

#### Split Folders by First Character
```bash
# Split folders by first character
bms-resource-toolbox root split-by-first-char <DIR>

# Examples
bms-resource-toolbox root split-by-first-char ./BMSRoot
```

#### Undo Split Operation
```bash
# Undo split operation
bms-resource-toolbox root undo-split <DIR>

# Examples
bms-resource-toolbox root undo-split ./BMSRoot
```

#### Merge Split Folders
```bash
# Merge split folders
bms-resource-toolbox root merge-split <DIR>

# Examples
bms-resource-toolbox root merge-split ./BMSRoot
```

#### Move Works
```bash
# Move works
bms-resource-toolbox root move-works <FROM> <TO>

# Examples
bms-resource-toolbox root move-works ./source ./target
```

#### Move Out One Directory Level
```bash
# Move out one directory level
bms-resource-toolbox root move-out-works <DIR>

# Examples
bms-resource-toolbox root move-out-works ./BMSRoot
```

#### Move Works with Same Name
```bash
# Move works with same name
bms-resource-toolbox root move-same-name <FROM> <TO>

# Examples
bms-resource-toolbox root move-same-name ./source ./target
```

#### Remove Unnecessary Media Files
```bash
# Remove unnecessary media files
bms-resource-toolbox root remove-unneed-media <DIR> [--rule <RULE>]

# Examples
bms-resource-toolbox root remove-unneed-media ./BMSRoot
bms-resource-toolbox root remove-unneed-media ./BMSRoot --rule oraja
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
bms-resource-toolbox pack raw-to-hq <DIR>

# Examples
bms-resource-toolbox pack raw-to-hq ./BMSRoot
```

#### HQ Version Large Package -> LQ Version Large Package
```bash
# Convert HQ version large package to LQ version large package (for LR2 players)
bms-resource-toolbox pack hq-to-lq <DIR>

# Examples
bms-resource-toolbox pack hq-to-lq ./BMSRoot
```

#### Large Package Generation Script
```bash
# Quickly create HQ version large package from raw package
bms-resource-toolbox pack setup-rawpack-to-hq <PACK_DIR> <ROOT_DIR>

# Examples
bms-resource-toolbox pack setup-rawpack-to-hq ./packs ./BMSRoot
```

#### Large Package Update Script
```bash
# Quickly update HQ version large package from raw package
bms-resource-toolbox pack update-rawpack-to-hq <PACK_DIR> <ROOT_DIR> <SYNC_DIR>

# Examples
bms-resource-toolbox pack update-rawpack-to-hq ./packs ./BMSRoot ./SyncDir
```

### 4. BMS File Related Operations (bms)

Parsing and checking operations for BMS files.

#### Parse BMS File
```bash
# Parse BMS file
bms-resource-toolbox bms parse-bms <FILE>

# Examples
bms-resource-toolbox bms parse-bms ./song.bms
```

#### Parse BMSON File
```bash
# Parse BMSON file
bms-resource-toolbox bms parse-bmson <FILE>

# Examples
bms-resource-toolbox bms parse-bmson ./song.bmson
```

#### Get BMS File List
```bash
# Get BMS file list in directory
bms-resource-toolbox bms get-bms-list <DIR>

# Examples
bms-resource-toolbox bms get-bms-list ./BMSFolder
```

#### Get BMS Information
```bash
# Get BMS information in directory
bms-resource-toolbox bms get-bms-info <DIR>

# Examples
bms-resource-toolbox bms get-bms-info ./BMSFolder
```

#### Check Directory Type
```bash
# Check if it's a work directory
bms-resource-toolbox bms is-work-dir <DIR>

# Check if it's a root directory
bms-resource-toolbox bms is-root-dir <DIR>

# Examples
bms-resource-toolbox bms is-work-dir ./MyBMSFolder
bms-resource-toolbox bms is-root-dir ./BMSRoot
```

### 5. File System Related Operations (fs)

Various operations for handling the file system.

#### File Comparison
```bash
# Check if two files have the same content
bms-resource-toolbox fs is-file-same <FILE1> <FILE2>

# Examples
bms-resource-toolbox fs is-file-same ./file1.txt ./file2.txt
```

#### Directory Check
```bash
# Check if directory contains files
bms-resource-toolbox fs is-dir-having-file <DIR>

# Examples
bms-resource-toolbox fs is-dir-having-file ./MyFolder
```

#### Cleanup Operations
```bash
# Remove empty folders
bms-resource-toolbox fs remove-empty-folders <DIR>

# Examples
bms-resource-toolbox fs remove-empty-folders ./BMSRoot
```

#### Similarity Calculation
```bash
# Calculate BMS directory similarity
bms-resource-toolbox fs bms-dir-similarity <DIR1> <DIR2>

# Examples
bms-resource-toolbox fs bms-dir-similarity ./folder1 ./folder2
```

### 6. Root Directory Event Related Operations (root-event)

Handling events and batch operations for root directories.

#### Numbered Folder Management
```bash
# Check numbered folders
bms-resource-toolbox root-event check-num-folder <DIR> <MAX>

# Create numbered folders
bms-resource-toolbox root-event create-num-folders <DIR> <COUNT>

# Examples
bms-resource-toolbox root-event check-num-folder ./BMSRoot 1000
bms-resource-toolbox root-event create-num-folders ./BMSRoot 100
```

#### Information Table Generation
```bash
# Generate work information table
bms-resource-toolbox root-event generate-work-info-table <DIR>

# Examples
bms-resource-toolbox root-event generate-work-info-table ./BMSRoot
```

#### Raw Package -> HQ Version Large Package
```bash
# Convert raw package to HQ version large package (for beatoraja/Qwilight players)
bms-resource-toolbox pack raw-to-hq <DIR>

# Examples
bms-resource-toolbox pack raw-to-hq ./BMSRoot
```

#### HQ Version Large Package -> LQ Version Large Package
```bash
# Convert HQ version large package to LQ version large package (for LR2 players)
bms-resource-toolbox pack hq-to-lq <DIR>

# Examples
bms-resource-toolbox pack hq-to-lq ./BMSRoot
```

#### Large Package Generation Script
```bash
# Quickly create HQ version large package from raw package
bms-resource-toolbox pack setup-rawpack-to-hq <PACK_DIR> <ROOT_DIR>

# Examples
bms-resource-toolbox pack setup-rawpack-to-hq ./packs ./BMSRoot
```

#### Large Package Update Script
```bash
# Quickly update HQ version large package from raw package
bms-resource-toolbox pack update-rawpack-to-hq <PACK_DIR> <ROOT_DIR> <SYNC_DIR>

# Examples
bms-resource-toolbox pack update-rawpack-to-hq ./packs ./BMSRoot ./SyncDir
```

## Common Workflows

### 1. Organize Individual BMS Folder
```bash
# Set directory name
bms-resource-toolbox work set-name ./MyBMSFolder

# Remove zero-byte files
bms-resource-toolbox work remove-empty-media ./MyBMSFolder
```

### 2. Organize Entire BMS Root Directory
```bash
# Split folders by first character
bms-resource-toolbox root split-by-first-char ./BMSRoot

# Remove unnecessary media files
bms-resource-toolbox root remove-unneed-media ./BMSRoot

# Merge split folders
bms-resource-toolbox root merge-split ./BMSRoot
```

### 3. Generate HQ Version Large Package
```bash
# Generate HQ version large package from raw package
bms-resource-toolbox pack setup-rawpack-to-hq ./packs ./BMSRoot

# Or directly convert existing directory
bms-resource-toolbox pack raw-to-hq ./BMSRoot
```

### 4. Generate LQ Version Large Package
```bash
# Generate LQ version large package from HQ version
bms-resource-toolbox pack hq-to-lq ./BMSRoot
```

### 5. BMS File Analysis
```bash
# Check directory type
bms-resource-toolbox bms is-work-dir ./MyBMSFolder
bms-resource-toolbox bms is-root-dir ./BMSRoot

# Get BMS information
bms-resource-toolbox bms get-bms-info ./MyBMSFolder

# Parse BMS file
bms-resource-toolbox bms parse-bms ./song.bms
```

### 6. File System Maintenance
```bash
# Check if files are the same
bms-resource-toolbox fs is-file-same ./file1.txt ./file2.txt

# Remove empty folders
bms-resource-toolbox fs remove-empty-folders ./BMSRoot

# Calculate directory similarity
bms-resource-toolbox fs bms-dir-similarity ./folder1 ./folder2
```

### 7. Batch Operations
```bash
# Create numbered folders
bms-resource-toolbox root-event create-num-folders ./BMSRoot 100

# Generate work information table
bms-resource-toolbox root-event generate-work-info-table ./BMSRoot
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
bms-resource-toolbox --help

# Subcommand help
bms-resource-toolbox work --help
bms-resource-toolbox root --help
bms-resource-toolbox pack --help
bms-resource-toolbox bms --help
bms-resource-toolbox fs --help
bms-resource-toolbox root-event --help

# Specific command help
bms-resource-toolbox work set-name --help
bms-resource-toolbox bms parse-bms --help
bms-resource-toolbox fs is-file-same --help
```
