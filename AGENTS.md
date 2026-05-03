# BMS Resource Toolbox — AGENTS.md

## 代码质量底线

以下规则优先级高于工具链的 auto-fix 建议。如果工具链（clippy、rustc）报告了相关 lint 问题，必须修复根本原因，**不得**通过 `#[allow(...)]` 或 `#[expect(...)]` 压制。

- **不得压制 `dead_code`**。未使用的代码必须删除，而非添加 `#[allow(dead_code)]` 或 `#[expect(dead_code)]`。
- **不得压制 `missing_docs`**。公有项缺少文档时，必须补写 `///` 文档注释，而非添加 `#![allow(missing_docs)]`。
- **不得压制 `clippy::missing_errors_doc`**。返回 `Result` 的公有函数必须有 `# Errors` 节。
- **不得压制 `clippy::missing_panics_doc`**。可能 panic 的公有函数必须有 `# Panics` 节。
- **不得压制 `clippy::pedantic` 系列 lint**。pedantic lint 已通过 `Cargo.toml` 启用为 warn，应通过修改代码满足要求，而非添加 `#[allow(clippy::*)]`。

例外：仅在第三方 crate 的宏展开确实无法避免时，可用 `#[allow(...)]` 包裹具体调用点，并注释说明原因。

## CI 检查

提交前必须通过以下全部检查：

```shell
cargo clippy --all-features --all-targets -- -D warnings
cargo build --all-features
cargo test --all-features --all-targets
cargo fmt --check
cargo doc --workspace --no-deps --all-features
```

任一检查失败，不得提交。
