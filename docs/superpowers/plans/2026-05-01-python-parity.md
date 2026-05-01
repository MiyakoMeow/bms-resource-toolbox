# Python 行为完全对齐 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 使 bms-resource-toolbox 的 Rust 实现与 bms-resource-scripts 的 Python 参考实现行为完全一致。

**Architecture:** 按7个模块（bms / fs / media / options / scripts / wasted / main+lib）独立修正。每模块先读Python源确认行为，再修Rust源使其匹配。各模块可并行。

**Tech Stack:** Rust 2024 edition, tokio async, thiserror/anyhow

---

## 差异清单（按模块 × 优先级）

### 模块 1: bms（6 critical, 5 medium, 5 low）

- [ ] **B1.** BMP 解析重写：遍历所有 `#BMP*` 行（含数字后缀），提取文件扩展名，不去重
  - Python: `parse.py:58-62` — `startswith("#BMP")` + `replace("#BMP","")`
  - Rust: `parse.rs:68-93` — 仅匹配裸键和非数字后缀
  - File: `src/bms/parse.rs`

- [ ] **B2.** PLAYLEVEL 解析修正：加 isdecimal 守卫、float→int、0..=99 范围、超范围返回 -1
  - Python: `parse.py:48-52`
  - Rust: `parse.rs:48-49`
  - File: `src/bms/parse.rs`

- [ ] **B3.** BMSON 传递编码：`parse_bmson_file` 接受 encoding 参数
  - Python: `__init__.py:35` 传 `encoding`
  - Rust: `dir.rs:56` 无 encoding
  - Files: `src/bms/parse.rs`, `src/bms/dir.rs`

- [ ] **B4.** BMSON 解析失败时 push Error BMSInfo（而非跳过）
  - Python: `parse.py:79-81`
  - Rust: `dir.rs:56` 静默跳过
  - File: `src/bms/dir.rs`

- [ ] **B5.** BMSON level 字段改为 `Option<f64>`（支持浮点 level）
  - Python: `parse.py:94` — `int(dict_get(...) or 0)`
  - Rust: `parse.rs:126-127` — `Option<i32>` serde 反序列化失败
  - File: `src/bms/parse.rs`

- [ ] **B6.** 头部解析改为区分大小写（去掉 `.to_uppercase()`）
  - Python: `parse.py:40-62` — 区分大小写
  - Rust: `parse.rs:27-39` — `.to_uppercase()` 不区分大小写
  - File: `src/bms/parse.rs`

- [ ] **B7.** 删除 Rust 多余字段 `total`、`stage_file`，或标注为有意扩展
  - Python: `parse.py:20-26` 无此字段
  - Rust: `types.rs:50-67` 有
  - File: `src/bms/types.rs`

- [ ] **B8.** 删除 dir.rs 中的解析回退逻辑（Python 无）
  - Rust: `dir.rs:46-55`
  - File: `src/bms/dir.rs`

- [ ] **B9.** MEDIA_FILE_EXTS 排序改为与 Python 一致：`.flac, .ogg, .wav`
  - Python: `__init__.py:19`
  - Rust: `types.rs:110-114`
  - File: `src/bms/types.rs`

### 模块 2: fs（4 critical, 6 high, 6 medium, 6 low）

- [ ] **F1.** `extract_zip` 添加路径遍历保护（zip slip）
  - Python: `rawpack.py:16-31,86` — `_safe_join`
  - Rust: `rawpack.rs:26-55` — `safe_join` 仅 `#[cfg(test)]`
  - File: `src/fs/rawpack.rs`

- [ ] **F2.** `extract_zip` 解压后设置文件修改时间
  - Python: `rawpack.py:34-43,89,94` — `_set_mtime`
  - Rust: `rawpack.rs:59-64` — `set_mtime` dead code
  - File: `src/fs/rawpack.rs`

- [ ] **F3.** 扩展名提取统一为 `rsplit(".")[-1]` 逻辑
  - 影响: `sync.rs:224-228`, `rawpack.rs:515-519`
  - Files: `src/fs/sync.rs`, `src/fs/rawpack.rs`

- [ ] **F4.** `get_work_folder_name` 增加 `id` 前缀：`"{id}. {title} [{artist}]"`
  - Python: `name.py:23-24`
  - Rust: `name.rs:35-41`
  - File: `src/fs/name.rs` + 所有调用点

- [ ] **F5.** `Rename`/`CheckReplace` 重命名时检查已存在目标的内容
  - Python: `move.py:112-117` — `is_same_content` 检查
  - Rust: `pack_move.rs:232-234` — 仅检查 exists
  - File: `src/fs/pack_move.rs`

- [ ] **F6.** `move_out_files_in_folder_in_cache_dir` mp4 检查时机：用循环内最后一次 `file_ext_count`
  - Python: `rawpack.py:231-233`
  - Rust: `rawpack.rs:612-620` — 重新扫描
  - File: `src/fs/rawpack.rs`

- [ ] **F7.** `sync_folder` SHA-512 空值保护：去掉 Rust 额外的 `!src_value.is_empty()` 检查
  - Python: `sync.py:218` — 空 == 空认为相同
  - Rust: `sync.rs:291` — 额外 `!is_empty()`
  - File: `src/fs/sync.rs`

- [ ] **F8.** `get_num_set_file_names` 去掉排序
  - Python: `rawpack.py:138-148` — 不排序
  - Rust: `rawpack.rs:87` — `names.sort()`
  - File: `src/fs/rawpack.rs`

### 模块 3: media（6 high, 7 medium, 7 low）

- [ ] **M1.** 音频 fallback 实现多级递归（当前仅一级）
  - Python: `audio.py:63-223` — 无限级 fallback
  - Rust: `convert.rs:118-234` — 仅一级
  - File: `src/media/convert.rs`

- [ ] **M2.** OGG_FFMPEG preset arg 改为 `Some("")`（非 None）
  - Python: `audio.py:25` — `arg=""`
  - Rust: `audio.rs:42-44` — `arg=None`
  - File: `src/media/audio.rs`

- [ ] **M3.** 视频添加 `use_prefered` 功能
  - Python: `video.py:159-236` — `get_prefered_preset_list`
  - Rust: `video.rs:363-454` — 缺失
  - File: `src/media/video.rs`

- [ ] **M4.** 视频处理捕获并输出 stderr
  - Python: `video.py:218-227` — 打印 stdout/stderr
  - Rust: `video.rs:329-355` — 丢弃
  - File: `src/media/video.rs`

- [ ] **M5.** 视频并发池改为 FIFO（`VecDeque` + `pop_front`）
  - Rust: `video.rs:396-399` — `Vec::pop()` LIFO bug
  - File: `src/media/video.rs`

- [ ] **M6.** 视频 convert 错误传播：收集 handle 结果，任一失败则返回 Err
  - Python: `video.py:282-283` — 返回 False 中断
  - Rust: `video.rs:453` — 永远 Ok
  - File: `src/media/video.rs`

- [ ] **M7.** 音频处理捕获 stderr 并输出
  - Python: `audio.py:115-117,210-216`
  - Rust: `convert.rs:188-189`
  - File: `src/media/convert.rs`

- [ ] **M8.** 音频处理添加 fallback 统计输出
  - Python: `audio.py:220-221` — 打印 fallback 统计
  - File: `src/media/convert.rs`

- [ ] **M9.** 删除 `convert.rs` 中重复的 `convert_video` 函数
  - File: `src/media/convert.rs`

### 模块 4: options（7 high, 10 medium, 7 low）

- [ ] **O1.** `set_file_num` 的 `.part` 检测逻辑修正：检查 `{file_name}.part` 伴随文件是否存在
  - Python: `rawpack.py:176-177`
  - Rust: `rawpack.rs:396` — 检查当前文件名以 `.part` 结尾
  - File: `src/options/rawpack.rs`

- [ ] **O2.** `set_file_num` 去掉排序
  - Python: `rawpack.py:166-191` — 不排序
  - Rust: `rawpack.rs:421` — `file_names.sort()`
  - File: `src/options/rawpack.rs`

- [ ] **O3.** `set_file_num` 改为循环实现（非递归）
  - Python: `rawpack.py:216-226` — `while True`
  - Rust: `rawpack.rs:475` — 递归
  - File: `src/options/rawpack.rs`

- [ ] **O4.** `generate_work_info_table` Excel sheet 名改为 "BMS List"
  - Python: `bms_folder_event.py:44-45`
  - Rust: `bms_folder_event.rs:88-89`
  - File: `src/options/bms_folder_event.rs`

- [ ] **O5.** `generate_work_info_table` ID 解析改为 `split(".")[0]`
  - Python: `bms_folder_event.py:59-63`
  - Rust: `bms_folder_event.rs:107-108` — `parse_work_dir_name`
  - File: `src/options/bms_folder_event.rs`

- [ ] **O6.** `move_works_with_same_name` 错误处理：输入目录不存在时应报错
  - Python: `bms_folder_bigpack.py:318-320` — `raise ValueError`
  - Rust: `bms_folder_bigpack.rs:541-543` — 静默 `Ok(())`
  - File: `src/options/bms_folder_bigpack.rs`

- [ ] **O7.** `remove_zero_sized_media_files` 统一为一个实现，删除 `bms_folder_bigpack.rs` 中的副本
  - File: `src/options/bms_folder.rs`, `src/options/bms_folder_bigpack.rs`

- [ ] **O8.** `_check_exec` 失败时打印诊断信息
  - Python: `options/__init__.py:228-232`
  - Rust: `validator.rs:15-47` — 静默返回 false
  - File: `src/options/validator.rs`

### 模块 5: scripts（1 critical, 4 high, 3 medium）

- [ ] **S1.** 音频/视频转换添加子目录遍历
  - Python: `media/audio.py:265-279` — `bms_folder_transfer_audio` 递归
  - Rust: `scripts/pack.rs:185-196` — 直接在 root_dir 上调用
  - Files: `src/scripts/pack.rs`

- [ ] **S2.** `pack.rs` 中 `remove_unneed_media_files` 复用 `bms_folder_bigpack.rs` 版本，删除重复实现
  - Files: `src/scripts/pack.rs`

- [ ] **S3.** 扩展名比较改为大小写敏感（去掉 `to_lowercase()`）
  - Python: `options/bms_folder_bigpack.py:187-189` — 大小写敏感
  - Rust: `scripts/pack.rs:74-77` — `to_lowercase()`
  - Files: `src/scripts/pack.rs`, `src/media/convert.rs`, `src/media/video.rs`

- [ ] **S4.** `pack_update_rawpack_to_hq` 添加自定义验证函数
  - Python: `scripts/pack.py:218` — 含 `_pack_update_rawpack_to_hq_check`
  - Rust: `main.rs:669` — 缺失
  - Files: `src/scripts/pack.rs`, `src/main.rs`

- [ ] **S5.** 错误处理：`.ok()` 改为打印错误
  - Rust: `main.rs:628,652,677,691`
  - File: `src/main.rs`

### 模块 6: wasted（3 medium, 2 low）

- [ ] **W1.** Aery 匹配大小写：确认 Python 的 `"Aery"/"AERY"` 限制，Rust 对齐
  - Python: `aery_fix.py:17`
  - Rust: `aery_fix.rs:41-42`
  - File: `src/wasted/aery_fix.rs`

### 模块 7: main+lib（2 critical, 3 high, 5 medium）

- [ ] **X1.** `transfer_audio`/`transfer_video` 使用 `args[0]`（非重新输入）
  - Python: `options/bms_folder_media.py:17` — 直接接收 root_dir
  - Rust: `main.rs:526-529` — 重新调用 `input_path`
  - File: `src/main.rs`

- [ ] **X2.** `is_root_dir` 检查所有路径参数（非仅 first）
  - Python: `options/__init__.py:206-211` — 遍历所有
  - Rust: `main.rs:159-183` — 只取 first
  - File: `src/main.rs`

- [ ] **X3.** 选项名文字："工作信息页" → "作品信息页"
  - File: `src/main.rs:219`

- [ ] **X4.** `error.rs` 填充统一错误枚举，`main.rs` 中 `let _ =` 改为打印错误
  - Files: `src/error.rs`, `src/main.rs`

- [ ] **X5.** 移除未使用的 `clap` 依赖
  - File: `Cargo.toml`

- [ ] **X6.** `Input.exec_input(Any)` 提示语统一：`"Input:"`（无空格）
  - File: `src/options/input.rs`

---

## 执行策略

各模块独立，可并行。按以下分组派子Agent：

1. **Agent A**: bms 模块（B1-B9）
2. **Agent B**: fs 模块（F1-F8）
3. **Agent C**: media 模块（M1-M9）
4. **Agent D**: options 模块（O1-O8）
5. **Agent E**: scripts 模块（S1-S5）
6. **Agent F**: wasted 模块（W1）
7. **Agent G**: main+lib（X1-X6）

每个Agent的任务：读取Python源文件确认行为 → 修改Rust源 → 确保编译通过。
