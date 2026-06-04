# Fix P0 auth — mount JWT middleware on protected routes

## Goal

修复两个 P0 安全缺陷:

1. **鉴权中间件从未挂载**:`auth::auth_middleware` 已实现,但 `main.rs` 没有应用到任何路由,导致所有写操作(创建/改状态/确认交换/上链)无需登录即可调用。
2. **弱密钥静默兜底**:`jwt_secret()` 在 `JWT_SECRET` 缺失时回退到硬编码 `"mutao-secret-change-in-production"`,生产环境签发的 token 可被任意伪造。

并顺带收紧越权:写操作的 owner 一律以 token 中的 `Claims.sub` 为准,而非客户端传值。

## Requirements

### R1 路由保护矩阵(选项 C:公开读 + 保护写 + WS 同步保护)

公开(匿名可访问):
- `GET /api/health`、`POST /api/auth/register`、`POST /api/auth/login`
- `GET /api/items`、`GET /api/items/:id`、`GET /api/demands`、`GET /api/cycles`、`GET /api/items/:id/history`
- `/swagger-ui`、`/api-docs/openapi.json`、`/uploads/*`

受保护(需有效 JWT,缺失/无效 → 401):
- `POST /api/items`、`POST /api/items/analyze`、`POST /api/items/:id/match`
- `POST /api/items/:id/image`、`PATCH /api/items/:id/status`
- `POST /api/items/:id/attest`、`POST /api/cycles/:id/confirm`、`POST /api/demands`

WebSocket:
- `GET /api/ws` 通过 query 参数 `?token=` 鉴权(浏览器无法为 WS 设自定义 header),无效/缺失 → 拒绝升级(401)。

### R2 弱密钥修复(选项:分环境校验)

- 引入 `APP_ENV` 环境变量。
- `APP_ENV=production`:启动时强校验 `JWT_SECRET` 必须存在且长度 ≥ 32 字节,否则应用拒绝启动(`main` 返回 `Err`)。
- 非生产(dev/test/未设):`JWT_SECRET` 缺失则 `tracing::warn!` 一次并使用内置 dev 默认值,保证本地与测试无需配置。
- 移除「静默」行为:dev 回退必须有显式 warning。

### R3 Owner 越权防护(选项 B:强制 owner 归属)

- `CreateItemReq` 移除 `owner_id`、`CreateDemandReq` 移除 `user_id`;owner 取 `Claims.sub`。
- `update_item_status`、`match_item`、`upload_image`、`attest_item`:加载目标物品,`item.owner_id != Claims.sub` → 403。
- `confirm_swap`:调用者必须是该交换环参与者之一(`cycle.swaps` 中存在 `leg.from_user_id == Claims.sub`),否则 403。
- 新增 `AppError::Forbidden(String)` → HTTP 403。

### R4 前端同步改造(选项 A:后端 + 前端一起)

- `api.ts` `request()` 与 `uploadImage()`:从 `localStorage["token"]` 读取并附加 `Authorization: Bearer <token>`。
- `connectWs()`:WS URL 追加 `?token=<token>`。
- `CreateItemReq` 去 `owner_id`、`CreateDemandReq` 去 `user_id`;对应发布表单(`/items/new`、demands 创建)移除这两个输入。

## Acceptance Criteria

- [ ] 无 token 调用任一受保护端点 → 401
- [ ] 有效 token 调用受保护端点 → 成功,且落库 owner = token 用户
- [ ] 在 `update_item_status`/`match`/`upload`/`attest` 操作非己物品 → 403
- [ ] 非参与者 `confirm_swap` → 403;参与者 → 成功
- [ ] 公开端点(health/register/login、GET items/demands/cycles/history、swagger)无 token 可访问
- [ ] WS 无/错 `?token=` → 拒绝(不升级);有效 → 正常推送
- [ ] `APP_ENV=production` 且 `JWT_SECRET` 缺失或 < 32 字节 → 应用拒绝启动;dev 缺失 → warn + 默认值
- [ ] `CreateItemReq`/`CreateDemandReq` 的 schema 不再包含 owner_id/user_id
- [ ] 前端登录后创建物品/意向、WS 连接端到端可用
- [ ] `cargo nextest run` 全绿;`cargo clippy -- -D warnings` 无告警;`pnpm build`(frontend)通过

## Definition of Done

- 新增/更新测试:受保护端点 401、越权 403、参与者校验、公开端点放行、密钥配置分环境行为
- `handler_test.rs`/`e2e_test.rs` 改为先 register+login 取 token 再带 `Authorization` 调用
- lint / clippy / 前端 build / CI 绿
- README 与 CLAUDE.md 的 API 表标注鉴权要求;`.env.example` 增加 `APP_ENV`、注明 `JWT_SECRET` 生产必填且 ≥ 32 字节

## Technical Approach

- **路由分组**:拆分 `public` 与 `protected` 两个 `Router`,`protected` 套 `.route_layer(axum::middleware::from_fn(auth::auth_middleware))`,再 `public.merge(protected).with_state(state)`。`/api/items` 的 GET(public)与 POST(protected)分别定义在两个 router、同路径不同 method,axum 合并即可。
- **取用户身份**:`auth_middleware` 已 `req.extensions_mut().insert(claims)`;受保护 handler 用 `axum::Extension<Claims>` 提取(`Claims` 已 derive `Clone`)。
- **WS 鉴权**:`ws_handler` 增加 `Query<WsAuthQuery { token: String }>`,`auth::verify_token` 失败则返回 `StatusCode::UNAUTHORIZED` 不升级。
- **密钥配置**:`main` 启动时调用 `auth::ensure_secret_for_env()`(读取 `APP_ENV` + `JWT_SECRET`,生产不合规返回 `Err`);`jwt_secret()` 保留 dev 默认 + 首次 warn。
- **测试注入**:Rust 测试默认非生产 → 用 dev 默认 secret,无需设环境变量;新增配置校验单测可用临时设置 env 的方式覆盖生产分支。

## Decision (ADR-lite)

- **Context**:鉴权已写好但未启用,且密钥兜底削弱了 JWT 安全性;P0 需立即堵住。
- **Decision**:公开读/保护写 + WS query token(R1);分环境密钥校验(R2);强制 owner 归属、移除客户端 owner 字段、confirm 参与者校验(R3);后端前端同批改造(R4)。
- **Consequences**:API 契约变化(owner_id/user_id 移除、写操作需 token)→ 前端与 HTTP 层测试需同步改;换来无伪造空间的鉴权与防越权。RBAC、token 刷新/撤销留后续。

## Out of Scope

- 刷新 token / token 撤销 / 黑名单
- 角色权限(RBAC)、细粒度资源策略
- WS 按用户定向推送(仍为广播)
- `attest` 的 from_user/to_user 由链上数据推导(保持现有字符串入参)

## Technical Notes

- 受影响后端:`src/main.rs`、`src/auth.rs`、`src/error.rs`、`src/ws.rs`、`src/handlers/{items,demands,cycles,upload,blockchain_handler}.rs`。
- 受影响测试:`tests/handler_test.rs`、`tests/e2e_test.rs`(HTTP 层);`store_test.rs`/`models_test.rs`/`matcher_test.rs` 不受影响(直接构造 struct)。
- 受影响前端:`frontend/src/lib/api.ts`、`frontend/src/hooks/use-auth.tsx`(token 已存 localStorage["token"])、`/items/new` 与 demands 创建表单。
- 配置:`.env.example`、`docker-compose.yml` 增加 `APP_ENV` / `JWT_SECRET`。
- 后端 spec(`.trellis/spec/backend/*`)目前为空模板,实际约定以本 PRD 为准。

## Research References

(无需外部调研;axum 0.7 中间件分组为标准用法)
