pub mod bms;
pub mod fs;
pub mod media;
pub mod options;

use clap::{Parser, Subcommand};
use log::info;
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
        rawpack::{set_file_num, unzip_numeric_to_bms_folder, unzip_with_name_to_bms_folder},
        root::{
            copy_numbered_workdir_names, scan_folder_similar_folders,
            set_name_by_bms as root_set_name_by_bms,
            undo_set_name_by_bms as root_undo_set_name_by_bms,
        },
        root_bigpack::{
            RemoveMediaPreset, get_remove_media_rule_by_preset, merge_split_folders,
            move_out_works, move_works_in_pack, move_works_with_same_name,
            remove_unneed_media_files, split_folders_with_first_char, undo_split_pack,
        },
        root_event::{check_num_folder, create_num_folders, generate_work_info_table},
        work::{
            BmsFolderSetNameType, remove_zero_sized_media_files, set_name_by_bms,
            undo_set_name_by_bms,
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
    /// Root directory related operations
    ///
    /// - Dir structure in this program: `root_dir/work_dir/xxx.bms`
    Root {
        #[command(subcommand)]
        command: RootCommands,
    },
    /// Work directory related operations
    ///
    /// - Dir structure in this program: `root_dir/work_dir/xxx.bms`
    Work {
        #[command(subcommand)]
        command: WorkCommands,
    },
    /// Pack processing related operations
    ///
    /// - Dir structure in this program: `root_dir/work_dir/xxx.bms`
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
    ///
    /// - Dir structure in this program: `root_dir/work_dir/xxx.bms`
    RootEvent {
        #[command(subcommand)]
        command: RootEventCommands,
    },
    /// Raw pack processing related operations
    Rawpack {
        #[command(subcommand)]
        command: RawpackCommands,
    },
}

#[derive(Subcommand)]
pub enum WorkCommands {
    /// Set directory name based on BMS file
    SetName {
        /// Work directory path
        #[arg(value_name = "Work directory")]
        dir: PathBuf,
        /// Set type: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "replace_title_artist", value_name = "Set type")]
        set_type: BmsFolderSetNameType,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Undo directory name setting
    UndoSetName {
        /// Work directory path
        #[arg(value_name = "Work directory")]
        dir: PathBuf,
        /// Set type: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "append_artist", value_name = "Set type")]
        set_type: BmsFolderSetNameType,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Remove zero-byte media files
    RemoveEmptyMedia {
        /// Work directory path
        #[arg(value_name = "Work directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum BmsCommands {
    /// Parse BMS file
    ParseBms {
        /// BMS file path
        #[arg(value_name = "BMS file")]
        file: PathBuf,
    },
    /// Parse BMSON file
    ParseBmson {
        /// BMSON file path
        #[arg(value_name = "BMSON file")]
        file: PathBuf,
    },
    /// Get BMS file list in directory
    GetBmsList {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
    /// Get BMS information in directory
    GetBmsInfo {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
    /// Check if it's a work directory
    IsWorkDir {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
    /// Check if it's a root directory
    IsRootDir {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum FsCommands {
    /// Check if two files have the same content
    IsFileSame {
        /// First file path
        #[arg(value_name = "First file")]
        file1: PathBuf,
        /// Second file path
        #[arg(value_name = "Second file")]
        file2: PathBuf,
    },
    /// Check if directory contains files
    IsDirHavingFile {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
    /// Remove empty folders
    RemoveEmptyFolders {
        /// Parent directory path
        #[arg(value_name = "Parent directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Calculate BMS directory similarity
    BmsDirSimilarity {
        /// First directory path
        #[arg(value_name = "First directory")]
        dir1: PathBuf,
        /// Second directory path
        #[arg(value_name = "Second directory")]
        dir2: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum RootEventCommands {
    /// Check numbered folders
    CheckNumFolder {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Maximum count
        #[arg(value_name = "Maximum count")]
        max: usize,
    },
    /// Create numbered folders
    CreateNumFolders {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Folder count
        #[arg(value_name = "Folder count")]
        count: usize,
    },
    /// Generate work information table
    GenerateWorkInfoTable {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum RawpackCommands {
    /// Extract numerically named pack files to BMS folders
    UnzipNumericToBmsFolder {
        /// Pack directory path
        #[arg(value_name = "Pack directory")]
        pack_dir: PathBuf,
        /// Cache directory path
        #[arg(value_name = "Cache directory")]
        cache_dir: PathBuf,
        /// Root directory path
        #[arg(value_name = "Root directory")]
        root_dir: PathBuf,
        /// Confirm before processing
        #[arg(long, value_name = "Confirm")]
        confirm: bool,
    },
    /// Extract files with names to BMS folders
    UnzipWithNameToBmsFolder {
        /// Pack directory path
        #[arg(value_name = "Pack directory")]
        pack_dir: PathBuf,
        /// Cache directory path
        #[arg(value_name = "Cache directory")]
        cache_dir: PathBuf,
        /// Root directory path
        #[arg(value_name = "Root directory")]
        root_dir: PathBuf,
        /// Confirm before processing
        #[arg(long, value_name = "Confirm")]
        confirm: bool,
    },
    /// Set file number (interactive)
    SetFileNum {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum RootCommands {
    /// Set directory name based on BMS file
    SetName {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Set type: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "replace_title_artist", value_name = "Set type")]
        set_type: BmsFolderSetNameType,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Undo directory name setting
    UndoSetName {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Set type: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "append_artist", value_name = "Set type")]
        set_type: BmsFolderSetNameType,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Copy numbered work directory names
    CopyNumberedNames {
        /// Source directory path
        #[arg(value_name = "Source directory")]
        from: PathBuf,
        /// Target directory path
        #[arg(value_name = "Target directory")]
        to: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Split folders by first character
    SplitByFirstChar {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Undo split operation
    UndoSplit {
        /// Target directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Merge split folders
    MergeSplit {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Move works
    MoveWorks {
        /// Source directory path
        #[arg(value_name = "Source directory")]
        from: PathBuf,
        /// Target directory path
        #[arg(value_name = "Target directory")]
        to: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Move out one level directory
    MoveOutWorks {
        /// Target root directory path
        #[arg(value_name = "Target root directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long)]
        dry_run: bool,
    },
    /// Move works with same name
    MoveSameName {
        /// Source directory path
        #[arg(value_name = "Source directory")]
        from: PathBuf,
        /// Target directory path
        #[arg(value_name = "Target directory")]
        to: PathBuf,
        /// Dry run: only print actions
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove unnecessary media files
    RemoveUnneedMedia {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Rule preset
        #[arg(long, value_enum, default_value = "oraja", value_name = "Rule preset")]
        rule: RemoveMediaPreset,
    },
    /// Scan similar folders
    ScanSimilarFolders {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Similarity threshold
        #[arg(long, default_value = "0.7", value_name = "Similarity")]
        similarity: f64,
    },
}

#[derive(Subcommand)]
pub enum PackCommands {
    /// Raw pack -> HQ pack
    RawToHq {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
    },
    /// HQ pack -> LQ pack
    HqToLq {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
    },
    /// Pack generation script: Raw pack -> HQ pack
    SetupRawpackToHq {
        /// Pack directory path
        #[arg(value_name = "Pack directory")]
        pack_dir: PathBuf,
        /// Root directory path
        #[arg(value_name = "Root directory")]
        root_dir: PathBuf,
    },
    /// Pack update script: Raw pack -> HQ pack
    UpdateRawpackToHq {
        /// Pack directory path
        #[arg(value_name = "Pack directory")]
        pack_dir: PathBuf,
        /// Root directory path
        #[arg(value_name = "Root directory")]
        root_dir: PathBuf,
        /// Sync directory path
        #[arg(value_name = "Sync directory")]
        sync_dir: PathBuf,
    },
}

pub async fn run_command(command: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Work { command } => match command {
            WorkCommands::SetName {
                dir,
                set_type,
                dry_run,
            } => {
                info!("Setting directory name: {}", dir.display());
                info!("Set type: {:?}", set_type);
                set_name_by_bms(dir, *set_type, *dry_run).await?;
                info!("Setting completed");
            }
            WorkCommands::UndoSetName {
                dir,
                set_type,
                dry_run,
            } => {
                info!("Undoing directory name setting: {}", dir.display());
                undo_set_name_by_bms(dir, *set_type, *dry_run).await?;
                info!("Undo completed");
            }
            WorkCommands::RemoveEmptyMedia { dir, dry_run } => {
                info!("Removing zero-byte media files: {}", dir.display());
                remove_zero_sized_media_files(dir, *dry_run).await?;
                info!("Removal completed");
            }
        },
        Commands::Root { command } => match command {
            RootCommands::SetName {
                dir,
                set_type,
                dry_run,
            } => {
                info!("Setting directory name: {}", dir.display());
                info!("Set type: {:?}", set_type);
                root_set_name_by_bms(dir, *set_type, *dry_run).await?;
                info!("Setting completed");
            }
            RootCommands::UndoSetName {
                dir,
                set_type,
                dry_run,
            } => {
                info!("Undoing directory name setting: {}", dir.display());
                root_undo_set_name_by_bms(dir, *set_type, *dry_run).await?;
                info!("Undo completed");
            }
            RootCommands::CopyNumberedNames { from, to, dry_run } => {
                info!(
                    "Copying numbered work directory names: {} -> {}",
                    from.display(),
                    to.display()
                );
                copy_numbered_workdir_names(from, to, *dry_run).await?;
                info!("Copy completed");
            }
            RootCommands::SplitByFirstChar { dir, dry_run } => {
                info!("Splitting folders by first character: {}", dir.display());
                split_folders_with_first_char(dir, *dry_run).await?;
                info!("Split completed");
            }
            RootCommands::UndoSplit { dir, dry_run } => {
                info!("Undoing split operation: {}", dir.display());
                undo_split_pack(dir, *dry_run).await?;
                info!("Undo completed");
            }
            RootCommands::MergeSplit { dir, dry_run } => {
                info!("Merging split folders: {}", dir.display());
                merge_split_folders(dir, *dry_run).await?;
                info!("Merge completed");
            }
            RootCommands::MoveWorks { from, to, dry_run } => {
                info!("Moving works: {} -> {}", from.display(), to.display());
                move_works_in_pack(from, to, *dry_run).await?;
                info!("Move completed");
            }
            RootCommands::MoveOutWorks { dir, dry_run } => {
                info!("Moving out one level directory: {}", dir.display());
                move_out_works(dir, *dry_run).await?;
                info!("Move out completed");
            }
            RootCommands::MoveSameName { from, to, dry_run } => {
                info!(
                    "Moving works with same name: {} -> {}",
                    from.display(),
                    to.display()
                );
                move_works_with_same_name(from, to, *dry_run).await?;
                info!("Move completed");
            }
            RootCommands::RemoveUnneedMedia { dir, rule } => {
                info!(
                    "Removing unnecessary media files: {} (rule: {:?})",
                    dir.display(),
                    rule
                );
                let rule_config = get_remove_media_rule_by_preset(*rule);
                remove_unneed_media_files(dir, rule_config).await?;
                info!("Removal completed");
            }
            RootCommands::ScanSimilarFolders { dir, similarity } => {
                info!(
                    "Scanning similar folders: {} (similarity threshold: {})",
                    dir.display(),
                    similarity
                );
                let results = scan_folder_similar_folders(dir, *similarity).await?;
                for (former, current, sim) in results {
                    info!("Similarity {:.3}: {} <-> {}", sim, former, current);
                }
                info!("Scan completed");
            }
        },
        Commands::Pack { command } => match command {
            PackCommands::RawToHq { dir } => {
                info!("Raw pack -> HQ pack: {}", dir.display());
                pack_raw_to_hq(dir).await?;
                info!("Conversion completed");
            }
            PackCommands::HqToLq { dir } => {
                info!("HQ pack -> LQ pack: {}", dir.display());
                pack_hq_to_lq(dir).await?;
                info!("Conversion completed");
            }
            PackCommands::SetupRawpackToHq { pack_dir, root_dir } => {
                info!(
                    "Pack generation script: {} -> {}",
                    pack_dir.display(),
                    root_dir.display()
                );
                pack_setup_rawpack_to_hq(pack_dir, root_dir).await?;
                info!("Generation completed");
            }
            PackCommands::UpdateRawpackToHq {
                pack_dir,
                root_dir,
                sync_dir,
            } => {
                info!(
                    "Pack update script: {} -> {} (sync: {})",
                    pack_dir.display(),
                    root_dir.display(),
                    sync_dir.display()
                );
                pack_update_rawpack_to_hq(pack_dir, root_dir, sync_dir).await?;
                info!("Update completed");
            }
        },
        Commands::Bms { command } => match command {
            BmsCommands::ParseBms { file } => {
                info!("Parsing BMS file: {}", file.display());
                let result = parse_bms_file(file).await?;
                info!("Parse result: {:?}", result);
            }
            BmsCommands::ParseBmson { file } => {
                info!("Parsing BMSON file: {}", file.display());
                let result = parse_bmson_file(file).await?;
                info!("Parse result: {:?}", result);
            }
            BmsCommands::GetBmsList { dir } => {
                info!("Getting BMS file list: {}", dir.display());
                let results = get_dir_bms_list(dir).await?;
                info!("Found {} BMS files", results.len());
                for (i, bms) in results.iter().enumerate() {
                    info!("  {}. {:?}", i + 1, bms);
                }
            }
            BmsCommands::GetBmsInfo { dir } => {
                info!("Getting BMS information: {}", dir.display());
                let result = get_dir_bms_info(dir).await?;
                match result {
                    Some(info) => info!("BMS information: {:?}", info),
                    None => info!("No BMS information found"),
                }
            }
            BmsCommands::IsWorkDir { dir } => {
                info!("Checking if it's a work directory: {}", dir.display());
                let result = is_work_dir(dir).await?;
                info!("Is work directory: {}", result);
            }
            BmsCommands::IsRootDir { dir } => {
                info!("Checking if it's a root directory: {}", dir.display());
                let result = is_root_dir(dir).await?;
                info!("Is root directory: {}", result);
            }
        },
        Commands::Fs { command } => match command {
            FsCommands::IsFileSame { file1, file2 } => {
                info!(
                    "Checking if files have same content: {} <-> {}",
                    file1.display(),
                    file2.display()
                );
                let result = is_file_same_content(file1, file2).await?;
                info!("Files have same content: {}", result);
            }
            FsCommands::IsDirHavingFile { dir } => {
                info!("Checking if directory contains files: {}", dir.display());
                let result = is_dir_having_file(dir).await?;
                info!("Directory contains files: {}", result);
            }
            FsCommands::RemoveEmptyFolders { dir, dry_run } => {
                info!("Removing empty folders: {}", dir.display());
                remove_empty_folders(dir, *dry_run).await?;
                info!("Removal completed");
            }
            FsCommands::BmsDirSimilarity { dir1, dir2 } => {
                info!(
                    "Calculating BMS directory similarity: {} <-> {}",
                    dir1.display(),
                    dir2.display()
                );
                let result = bms_dir_similarity(&dir1, &dir2).await?;
                info!("Similarity: {:.3}", result);
            }
        },
        Commands::RootEvent { command } => match command {
            RootEventCommands::CheckNumFolder { dir, max } => {
                info!(
                    "Checking numbered folders: {} (max count: {})",
                    dir.display(),
                    max
                );
                let results = check_num_folder(dir, *max).await?;
                info!("Found {} numbered folders", results.len());
                for path in results {
                    info!("  {}", path.display());
                }
            }
            RootEventCommands::CreateNumFolders { dir, count } => {
                info!(
                    "Creating numbered folders: {} (count: {})",
                    dir.display(),
                    count
                );
                create_num_folders(dir, *count).await?;
                info!("Creation completed");
            }
            RootEventCommands::GenerateWorkInfoTable { dir } => {
                info!("Generating work information table: {}", dir.display());
                generate_work_info_table(dir).await?;
                info!("Generation completed");
            }
        },
        Commands::Rawpack { command } => match command {
            RawpackCommands::UnzipNumericToBmsFolder {
                pack_dir,
                cache_dir,
                root_dir,
                confirm,
            } => {
                info!(
                    "Extracting numerically named pack files: {} -> {} (cache: {})",
                    pack_dir.display(),
                    root_dir.display(),
                    cache_dir.display()
                );
                unzip_numeric_to_bms_folder(pack_dir, cache_dir, root_dir, *confirm).await?;
                info!("Extraction completed");
            }
            RawpackCommands::UnzipWithNameToBmsFolder {
                pack_dir,
                cache_dir,
                root_dir,
                confirm,
            } => {
                info!(
                    "Extracting files with names: {} -> {} (cache: {})",
                    pack_dir.display(),
                    root_dir.display(),
                    cache_dir.display()
                );
                unzip_with_name_to_bms_folder(pack_dir, cache_dir, root_dir, *confirm).await?;
                info!("Extraction completed");
            }
            RawpackCommands::SetFileNum { dir } => {
                info!("Setting file numbers: {}", dir.display());
                set_file_num(dir).await?;
                info!("Setting completed");
            }
        },
    }

    Ok(())
}
