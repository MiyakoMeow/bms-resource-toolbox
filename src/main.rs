//! BMS Resource Toolbox - BMS chart resource management CLI
//!
//! A command-line tool for managing BMS (Beatmania) chart resources,
//! including audio/video conversion, pack generation, and archive handling.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod bms;
mod error;
mod fs;
mod media;
mod options;
mod scripts;

use scripts::pack::{pack_hq_to_lq, pack_raw_to_hq, pack_setup_rawpack_to_hq, pack_update_rawpack_to_hq};

/// BMS Resource Toolbox - BMS chart resource management tool
#[derive(Parser, Debug)]
#[command(name = "bms-resource-toolbox")]
#[command(about = "BMS Resource Management Tool", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Pack raw BMS to HQ version (for beatoraja/Qwilight)
    PackRawToHq {
        /// Root directory containing BMS folders
        #[arg(short, long)]
        root: PathBuf,
    },
    /// Pack HQ BMS to LQ version (for LR2)
    PackHqToLq {
        /// Root directory containing BMS folders
        #[arg(short, long)]
        root: PathBuf,
    },
    /// Setup new HQ pack from raw packs
    PackSetupRawpackToHq {
        /// Pack directory containing numbered archives
        #[arg(short, long)]
        pack_dir: PathBuf,
        /// Root directory for output
        #[arg(short, long)]
        root: PathBuf,
    },
    /// Update existing HQ pack from raw packs
    PackUpdateRawpackToHq {
        /// Pack directory containing numbered archives
        #[arg(short, long)]
        pack_dir: PathBuf,
        /// Root directory for output
        #[arg(short, long)]
        root: PathBuf,
        /// Existing BMS directory for sync
        #[arg(short, long)]
        sync_dir: PathBuf,
    },
    /// Parse a BMS file and display info
    Parse {
        /// BMS file path
        #[arg(short, long)]
        file: PathBuf,
        /// Force encoding (e.g., shift-jis, gbk)
        #[arg(short, long)]
        encoding: Option<String>,
    },
    /// Check external tool availability
    CheckTools,
    /// Convert audio files
    ConvertAudio {
        /// Input directory
        #[arg(short, long)]
        dir: PathBuf,
        /// Input extension (e.g., wav, flac)
        #[arg(short, long)]
        input_ext: String,
        /// Output extension (e.g., flac, ogg)
        #[arg(short, long)]
        output_ext: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    match cli.command {
        Commands::PackRawToHq { root } => {
            info!("Pack RAW -> HQ: {:?}", root);
            pack_raw_to_hq(&root).await?;
        }
        Commands::PackHqToLq { root } => {
            info!("Pack HQ -> LQ: {:?}", root);
            pack_hq_to_lq(&root).await?;
        }
        Commands::PackSetupRawpackToHq { pack_dir, root } => {
            info!("Pack Setup RAW -> HQ: {:?} -> {:?}", pack_dir, root);
            pack_setup_rawpack_to_hq(&pack_dir, &root).await?;
        }
        Commands::PackUpdateRawpackToHq {
            pack_dir,
            root,
            sync_dir,
        } => {
            info!(
                "Pack Update RAW -> HQ: {:?} -> {:?} (sync from {:?})",
                pack_dir, root, sync_dir
            );
            pack_update_rawpack_to_hq(&pack_dir, &root, &sync_dir).await?;
        }
        Commands::Parse { file, encoding: _ } => {
            info!("Parse BMS file: {:?}", file);
            let content = bms::encoding::read_bms_file(&file)?;
            let info = bms::parse::parse_bms_content(&content);
            println!("Title: {}", info.title);
            println!("Artist: {}", info.artist);
            println!("Genre: {}", info.genre);
            println!("Playlevel: {}", info.playlevel);
            println!("Difficulty: {:?}", info.difficulty);
        }
        Commands::CheckTools => {
            println!("Checking external tools...");
            println!("ffmpeg: {}", options::check_ffmpeg().await);
            println!("flac: {}", options::check_flac().await);
            println!("oggenc: {}", options::check_oggenc().await);
        }
        Commands::ConvertAudio {
            dir,
            input_ext,
            output_ext,
        } => {
            info!(
                "Convert audio: {:?} from {} to {}",
                dir, input_ext, output_ext
            );
            // TODO: Implement audio conversion with custom preset
        }
    }

    Ok(())
}
