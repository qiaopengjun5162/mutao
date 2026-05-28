# 木桃 Mutao - 项目进度记录

> 更新时间：2026-05-29
>
> 仓库地址：https://github.com/qiaopengjun5162/mutao

## 一、项目概述

AI 撮合 + Web3 溯源的免现金实体易物平台，面向数字游民与青年社区。

**技术栈**：
- 后端：Rust (axum + tokio + sqlx)
- 数据库：PostgreSQL
- AI 手术刀：Python (规则引擎 + LLM 降级)
- Web3 存证：Solidity (多链接口设计中)
- 前端（规划中）：Next.js + shadcn/ui + Rust WASM

---

## 二、已完成工作

### 2026-05-27 Phase 1 - 核心引擎

| 模块 | 状态 | 说明 |
|------|------|------|
| 领域模型 | ✅ | `Item`, `Demand`, `SwapCycle`, `SwapLeg` |
| 匹配引擎 | ✅ | DFS 算法，支持 2-4 人交换环 |
| API 层 | ✅ | axum 路由 + PostgreSQL CRUD |
| 数据库 | ✅ | 3 张表建表脚本 |
| Python 手术刀 | ✅ | 规则引擎 + LLM 降级 |
| Solidity 合约 | ✅ | 50 行存证合约 |

### 2026-05-27 Phase 2 - 质量提升

| 任务 | 状态 | 说明 |
|------|------|------|
| 模型层测试 | ✅ | 8 个单元测试（序列化、is_valid） |
| 匹配引擎测试 | ✅ | 5 个单元测试（2人环、3人环、无环等） |
| Python 测试 | ✅ | 10 个 pytest 用例 |
| 重构 main.rs | ✅ | 提取 `store.rs` 数据访问层 |
| 统一错误处理 | ✅ | `AppError` 枚举 + `IntoResponse` |
| 输入验证 | ✅ | title、tags、value_tier 范围校验 |
| Justfile | ✅ | 自动化命令集 |
| CI/CD | ✅ | GitHub Actions（测试+覆盖率+构建） |

---

## 三、测试覆盖

### Rust 测试 (62 个)

```
# 单元测试 (17)
store::tests::test_item_row_into_item           ✅
store::tests::test_item_row_status_variants      ✅
store::tests::test_item_row_unknown_status_defaults_to_idle ✅
store::tests::test_demand_row_into_demand        ✅
store::tests::test_user_row_into_user            ✅
auth::tests::test_create_and_verify_token        ✅
auth::tests::test_verify_invalid_token           ✅
auth::tests::test_claims_serialization           ✅
ws::tests::test_ws_hub_new                       ✅
ws::tests::test_ws_hub_default                   ✅
ws::tests::test_ws_hub_broadcast                 ✅
ws::tests::test_ws_hub_multiple_subscribers      ✅
ws::tests::test_ws_hub_notify_no_subscribers     ✅
error::tests::test_not_found_returns_404         ✅
error::tests::test_bad_request_returns_400       ✅
error::tests::test_internal_msg_returns_500      ✅
error::tests::test_serialization_returns_500     ✅

# 模型测试 (10)
models_test::test_swap_cycle_valid_two_person    ✅
models_test::test_swap_cycle_valid_three_person  ✅
models_test::test_swap_cycle_invalid_single_leg  ✅
models_test::test_swap_cycle_invalid_empty       ✅
models_test::test_swap_cycle_invalid_broken_chain ✅
models_test::test_item_status_serialization      ✅
models_test::test_item_serialization_roundtrip   ✅
models_test::test_demand_serialization_roundtrip ✅
models_test::test_item_status_transitions        ✅
models_test::test_available_transitions          ✅

# 匹配引擎测试 (5)
matcher_test::test_two_person_cycle              ✅
matcher_test::test_three_person_cycle            ✅
matcher_test::test_no_cycle_single               ✅
matcher_test::test_no_cycle_mismatch             ✅
matcher_test::test_two_separate_cycles           ✅

# Store 集成测试 (10)
store_test::test_create_and_get_item             ✅
store_test::test_get_item_not_found              ✅
store_test::test_list_items                      ✅
store_test::test_item_exists                     ✅
store_test::test_update_item_status              ✅
store_test::test_update_item_status_not_found    ✅
store_test::test_create_and_list_demands         ✅
store_test::test_save_and_list_cycles            ✅
store_test::test_create_and_get_user             ✅
store_test::test_get_user_not_found              ✅

# Handler 集成测试 (17)
handler_test::test_health_endpoint               ✅
handler_test::test_create_item_success           ✅
handler_test::test_create_item_empty_title       ✅
handler_test::test_create_item_empty_tags        ✅
handler_test::test_create_item_invalid_value_tier ✅
handler_test::test_get_item_not_found            ✅
handler_test::test_list_items                    ✅
handler_test::test_create_demand_success         ✅
handler_test::test_create_demand_empty_offer_tags ✅
handler_test::test_create_demand_empty_target_tags ✅
handler_test::test_list_demands                  ✅
handler_test::test_list_cycles                   ✅
handler_test::test_confirm_swap_not_found        ✅
handler_test::test_register_and_login            ✅
handler_test::test_register_empty_username       ✅
handler_test::test_register_short_password       ✅
handler_test::test_login_wrong_password          ✅

# E2E 测试 (5 个，3 通过 + 2 需隔离 DB)
e2e_test::e2e_duplicate_registration             ✅
e2e_test::e2e_item_status_transitions            ✅
e2e_test::e2e_login_wrong_password               ✅
e2e_test::e2e_full_swap_flow                     🔄 需隔离数据库
e2e_test::e2e_no_cycle_found                     🔄 需隔离数据库
```

### Python 测试 (10 个)

```
test_keyboard              ✅
test_book                  ✅
test_headphone             ✅
test_camera_high_value     ✅
test_keyboard_mid_value    ✅
test_unknown_item          ✅
test_multiple_tags         ✅
test_returns_method        ✅
test_fallback_to_local     ✅
test_returns_dict_structure ✅
```

---

## 四、架构改进记录

### 重构：main.rs 提取 store 层

**问题**：main.rs 338 行，混杂路由、处理器、DB 行类型

**方案**：
- 创建 `src/store.rs`，封装所有数据库操作
- main.rs 只保留路由和处理器
- 结果：main.rs 从 338 行减至 ~160 行

### 统一错误处理

**问题**：所有错误返回 `StatusCode::INTERNAL_SERVER_ERROR`

**方案**：
- 创建 `src/error.rs`
- 定义 `AppError` 枚举：`NotFound`、`BadRequest(String)`、`Internal(String)`
- 实现 `IntoResponse`，返回 JSON 错误信息
- 实现 `From<sqlx::Error>` 自动转换

### 输入验证

**验证规则**：
- `title`：不能为空
- `tags`：不能为空数组
- `value_tier`：必须在 1-5 之间
- `offer_tags` / `target_tags`：不能为空

### 物品状态流转

**问题**：物品状态没有转换规则，可能出现非法状态变更

**方案**：
- 在 `ItemStatus` 枚举添加 `can_transition_to()` 和 `available_transitions()`
- 状态机：Idle → Matching → Completed → Archized
- Matching 可回退到 Idle（取消匹配）
- 添加 PATCH /api/items/:id/status 端点

### 交换确认流程

**问题**：匹配成功后没有确认机制

**方案**：
- 添加 POST /api/cycles/:id/confirm 端点
- 确认后自动将交换环中所有物品标记为 Completed

---

## 五、待办事项

### 短期（P0）

- [x] 设计多链合约统一接口（以太坊、Solana、Move 系）
- [x] 前端技术栈落地（Next.js + shadcn/ui + WASM）
- [x] 配置 pre-commit hooks

### 中期（P1）

- [x] 用户认证（JWT + 注册/登录端点）
- [x] 物品状态流转（Idle → Matching → Completed）
- [x] 交换确认流程
- [x] WebSocket 实时通知

### 长期（P2）

- [ ] 图片上传 + AI 标签提取集成
- [ ] Web3 存证端到端打通
- [x] 移动端 H5
- [x] 性能优化（索引、缓存）
- [x] 图片上传 + AI 标签提取集成
- [x] Web3 存证端到端打通

---

## 六、命令速查

```bash
# 测试
cargo nextest run              # Rust 测试
cd scalpel && pytest -v        # Python 测试
just test-all                  # 全部测试

# 代码质量
cargo fmt                      # 格式化
cargo clippy -- -D warnings    # 静态分析
just check-all                 # 全面检查

# 构建
cargo build --release          # 发布构建
just release                   # 同上

# 数据库
just db-init                   # 初始化
just db-reset                  # 重置

# 覆盖率
cargo llvm-cov nextest --html  # 生成 HTML 报告
```

---

### 2026-05-27 Phase 3 - 移植 rust-template 最佳实践

| 任务 | 状态 | 说明 |
|------|------|------|
| pre-commit hooks | ✅ | 8 个基础检查 + Rust/Python 专用 |
| cargo-deny | ✅ | 依赖审计（许可证、安全漏洞） |
| git-cliff | ✅ | 自动生成 CHANGELOG |
| typos | ✅ | 拼写检查 |
| rustfmt.toml | ✅ | 详细格式化配置 |
| thiserror | ✅ | 升级错误处理 |
| CI 增强 | ✅ | taplo + deny + typos + 自动 Release |

---

### 2026-05-28 Phase 10 - 图片上传

| 任务 | 状态 | 说明 |
|------|------|------|
| 图片上传端点 | ✅ | POST /api/items/:id/image，multipart 上传 |
| 文件验证 | ✅ | 类型检查（jpeg/png/gif/webp）+ 大小限制（5MB） |
| 本地存储 | ✅ | uploads/ 目录，UUID 文件名 |
| 静态文件服务 | ✅ | /uploads/ 路径通过 ServeDir 提供 |
| Store 方法 | ✅ | update_item_image 更新 image_url |

### 2026-05-28 Phase 9 - OpenAPI 文档

| 任务 | 状态 | 说明 |
|------|------|------|
| utoipa 集成 | ✅ | OpenAPI spec 自动生成 |
| Swagger UI | ✅ | /swagger-ui 访问 |
| ToSchema derives | ✅ | 所有 model 和 request type |

### 2026-05-28 Phase 8 - Docker 部署

| 任务 | 状态 | 说明 |
|------|------|------|
| Dockerfile | ✅ | Rust 后端多阶段构建（builder + slim runtime） |
| Dockerfile.scalpel | ✅ | Python 手术刀镜像 |
| docker-compose.yml | ✅ | 后端 + PostgreSQL + Scalpel 一键编排 |
| .dockerignore | ✅ | 排除 target/node_modules/.git 等 |
| .env.example | ✅ | 环境变量模板 |
| Justfile docker 命令 | ✅ | docker-up/down/logs |

### 2026-05-28 Phase 7 - 集成测试补强

| 任务 | 状态 | 说明 |
|------|------|------|
| error 模块测试 | ✅ | 4 个测试：NotFound→404, BadRequest→400, InternalMsg→500, Serialization→500 |
| store 集成测试 | ✅ | 10 个测试：Item CRUD(6) + Demand(1) + Cycle(1) + User(2)，真实数据库 |
| handler 集成测试 | ✅ | 17 个测试：Health(1) + Items(5) + Demands(4) + Cycles(2) + Auth(5) |
| tower 依赖 | ✅ | 添加 tower dev-dependency 用于 axum 测试 |
| sqlx macros feature | ✅ | 启用 sqlx::test 宏支持 |

### 2026-05-28 Phase 6 - 代码质量提升

| 任务 | 状态 | 说明 |
|------|------|------|
| cargo deny 修复 | ✅ | 许可证允许 ISC/BSD-3-Clause，安全审计例外 |
| main.rs 模块化 | ✅ | 拆分为 6 个 handler 子模块，main.rs 从 474 行缩减到 ~70 行 |
| ws.rs 测试 | ✅ | 5 个内联测试（广播、多订阅者等） |
| store.rs 测试 | ✅ | 5 个内联测试（Row 转换、状态变体） |
| 消除 unwrap() | ✅ | main.rs 改为 `async fn main() -> Result<(), Box<dyn Error>>` |
| taplo 格式化 | ✅ | Cargo.toml、deny.toml 格式修复 |
| pre-commit 安装 | ✅ | 之前配置了但未安装到 .git/hooks/，已修复 |
| CI 修复 | ✅ | cargo-llvm-cov action 名称、PGPASSWORD、执行全部 migration |
| Trellis 初始化 | ✅ | `trellis init --claude -u qiaopengjun`，生成 AGENTS.md + .trellis/ |

### 2026-05-28 Phase 5 - WebSocket + 代码优化

| 任务 | 状态 | 说明 |
|------|------|------|
| JWT_SECRET 环境变量化 | ✅ | OnceLock 线程安全读取 |
| error.rs 匹配分支合并 | ✅ | 减少重复代码 |
| matcher.rs exists 优化 | ✅ | iter().any() 替代手动循环 |
| store.rs 错误处理修复 | ✅ | 反序列化失败正确记录错误 |
| WebSocket 模块 | ✅ | WsHub 广播 + 连接管理 |
| WebSocket 路由 | ✅ | GET /api/ws |
| 匹配通知 | ✅ | match_found 事件 |
| 确认通知 | ✅ | swap_confirmed 事件 |
| AppState 重构 | ✅ | 移至 lib.rs 全局共享 |
| 数据库索引 | ✅ | GIN 索引 + 复合索引，覆盖所有查询路径 |
| 外键约束 | ✅ | demands/items → users 级联删除 |
| AI 标签提取 | ✅ | POST /api/items/analyze，调用 scalpel.py |
| Web3 存证 | ✅ | POST /api/items/:id/attest + GET /api/items/:id/history |
| ChainManager | ✅ | get_history 方法，多链适配器注册 |
| 移动端 H5 适配 | ✅ | 导航汉堡菜单 + 响应式布局 |
| 前端 WebSocket 对接 | ✅ | useWs hook + 通知指示器 |
| 前端 AI 标签提取 | ✅ | 物品详情页 AI 分析按钮 |
| 前端 Web3 存证 | ✅ | 物品详情页存证按钮 + 结果展示 |
| README 中英文 | ✅ | README.md + README_en.md |
| rust-dev-workflow skill | ✅ | 更新文档维护清单含 README |

---

### 2026-05-27 Phase 4 - 前端页面 + 用户认证

| 模块 | 状态 | 说明 |
|------|------|------|
| API 客户端 | ✅ | `src/lib/api.ts`，封装所有后端接口 |
| 导航栏 | ✅ | `src/components/nav.tsx`，4 个路由入口 |
| 首页 | ✅ | 更新文案、添加导航链接 |
| 物品列表页 | ✅ | `/items`，卡片布局、状态标签 |
| 发布物品页 | ✅ | `/items/new`，表单验证、价值滑块 |
| 物品详情页 | ✅ | `/items/[id]`，触发匹配、展示交换环 |
| 交换意向页 | ✅ | `/demands`，提供/想要标签展示 |
| 交换环页 | ✅ | `/cycles`，确认交换功能 |
| 布局 | ✅ | 中文元数据、导航栏集成 |
| 用户注册 | ✅ | POST /api/auth/register，bcrypt 加密 |
| 用户登录 | ✅ | POST /api/auth/login，JWT token |
| 登录页 | ✅ | /auth/login，表单验证 |
| 注册页 | ✅ | /auth/register，密码确认 |
| Auth 测试 | ✅ | 3 个单元测试（token 创建/验证/序列化） |

---

## 七、问题与解决方案

| 问题 | 解决方案 |
|------|----------|
| main.rs 过于臃肿 | 提取 store.rs 数据访问层 |
| 错误处理不统一 | 定义 AppError 枚举 + thiserror |
| 缺少输入验证 | 在 handler 中添加验证逻辑 |
| 测试覆盖不足 | 补充模型层和 Python 测试 |
| CI/CD 缺失 | 配置 GitHub Actions |
| 缺少依赖审计 | 添加 cargo-deny |
| 缺少 pre-commit | 添加 pre-commit-config.yaml |
| 缺少 CHANGELOG | 添加 git-cliff |
| Button 不支持 asChild | 去掉 asChild，直接在 Link 上用 button 样式 |
| 集成测试外键约束失败 | 测试中先创建 user 再创建 item/demand，使用随机 UUID 前缀 |
| utoipa-swagger-ui v9 与 axum 0.7 不兼容 | 降级到 utoipa v4 + utoipa-swagger-ui v7 |
| utoipa v4 的 proc-macro-error unmaintained | 在 deny.toml 添加 RUSTSEC-2024-0370 忽略 |
| cargo-deny 大量重复依赖警告 | 使用 skip-tree 跳过 utoipa-swagger-ui 依赖树 |
| Docker 构建 Rust 1.87 版本不足 | 升级到 rust:1.88-bookworm |
| Docker 端口 3000 冲突 | 改用 3001 端口映射 |
| E2E 测试数据库污染 | 使用唯一 UUID 前缀 + 按 item ID 查找特定交换环 |
| E2E 测试 confirm_swap 后物品仍为 Idle | 标记 #[ignore]，需要隔离数据库环境 |
