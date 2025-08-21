//! be-music-cabinet 基本使用示例
//!
//! 这个示例展示了如何使用 be-music-cabinet 的各种功能

use be_music_cabinet::options::{
    pack::{pack_hq_to_lq, pack_raw_to_hq},
    root_bigpack::{get_remove_media_rule_oraja, remove_unneed_media_files},
    work::{BmsFolderSetNameType, set_name_by_bms},
};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("be-music-cabinet 基本使用示例");
    println!("================================");

    // 示例1: 设置BMS文件夹名称
    println!("\n1. 设置BMS文件夹名称");
    let bms_dir = Path::new("./example_bms_folder");
    if bms_dir.exists() {
        println!("设置目录名: {}", bms_dir.display());
        set_name_by_bms(bms_dir, BmsFolderSetNameType::AppendTitleArtist).await?;
        println!("设置完成");
    } else {
        println!("示例目录不存在: {}", bms_dir.display());
    }

    // 示例2: 移除不需要的媒体文件
    println!("\n2. 移除不需要的媒体文件");
    let root_dir = Path::new("./example_root");
    if root_dir.exists() {
        println!("移除不需要的媒体文件: {}", root_dir.display());
        remove_unneed_media_files(root_dir, Some(get_remove_media_rule_oraja())).await?;
        println!("移除完成");
    } else {
        println!("示例目录不存在: {}", root_dir.display());
    }

    // 示例3: 原包转HQ版大包
    println!("\n3. 原包转HQ版大包");
    let raw_dir = Path::new("./example_raw");
    if raw_dir.exists() {
        println!("转换原包到HQ版: {}", raw_dir.display());
        pack_raw_to_hq(raw_dir).await?;
        println!("转换完成");
    } else {
        println!("示例目录不存在: {}", raw_dir.display());
    }

    // 示例4: HQ版转LQ版大包
    println!("\n4. HQ版转LQ版大包");
    let hq_dir = Path::new("./example_hq");
    if hq_dir.exists() {
        println!("转换HQ版到LQ版: {}", hq_dir.display());
        pack_hq_to_lq(hq_dir).await?;
        println!("转换完成");
    } else {
        println!("示例目录不存在: {}", hq_dir.display());
    }

    println!("\n示例执行完成！");
    println!("\n要使用命令行版本，请运行:");
    println!("  be-music-cabinet --help");
    println!("  be-music-cabinet work --help");
    println!("  be-music-cabinet root --help");
    println!("  be-music-cabinet pack --help");

    Ok(())
}
