# be-music-cabinet

BMS音乐文件管理工具，提供丰富的功能来组织和处理BMS文件。

## 功能特性

- **工作目录管理**: 设置目录名、移除零字节文件等
- **根目录操作**: 分割/合并文件夹、移动作品、清理媒体文件等
- **大包处理**: 原包/HQ版/LQ版之间的转换
- **命令行界面**: 提供完整的命令行操作界面
- **异步处理**: 高效的异步文件操作
- **多格式支持**: 支持ZIP、7Z、RAR等压缩格式

## 快速开始

### 安装

```bash
git clone <repository-url>
cd be-music-cabinet
cargo build --release
```

### 命令行使用

```bash
# 查看帮助
./target/release/be-music-cabinet --help

# 设置BMS文件夹名称
./target/release/be-music-cabinet work set-name ./MyBMSFolder

# 移除不需要的媒体文件
./target/release/be-music-cabinet root remove-unneed-media ./BMSRoot

# 原包转HQ版大包
./target/release/be-music-cabinet pack raw-to-hq ./BMSRoot
```

### 编程接口使用

```rust
use be_music_cabinet::options::{
    work::{set_name_by_bms, BmsFolderSetNameType},
    root_bigpack::{remove_unneed_media_files, get_remove_media_rule_oraja},
    pack::pack_raw_to_hq,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置目录名
    set_name_by_bms("./MyBMSFolder", BmsFolderSetNameType::AppendTitleArtist).await?;
    
    // 移除不需要的媒体文件
    remove_unneed_media_files("./BMSRoot", Some(get_remove_media_rule_oraja())).await?;
    
    // 原包转HQ版大包
    pack_raw_to_hq("./BMSRoot").await?;
    
    Ok(())
}
```

## 详细文档

- [命令行使用指南](README_CLI.md) - 完整的命令行操作说明
- [Pack模块文档](README_PACK.md) - 大包处理功能详细说明

## 模块结构

```
be-music-cabinet/
├── src/
│   ├── main.rs              # 命令行应用入口
│   ├── lib.rs               # 库入口
│   ├── options/             # 主要功能模块
│   │   ├── work.rs          # 工作目录操作
│   │   ├── root.rs          # 根目录操作
│   │   ├── root_bigpack.rs  # 大包根目录操作
│   │   └── pack.rs          # 大包处理
│   ├── fs/                  # 文件系统操作
│   ├── media/               # 媒体处理
│   └── bms/                 # BMS文件处理
├── examples/                # 使用示例
└── README_CLI.md           # 命令行使用指南
```

## 开发

### 运行测试

```bash
cargo test
```

### 运行示例

```bash
cargo run --example basic_usage
```

### 构建发布版本

```bash
cargo build --release
```

## 依赖

- **smol**: 异步运行时
- **clap**: 命令行参数解析
- **tokio**: 异步运行时（命令行应用）
- **regex**: 正则表达式支持
- **zip/sevenz-rust/unrar**: 压缩文件支持

## 外部工具依赖

某些功能需要外部工具：
- **ffmpeg**: 音频和视频转换
- **flac**: FLAC格式处理
- **oggenc**: OGG格式处理

## 许可证

Apache License 2.0
