# BMS Resource Toolbox — AGENTS.md

## Lint 配置

### 以下 Lint 禁止使用 `#[allow(...)]` 或 `#[expect(...)]` 等方式压制

- `dead_code`
- `missing_docs`
- `clippy::missing_errors_doc`
- `clippy::missing_panics_doc`

## CI 检查

提交前必须运行在参考文件中的所有检查（运行时不需要 `--verbose` 参数）。
任一检查失败，不得提交。

### 参考文件

- [rust.yml](.github/workflows/rust.yml)
- [cargo-deny.yml](.github/workflows/cargo-deny.yml)

