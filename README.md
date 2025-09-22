# bms-resource-toolbox

BMS music file management tool that provides rich functionality to organize and process BMS files.

## Features

- **Work directory management**: Set directory names, remove zero-byte files, etc.
- **Root directory operations**: Split/merge folders, move works, clean media files, etc.
- **Pack processing**: Conversion between raw pack/HQ pack/LQ pack
- **Command line interface**: Complete command line operation interface
- **Async processing**: Efficient asynchronous file operations
- **Multi-format support**: Support for ZIP, 7Z, RAR and other compression formats

## Quick Start

### Installation

```bash
git clone <repository-url>
cd bms-resource-toolbox
cargo build --release
```

### Command Line Usage

```bash
# View help
./target/release/bms-resource-toolbox --help

# Set BMS folder name
./target/release/bms-resource-toolbox work set-name ./MyBMSFolder

# Remove unnecessary media files
./target/release/bms-resource-toolbox root remove-unneed-media ./BMSRoot

# Raw pack to HQ pack
./target/release/bms-resource-toolbox pack raw-to-hq ./BMSRoot
```

### Programming Interface Usage

```rust
use be_music_cabinet::options::{
    work::{set_name_by_bms, BmsFolderSetNameType},
    root_bigpack::{remove_unneed_media_files, get_remove_media_rule_oraja},
    pack::pack_raw_to_hq,
    fs::moving::ReplacePreset,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set directory name
    set_name_by_bms("./MyBMSFolder", BmsFolderSetNameType::AppendTitleArtist, false, ReplacePreset::UpdatePack, true).await?;
    
    // Remove unnecessary media files
    remove_unneed_media_files("./BMSRoot", Some(get_remove_media_rule_oraja())).await?;
    
    // Raw pack to HQ pack
    pack_raw_to_hq("./BMSRoot").await?;
    
    Ok(())
}
```

## Detailed Documentation

- [Command Line Usage Guide](README_CLI.md) - Complete command line operation instructions
- [Pack Module Documentation](README_PACK.md) - Detailed explanation of pack processing functionality

## Module Structure

```
bms-resource-toolbox/
├── src/
│   ├── main.rs              # Command line application entry
│   ├── lib.rs               # Library entry
│   ├── options/             # Main functionality modules
│   │   ├── work.rs          # Work directory operations
│   │   ├── root.rs          # Root directory operations
│   │   ├── root_bigpack.rs  # Big pack root directory operations
│   │   └── pack.rs          # Pack processing
│   ├── fs/                  # File system operations
│   ├── media/               # Media processing
│   └── bms/                 # BMS file processing
├── examples/                # Usage examples
└── README_CLI.md           # Command line usage guide
```

## Development

This project includes automatic Nix integration using rust-overlay for a consistent and up-to-date development environment.

### Quick Setup

#### With Nix (Recommended)
```bash
# Enter development environment (includes all dependencies)
nix develop

# Build the project
cargo build

# Run tests
cargo test
```

#### Without Nix
```bash
# Ensure LIBCLANG_PATH is set for bindgen
export LIBCLANG_PATH=/path/to/llvm/lib

# Build the project
cargo build
```

### Run Tests

```bash
cargo test
```

### Run Examples

```bash
cargo run --example basic_usage
```

### Build Release Version

```bash
cargo build --release
```

### Cursor IDE Integration

This project includes Cursor rules for enhanced development experience:

- **Nix Integration**: Automatic Nix environment detection
- **Rust Development**: Common cargo commands and workflows
- **BMS Project**: Project-specific knowledge and workflows

Rules are located in `.cursor/rules/` and provide contextual assistance for:
- Development environment setup
- Build and test commands
- BMS-specific workflows
- Project structure understanding

## Dependencies

- **smol**: Async runtime
- **clap**: Command line argument parsing
- **tokio**: Async runtime (command line application)
- **regex**: Regular expression support
- **zip/sevenz-rust/unrar**: Compression file support

## External Tool Dependencies

Some features require external tools:
- **ffmpeg**: Audio and video conversion
- **flac**: FLAC format processing
- **oggenc**: OGG format processing

## License

Apache License 2.0
