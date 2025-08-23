pub mod bms;
pub mod fs;
pub mod media;
pub mod options;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::{
    bms::{
        get_dir_bms_info, get_dir_bms_list, is_root_dir, is_work_dir, parse_bms_file,
        parse_bmson_file,
    },
    fs::{bms_dir_similarity, is_dir_having_file, is_file_same_content, remove_empty_folders},
    options::{
        pack::{
            pack_hq_to_lq, pack_raw_to_hq, pack_setup_rawpack_to_hq, pack_update_rawpack_to_hq,
        },
        root::{
            copy_numbered_workdir_names, scan_folder_similar_folders,
            set_name_by_bms as root_set_name_by_bms,
        },
        root_bigpack::{
            get_remove_media_rule_mpg_fill_wmv, get_remove_media_rule_oraja,
            get_remove_media_rule_wav_fill_flac, merge_split_folders, move_out_works,
            move_works_in_pack, move_works_with_same_name, remove_unneed_media_files,
            split_folders_with_first_char, undo_split_pack,
        },
        root_event::{check_num_folder, create_num_folders, generate_work_info_table},
        work::{
            BmsFolderSetNameType, remove_zero_sized_media_files, set_name_by_bms, undo_set_name,
        },
    },
};

#[derive(Parser)]
#[command(name = "be-music-cabinet")]
#[command(about = "Be-Music Source File Manager")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Work directory related operations
    Work {
        #[command(subcommand)]
        command: WorkCommands,
    },
    /// Root directory related operations
    Root {
        #[command(subcommand)]
        command: RootCommands,
    },
    /// Pack processing related operations
    Pack {
        #[command(subcommand)]
        command: PackCommands,
    },
    /// BMS file related operations
    Bms {
        #[command(subcommand)]
        command: BmsCommands,
    },
    /// File system related operations
    Fs {
        #[command(subcommand)]
        command: FsCommands,
    },
    /// Root directory event related operations
    RootEvent {
        #[command(subcommand)]
        command: RootEventCommands,
    },
}

#[derive(Subcommand)]
pub enum WorkCommands {
    /// Set directory name based on BMS file
    SetName {
        /// Work directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// Set type: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "append_title_artist")]
        set_type: String,
    },
    /// Undo directory name setting
    UndoSetName {
        /// Work directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// Set type: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "append_title_artist")]
        set_type: String,
    },
    /// Remove zero-byte media files
    RemoveEmptyMedia {
        /// Work directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum BmsCommands {
    /// Parse BMS file
    ParseBms {
        /// BMS file path
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Parse BMSON file
    ParseBmson {
        /// BMSON file path
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Get BMS file list in directory
    GetBmsList {
        /// Directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Get BMS information in directory
    GetBmsInfo {
        /// Directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Check if it's a work directory
    IsWorkDir {
        /// Directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Check if it's a root directory
    IsRootDir {
        /// Directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum FsCommands {
    /// Check if two files have the same content
    IsFileSame {
        /// First file path
        #[arg(value_name = "FILE1")]
        file1: PathBuf,
        /// Second file path
        #[arg(value_name = "FILE2")]
        file2: PathBuf,
    },
    /// Check if directory contains files
    IsDirHavingFile {
        /// Directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Remove empty folders
    RemoveEmptyFolders {
        /// Parent directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Calculate BMS directory similarity
    BmsDirSimilarity {
        /// First directory path
        #[arg(value_name = "DIR1")]
        dir1: PathBuf,
        /// Second directory path
        #[arg(value_name = "DIR2")]
        dir2: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum RootEventCommands {
    /// Check numbered folders
    CheckNumFolder {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// Maximum count
        #[arg(value_name = "MAX")]
        max: usize,
    },
    /// Create numbered folders
    CreateNumFolders {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// Folder count
        #[arg(value_name = "COUNT")]
        count: usize,
    },
    /// Generate work information table
    GenerateWorkInfoTable {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum RootCommands {
    /// Set directory name based on BMS file
    SetName {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// Set type: replace_title_artist, append_title_artist, append_artist
        /// See [`get_set_name_type`] for details
        #[arg(long, default_value = "replace")]
        set_type: String,
    },
    /// Copy numbered work directory names
    CopyNumberedNames {
        /// Source directory path
        #[arg(value_name = "FROM")]
        from: PathBuf,
        /// Target directory path
        #[arg(value_name = "TO")]
        to: PathBuf,
    },
    /// Split folders by first character
    SplitByFirstChar {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Undo split operation
    UndoSplit {
        /// Target directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Merge split folders
    MergeSplit {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Move works
    MoveWorks {
        /// Source directory path
        #[arg(value_name = "FROM")]
        from: PathBuf,
        /// Target directory path
        #[arg(value_name = "TO")]
        to: PathBuf,
    },
    /// Move out one level directory
    MoveOutWorks {
        /// Target root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Move works with same name
    MoveSameName {
        /// Source directory path
        #[arg(value_name = "FROM")]
        from: PathBuf,
        /// Target directory path
        #[arg(value_name = "TO")]
        to: PathBuf,
    },
    /// Remove unnecessary media files
    RemoveUnneedMedia {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// Rule type: oraja, wav_fill_flac, mpg_fill_wmv
        #[arg(long, default_value = "oraja")]
        rule: String,
    },
    /// Scan similar folders
    ScanSimilarFolders {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// Similarity threshold
        #[arg(long, default_value = "0.7")]
        similarity: f64,
    },
}

#[derive(Subcommand)]
pub enum PackCommands {
    /// Raw pack -> HQ pack
    RawToHq {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// HQ pack -> LQ pack
    HqToLq {
        /// Root directory path
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// Pack generation script: Raw pack -> HQ pack
    SetupRawpackToHq {
        /// Pack directory path
        #[arg(value_name = "PACK_DIR")]
        pack_dir: PathBuf,
        /// Root directory path
        #[arg(value_name = "ROOT_DIR")]
        root_dir: PathBuf,
    },
    /// Pack update script: Raw pack -> HQ pack
    UpdateRawpackToHq {
        /// Pack directory path
        #[arg(value_name = "PACK_DIR")]
        pack_dir: PathBuf,
        /// Root directory path
        #[arg(value_name = "ROOT_DIR")]
        root_dir: PathBuf,
        /// Sync directory path
        #[arg(value_name = "SYNC_DIR")]
        sync_dir: PathBuf,
    },
}

fn get_set_name_type(set_type: &str) -> BmsFolderSetNameType {
    match set_type {
        "replace" | "replace_title_artist" => BmsFolderSetNameType::ReplaceTitleArtist,
        "append" | "append_title_artist" => BmsFolderSetNameType::AppendTitleArtist,
        "append_artist" => BmsFolderSetNameType::AppendArtist,
        _ => BmsFolderSetNameType::ReplaceTitleArtist,
    }
}

pub async fn run_command(command: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Work { command } => match command {
            WorkCommands::SetName { dir, set_type } => {
                println!("Setting directory name: {}", dir.display());
                set_name_by_bms(dir, get_set_name_type(set_type)).await?;
                println!("Setting completed");
            }
            WorkCommands::UndoSetName { dir, set_type } => {
                println!("Undoing directory name setting: {}", dir.display());
                undo_set_name(dir, get_set_name_type(set_type)).await?;
                println!("Undo completed");
            }
            WorkCommands::RemoveEmptyMedia { dir } => {
                println!("Removing zero-byte media files: {}", dir.display());
                remove_zero_sized_media_files(dir).await?;
                println!("Removal completed");
            }
        },
        Commands::Root { command } => match command {
            RootCommands::SetName { dir, set_type } => {
                println!("Setting directory name: {}", dir.display());
                let set_type = get_set_name_type(set_type);
                println!("Set type: {:?}", set_type);
                root_set_name_by_bms(dir, set_type).await?;
                println!("Setting completed");
            }
            RootCommands::CopyNumberedNames { from, to } => {
                println!(
                    "Copying numbered work directory names: {} -> {}",
                    from.display(),
                    to.display()
                );
                copy_numbered_workdir_names(from, to).await?;
                println!("Copy completed");
            }
            RootCommands::SplitByFirstChar { dir } => {
                println!("Splitting folders by first character: {}", dir.display());
                split_folders_with_first_char(dir).await?;
                println!("Split completed");
            }
            RootCommands::UndoSplit { dir } => {
                println!("Undoing split operation: {}", dir.display());
                undo_split_pack(dir).await?;
                println!("Undo completed");
            }
            RootCommands::MergeSplit { dir } => {
                println!("Merging split folders: {}", dir.display());
                merge_split_folders(dir).await?;
                println!("Merge completed");
            }
            RootCommands::MoveWorks { from, to } => {
                println!("Moving works: {} -> {}", from.display(), to.display());
                move_works_in_pack(from, to).await?;
                println!("Move completed");
            }
            RootCommands::MoveOutWorks { dir } => {
                println!("Moving out one level directory: {}", dir.display());
                move_out_works(dir).await?;
                println!("Move out completed");
            }
            RootCommands::MoveSameName { from, to } => {
                println!(
                    "Moving works with same name: {} -> {}",
                    from.display(),
                    to.display()
                );
                move_works_with_same_name(from, to).await?;
                println!("Move completed");
            }
            RootCommands::RemoveUnneedMedia { dir, rule } => {
                println!(
                    "Removing unnecessary media files: {} (rule: {})",
                    dir.display(),
                    rule
                );
                let rule_config = match rule.as_str() {
                    "oraja" => Some(get_remove_media_rule_oraja()),
                    "wav_fill_flac" => Some(get_remove_media_rule_wav_fill_flac()),
                    "mpg_fill_wmv" => Some(get_remove_media_rule_mpg_fill_wmv()),
                    _ => None,
                };
                remove_unneed_media_files(dir, rule_config).await?;
                println!("Removal completed");
            }
            RootCommands::ScanSimilarFolders { dir, similarity } => {
                println!(
                    "Scanning similar folders: {} (similarity threshold: {})",
                    dir.display(),
                    similarity
                );
                let results = scan_folder_similar_folders(dir, *similarity).await?;
                for (former, current, sim) in results {
                    println!("Similarity {:.3}: {} <-> {}", sim, former, current);
                }
                println!("Scan completed");
            }
        },
        Commands::Pack { command } => match command {
            PackCommands::RawToHq { dir } => {
                println!("Raw pack -> HQ pack: {}", dir.display());
                pack_raw_to_hq(dir).await?;
                println!("Conversion completed");
            }
            PackCommands::HqToLq { dir } => {
                println!("HQ pack -> LQ pack: {}", dir.display());
                pack_hq_to_lq(dir).await?;
                println!("Conversion completed");
            }
            PackCommands::SetupRawpackToHq { pack_dir, root_dir } => {
                println!(
                    "Pack generation script: {} -> {}",
                    pack_dir.display(),
                    root_dir.display()
                );
                pack_setup_rawpack_to_hq(pack_dir, root_dir).await?;
                println!("Generation completed");
            }
            PackCommands::UpdateRawpackToHq {
                pack_dir,
                root_dir,
                sync_dir,
            } => {
                println!(
                    "Pack update script: {} -> {} (sync: {})",
                    pack_dir.display(),
                    root_dir.display(),
                    sync_dir.display()
                );
                pack_update_rawpack_to_hq(pack_dir, root_dir, sync_dir).await?;
                println!("Update completed");
            }
        },
        Commands::Bms { command } => match command {
            BmsCommands::ParseBms { file } => {
                println!("Parsing BMS file: {}", file.display());
                let result = parse_bms_file(file).await?;
                println!("Parse result: {:?}", result);
            }
            BmsCommands::ParseBmson { file } => {
                println!("Parsing BMSON file: {}", file.display());
                let result = parse_bmson_file(file).await?;
                println!("Parse result: {:?}", result);
            }
            BmsCommands::GetBmsList { dir } => {
                println!("Getting BMS file list: {}", dir.display());
                let results = get_dir_bms_list(dir).await?;
                println!("Found {} BMS files", results.len());
                for (i, bms) in results.iter().enumerate() {
                    println!("  {}. {:?}", i + 1, bms);
                }
            }
            BmsCommands::GetBmsInfo { dir } => {
                println!("Getting BMS information: {}", dir.display());
                let result = get_dir_bms_info(dir).await?;
                match result {
                    Some(info) => println!("BMS information: {:?}", info),
                    None => println!("No BMS information found"),
                }
            }
            BmsCommands::IsWorkDir { dir } => {
                println!("Checking if it's a work directory: {}", dir.display());
                let result = is_work_dir(dir).await?;
                println!("Is work directory: {}", result);
            }
            BmsCommands::IsRootDir { dir } => {
                println!("Checking if it's a root directory: {}", dir.display());
                let result = is_root_dir(dir).await?;
                println!("Is root directory: {}", result);
            }
        },
        Commands::Fs { command } => match command {
            FsCommands::IsFileSame { file1, file2 } => {
                println!(
                    "Checking if files have same content: {} <-> {}",
                    file1.display(),
                    file2.display()
                );
                let result = is_file_same_content(file1, file2).await?;
                println!("Files have same content: {}", result);
            }
            FsCommands::IsDirHavingFile { dir } => {
                println!("Checking if directory contains files: {}", dir.display());
                let result = is_dir_having_file(dir).await?;
                println!("Directory contains files: {}", result);
            }
            FsCommands::RemoveEmptyFolders { dir } => {
                println!("Removing empty folders: {}", dir.display());
                remove_empty_folders(dir).await?;
                println!("Removal completed");
            }
            FsCommands::BmsDirSimilarity { dir1, dir2 } => {
                println!(
                    "Calculating BMS directory similarity: {} <-> {}",
                    dir1.display(),
                    dir2.display()
                );
                let result = bms_dir_similarity(&dir1, &dir2).await?;
                println!("Similarity: {:.3}", result);
            }
        },
        Commands::RootEvent { command } => match command {
            RootEventCommands::CheckNumFolder { dir, max } => {
                println!(
                    "Checking numbered folders: {} (max count: {})",
                    dir.display(),
                    max
                );
                let results = check_num_folder(dir, *max).await?;
                println!("Found {} numbered folders", results.len());
                for path in results {
                    println!("  {}", path.display());
                }
            }
            RootEventCommands::CreateNumFolders { dir, count } => {
                println!(
                    "Creating numbered folders: {} (count: {})",
                    dir.display(),
                    count
                );
                create_num_folders(dir, *count).await?;
                println!("Creation completed");
            }
            RootEventCommands::GenerateWorkInfoTable { dir } => {
                println!("Generating work information table: {}", dir.display());
                generate_work_info_table(dir).await?;
                println!("Generation completed");
            }
        },
    }

    Ok(())
}
