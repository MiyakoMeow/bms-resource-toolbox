# BMS大包处理模块

这个模块提供了用于处理BMS大包的各种功能，从Python版本同步而来。

## 功能列表

### 1. 原包 -> HQ版大包
```rust
use be_music_cabinet::options::pack::pack_raw_to_hq;

// 将原包转换为HQ版大包（适用于beatoraja/Qwilight播放器）
pack_raw_to_hq("path/to/root/dir").await?;
```

### 2. HQ版大包 -> LQ版大包
```rust
use be_music_cabinet::options::pack::pack_hq_to_lq;

// 将HQ版大包转换为LQ版大包（适用于LR2播放器）
pack_hq_to_lq("path/to/root/dir").await?;
```

### 3. 大包生成脚本：原包 -> HQ版大包
```rust
use be_music_cabinet::options::pack::pack_setup_rawpack_to_hq;

// 从原包快速创建HQ版大包
pack_setup_rawpack_to_hq("path/to/pack/dir", "path/to/root/dir").await?;
```

### 4. 大包更新脚本：原包 -> HQ版大包
```rust
use be_music_cabinet::options::pack::pack_update_rawpack_to_hq;

// 从原包快速更新HQ版大包
pack_update_rawpack_to_hq("path/to/pack/dir", "path/to/root/dir", "path/to/sync/dir").await?;
```

## 处理流程

### 原包 -> HQ版大包流程
1. **音频处理**: WAV -> FLAC转换
2. **清理文件**: 移除不需要的媒体文件

### HQ版大包 -> LQ版大包流程
1. **音频处理**: FLAC -> OGG转换
2. **视频处理**: MP4 -> 多种格式转换（MPEG1、WMV2、AVI）

### 大包生成脚本流程
1. **解压**: 从包目录解压数字命名的文件到目标目录
2. **命名**: 根据BMS文件设置目录名
3. **音频处理**: WAV -> FLAC转换
4. **清理文件**: 移除不需要的媒体文件

### 大包更新脚本流程
1. **解压**: 从包目录解压数字命名的文件到目标目录
2. **同步命名**: 从现有目录同步目录名
3. **音频处理**: WAV -> FLAC转换
4. **清理文件**: 移除不需要的媒体文件
5. **软同步**: 同步目录文件
6. **清理空文件夹**: 移除空文件夹

## 音频预设

系统支持以下音频处理预设：

- **FLAC**: 使用flac工具进行无损压缩
- **FLAC_FFMPEG**: 使用ffmpeg进行FLAC转换
- **OGG_Q10**: 使用oggenc进行OGG转换（质量10）

## 视频预设

系统支持以下视频处理预设：

- **MPEG1VIDEO_512X512**: MPEG1格式，512x512分辨率
- **WMV2_512X512**: WMV2格式，512x512分辨率
- **AVI_512X512**: AVI格式，512x512分辨率

## 解压功能

系统支持以下解压功能：

- **数字命名文件识别**: 自动识别以数字开头的包文件
- **多格式支持**: ZIP、7Z、RAR格式的自动解压
- **智能目录整理**: 自动整理解压后的目录结构
- **目标目录创建**: 按数字ID自动创建目标目录

## 注意事项

- 所有操作都是异步的，需要使用`.await`
- 音频和视频处理需要相应的外部工具（ffmpeg、flac、oggenc等）
- 解压功能已完全实现，支持ZIP、7Z、RAR格式
- 文件同步使用智能策略，避免覆盖重要文件
- 支持递归清理空文件夹
- 包文件必须以数字开头才会被处理

## 依赖的外部工具

- **ffmpeg**: 用于音频和视频转换
- **flac**: 用于FLAC格式处理
- **oggenc**: 用于OGG格式处理
- **zip/7z/rar**: 用于解压功能

## 错误处理

所有函数都返回`io::Result<()>`，包含详细的错误信息。常见的错误包括：

- 文件不存在或权限不足
- 外部工具未安装或执行失败
- 磁盘空间不足
- 文件格式不支持
