# be-music-cabinet 命令行应用

be-music-cabinet 是一个用于管理BMS音乐文件的命令行工具，提供了丰富的功能来组织和处理BMS文件。

## 安装

```bash
cargo build --release
```

编译后的可执行文件位于 `target/release/be-music-cabinet.exe`

## 基本用法

```bash
be-music-cabinet <COMMAND>
```

## 命令分类

### 1. 工作目录操作 (work)

处理单个BMS工作目录的操作。

#### 设置目录名
```bash
# 根据BMS文件设置目录名
be-music-cabinet work set-name <DIR> [--set-type <TYPE>]

# 示例
be-music-cabinet work set-name ./MyBMSFolder
be-music-cabinet work set-name ./MyBMSFolder --set-type append_title_artist
```

设置类型选项：
- `replace_title_artist`: 替换为标题+艺术家
- `append_title_artist`: 追加标题+艺术家（默认）
- `append_artist`: 仅追加艺术家

#### 撤销设置目录名
```bash
# 撤销设置目录名
be-music-cabinet work undo-set-name <DIR> [--set-type <TYPE>]

# 示例
be-music-cabinet work undo-set-name ./MyBMSFolder
```

#### 移除零字节媒体文件
```bash
# 移除零字节媒体文件
be-music-cabinet work remove-empty-media <DIR>

# 示例
be-music-cabinet work remove-empty-media ./MyBMSFolder
```

### 2. 根目录操作 (root)

处理BMS根目录的批量操作。

#### 复制编号工作目录名
```bash
# 复制编号工作目录名
be-music-cabinet root copy-numbered-names <FROM> <TO>

# 示例
be-music-cabinet root copy-numbered-names ./source ./target
```

#### 按首字符分割文件夹
```bash
# 按首字符分割文件夹
be-music-cabinet root split-by-first-char <DIR>

# 示例
be-music-cabinet root split-by-first-char ./BMSRoot
```

#### 撤销分割操作
```bash
# 撤销分割操作
be-music-cabinet root undo-split <DIR>

# 示例
be-music-cabinet root undo-split ./BMSRoot
```

#### 合并分割的文件夹
```bash
# 合并分割的文件夹
be-music-cabinet root merge-split <DIR>

# 示例
be-music-cabinet root merge-split ./BMSRoot
```

#### 移动作品
```bash
# 移动作品
be-music-cabinet root move-works <FROM> <TO>

# 示例
be-music-cabinet root move-works ./source ./target
```

#### 移出一层目录
```bash
# 移出一层目录
be-music-cabinet root move-out-works <DIR>

# 示例
be-music-cabinet root move-out-works ./BMSRoot
```

#### 移动同名作品
```bash
# 移动同名作品
be-music-cabinet root move-same-name <FROM> <TO>

# 示例
be-music-cabinet root move-same-name ./source ./target
```

#### 移除不需要的媒体文件
```bash
# 移除不需要的媒体文件
be-music-cabinet root remove-unneed-media <DIR> [--rule <RULE>]

# 示例
be-music-cabinet root remove-unneed-media ./BMSRoot
be-music-cabinet root remove-unneed-media ./BMSRoot --rule oraja
```

规则类型选项：
- `oraja`: beatoraja规则（默认）
- `wav_fill_flac`: WAV填充FLAC规则
- `mpg_fill_wmv`: MPG填充WMV规则

### 3. 大包处理操作 (pack)

处理BMS大包的转换和生成。

#### 原包 -> HQ版大包
```bash
# 原包转换为HQ版大包（适用于beatoraja/Qwilight播放器）
be-music-cabinet pack raw-to-hq <DIR>

# 示例
be-music-cabinet pack raw-to-hq ./BMSRoot
```

#### HQ版大包 -> LQ版大包
```bash
# HQ版大包转换为LQ版大包（适用于LR2播放器）
be-music-cabinet pack hq-to-lq <DIR>

# 示例
be-music-cabinet pack hq-to-lq ./BMSRoot
```

#### 大包生成脚本
```bash
# 从原包快速创建HQ版大包
be-music-cabinet pack setup-rawpack-to-hq <PACK_DIR> <ROOT_DIR>

# 示例
be-music-cabinet pack setup-rawpack-to-hq ./packs ./BMSRoot
```

#### 大包更新脚本
```bash
# 从原包快速更新HQ版大包
be-music-cabinet pack update-rawpack-to-hq <PACK_DIR> <ROOT_DIR> <SYNC_DIR>

# 示例
be-music-cabinet pack update-rawpack-to-hq ./packs ./BMSRoot ./SyncDir
```

### 4. BMS文件相关操作 (bms)

处理BMS文件的解析和检查。

#### 解析BMS文件
```bash
# 解析BMS文件
be-music-cabinet bms parse-bms <FILE>

# 示例
be-music-cabinet bms parse-bms ./song.bms
```

#### 解析BMSON文件
```bash
# 解析BMSON文件
be-music-cabinet bms parse-bmson <FILE>

# 示例
be-music-cabinet bms parse-bmson ./song.bmson
```

#### 获取BMS文件列表
```bash
# 获取目录中的BMS文件列表
be-music-cabinet bms get-bms-list <DIR>

# 示例
be-music-cabinet bms get-bms-list ./BMSFolder
```

#### 获取BMS信息
```bash
# 获取目录中的BMS信息
be-music-cabinet bms get-bms-info <DIR>

# 示例
be-music-cabinet bms get-bms-info ./BMSFolder
```

#### 检查目录类型
```bash
# 检查是否为工作目录
be-music-cabinet bms is-work-dir <DIR>

# 检查是否为根目录
be-music-cabinet bms is-root-dir <DIR>

# 示例
be-music-cabinet bms is-work-dir ./MyBMSFolder
be-music-cabinet bms is-root-dir ./BMSRoot
```

### 5. 文件系统相关操作 (fs)

处理文件系统的各种操作。

#### 文件比较
```bash
# 检查两个文件内容是否相同
be-music-cabinet fs is-file-same <FILE1> <FILE2>

# 示例
be-music-cabinet fs is-file-same ./file1.txt ./file2.txt
```

#### 目录检查
```bash
# 检查目录是否包含文件
be-music-cabinet fs is-dir-having-file <DIR>

# 示例
be-music-cabinet fs is-dir-having-file ./MyFolder
```

#### 清理操作
```bash
# 移除空文件夹
be-music-cabinet fs remove-empty-folders <DIR>

# 示例
be-music-cabinet fs remove-empty-folders ./BMSRoot
```

#### 相似度计算
```bash
# 计算BMS目录相似度
be-music-cabinet fs bms-dir-similarity <DIR1> <DIR2>

# 示例
be-music-cabinet fs bms-dir-similarity ./folder1 ./folder2
```

### 6. 根目录事件相关操作 (root-event)

处理根目录的事件和批量操作。

#### 编号文件夹管理
```bash
# 检查编号文件夹
be-music-cabinet root-event check-num-folder <DIR> <MAX>

# 创建编号文件夹
be-music-cabinet root-event create-num-folders <DIR> <COUNT>

# 示例
be-music-cabinet root-event check-num-folder ./BMSRoot 1000
be-music-cabinet root-event create-num-folders ./BMSRoot 100
```

#### 信息表生成
```bash
# 生成工作信息表
be-music-cabinet root-event generate-work-info-table <DIR>

# 示例
be-music-cabinet root-event generate-work-info-table ./BMSRoot
```

#### 原包 -> HQ版大包
```bash
# 原包转换为HQ版大包（适用于beatoraja/Qwilight播放器）
be-music-cabinet pack raw-to-hq <DIR>

# 示例
be-music-cabinet pack raw-to-hq ./BMSRoot
```

#### HQ版大包 -> LQ版大包
```bash
# HQ版大包转换为LQ版大包（适用于LR2播放器）
be-music-cabinet pack hq-to-lq <DIR>

# 示例
be-music-cabinet pack hq-to-lq ./BMSRoot
```

#### 大包生成脚本
```bash
# 从原包快速创建HQ版大包
be-music-cabinet pack setup-rawpack-to-hq <PACK_DIR> <ROOT_DIR>

# 示例
be-music-cabinet pack setup-rawpack-to-hq ./packs ./BMSRoot
```

#### 大包更新脚本
```bash
# 从原包快速更新HQ版大包
be-music-cabinet pack update-rawpack-to-hq <PACK_DIR> <ROOT_DIR> <SYNC_DIR>

# 示例
be-music-cabinet pack update-rawpack-to-hq ./packs ./BMSRoot ./SyncDir
```

## 常用工作流程

### 1. 整理单个BMS文件夹
```bash
# 设置目录名
be-music-cabinet work set-name ./MyBMSFolder

# 移除零字节文件
be-music-cabinet work remove-empty-media ./MyBMSFolder
```

### 2. 整理整个BMS根目录
```bash
# 按首字符分割文件夹
be-music-cabinet root split-by-first-char ./BMSRoot

# 移除不需要的媒体文件
be-music-cabinet root remove-unneed-media ./BMSRoot

# 合并分割的文件夹
be-music-cabinet root merge-split ./BMSRoot
```

### 3. 生成HQ版大包
```bash
# 从原包生成HQ版大包
be-music-cabinet pack setup-rawpack-to-hq ./packs ./BMSRoot

# 或者直接转换现有目录
be-music-cabinet pack raw-to-hq ./BMSRoot
```

### 4. 生成LQ版大包
```bash
# 从HQ版生成LQ版大包
be-music-cabinet pack hq-to-lq ./BMSRoot
```

### 5. BMS文件分析
```bash
# 检查目录类型
be-music-cabinet bms is-work-dir ./MyBMSFolder
be-music-cabinet bms is-root-dir ./BMSRoot

# 获取BMS信息
be-music-cabinet bms get-bms-info ./MyBMSFolder

# 解析BMS文件
be-music-cabinet bms parse-bms ./song.bms
```

### 6. 文件系统维护
```bash
# 检查文件是否相同
be-music-cabinet fs is-file-same ./file1.txt ./file2.txt

# 移除空文件夹
be-music-cabinet fs remove-empty-folders ./BMSRoot

# 计算目录相似度
be-music-cabinet fs bms-dir-similarity ./folder1 ./folder2
```

### 7. 批量操作
```bash
# 创建编号文件夹
be-music-cabinet root-event create-num-folders ./BMSRoot 100

# 生成工作信息表
be-music-cabinet root-event generate-work-info-table ./BMSRoot
```

## 注意事项

1. **备份重要文件**: 在执行任何操作前，请备份重要的BMS文件
2. **路径格式**: 支持相对路径和绝对路径
3. **异步操作**: 所有操作都是异步的，大文件处理可能需要一些时间
4. **外部依赖**: 某些功能需要外部工具（如ffmpeg、flac等）
5. **权限要求**: 确保对目标目录有读写权限

## 错误处理

如果遇到错误，程序会显示详细的错误信息。常见的错误包括：
- 文件不存在或权限不足
- 外部工具未安装
- 磁盘空间不足
- 文件格式不支持

## 帮助信息

获取帮助信息：
```bash
# 主帮助
be-music-cabinet --help

# 子命令帮助
be-music-cabinet work --help
be-music-cabinet root --help
be-music-cabinet pack --help
be-music-cabinet bms --help
be-music-cabinet fs --help
be-music-cabinet root-event --help

# 具体命令帮助
be-music-cabinet work set-name --help
be-music-cabinet bms parse-bms --help
be-music-cabinet fs is-file-same --help
```
