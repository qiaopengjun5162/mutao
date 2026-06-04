# Authentication & Authorization

> JWT bearer auth, route protection matrix, ownership enforcement, and
> environment-gated secret validation. Established by the P0 auth hardening
> task (2026-06-04). Code lives in `src/auth.rs`, wired in `src/main.rs`.

---

## 1. Scope / Trigger

Apply this contract when changing any of: route registration in `src/main.rs`,
the auth layer `src/auth.rs`, any protected handler under `src/handlers/`,
WebSocket auth in `src/ws.rs`, or the `JWT_SECRET` / `APP_ENV` env wiring.

This is infra + cross-layer: the HTTP contract, the frontend client, and the
test suite all depend on it.

## 2. Signatures

```rust
// src/auth.rs
pub fn ensure_secret_for_env() -> Result<(), String>;            // startup gate, call once in main()
pub fn create_token(user_id: Uuid, username: &str) -> Result<String, StatusCode>;
pub fn verify_token(token: &str) -> Result<Claims, StatusCode>;  // Err(401) on ANY failure
pub async fn auth_middleware(req: Request, next: Next) -> Result<Response, StatusCode>;

pub struct Claims { pub sub: Uuid, pub username: String, pub exp: usize }
```

Protected handlers read the verified identity via `Extension<Claims>` — the
middleware inserts it with `req.extensions_mut().insert(claims)`:

```rust
pub async fn create_item(
    State(s): State<SharedState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateItemReq>,
) -> Result<Json<Item>, AppError> { /* owner_id = claims.sub */ }
```

## 3. Contracts

### Route protection matrix

`main()` builds two routers and merges them. `protected` carries the middleware
via `.route_layer(middleware::from_fn(auth::auth_middleware))`.

Public (anonymous):
- `GET /api/health`, `POST /api/auth/register`, `POST /api/auth/login`
- `GET /api/items`, `GET /api/items/:id`, `GET /api/items/:id/history`,
  `GET /api/demands`, `GET /api/cycles`
- `/swagger-ui`, `/api-docs/openapi.json`, `/uploads/*`
- `GET /api/ws` (self-authenticates inside the handler — see below)

Protected (valid Bearer JWT, else 401):
- `POST /api/items`, `POST /api/items/analyze`, `POST /api/items/:id/match`
- `POST /api/items/:id/image`, `PATCH /api/items/:id/status`
- `POST /api/items/:id/attest`, `POST /api/cycles/:id/confirm`, `POST /api/demands`

> Same path / different method splits across the two routers (e.g. `GET /api/items`
> is public, `POST /api/items` is protected). Register each method on its own
> router; axum merges them.

### Ownership: never trust a client-supplied owner

- `CreateItemReq` / `CreateDemandReq` MUST NOT contain `owner_id` / `user_id`.
  Owner is always `claims.sub`.
- Mutating handlers (`update_item_status`, `match_item`, `upload_image`,
  `attest_item`) load the target first and reject `item.owner_id != claims.sub`
  with `AppError::Forbidden`.
- `confirm_swap` requires the caller to be a participant: some
  `leg.from_user_id == claims.sub` in `cycle.swaps`, else `Forbidden`.

### WebSocket auth (browsers can't set WS headers)

```rust
pub struct WsAuthQuery { pub token: Option<String> }
// authorize BEFORE upgrade; on failure return 401 and do NOT upgrade
let authorized = query.token.as_deref().is_some_and(|t| verify_token(t).is_ok());
if !authorized { return StatusCode::UNAUTHORIZED.into_response(); }
```

Client connects with `/api/ws?token=<jwt>`.

### Environment keys

| Key | Required | Behavior |
|-----|----------|----------|
| `APP_ENV` | optional | value `production` (case-insensitive) enables strict secret validation |
| `JWT_SECRET` | required in prod | prod: must exist AND `len >= 32` bytes, else `main()` returns `Err` (refuse start). Non-prod missing → `tracing::warn!` once + built-in dev fallback |

`ensure_secret_for_env()` runs first in `main()`, before the DB connect.
`MIN_SECRET_LEN = 32`. Dev fallback secret is for local/test ONLY.

## 4. Validation & Error Matrix

| Condition | Result |
|-----------|--------|
| Protected route, missing/malformed `Authorization: Bearer` | 401 (middleware) |
| Protected route, invalid/expired token | 401 (`verify_token` → `Err`) |
| Mutating own resource, valid token | 200 |
| Mutating another user's item | 403 `AppError::Forbidden` |
| `confirm_swap` by non-participant | 403 |
| WS without / with bad `?token=` | 401, no upgrade |
| `APP_ENV=production` + `JWT_SECRET` missing or `< 32` bytes | startup `Err`, process exits |
| Non-prod + `JWT_SECRET` missing | warn + dev fallback, starts |

## 5. Good/Base/Bad Cases

- Good: `POST /api/items` with valid token, body has no `owner_id` → item
  persisted with `owner_id = claims.sub`.
- Base: `GET /api/items` anonymous → 200 list.
- Bad: `POST /api/items` no token → 401; `PATCH /api/items/:id/status` on
  another user's item → 403.

## 6. Tests Required

- 401: any protected endpoint without a token.
- 403: `test_update_status_forbidden_for_non_owner`,
  `e2e_confirm_swap_forbidden_for_outsider`.
- Public passthrough: anonymous GET items/demands/cycles + register/login.
- Secret gating: `test_ensure_secret_production_missing_fails`,
  `_short_fails`, `_strong_ok`, `_dev_missing_ok`. These mutate process env →
  rely on `cargo nextest`'s per-test process isolation; do NOT run under
  `cargo test` (shared process pollutes env).
- HTTP-layer tests must register+login to obtain a token, then call with
  `Authorization: Bearer <token>`.

## 7. Wrong vs Correct

### Wrong
```rust
// Trusting a client-supplied owner — forgeable, lets the caller impersonate anyone
let item = Item { owner_id: req.owner_id, /* ... */ };
```

### Correct
```rust
// Owner derived only from the verified token
let item = Item { owner_id: claims.sub, /* ... */ };
```

---

## Out of Scope (deferred)

Refresh tokens, token revocation/blacklist, RBAC / fine-grained resource
policies, per-user WS targeting (still broadcast).
