//! CLI interface using clap derive.
//!
//! Provides subcommands for each option function in the toolbox.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// BMS Resource Toolbox - CLI interface
#[derive(Parser)]
#[command(name = "bms-resource-toolbox")]
#[command(version, about = "BMS Resource Toolbox")]
pub struct Cli {
    /// Launch interactive TUI (default when no subcommand)
    #[arg(short, long)]
    pub tui: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// BMS活动：跳转至作品信息页
    JumpToWorkInfo,

    /// BMS根目录：按照BMS设置文件夹名
    SetNameByBms {
        /// Root directory path
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS根目录：按照BMS追加文件夹名
    AppendNameByBms {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS根目录：按照BMS追加文件夹艺术家名
    AppendArtistNameByBms {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS根目录：克隆带编号的文件夹名
    CopyNumberedWorkdirNames {
        #[arg(short, long)]
        from: PathBuf,
        #[arg(short, long)]
        to: PathBuf,
    },

    /// BMS根目录：扫描相似文件夹名
    ScanFolderSimilarFolders {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS根目录：撤销重命名
    UndoSetName {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS根目录：移除大小为0的媒体文件和临时文件
    RemoveZeroSizedMediaFiles {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS大包目录：按照首字符分成多个文件夹
    SplitFoldersWithFirstChar {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS大包目录：（撤销）按照首字符分成多个文件夹
    UndoSplitPack {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS大包目录：将目录A下的作品移动到目录B
    MoveWorksInPack {
        #[arg(short, long)]
        from: PathBuf,
        #[arg(short, long)]
        to: PathBuf,
    },

    /// BMS大包父目录：移出一层目录
    MoveOutWorks {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS大包目录：合并文件名相似的子文件夹到目标
    MoveWorksWithSameName {
        #[arg(short, long)]
        from: PathBuf,
        #[arg(short, long)]
        to: PathBuf,
    },

    /// BMS大包目录：将文件名相似的子文件夹合并到各平级目录
    MoveWorksWithSameNameToSiblings {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS大包目录：合并被拆分的文件夹
    MergeSplitFolders {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS活动目录：检查编号对应文件夹是否存在
    CheckNumFolder {
        #[arg(short, long)]
        path: PathBuf,
        #[arg(short, long)]
        count: i32,
    },

    /// BMS活动目录：创建只带有编号的空文件夹
    CreateNumFolders {
        #[arg(short, long)]
        path: PathBuf,
        #[arg(short, long)]
        count: i32,
    },

    /// BMS活动目录：生成活动作品xlsx表格
    GenerateWorkInfoTable {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS根目录：音频文件转换
    TransferAudio {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS根目录：视频文件转换
    TransferVideo {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS原文件：解压编号文件至根目录
    UnzipNumericToBmsFolder {
        #[arg(short, long)]
        pack: PathBuf,
        #[arg(short, long)]
        cache: PathBuf,
        #[arg(short, long)]
        root: PathBuf,
    },

    /// BMS原文件：解压文件至根目录（按原名）
    UnzipWithNameToBmsFolder {
        #[arg(short, long)]
        pack: PathBuf,
        #[arg(short, long)]
        cache: PathBuf,
        #[arg(short, long)]
        root: PathBuf,
    },

    /// BMS原文件：赋予编号
    SetFileNum {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// 大包生成脚本：原包 -> HQ版大包
    PackSetupRawpackToHq {
        #[arg(short, long)]
        pack: PathBuf,
        #[arg(short, long)]
        root: PathBuf,
    },

    /// 大包更新脚本：原包 -> HQ版大包
    PackUpdateRawpackToHq {
        #[arg(short, long)]
        pack: PathBuf,
        #[arg(short, long)]
        root: PathBuf,
        #[arg(short, long)]
        sync: PathBuf,
    },

    /// BMS大包脚本：原包 -> HQ版大包
    PackRawToHq {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// BMS大包脚本：HQ版大包 -> LQ版大包
    PackHqToLq {
        #[arg(short, long)]
        path: PathBuf,
    },
}

/// Dispatch a CLI command to the appropriate function
pub fn dispatch(cmd: &Commands) {
    use tokio::runtime::Handle;

    match cmd {
        Commands::JumpToWorkInfo => {
            crate::options::bms_events::jump_to_work_info(&[]);
        }
        Commands::SetNameByBms { path } => {
            if let Err(e) =
                Handle::current().block_on(crate::options::bms_folder::set_name_by_bms(path))
            {
                eprintln!("{e}");
            }
        }
        Commands::AppendNameByBms { path } => {
            if let Err(e) =
                Handle::current().block_on(crate::options::bms_folder::append_name_by_bms(path))
            {
                eprintln!("{e}");
            }
        }
        Commands::AppendArtistNameByBms { path } => {
            if let Err(e) = Handle::current()
                .block_on(crate::options::bms_folder::append_artist_name_by_bms(path))
            {
                eprintln!("{e}");
            }
        }
        Commands::CopyNumberedWorkdirNames { from, to } => {
            if let Err(e) = crate::options::bms_folder::copy_numbered_workdir_names(from, to) {
                eprintln!("{e}");
            }
        }
        Commands::ScanFolderSimilarFolders { path } => {
            if let Err(e) = crate::options::bms_folder::scan_folder_similar_folders(path, 0.7) {
                eprintln!("{e}");
            }
        }
        Commands::UndoSetName { path } => {
            if let Err(e) = crate::options::bms_folder::undo_set_name(path) {
                eprintln!("{e}");
            }
        }
        Commands::RemoveZeroSizedMediaFiles { path } => {
            if let Err(e) = crate::options::bms_folder::remove_zero_sized_media_files(path, false) {
                eprintln!("{e}");
            }
        }
        Commands::SplitFoldersWithFirstChar { path } => {
            if let Err(e) = crate::options::bms_folder_bigpack::split_folders_with_first_char(path)
            {
                eprintln!("{e}");
            }
        }
        Commands::UndoSplitPack { path } => {
            if let Err(e) = crate::options::bms_folder_bigpack::undo_split_pack(path) {
                eprintln!("{e}");
            }
        }
        Commands::MoveWorksInPack { from, to } => {
            if let Err(e) = crate::options::bms_folder_bigpack::move_works_in_pack(from, to) {
                eprintln!("{e}");
            }
        }
        Commands::MoveOutWorks { path } => {
            if let Err(e) = crate::options::bms_folder_bigpack::move_out_works(path) {
                eprintln!("{e}");
            }
        }
        Commands::MoveWorksWithSameName { from, to } => {
            if let Err(e) = crate::options::bms_folder_bigpack::move_works_with_same_name(from, to)
            {
                eprintln!("{e}");
            }
        }
        Commands::MoveWorksWithSameNameToSiblings { path } => {
            if let Err(e) =
                crate::options::bms_folder_bigpack::move_works_with_same_name_to_siblings(path)
            {
                eprintln!("{e}");
            }
        }
        Commands::MergeSplitFolders { path } => {
            if let Err(e) = crate::options::bms_folder_bigpack::merge_split_folders(path) {
                eprintln!("{e}");
            }
        }
        Commands::CheckNumFolder { path, count } => {
            crate::options::bms_folder_event::check_num_folder(path, *count);
        }
        Commands::CreateNumFolders { path, count } => {
            if let Err(e) = crate::options::bms_folder_event::create_num_folders(path, *count) {
                eprintln!("{e}");
            }
        }
        Commands::GenerateWorkInfoTable { path } => {
            if let Err(e) = Handle::current().block_on(
                crate::options::bms_folder_event::generate_work_info_table(path),
            ) {
                eprintln!("{e}");
            }
        }
        Commands::TransferAudio { path } => {
            if let Err(e) =
                Handle::current().block_on(crate::options::bms_folder_media::transfer_audio(path))
            {
                eprintln!("{e}");
            }
        }
        Commands::TransferVideo { path } => {
            if let Err(e) =
                Handle::current().block_on(crate::options::bms_folder_media::transfer_video(path))
            {
                eprintln!("{e}");
            }
        }
        Commands::UnzipNumericToBmsFolder { pack, cache, root } => {
            if let Err(e) =
                crate::options::rawpack::unzip_numeric_to_bms_folder(pack, cache, root, false)
            {
                eprintln!("{e}");
            }
        }
        Commands::UnzipWithNameToBmsFolder { pack, cache, root } => {
            if let Err(e) =
                crate::options::rawpack::unzip_with_name_to_bms_folder(pack, cache, root, false)
            {
                eprintln!("{e}");
            }
        }
        Commands::SetFileNum { path } => {
            if let Err(e) = crate::options::rawpack::set_file_num(path) {
                eprintln!("{e}");
            }
        }
        Commands::PackSetupRawpackToHq { pack, root } => {
            if let Err(e) = Handle::current()
                .block_on(crate::scripts::pack::pack_setup_rawpack_to_hq(pack, root))
            {
                eprintln!("{e}");
            }
        }
        Commands::PackUpdateRawpackToHq { pack, root, sync } => {
            if let Err(e) = Handle::current().block_on(
                crate::scripts::pack::pack_update_rawpack_to_hq(pack, root, sync),
            ) {
                eprintln!("{e}");
            }
        }
        Commands::PackRawToHq { path } => {
            if let Err(e) = Handle::current().block_on(crate::scripts::pack::pack_raw_to_hq(path)) {
                eprintln!("{e}");
            }
        }
        Commands::PackHqToLq { path } => {
            if let Err(e) = Handle::current().block_on(crate::scripts::pack::pack_hq_to_lq(path)) {
                eprintln!("{e}");
            }
        }
    }
}
