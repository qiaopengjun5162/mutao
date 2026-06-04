# 木桃 Mutao

[![CI](https://github.com/qiaopengjun5162/mutao/actions/workflows/ci.yml/badge.svg)](https://github.com/qiaopengjun5162/mutao/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.87+-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/Tests-62-green.svg)](#测试)

> "投我以木桃，报之以琼瑶。" ——《诗经》

AI 撮合 + Web3 溯源的免现金实体易物平台，面向数字游民与青年社区。

[English](README.md)

## 技术栈

| 层 | 技术 | 职责 |
|---|---|---|
| 核心引擎 | Rust 1.87+ (axum 0.7 + tokio) | API 路由、图匹配算法、状态机 |
| 持久化 | PostgreSQL 16 (sqlx 0.7) | 物品、需求、交换环存盘 |
| AI 手术刀 | Python 3.12+ | 图片/文本 → 标签 + 价值梯度 |
| Web3 存证 | Solidity | 物品流转履历上链（多链接口） |
| 前端 | Next.js 16 + shadcn/ui | 用户界面（Turbopack） |

## 快速开始

```bash
# 方式一：Docker 一键启动（推荐）
cp .env.example .env   # 编辑 JWT_SECRET
just docker-up         # 启动所有服务

# 方式二：本地开发
# 前置条件：PostgreSQL 运行中，Rust 1.87+
cp .env.example .env   # 编辑 DATABASE_URL
just db-init           # 初始化数据库
cargo run              # 启动后端 http://localhost:3000

# 前端
cd frontend && pnpm install && pnpm run dev   # http://localhost:3001
```

## API 端点

| 方法 | 路径 | 说明 | 鉴权 |
|---|---|---|---|
| GET | /api/health | 健康检查 | 公开 |
| POST | /api/auth/register | 用户注册 | 公开 |
| POST | /api/auth/login | 用户登录 | 公开 |
| POST | /api/items | 创建物品 | Bearer |
| GET | /api/items | 物品列表 | 公开 |
| GET | /api/items/:id | 物品详情 | 公开 |
| POST | /api/items/:id/match | 触发匹配 | Bearer |
| PATCH | /api/items/:id/status | 更新物品状态 | Bearer |
| POST | /api/items/analyze | AI 标签提取 | Bearer |
| POST | /api/items/:id/attest | Web3 存证 | Bearer |
| GET | /api/items/:id/history | 链上历史 | 公开 |
| POST | /api/demands | 创建交换意向 | Bearer |
| GET | /api/demands | 意向列表 | 公开 |
| GET | /api/cycles | 交换环列表 | 公开 |
| POST | /api/cycles/:id/confirm | 确认交换 | Bearer |
| GET | /api/ws | WebSocket 实时通知 | ?token= |

> 鉴权：**公开** = 无需 token；**Bearer** = 需 `Authorization: Bearer <JWT>`，否则 401；**?token=** = JWT 通过 WebSocket 查询参数传入。物主限定的写操作对非物主返回 403。

## 项目结构

```
mutao/
├── src/
│   ├── main.rs          # axum 路由 + 处理器
│   ├── lib.rs           # AppState + 库入口
│   ├── models.rs        # 领域模型
│   ├── matcher.rs       # DFS 多节点交换环发现
│   ├── store.rs         # 数据访问层
│   ├── auth.rs          # JWT 认证
│   ├── error.rs         # 统一错误处理
│   ├── ws.rs            # WebSocket 广播
│   └── blockchain/      # 多链适配器
├── frontend/src/app/    # Next.js 页面（9 个路由）
├── scalpel/             # Python AI 手术刀
├── contracts/           # Solidity 存证合约
├── migrations/          # 数据库迁移
├── tests/               # Rust 测试（59 个）
├── Dockerfile           # Rust 后端镜像（多阶段构建）
├── Dockerfile.scalpel   # Python 手术刀镜像
└── docker-compose.yml   # 后端 + DB + 手术刀编排
```

## 测试

```bash
cargo nextest run             # Rust 测试 (59 个)
cd scalpel && pytest -v       # Python 测试 (10 个)
just test-all                 # 全部测试
cargo llvm-cov nextest --html # 覆盖率报告
```

## 核心算法

`matcher::Matcher::find_cycles()` 使用 DFS 在有向图中搜索长度 2~4 的交换环：

1. **建边**：`Demand[i].offer_tags` 与 `Demand[j].target_tags` 有交集时，`i → j`
2. **DFS**：从每个节点出发，搜索回到起点的有向环
3. **过滤**：去重、同用户去重、验证环闭合

## 许可证

MIT
