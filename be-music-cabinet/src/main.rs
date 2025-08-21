use clap::{Parser, Subcommand};
use std::path::PathBuf;

use be_music_cabinet::{
    bms::{get_dir_bms_info, get_dir_bms_list, is_root_dir, is_work_dir, parse_bms_file, parse_bmson_file},
    fs::{is_dir_having_file, is_file_same_content, remove_empty_folders, bms_dir_similarity},
    options::{
        pack::{pack_hq_to_lq, pack_raw_to_hq, pack_setup_rawpack_to_hq, pack_update_rawpack_to_hq},
        root::{copy_numbered_workdir_names, scan_folder_similar_folders},
        root_bigpack::{
            get_remove_media_rule_oraja, merge_split_folders, move_out_works, move_works_in_pack,
            move_works_with_same_name, remove_unneed_media_files, split_folders_with_first_char,
            undo_split_pack,
        },
        root_event::{check_num_folder, create_num_folders, generate_work_info_table},
        work::{BmsFolderSetNameType, remove_zero_sized_media_files, set_name_by_bms, undo_set_name},
    },
};

#[derive(Parser)]
#[command(name = "be-music-cabinet")]
#[command(about = "BMS音乐文件管理工具")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 工作目录相关操作
    Work {
        #[command(subcommand)]
        command: WorkCommands,
    },
    /// 根目录相关操作
    Root {
        #[command(subcommand)]
        command: RootCommands,
    },
    /// 大包处理相关操作
    Pack {
        #[command(subcommand)]
        command: PackCommands,
    },
    /// BMS文件相关操作
    Bms {
        #[command(subcommand)]
        command: BmsCommands,
    },
    /// 文件系统相关操作
    Fs {
        #[command(subcommand)]
        command: FsCommands,
    },
    /// 根目录事件相关操作
    RootEvent {
        #[command(subcommand)]
        command: RootEventCommands,
    },
}

#[derive(Subcommand)]
enum WorkCommands {
    /// 根据BMS文件设置目录名
    SetName {
        /// 工作目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// 设置类型: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "append_title_artist")]
        set_type: String,
    },
    /// 撤销设置目录名
    UndoSetName {
        /// 工作目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// 设置类型: replace_title_artist, append_title_artist, append_artist
        #[arg(long, default_value = "append_title_artist")]
        set_type: String,
    },
    /// 移除零字节媒体文件
    RemoveEmptyMedia {
        /// 工作目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum BmsCommands {
    /// 解析BMS文件
    ParseBms {
        /// BMS文件路径
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// 解析BMSON文件
    ParseBmson {
        /// BMSON文件路径
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// 获取目录中的BMS文件列表
    GetBmsList {
        /// 目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 获取目录中的BMS信息
    GetBmsInfo {
        /// 目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 检查是否为工作目录
    IsWorkDir {
        /// 目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 检查是否为根目录
    IsRootDir {
        /// 目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum FsCommands {
    /// 检查两个文件内容是否相同
    IsFileSame {
        /// 第一个文件路径
        #[arg(value_name = "FILE1")]
        file1: PathBuf,
        /// 第二个文件路径
        #[arg(value_name = "FILE2")]
        file2: PathBuf,
    },
    /// 检查目录是否包含文件
    IsDirHavingFile {
        /// 目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 移除空文件夹
    RemoveEmptyFolders {
        /// 父目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 计算BMS目录相似度
    BmsDirSimilarity {
        /// 第一个目录路径
        #[arg(value_name = "DIR1")]
        dir1: PathBuf,
        /// 第二个目录路径
        #[arg(value_name = "DIR2")]
        dir2: PathBuf,
    },
}

#[derive(Subcommand)]
enum RootEventCommands {
    /// 检查编号文件夹
    CheckNumFolder {
        /// 根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// 最大数量
        #[arg(value_name = "MAX")]
        max: usize,
    },
    /// 创建编号文件夹
    CreateNumFolders {
        /// 根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// 文件夹数量
        #[arg(value_name = "COUNT")]
        count: usize,
    },
    /// 生成工作信息表
    GenerateWorkInfoTable {
        /// 根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum RootCommands {
    /// 复制编号工作目录名
    CopyNumberedNames {
        /// 源目录路径
        #[arg(value_name = "FROM")]
        from: PathBuf,
        /// 目标目录路径
        #[arg(value_name = "TO")]
        to: PathBuf,
    },
    /// 按首字符分割文件夹
    SplitByFirstChar {
        /// 根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 撤销分割操作
    UndoSplit {
        /// 目标目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 合并分割的文件夹
    MergeSplit {
        /// 根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 移动作品
    MoveWorks {
        /// 源目录路径
        #[arg(value_name = "FROM")]
        from: PathBuf,
        /// 目标目录路径
        #[arg(value_name = "TO")]
        to: PathBuf,
    },
    /// 移出一层目录
    MoveOutWorks {
        /// 目标根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 移动同名作品
    MoveSameName {
        /// 源目录路径
        #[arg(value_name = "FROM")]
        from: PathBuf,
        /// 目标目录路径
        #[arg(value_name = "TO")]
        to: PathBuf,
    },
    /// 移除不需要的媒体文件
    RemoveUnneedMedia {
        /// 根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// 规则类型: oraja, wav_fill_flac, mpg_fill_wmv
        #[arg(long, default_value = "oraja")]
        rule: String,
    },
    /// 扫描相似文件夹
    ScanSimilarFolders {
        /// 根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        /// 相似度阈值
        #[arg(long, default_value = "0.7")]
        similarity: f64,
    },
}

#[derive(Subcommand)]
enum PackCommands {
    /// 原包 -> HQ版大包
    RawToHq {
        /// 根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// HQ版大包 -> LQ版大包
    HqToLq {
        /// 根目录路径
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
    /// 大包生成脚本：原包 -> HQ版大包
    SetupRawpackToHq {
        /// 包目录路径
        #[arg(value_name = "PACK_DIR")]
        pack_dir: PathBuf,
        /// 根目录路径
        #[arg(value_name = "ROOT_DIR")]
        root_dir: PathBuf,
    },
    /// 大包更新脚本：原包 -> HQ版大包
    UpdateRawpackToHq {
        /// 包目录路径
        #[arg(value_name = "PACK_DIR")]
        pack_dir: PathBuf,
        /// 根目录路径
        #[arg(value_name = "ROOT_DIR")]
        root_dir: PathBuf,
        /// 同步目录路径
        #[arg(value_name = "SYNC_DIR")]
        sync_dir: PathBuf,
    },
}

fn get_set_name_type(set_type: &str) -> BmsFolderSetNameType {
    match set_type {
        "replace_title_artist" => BmsFolderSetNameType::ReplaceTitleArtist,
        "append_title_artist" => BmsFolderSetNameType::AppendTitleArtist,
        "append_artist" => BmsFolderSetNameType::AppendArtist,
        _ => BmsFolderSetNameType::AppendTitleArtist,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Work { command } => {
            match command {
                WorkCommands::SetName { dir, set_type } => {
                    println!("设置目录名: {}", dir.display());
                    set_name_by_bms(dir, get_set_name_type(set_type)).await?;
                    println!("设置完成");
                }
                WorkCommands::UndoSetName { dir, set_type } => {
                    println!("撤销设置目录名: {}", dir.display());
                    undo_set_name(&dir, get_set_name_type(&set_type)).await?;
                    println!("撤销完成");
                }
                WorkCommands::RemoveEmptyMedia { dir } => {
                    println!("移除零字节媒体文件: {}", dir.display());
                    remove_zero_sized_media_files(dir).await?;
                    println!("移除完成");
                }
            }
        }
        Commands::Root { command } => match command {
            RootCommands::CopyNumberedNames { from, to } => {
                println!("复制编号工作目录名: {} -> {}", from.display(), to.display());
                copy_numbered_workdir_names(from, to).await?;
                println!("复制完成");
            }
            RootCommands::SplitByFirstChar { dir } => {
                println!("按首字符分割文件夹: {}", dir.display());
                split_folders_with_first_char(dir).await?;
                println!("分割完成");
            }
            RootCommands::UndoSplit { dir } => {
                println!("撤销分割操作: {}", dir.display());
                undo_split_pack(dir).await?;
                println!("撤销完成");
            }
            RootCommands::MergeSplit { dir } => {
                println!("合并分割的文件夹: {}", dir.display());
                merge_split_folders(dir).await?;
                println!("合并完成");
            }
            RootCommands::MoveWorks { from, to } => {
                println!("移动作品: {} -> {}", from.display(), to.display());
                move_works_in_pack(from, to).await?;
                println!("移动完成");
            }
            RootCommands::MoveOutWorks { dir } => {
                println!("移出一层目录: {}", dir.display());
                move_out_works(dir).await?;
                println!("移出完成");
            }
            RootCommands::MoveSameName { from, to } => {
                println!("移动同名作品: {} -> {}", from.display(), to.display());
                move_works_with_same_name(from, to).await?;
                println!("移动完成");
            }
            RootCommands::RemoveUnneedMedia { dir, rule } => {
                println!("移除不需要的媒体文件: {} (规则: {})", dir.display(), rule);
                let rule_config = match rule.as_str() {
                        "oraja" => Some(get_remove_media_rule_oraja()),
                        "wav_fill_flac" => Some(be_music_cabinet::options::root_bigpack::get_remove_media_rule_wav_fill_flac()),
                        "mpg_fill_wmv" => Some(be_music_cabinet::options::root_bigpack::get_remove_media_rule_mpg_fill_wmv()),
                        _ => None,
                    };
                remove_unneed_media_files(dir, rule_config).await?;
                println!("移除完成");
            }
            RootCommands::ScanSimilarFolders { dir, similarity } => {
                println!("扫描相似文件夹: {} (相似度阈值: {})", dir.display(), similarity);
                let results = scan_folder_similar_folders(dir, *similarity).await?;
                for (former, current, sim) in results {
                    println!("相似度 {:.3}: {} <-> {}", sim, former, current);
                }
                println!("扫描完成");
            }
        },
        Commands::Pack { command } => match command {
            PackCommands::RawToHq { dir } => {
                println!("原包 -> HQ版大包: {}", dir.display());
                pack_raw_to_hq(dir).await?;
                println!("转换完成");
            }
            PackCommands::HqToLq { dir } => {
                println!("HQ版大包 -> LQ版大包: {}", dir.display());
                pack_hq_to_lq(dir).await?;
                println!("转换完成");
            }
            PackCommands::SetupRawpackToHq { pack_dir, root_dir } => {
                println!(
                    "大包生成脚本: {} -> {}",
                    pack_dir.display(),
                    root_dir.display()
                );
                pack_setup_rawpack_to_hq(pack_dir, root_dir).await?;
                println!("生成完成");
            }
            PackCommands::UpdateRawpackToHq {
                pack_dir,
                root_dir,
                sync_dir,
            } => {
                println!(
                    "大包更新脚本: {} -> {} (同步: {})",
                    pack_dir.display(),
                    root_dir.display(),
                    sync_dir.display()
                );
                pack_update_rawpack_to_hq(pack_dir, root_dir, sync_dir).await?;
                println!("更新完成");
            }
        },
        Commands::Bms { command } => match command {
            BmsCommands::ParseBms { file } => {
                println!("解析BMS文件: {}", file.display());
                let result = parse_bms_file(&file).await?;
                println!("解析结果: {:?}", result);
            }
            BmsCommands::ParseBmson { file } => {
                println!("解析BMSON文件: {}", file.display());
                let result = parse_bmson_file(&file).await?;
                println!("解析结果: {:?}", result);
            }
            BmsCommands::GetBmsList { dir } => {
                println!("获取BMS文件列表: {}", dir.display());
                let results = get_dir_bms_list(&dir).await?;
                println!("找到 {} 个BMS文件", results.len());
                for (i, bms) in results.iter().enumerate() {
                    println!("  {}. {:?}", i + 1, bms);
                }
            }
            BmsCommands::GetBmsInfo { dir } => {
                println!("获取BMS信息: {}", dir.display());
                let result = get_dir_bms_info(&dir).await?;
                match result {
                    Some(info) => println!("BMS信息: {:?}", info),
                    None => println!("未找到BMS信息"),
                }
            }
            BmsCommands::IsWorkDir { dir } => {
                println!("检查是否为工作目录: {}", dir.display());
                let result = is_work_dir(&dir).await?;
                println!("是否为工作目录: {}", result);
            }
            BmsCommands::IsRootDir { dir } => {
                println!("检查是否为根目录: {}", dir.display());
                let result = is_root_dir(&dir).await?;
                println!("是否为根目录: {}", result);
            }
        },
        Commands::Fs { command } => match command {
            FsCommands::IsFileSame { file1, file2 } => {
                println!("检查文件内容是否相同: {} <-> {}", file1.display(), file2.display());
                let result = is_file_same_content(&file1, &file2).await?;
                println!("文件内容是否相同: {}", result);
            }
            FsCommands::IsDirHavingFile { dir } => {
                println!("检查目录是否包含文件: {}", dir.display());
                let result = is_dir_having_file(&dir).await?;
                println!("目录是否包含文件: {}", result);
            }
            FsCommands::RemoveEmptyFolders { dir } => {
                println!("移除空文件夹: {}", dir.display());
                remove_empty_folders(dir).await?;
                println!("移除完成");
            }
            FsCommands::BmsDirSimilarity { dir1, dir2 } => {
                println!("计算BMS目录相似度: {} <-> {}", dir1.display(), dir2.display());
                let result = bms_dir_similarity(&dir1, &dir2).await?;
                println!("相似度: {:.3}", result);
            }
        },
        Commands::RootEvent { command } => match command {
            RootEventCommands::CheckNumFolder { dir, max } => {
                println!("检查编号文件夹: {} (最大数量: {})", dir.display(), max);
                let results = check_num_folder(dir, *max).await?;
                println!("找到 {} 个编号文件夹", results.len());
                for path in results {
                    println!("  {}", path.display());
                }
            }
            RootEventCommands::CreateNumFolders { dir, count } => {
                println!("创建编号文件夹: {} (数量: {})", dir.display(), count);
                create_num_folders(dir, *count).await?;
                println!("创建完成");
            }
            RootEventCommands::GenerateWorkInfoTable { dir } => {
                println!("生成工作信息表: {}", dir.display());
                generate_work_info_table(dir).await?;
                println!("生成完成");
            }
        },
    }

    Ok(())
}
