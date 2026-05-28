# 木桃 Mutao - 自动化命令集

# 默认列出所有命令
default:
    @just --list

# ---- 构建 ----

# 检查代码
check:
    cargo check --all-features

# 构建项目
build:
    cargo build --all-features

# 构建发布版本
release:
    cargo build --release --all-features

# ---- 测试 ----

# 运行所有 Rust 测试
test:
    cargo nextest run --all-features

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
    cargo fmt --all -- --check

# TOML 格式化
toml-fmt:
    taplo fmt --option reorder_keys=true

# 检查 TOML 格式
toml-fmt-check:
    taplo fmt --option reorder_keys=true --check

# Clippy 静态分析
clippy:
    cargo clippy --all-targets --all-features --tests -- -D warnings

# 依赖审计
deny:
    cargo deny check

# 拼写检查
typos:
    typos

# 全面检查（格式 + Clippy + 测试）
check-all: fmt-check toml-fmt-check clippy deny test

# ---- 数据库 ----

# 初始化数据库（按顺序执行所有迁移）
db-init:
    createdb mutao 2>/dev/null || true
    for f in migrations/*.sql; do psql mutao < "$$f"; done

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

# ---- Docker ----

# Docker 构建并启动所有服务
docker-up:
    docker-compose up --build -d

# Docker 停止所有服务
docker-down:
    docker-compose down

# Docker 查看日志
docker-logs:
    docker-compose logs -f

# ---- 清理 ----

# 清理构建产物
clean:
    cargo clean

# ---- 文档 ----

# 生成并打开文档
docs:
    cargo doc --open

# 生成 CHANGELOG
changelog:
    git cliff -o CHANGELOG.md

# ---- Git ----

# 提交前检查
pre-commit: fmt-check toml-fmt-check clippy deny test
    @echo "Pre-commit 检查通过！"

# 创建提交
commit msg:
    git add -A
    git commit -m "{{msg}}"

# 帮助
help:
    @just --list
