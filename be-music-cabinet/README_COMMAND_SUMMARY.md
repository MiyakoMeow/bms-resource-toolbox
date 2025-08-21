# be-music-cabinet 命令总结

本文档总结了be-music-cabinet命令行应用的所有可用命令。

## 命令分类

### 1. 工作目录操作 (work)
处理单个BMS工作目录的操作。

| 命令 | 功能 | 参数 |
|------|------|------|
| `set-name` | 根据BMS文件设置目录名 | `DIR` + `--set-type` |
| `undo-set-name` | 撤销设置目录名 | `DIR` + `--set-type` |
| `remove-empty-media` | 移除零字节媒体文件 | `DIR` |

### 2. 根目录操作 (root)
处理BMS根目录的批量操作。

| 命令 | 功能 | 参数 |
|------|------|------|
| `copy-numbered-names` | 复制编号工作目录名 | `FROM` `TO` |
| `split-by-first-char` | 按首字符分割文件夹 | `DIR` |
| `undo-split` | 撤销分割操作 | `DIR` |
| `merge-split` | 合并分割的文件夹 | `DIR` |
| `move-works` | 移动作品 | `FROM` `TO` |
| `move-out-works` | 移出一层目录 | `DIR` |
| `move-same-name` | 移动同名作品 | `FROM` `TO` |
| `remove-unneed-media` | 移除不需要的媒体文件 | `DIR` + `--rule` |
| `scan-similar-folders` | 扫描相似文件夹 | `DIR` + `--similarity` |

### 3. 大包处理操作 (pack)
处理BMS大包的转换和生成。

| 命令 | 功能 | 参数 |
|------|------|------|
| `raw-to-hq` | 原包转HQ版大包 | `DIR` |
| `hq-to-lq` | HQ版转LQ版大包 | `DIR` |
| `setup-rawpack-to-hq` | 大包生成脚本 | `PACK_DIR` `ROOT_DIR` |
| `update-rawpack-to-hq` | 大包更新脚本 | `PACK_DIR` `ROOT_DIR` `SYNC_DIR` |

### 4. BMS文件相关操作 (bms)
处理BMS文件的解析和检查。

| 命令 | 功能 | 参数 |
|------|------|------|
| `parse-bms` | 解析BMS文件 | `FILE` |
| `parse-bmson` | 解析BMSON文件 | `FILE` |
| `get-bms-list` | 获取BMS文件列表 | `DIR` |
| `get-bms-info` | 获取BMS信息 | `DIR` |
| `is-work-dir` | 检查是否为工作目录 | `DIR` |
| `is-root-dir` | 检查是否为根目录 | `DIR` |

### 5. 文件系统相关操作 (fs)
处理文件系统的各种操作。

| 命令 | 功能 | 参数 |
|------|------|------|
| `is-file-same` | 检查文件内容是否相同 | `FILE1` `FILE2` |
| `is-dir-having-file` | 检查目录是否包含文件 | `DIR` |
| `remove-empty-folders` | 移除空文件夹 | `DIR` |
| `bms-dir-similarity` | 计算BMS目录相似度 | `DIR1` `DIR2` |

### 6. 根目录事件相关操作 (root-event)
处理根目录的事件和批量操作。

| 命令 | 功能 | 参数 |
|------|------|------|
| `check-num-folder` | 检查编号文件夹 | `DIR` `MAX` |
| `create-num-folders` | 创建编号文件夹 | `DIR` `COUNT` |
| `generate-work-info-table` | 生成工作信息表 | `DIR` |

## 常用参数说明

### 设置类型 (--set-type)
- `replace_title_artist`: 替换为标题+艺术家
- `append_title_artist`: 追加标题+艺术家（默认）
- `append_artist`: 仅追加艺术家

### 规则类型 (--rule)
- `oraja`: beatoraja规则（默认）
- `wav_fill_flac`: WAV填充FLAC规则
- `mpg_fill_wmv`: MPG填充WMV规则

### 相似度阈值 (--similarity)
- 范围：0.0 - 1.0
- 默认值：0.7

## 快速参考

### 基本操作
```bash
# 设置目录名
be-music-cabinet work set-name ./MyBMSFolder

# 移除不需要的文件
be-music-cabinet root remove-unneed-media ./BMSRoot

# 转换大包
be-music-cabinet pack raw-to-hq ./BMSRoot
```

### 文件检查
```bash
# 检查目录类型
be-music-cabinet bms is-work-dir ./MyBMSFolder

# 检查文件是否相同
be-music-cabinet fs is-file-same ./file1.txt ./file2.txt

# 移除空文件夹
be-music-cabinet fs remove-empty-folders ./BMSRoot
```

### 批量操作
```bash
# 分割文件夹
be-music-cabinet root split-by-first-char ./BMSRoot

# 创建编号文件夹
be-music-cabinet root-event create-num-folders ./BMSRoot 100

# 生成信息表
be-music-cabinet root-event generate-work-info-table ./BMSRoot
```

## 注意事项

1. 所有路径都支持相对路径和绝对路径
2. 所有操作都是异步的，大文件处理可能需要时间
3. 某些功能需要外部工具（ffmpeg、flac等）
4. 执行前请备份重要文件
5. 确保对目标目录有读写权限

## 获取帮助

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
```
