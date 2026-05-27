# 木桃 Mutao - 自动化命令集

# 默认列出所有命令
default:
    @just --list

# ---- 构建 ----

# 检查代码
check:
    cargo check

# 构建项目
build:
    cargo build

# 构建发布版本
release:
    cargo build --release

# ---- 测试 ----

# 运行所有 Rust 测试
test:
    cargo nextest run

# 运行测试并生成覆盖率报告
test-coverage:
    cargo llvm-cov nextest --html

# 运行 Python 测试
test-python:
    cd scalpel && python -m pytest -v

# 运行所有测试（Rust + Python）
test-all: test test-python

# ---- 代码质量 ----

# 格式化代码
fmt:
    cargo fmt

# 检查格式
fmt-check:
    cargo fmt --check

# Clippy 静态分析
clippy:
    cargo clippy -- -D warnings

# 全面检查（格式 + Clippy + 测试）
check-all: fmt-check clippy test

# ---- 数据库 ----

# 初始化数据库
db-init:
    createdb mutao 2>/dev/null || true
    psql mutao < migrations/001_init.sql

# 重置数据库
db-reset:
    dropdb mutao 2>/dev/null || true
    just db-init

# 查看表结构
db-schema:
    psql mutao -c "\dt"

# ---- 运行 ----

# 启动服务
run:
    cargo run

# 启动开发模式（热重载需要 cargo-watch）
dev:
    cargo watch -x run

# ---- 清理 ----

# 清理构建产物
clean:
    cargo clean

# ---- 文档 ----

# 生成并打开文档
docs:
    cargo doc --open

# ---- Git ----

# 提交前检查
pre-commit: fmt-check clippy test
    @echo "Pre-commit 检查通过！"

# 创建提交
commit msg:
    git add -A
    git commit -m "{{msg}}"
