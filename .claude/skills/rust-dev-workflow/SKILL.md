---
name: rust-dev-workflow
description: Rust 项目开发工作流 — 测试、审计、格式化、CI/CD、commit 规范。适用于所有 Rust 项目。参考 qiaopengjun5162/rust-template。
---

# Rust 开发工作流

参考模板：https://github.com/qiaopengjun5162/rust-template

## 核心原则

每次代码变更后，按以下顺序执行：

1. **格式化** → `cargo fmt` + `taplo fmt`
2. **静态分析** → `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`
3. **测试** → `cargo nextest run`（不用 cargo test）
4. **审计** → `cargo deny check`
5. **提交** → git commit + push
6. **文档** → 更新 PROGRESS.md / CLAUDE.md

## 测试要求

- 测试运行器：**cargo-nextest**（不是 cargo test）
- 覆盖率：**cargo-llvm-cov**
- 目标：尽可能 100% 覆盖率
- 测试文件放在 `tests/` 目录，命名 `*_test.rs`

```bash
cargo nextest run --all-features                              # 运行测试
cargo nextest run --all-features -- --include-ignored         # 含 ignored 测试
cargo llvm-cov nextest --html                                 # 覆盖率 HTML 报告
cargo llvm-cov nextest --lcov --output-path lcov.info         # 覆盖率 LCOV（CI 用）
```

## 代码质量工具链

| 工具 | 用途 | 命令 |
|------|------|------|
| rustfmt | Rust 格式化 | `cargo fmt` |
| taplo | TOML 格式化 | `taplo fmt --option reorder_keys=true` |
| clippy | 静态分析 | `cargo clippy --all-targets --all-features --tests --benches -- -D warnings` |
| cargo-deny | 依赖审计（许可证、漏洞） | `cargo deny check` |
| typos | 拼写检查 | `typos` |
| git-cliff | CHANGELOG 生成 | `git cliff -o CHANGELOG.md` |
| cargo-generate | 项目模板生成 | `cargo generate rust-template` |

## Commit 规范

- 使用 Conventional Commits：`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`
- 每次修改都要 commit 和 push
- commit message 用英文

## 文档维护

每次重大变更后更新：
- `PROGRESS.md` — 进度记录（已完成、待办、测试覆盖）
- `CLAUDE.md` — 项目文档（架构、API、命令速查）
- `README.md` — 中文 README（项目介绍、快速开始、API、架构）
- `README_en.md` — 英文 README（面向国际社区）
- `CONTRIBUTING.md` — 贡献指南（开源项目必备）

## 错误处理模式

使用 `thiserror` 定义错误枚举，实现 `IntoResponse`：

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("资源不存在")]
    NotFound,
    #[error("{0}")]
    BadRequest(String),
    #[error("服务器内部错误")]
    Internal(#[from] sqlx::Error),
}
```

## Cargo 配置

### `.cargo/config.toml` — 编译优化

```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

### `Cargo.toml` — Release Profile

```toml
[profile.release]
codegen-units = 1
lto = "fat"
panic = "abort"
strip = true

[profile.profiling]
debug = "full"
inherits = "release"
strip = false
```

## CI/CD 模板

GitHub Actions 流水线：fmt → taplo → clippy → nextest → release

- fmt + taplo：格式检查
- clippy：静态分析（含 `--benches`）
- nextest：测试运行
- llvm-cov：覆盖率上传 Codecov
- release：git-cliff CHANGELOG + GitHub Release（仅 tag 触发）

## 自动化命令

Justfile 或 Makefile 二选一：

### Justfile（推荐）

```
just test          # cargo nextest run
just check-all     # fmt + clippy + deny + test
just pre-commit    # 提交前完整检查
just changelog     # 生成 CHANGELOG
just db-init       # 初始化数据库
just db-reset      # 重置数据库
```

### Makefile

```makefile
CARGO := cargo
MAIN_BRANCH := main

build:
	@$(CARGO) build --all-features

test:
	@$(CARGO) nextest run --all-features

clippy:
	@$(CARGO) clippy --all-features -- -D warnings

format:
	@$(CARGO) fmt --all -- --check

release: test
	@cargo release --execute
	@git cliff -o CHANGELOG.md
```

## 配置文件清单

每个 Rust 项目应包含：

| 文件 | 用途 |
|------|------|
| `.cargo/config.toml` | 编译优化（target-cpu=native） |
| `.rustfmt.toml` | Rust 2024 格式化配置 |
| `.pre-commit-config.yaml` | Git hooks（含 clippy --benches、nextest --include-ignored） |
| `deny.toml` | 依赖审计 |
| `cliff.toml` | CHANGELOG 生成 |
| `_typos.toml` | 拼写检查白名单 |
| `cargo-generate.toml` | 项目模板变量（可选） |
| `Justfile` 或 `Makefile` | 自动化命令 |
| `.github/workflows/build.yml` | CI/CD |
| `CONTRIBUTING.md` | 贡献指南 |
| `LICENSE` | MIT 许可证 |
