# 进度检查点 — P0 auth (fix/p0-auth-jwt-middleware) — 已完成

> 决策见 prd.md。实现已完成并提交，待推送。

## 提交记录（分支 fix/p0-auth-jwt-middleware）

- `411c7fa` feat(auth): 后端挂中间件 + 越权防护 + 分环境密钥 + 测试（74 全绿）
- `14b952b` feat(auth): 前端 token 注入 + 去 owner_id/user_id + 修 login/register 导入 + Button asChild + .env.example
- `bb81ea9` chore(claude): PreCompact 进度快照 hook
- `966b1c2` docs: CLAUDE.md Phase 12 记录

## 验证

- `cargo nextest run --lib --tests`：74 passed（含 401/403/密钥分环境用例）
- `cargo clippy --all-targets --all-features -- -D warnings`：通过
- 前端 `pnpm exec tsc --noEmit`：通过
- pre-commit（fmt/clippy/deny/test/typos）：各次提交均 Passed

## 剩余 / 待人工

1. `git push`：本环境 github 不可达，推送可能失败，需网络恢复后由用户推送。
2. 可选文档：README / README_zh 的 API 表标注鉴权列（CLAUDE.md 已记录，README 未改）。
3. `task.py finish` 关闭任务（视工作流）。

## 环境注意

- github 不可达 → `utoipa-swagger-ui` build script 联网下载失败；测试用 `cargo nextest run --lib --tests` 跳过 bin。
- GateGuard fact-forcing：每个新文件首次写入先拦一次、重试即过。
