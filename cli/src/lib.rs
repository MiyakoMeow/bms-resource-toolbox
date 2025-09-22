pub mod bms;
pub mod fs;
pub mod media;
pub mod options;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use lang_derive::Localized;
use log::info;

use crate::{
    bms::{
        get_dir_bms_info, get_dir_bms_list, is_root_dir, is_work_dir, parse_bms_file,
        parse_bmson_file,
    },
    fs::{
        bms_dir_similarity, is_dir_having_file, is_file_same_content, moving::ReplacePreset,
        remove_empty_folders,
    },
    options::{
        bms_event::BMSEvent,
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
#[command(name = "bms-resource-toolbox")]
#[command(about = "Be-Music Source File Manager")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Localized)]
pub enum Commands {
    /// Root directory related operations
    ///
    /// - Dir structure in this program: `root_dir/work_dir/xxx.bms`
    #[lang_chinese(name = "根目录", desc = "根目录相关操作")]
    #[lang_english(name = "Root", desc = "Root directory related operations")]
    Root {
        #[command(subcommand)]
        command: RootCommands,
    },
    /// Work directory related operations
    ///
    /// - Dir structure in this program: `root_dir/work_dir/xxx.bms`
    #[lang_chinese(name = "作品目录", desc = "作品目录相关操作")]
    #[lang_english(name = "Work", desc = "Work directory related operations")]
    Work {
        #[command(subcommand)]
        command: WorkCommands,
    },
    /// Pack processing related operations
    ///
    /// - Dir structure in this program: `root_dir/work_dir/xxx.bms`
    #[lang_chinese(name = "打包处理", desc = "打包处理相关操作")]
    #[lang_english(name = "Pack", desc = "Pack processing related operations")]
    Pack {
        #[command(subcommand)]
        command: PackCommands,
    },
    /// BMS file related operations
    #[lang_chinese(name = "BMS 文件", desc = "BMS 文件相关操作")]
    #[lang_english(name = "Bms", desc = "BMS file related operations")]
    Bms {
        #[command(subcommand)]
        command: BmsCommands,
    },
    /// File system related operations
    #[lang_chinese(name = "文件系统", desc = "文件系统相关操作")]
    #[lang_english(name = "Fs", desc = "File system related operations")]
    Fs {
        #[command(subcommand)]
        command: FsCommands,
    },
    /// Root directory event related operations
    ///
    /// - Dir structure in this program: `root_dir/work_dir/xxx.bms`
    #[lang_chinese(name = "根目录事件", desc = "根目录事件相关操作")]
    #[lang_english(name = "RootEvent", desc = "Root directory event related operations")]
    RootEvent {
        #[command(subcommand)]
        command: RootEventCommands,
    },
    /// Raw pack processing related operations
    #[lang_chinese(name = "原始包处理", desc = "原始包处理相关操作")]
    #[lang_english(name = "Rawpack", desc = "Raw pack processing related operations")]
    Rawpack {
        #[command(subcommand)]
        command: RawpackCommands,
    },
    /// BMS event related operations
    #[lang_chinese(name = "BMS 活动", desc = "BMS 活动相关操作")]
    #[lang_english(name = "BmsEvent", desc = "BMS event related operations")]
    BmsEvent {
        #[command(subcommand)]
        command: BmsEventCommands,
    },
}

#[derive(Subcommand, Localized)]
pub enum WorkCommands {
    /// Set directory name based on BMS file
    #[lang_chinese(name = "按BMS设置目录名", desc = "根据BMS文件设置作品目录名称")]
    #[lang_english(name = "SetName", desc = "Set directory name based on BMS file")]
    SetName {
        /// Work directory path
        #[arg(value_name = "Work directory")]
        dir: PathBuf,
        /// Set type: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "replace_title_artist", value_name = "Set type")]
        set_type: BmsFolderSetNameType,
        /// Replace preset: default, update_pack
        #[arg(
            long,
            value_enum,
            default_value = "update_pack",
            value_name = "Replace preset"
        )]
        replace: ReplacePreset,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
        /// Skip directories that are already formatted
        #[arg(long, default_value = "true", value_name = "Skip already formatted")]
        skip_already_formatted: bool,
    },
    /// Undo directory name setting
    #[lang_chinese(name = "撤销设置目录名", desc = "撤销目录名设置")]
    #[lang_english(name = "UndoSetName", desc = "Undo directory name setting")]
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
    #[lang_chinese(name = "移除零字节媒体文件", desc = "移除零字节媒体文件")]
    #[lang_english(name = "RemoveEmptyMedia", desc = "Remove zero-byte media files")]
    RemoveEmptyMedia {
        /// Work directory path
        #[arg(value_name = "Work directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
}

#[derive(Subcommand, Localized)]
pub enum BmsCommands {
    /// Parse BMS file
    #[lang_chinese(name = "解析BMS", desc = "解析BMS文件")]
    #[lang_english(name = "ParseBms", desc = "Parse BMS file")]
    ParseBms {
        /// BMS file path
        #[arg(value_name = "BMS file")]
        file: PathBuf,
    },
    /// Parse BMSON file
    #[lang_chinese(name = "解析BMSON", desc = "解析BMSON文件")]
    #[lang_english(name = "ParseBmson", desc = "Parse BMSON file")]
    ParseBmson {
        /// BMSON file path
        #[arg(value_name = "BMSON file")]
        file: PathBuf,
    },
    /// Get BMS file list in directory
    #[lang_chinese(name = "获取BMS列表", desc = "获取目录中的BMS文件列表")]
    #[lang_english(name = "GetBmsList", desc = "Get BMS file list in directory")]
    GetBmsList {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
    /// Get BMS information in directory
    #[lang_chinese(name = "获取BMS信息", desc = "获取目录中的BMS信息")]
    #[lang_english(name = "GetBmsInfo", desc = "Get BMS information in directory")]
    GetBmsInfo {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
    /// Check if it's a work directory
    #[lang_chinese(name = "检测作品目录", desc = "检查是否为作品目录")]
    #[lang_english(name = "IsWorkDir", desc = "Check if it's a work directory")]
    IsWorkDir {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
    /// Check if it's a root directory
    #[lang_chinese(name = "检测根目录", desc = "检查是否为根目录")]
    #[lang_english(name = "IsRootDir", desc = "Check if it's a root directory")]
    IsRootDir {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
}

#[derive(Subcommand, Localized)]
pub enum FsCommands {
    /// Check if two files have the same content
    #[lang_chinese(name = "判断文件相同", desc = "检查两个文件是否内容相同")]
    #[lang_english(name = "IsFileSame", desc = "Check if two files have the same content")]
    IsFileSame {
        /// First file path
        #[arg(value_name = "First file")]
        file1: PathBuf,
        /// Second file path
        #[arg(value_name = "Second file")]
        file2: PathBuf,
    },
    /// Check if directory contains files
    #[lang_chinese(name = "检查目录有文件", desc = "检查目录是否包含文件")]
    #[lang_english(name = "IsDirHavingFile", desc = "Check if directory contains files")]
    IsDirHavingFile {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
    },
    /// Remove empty folders
    #[lang_chinese(name = "删除空文件夹", desc = "删除空文件夹")]
    #[lang_english(name = "RemoveEmptyFolders", desc = "Remove empty folders")]
    RemoveEmptyFolders {
        /// Parent directory path
        #[arg(value_name = "Parent directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Calculate BMS directory similarity
    #[lang_chinese(name = "计算目录相似度", desc = "计算两个目录的BMS相似度")]
    #[lang_english(name = "BmsDirSimilarity", desc = "Calculate BMS directory similarity")]
    BmsDirSimilarity {
        /// First directory path
        #[arg(value_name = "First directory")]
        dir1: PathBuf,
        /// Second directory path
        #[arg(value_name = "Second directory")]
        dir2: PathBuf,
    },
}

#[derive(Subcommand, Localized)]
pub enum RootEventCommands {
    /// Check numbered folders
    #[lang_chinese(name = "检查编号文件夹", desc = "检查编号文件夹")]
    #[lang_english(name = "CheckNumFolder", desc = "Check numbered folders")]
    CheckNumFolder {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Maximum count
        #[arg(value_name = "Maximum count")]
        max: usize,
    },
    /// Create numbered folders
    #[lang_chinese(name = "创建编号文件夹", desc = "创建编号文件夹")]
    #[lang_english(name = "CreateNumFolders", desc = "Create numbered folders")]
    CreateNumFolders {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Folder count
        #[arg(value_name = "Folder count")]
        count: usize,
    },
    /// Generate work information table
    #[lang_chinese(name = "生成作品信息表", desc = "生成作品信息表")]
    #[lang_english(
        name = "GenerateWorkInfoTable",
        desc = "Generate work information table"
    )]
    GenerateWorkInfoTable {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
    },
}

#[derive(Subcommand, Localized)]
pub enum RawpackCommands {
    /// Extract numerically named pack files to BMS folders
    #[lang_chinese(name = "编号解压到BMS目录", desc = "将按数字命名的包解压到BMS目录")]
    #[lang_english(
        name = "UnzipNumericToBmsFolder",
        desc = "Extract numerically named pack files to BMS folders"
    )]
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
        /// Replace preset: default, update_pack
        #[arg(
            long,
            value_enum,
            default_value = "update_pack",
            value_name = "Replace preset"
        )]
        replace: ReplacePreset,
        /// Confirm before processing
        #[arg(long, value_name = "Confirm")]
        confirm: bool,
    },
    /// Extract files with names to BMS folders
    #[lang_chinese(name = "按名称解压到BMS目录", desc = "将具名文件解压到BMS目录")]
    #[lang_english(
        name = "UnzipWithNameToBmsFolder",
        desc = "Extract files with names to BMS folders"
    )]
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
        /// Replace preset: default, update_pack
        #[arg(
            long,
            value_enum,
            default_value = "update_pack",
            value_name = "Replace preset"
        )]
        replace: ReplacePreset,
        /// Confirm before processing
        #[arg(long, value_name = "Confirm")]
        confirm: bool,
    },
    /// Set file number (interactive)
    #[lang_chinese(name = "设置文件编号", desc = "设置文件编号（交互式）")]
    #[lang_english(name = "SetFileNum", desc = "Set file number (interactive)")]
    SetFileNum {
        /// Directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
        /// Allowed file extensions
        #[arg(
            long,
            value_name = "Allowed extensions",
            default_value = "zip,7z,rar,mp4,bms,bme,bml,pms"
        )]
        allowed_exts: Vec<String>,
    },
}

#[derive(Subcommand, Localized)]
pub enum RootCommands {
    /// Set directory name based on BMS file
    #[lang_chinese(name = "按BMS设置目录名(根)", desc = "为根目录下作品设置名称")]
    #[lang_english(name = "SetName", desc = "Set directory name based on BMS file")]
    SetName {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Set type: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "replace_title_artist", value_name = "Set type")]
        set_type: BmsFolderSetNameType,
        /// Replace preset: default, update_pack
        #[arg(
            long,
            value_enum,
            default_value = "update_pack",
            value_name = "Replace preset"
        )]
        replace: ReplacePreset,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
        /// Skip directories that are already formatted
        #[arg(long, default_value = "true", value_name = "Skip already formatted")]
        skip_already_formatted: bool,
    },
    /// Undo directory name setting
    #[lang_chinese(name = "撤销设置目录名(根)", desc = "撤销目录名设置")]
    #[lang_english(name = "UndoSetName", desc = "Undo directory name setting")]
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
    #[lang_chinese(name = "复制编号作品名", desc = "复制编号的作品目录名称")]
    #[lang_english(
        name = "CopyNumberedNames",
        desc = "Copy numbered work directory names"
    )]
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
    #[lang_chinese(name = "按首字符拆分文件夹", desc = "按首字符拆分根目录下文件夹")]
    #[lang_english(name = "SplitByFirstChar", desc = "Split folders by first character")]
    SplitByFirstChar {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Undo split operation
    #[lang_chinese(name = "撤销拆分", desc = "撤销拆分操作")]
    #[lang_english(name = "UndoSplit", desc = "Undo split operation")]
    UndoSplit {
        /// Target directory path
        #[arg(value_name = "Target directory")]
        dir: PathBuf,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Merge split folders
    #[lang_chinese(name = "合并拆分文件夹", desc = "合并已拆分的文件夹")]
    #[lang_english(name = "MergeSplit", desc = "Merge split folders")]
    MergeSplit {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Replace preset: default, update_pack
        #[arg(
            long,
            value_enum,
            default_value = "update_pack",
            value_name = "Replace preset"
        )]
        replace: ReplacePreset,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Move works
    #[lang_chinese(name = "移动作品", desc = "移动作品目录")]
    #[lang_english(name = "MoveWorks", desc = "Move works")]
    MoveWorks {
        /// Source directory path
        #[arg(value_name = "Source directory")]
        from: PathBuf,
        /// Target directory path
        #[arg(value_name = "Target directory")]
        to: PathBuf,
        /// Replace preset: default, update_pack
        #[arg(
            long,
            value_enum,
            default_value = "update_pack",
            value_name = "Replace preset"
        )]
        replace: ReplacePreset,
        /// Dry run: only print actions
        #[arg(long, value_name = "Dry run")]
        dry_run: bool,
    },
    /// Move out one level directory
    #[lang_chinese(name = "上移一层目录", desc = "将作品目录上移一层")]
    #[lang_english(name = "MoveOutWorks", desc = "Move out one level directory")]
    MoveOutWorks {
        /// Target root directory path
        #[arg(value_name = "Target root directory")]
        dir: PathBuf,
        /// Replace preset: default, update_pack
        #[arg(
            long,
            value_enum,
            default_value = "update_pack",
            value_name = "Replace preset"
        )]
        replace: ReplacePreset,
        /// Dry run: only print actions
        #[arg(long)]
        dry_run: bool,
    },
    /// Move works with same name
    #[lang_chinese(name = "移动同名作品", desc = "移动同名的作品目录")]
    #[lang_english(name = "MoveSameName", desc = "Move works with same name")]
    MoveSameName {
        /// Source directory path
        #[arg(value_name = "Source directory")]
        from: PathBuf,
        /// Target directory path
        #[arg(value_name = "Target directory")]
        to: PathBuf,
        /// Replace preset: default, update_pack
        #[arg(
            long,
            value_enum,
            default_value = "update_pack",
            value_name = "Replace preset"
        )]
        replace: ReplacePreset,
        /// Dry run: only print actions
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove unnecessary media files
    #[lang_chinese(name = "移除不必要媒体", desc = "移除不必要的媒体文件")]
    #[lang_english(name = "RemoveUnneedMedia", desc = "Remove unnecessary media files")]
    RemoveUnneedMedia {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Rule preset
        #[arg(long, value_enum, default_value = "oraja", value_name = "Rule preset")]
        rule: RemoveMediaPreset,
    },
    /// Scan similar folders
    #[lang_chinese(name = "扫描相似文件夹", desc = "扫描相似的文件夹")]
    #[lang_english(name = "ScanSimilarFolders", desc = "Scan similar folders")]
    ScanSimilarFolders {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
        /// Similarity threshold
        #[arg(long, default_value = "0.7", value_name = "Similarity")]
        similarity: f64,
    },
}

#[derive(Subcommand, Localized)]
pub enum BmsEventCommands {
    /// Open BMS event list page
    #[lang_chinese(name = "打开活动列表页", desc = "打开BMS活动列表页面")]
    #[lang_english(name = "OpenList", desc = "Open BMS event list page")]
    OpenList {
        /// BMS event type
        #[arg(value_name = "Event type")]
        event: BMSEvent,
    },
    /// Open multiple BMS event work details pages
    #[lang_chinese(name = "打开作品详情页", desc = "打开多个作品详情页面")]
    #[lang_english(
        name = "OpenWorks",
        desc = "Open multiple BMS event work details pages"
    )]
    OpenWorks {
        /// BMS event type
        #[arg(value_name = "Event type")]
        event: BMSEvent,
        /// Work IDs (space or comma separated)
        #[arg(value_name = "Work IDs")]
        work_ids: Vec<u32>,
    },
}

#[derive(Subcommand, Localized)]
pub enum PackCommands {
    /// Raw pack -> HQ pack
    #[lang_chinese(name = "原始包转HQ", desc = "原始包 -> HQ 包")]
    #[lang_english(name = "RawToHq", desc = "Raw pack -> HQ pack")]
    RawToHq {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
    },
    /// HQ pack -> LQ pack
    #[lang_chinese(name = "HQ转LQ", desc = "HQ 包 -> LQ 包")]
    #[lang_english(name = "HqToLq", desc = "HQ pack -> LQ pack")]
    HqToLq {
        /// Root directory path
        #[arg(value_name = "Root directory")]
        dir: PathBuf,
    },
    /// Pack generation script: Raw pack -> HQ pack
    #[lang_chinese(name = "生成脚本：原始->HQ", desc = "打包生成脚本：原始包 -> HQ 包")]
    #[lang_english(
        name = "SetupRawpackToHq",
        desc = "Pack generation script: Raw pack -> HQ pack"
    )]
    SetupRawpackToHq {
        /// Pack directory path
        #[arg(value_name = "Pack directory")]
        pack_dir: PathBuf,
        /// Root directory path
        #[arg(value_name = "Root directory")]
        root_dir: PathBuf,
    },
    /// Pack update script: Raw pack -> HQ pack
    #[lang_chinese(name = "更新脚本：原始->HQ", desc = "打包更新脚本：原始包 -> HQ 包")]
    #[lang_english(
        name = "UpdateRawpackToHq",
        desc = "Pack update script: Raw pack -> HQ pack"
    )]
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
                replace,
                dry_run,
                skip_already_formatted,
            } => {
                info!("Setting directory name: {}", dir.display());
                info!("Set type: {:?}", set_type);
                set_name_by_bms(dir, *set_type, *dry_run, *replace, *skip_already_formatted)
                    .await?;
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
                replace,
                dry_run,
                skip_already_formatted,
            } => {
                info!("Setting directory name: {}", dir.display());
                info!("Set type: {:?}", set_type);
                root_set_name_by_bms(dir, *set_type, *dry_run, *replace, *skip_already_formatted)
                    .await?;
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
                undo_split_pack(dir, *dry_run, crate::fs::moving::ReplacePreset::UpdatePack)
                    .await?;
                info!("Undo completed");
            }
            RootCommands::MergeSplit {
                dir,
                dry_run,
                replace,
            } => {
                info!("Merging split folders: {}", dir.display());
                merge_split_folders(dir, *dry_run, *replace).await?;
                info!("Merge completed");
            }
            RootCommands::MoveWorks {
                from,
                to,
                dry_run,
                replace,
            } => {
                info!("Moving works: {} -> {}", from.display(), to.display());
                move_works_in_pack(from, to, *dry_run, *replace).await?;
                info!("Move completed");
            }
            RootCommands::MoveOutWorks {
                dir,
                dry_run,
                replace,
            } => {
                info!("Moving out one level directory: {}", dir.display());
                move_out_works(dir, *dry_run, *replace).await?;
                info!("Move out completed");
            }
            RootCommands::MoveSameName {
                from,
                to,
                dry_run,
                replace,
            } => {
                info!(
                    "Moving works with same name: {} -> {}",
                    from.display(),
                    to.display()
                );
                move_works_with_same_name(from, to, *dry_run, *replace).await?;
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
                replace,
                confirm,
            } => {
                info!(
                    "Extracting numerically named pack files: {} -> {} (cache: {})",
                    pack_dir.display(),
                    root_dir.display(),
                    cache_dir.display()
                );
                unzip_numeric_to_bms_folder(pack_dir, cache_dir, root_dir, *confirm, *replace)
                    .await?;
                info!("Extraction completed");
            }
            RawpackCommands::UnzipWithNameToBmsFolder {
                pack_dir,
                cache_dir,
                root_dir,
                replace,
                confirm,
            } => {
                info!(
                    "Extracting files with names: {} -> {} (cache: {})",
                    pack_dir.display(),
                    root_dir.display(),
                    cache_dir.display()
                );
                unzip_with_name_to_bms_folder(pack_dir, cache_dir, root_dir, *confirm, *replace)
                    .await?;
                info!("Extraction completed");
            }
            RawpackCommands::SetFileNum { dir, allowed_exts } => {
                info!("Setting file numbers: {}", dir.display());
                let allowed_exts_slice: &[&str] =
                    &allowed_exts.iter().map(|s| s.as_str()).collect::<Vec<_>>();
                set_file_num(dir, allowed_exts_slice).await?;
                info!("Setting completed");
            }
        },
        Commands::BmsEvent { command } => match command {
            BmsEventCommands::OpenList { event } => {
                crate::options::bms_event::open_event_list(*event).await?;
                info!("List opened");
            }
            BmsEventCommands::OpenWorks { event, work_ids } => {
                crate::options::bms_event::open_event_works(*event, work_ids).await?;
                info!("All work pages opened");
            }
        },
    }

    Ok(())
}
