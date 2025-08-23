//! be-music-cabinet basic usage example
//!
//! This example demonstrates how to use various features of be-music-cabinet

use be_music_cabinet::options::{
    pack::{pack_hq_to_lq, pack_raw_to_hq},
    root_bigpack::{get_remove_media_rule_oraja, remove_unneed_media_files},
    work::{BmsFolderSetNameType, set_name_by_bms},
};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        println!("be-music-cabinet basic usage example");
        println!("================================");

        // Example 1: Set BMS folder name
        println!("\n1. Set BMS folder name");
        let bms_dir = Path::new("./example_bms_folder");
        if bms_dir.exists() {
            println!("Setting directory name: {}", bms_dir.display());
            set_name_by_bms(bms_dir, BmsFolderSetNameType::AppendTitleArtist).await?;
            println!("Setting completed");
        } else {
            println!("Example directory does not exist: {}", bms_dir.display());
        }

        // Example 2: Remove unnecessary media files
        println!("\n2. Remove unnecessary media files");
        let root_dir = Path::new("./example_root");
        if root_dir.exists() {
            println!("Removing unnecessary media files: {}", root_dir.display());
            remove_unneed_media_files(root_dir, Some(get_remove_media_rule_oraja())).await?;
            println!("Removal completed");
        } else {
            println!("Example directory does not exist: {}", root_dir.display());
        }

        // Example 3: Raw pack to HQ pack
        println!("\n3. Raw pack to HQ pack");
        let raw_dir = Path::new("./example_raw");
        if raw_dir.exists() {
            println!("Converting raw pack to HQ: {}", raw_dir.display());
            pack_raw_to_hq(raw_dir).await?;
            println!("Conversion completed");
        } else {
            println!("Example directory does not exist: {}", raw_dir.display());
        }

        // Example 4: HQ pack to LQ pack
        println!("\n4. HQ pack to LQ pack");
        let hq_dir = Path::new("./example_hq");
        if hq_dir.exists() {
            println!("Converting HQ pack to LQ: {}", hq_dir.display());
            pack_hq_to_lq(hq_dir).await?;
            println!("Conversion completed");
        } else {
            println!("Example directory does not exist: {}", hq_dir.display());
        }

        println!("\nExample execution completed!");
        println!("\nTo use the command line version, run:");
        println!("  be-music-cabinet --help");
        println!("  be-music-cabinet work --help");
        println!("  be-music-cabinet root --help");
        println!("  be-music-cabinet pack --help");

        Ok(())
    })
}
