# 木桃 Mutao - 项目进度记录

> 更新时间：2026-05-27

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

### Rust 测试 (13 个)

```
matcher_test::test_two_person_cycle      ✅
matcher_test::test_three_person_cycle    ✅
matcher_test::test_no_cycle_single       ✅
matcher_test::test_no_cycle_mismatch     ✅
matcher_test::test_two_separate_cycles   ✅
models_test::test_swap_cycle_valid_two_person    ✅
models_test::test_swap_cycle_valid_three_person  ✅
models_test::test_swap_cycle_invalid_single_leg  ✅
models_test::test_swap_cycle_invalid_empty       ✅
models_test::test_swap_cycle_invalid_broken_chain ✅
models_test::test_item_status_serialization      ✅
models_test::test_item_serialization_roundtrip   ✅
models_test::test_demand_serialization_roundtrip ✅
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

---

## 五、待办事项

### 短期（P0）

- [ ] 设计多链合约统一接口（以太坊、Solana、Move 系）
- [ ] 前端技术栈落地（Next.js + shadcn/ui + WASM）
- [ ] 配置 pre-commit hooks

### 中期（P1）

- [ ] 用户认证（JWT）
- [ ] 物品状态流转（Idle → Matching → Completed）
- [ ] 交换确认流程
- [ ] WebSocket 实时通知

### 长期（P2）

- [ ] 图片上传 + AI 标签提取集成
- [ ] Web3 存证端到端打通
- [ ] 移动端 H5
- [ ] 性能优化（索引、缓存）

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

## 七、问题与解决方案

| 问题 | 解决方案 |
|------|----------|
| main.rs 过于臃肿 | 提取 store.rs 数据访问层 |
| 错误处理不统一 | 定义 AppError 枚举 |
| 缺少输入验证 | 在 handler 中添加验证逻辑 |
| 测试覆盖不足 | 补充模型层和 Python 测试 |
| CI/CD 缺失 | 配置 GitHub Actions |
