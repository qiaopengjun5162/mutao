# 木桃 (Mutao) — 项目文档

> "投我以木桃，报之以琼瑶。" —《诗经》
>
> AI 撮合 + Web3 溯源的免现金实体易物平台，面向数字游民与青年社区。

## 快速开始

```bash
# 确保 PostgreSQL 运行中
just db-init

# 启动
cp .env.example .env  # 编辑 DATABASE_URL
cargo run

# API 在 http://localhost:3000
```

### Docker 一键启动

```bash
cp .env.example .env  # 编辑 JWT_SECRET 等
just docker-up        # 或 docker-compose up --build -d
# API: http://localhost:3000
# Scalpel: http://localhost:8000
# PostgreSQL: localhost:5432
```

## 技术栈

| 层 | 技术 | 职责 |
|---|---|---|
| 核心引擎 | Rust (axum + tokio) | API 路由、图匹配算法、状态机 |
| 持久化 | PostgreSQL (sqlx) | 物品、需求、交换环存盘 |
| AI 手术刀 | Python | 图片/文本 → 标签 + 价值梯度 |
| Web3 存证 | Solidity | 物品流转履历上链（多链接口） |
| 前端 | Next.js + shadcn/ui | 用户界面（Turbopack） |

## 项目结构

```
mutao/
├── src/
│   ├── main.rs         # 启动入口 + 路由注册（~70 行）
│   ├── lib.rs          # 库入口 + AppState
│   ├── handlers/       # 请求处理器（按领域拆分）
│   │   ├── mod.rs      # SharedState 类型定义
│   │   ├── items.rs    # 物品 CRUD + 健康检查
│   │   ├── demands.rs  # 交换意向
│   │   ├── cycles.rs   # 匹配 + 交换环确认
│   │   ├── auth_handler.rs # 注册/登录
│   │   ├── ai.rs       # AI 标签提取
│   │   └── blockchain_handler.rs # Web3 存证
│   ├── models.rs       # 领域模型：Item, Demand, SwapCycle, SwapLeg
│   ├── matcher.rs      # 图匹配引擎：DFS 多节点交换环发现
│   ├── store.rs        # 数据访问层
│   ├── auth.rs         # JWT 认证 + 中间件
│   ├── ws.rs           # WebSocket 实时通知
│   ├── error.rs        # 统一错误处理
│   └── blockchain/     # 多链适配器（Ethereum, Solana, Move）
├── frontend/
│   ├── src/
│   │   ├── app/
│   │   │   ├── page.tsx           # 首页
│   │   │   ├── items/page.tsx     # 物品列表
│   │   │   ├── items/new/page.tsx # 发布物品
│   │   │   ├── items/[id]/page.tsx# 物品详情
│   │   │   ├── demands/page.tsx   # 交换意向
│   │   │   └── cycles/page.tsx    # 交换环
│   │   ├── components/
│   │   │   ├── nav.tsx            # 导航栏（含 WebSocket 通知指示器）
│   │   │   └── ui/button.tsx      # 按钮组件
│   │   ├── hooks/
│   │   │   └── use-ws.ts          # WebSocket 实时通知 hook
│   │   └── lib/
│   │       ├── api.ts             # API 客户端（含 WS、AI、Web3）
│   │       └── utils.ts           # 工具函数
│   └── package.json
├── scalpel/
│   ├── scalpel.py      # Python AI 手术刀（规则引擎 + LLM 降级）
│   └── test_scalpel.py # Python 测试
├── contracts/
│   └── MutaoSwapHistory.sol  # Web3 存证合约
├── tests/
│   ├── matcher_test.rs  # 匹配引擎测试 (5 个)
│   └── models_test.rs   # 模型层测试 (13 个)
│                        # 内联测试: auth(3) + ws(5) + store(5) = 13
├── migrations/
│   ├── 001_init.sql     # 数据库建表
│   ├── 002_users.sql    # 用户表 + 外键
│   └── 003_indexes.sql  # 性能索引 + 级联删除
├── .github/workflows/
│   └── ci.yml           # CI/CD 配置
├── Dockerfile           # Rust 后端镜像（多阶段构建）
├── Dockerfile.scalpel   # Python 手术刀镜像
├── docker-compose.yml   # 一键启动：后端 + DB + 手术刀
├── Justfile             # 自动化命令集
├── PROGRESS.md          # 项目进度记录
├── .env                 # 环境变量
└── Cargo.toml
```

## API 端点

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | /api/health | 健康检查 |
| POST | /api/items | 创建物品（含验证） |
| POST | /api/items/analyze | AI 标签提取（调用 scalpel） |
| GET | /api/items | 物品列表 |
| GET | /api/items/:id | 物品详情 |
| POST | /api/items/:id/match | 触发匹配 |
| POST | /api/items/:id/attest | Web3 存证上链 |
| GET | /api/items/:id/history | 查询链上存证历史 |
| PATCH | /api/items/:id/status | 更新物品状态 |
| POST | /api/demands | 创建交换意向（含验证） |
| GET | /api/demands | 意向列表 |
| GET | /api/cycles | 交换环列表 |
| POST | /api/cycles/:id/confirm | 确认交换 |
| POST | /api/auth/register | 用户注册 |
| POST | /api/auth/login | 用户登录 |
| GET | /api/ws | WebSocket 实时通知 |

## 前端路由

| 路径 | 说明 |
|---|---|
| / | 首页（导航入口） |
| /items | 物品列表（卡片布局） |
| /items/new | 发布物品（表单） |
| /items/[id] | 物品详情（触发匹配、AI 分析、Web3 存证） |
| /demands | 交换意向列表 |
| /cycles | 交换环列表（确认交换） |
| /auth/login | 登录页 |
| /auth/register | 注册页 |

## 测试

```bash
cargo nextest run             # Rust 测试 (18 个)
cd scalpel && pytest -v       # Python 测试 (10 个)
just test-all                 # 全部测试
cargo llvm-cov nextest --html # 覆盖率报告
```

## 代码质量

```bash
cargo fmt                     # 格式化
cargo clippy -- -D warnings   # 静态分析
just check-all                # 全面检查（格式 + Clippy + 测试）
```

## 数据库

```bash
just db-init                  # 初始化
just db-reset                 # 重置
just db-schema                # 查看表结构
```

## 核心算法

`matcher::Matcher::find_cycles()` 在有向图中使用 DFS 搜索长度 2~4 的交换环：

1. 建边：`Demand[i].offer_tags` 与 `Demand[j].target_tags` 有交集时，`i → j`
2. DFS：从每个节点出发，搜索回到起点的有向环
3. 过滤：去重、同用户去重、验证环闭合

## Session 记录 — 2026-05-27

### Phase 1 - 核心引擎
- Rust 匹配引擎 + axum API
- 重构：内存存储 → PostgreSQL (sqlx)
- 修复：`build_cycle` 中 `want_item_id` 多人环计算错误
- 修复：同一用户多个 Demand 出现自己换自己

### Phase 2 - 质量提升
- 补充模型层测试 (8 个)、Python 测试 (10 个)
- 重构：提取 `store.rs` 数据访问层
- 统一错误处理：`AppError` 枚举
- 添加输入验证：title、tags、value_tier
- 配置 Justfile 自动化命令集
- 配置 GitHub Actions CI/CD

### Phase 3 - 移植 rust-template 最佳实践
- pre-commit hooks、cargo-deny、git-cliff、typos
- rustfmt.toml、thiserror 错误处理升级
- CI 增强：taplo + deny + typos + 自动 Release

### Phase 4 - 前端页面 + 用户认证
- Next.js + shadcn/ui 前端（9 个路由）
- JWT 认证（注册/登录）
- API 客户端封装

### Phase 5 - WebSocket + 代码优化 + 全链路打通
- JWT_SECRET 环境变量化（OnceLock）
- WebSocket 实时通知（tokio::sync::broadcast）
- 数据库性能索引（GIN + 复合索引）+ 外键级联删除
- AI 标签提取（POST /api/items/analyze，调用 scalpel.py）
- Web3 存证端到端（ChainManager + 多链适配器）
- 移动端 H5 响应式适配（导航汉堡菜单）
- 前端对接 WebSocket + AI + Web3
- README 中英文

### Phase 6 - 代码质量提升（2026-05-28）
- 修复 cargo deny check：许可证允许 ISC/BSD-3-Clause，安全审计例外
- 模块化 main.rs：拆分为 6 个 handler 子模块（items/demands/cycles/auth/ai/blockchain）
- 补充测试：ws.rs(5) + store.rs(5) = 10 个新测试
- 消除 unwrap()：main.rs 改为 `async fn main() -> Result<(), Box<dyn Error>>`
- taplo 格式化修复
- 安装 pre-commit hooks（之前配置了但未安装到 .git/hooks/）
- 修复 CI：cargo-llvm-cov action 名称、PGPASSWORD、执行全部 migration
- 安装 Trellis AI 工作流框架（`trellis init --claude -u qiaopengjun`）

### 当前状态（2026-05-28）
- Rust 测试：59 个全部通过（单元 17 + 模型 10 + 匹配 5 + Store 集成 10 + Handler 集成 17）
- Python 测试：10 个全部通过
- 前端路由：9 个
- 所有 P0/P1/P2 功能已完成
- Trellis 已初始化
