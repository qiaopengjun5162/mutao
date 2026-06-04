# Error Handling

> Every fallible handler returns `Result<T, AppError>`. `AppError` implements
> `IntoResponse`, mapping each variant to an HTTP status plus a uniform JSON
> body `{"error": "<message>"}`. Defined in `src/error.rs`.

---

## Error Types

```rust
pub enum AppError {
    NotFound,                                   // 404, fixed message "资源不存在"
    BadRequest(String),                         // 400, message = the string (client-safe)
    Forbidden(String),                          // 403, message = the string (client-safe)
    Internal(#[from] sqlx::Error),              // 500
    InternalMsg(String),                        // 500
    Serialization(#[from] serde_json::Error),   // 500
}
```

## Status Mapping

| Variant | HTTP | Body `error` |
|---------|------|--------------|
| `NotFound` | 404 | `资源不存在` |
| `BadRequest(msg)` | 400 | `msg` |
| `Forbidden(msg)` | 403 | `msg` |
| `Internal` / `InternalMsg` / `Serialization` | 500 | `服务器内部错误` (real cause logged via `tracing::error!`, never leaked) |

## Error Handling Patterns

- `?` auto-converts `sqlx::Error` → `Internal` and `serde_json::Error` →
  `Serialization` (both 500) via `#[from]`. Just propagate with `?`.
- Validation failures → `BadRequest`. The message IS returned to the client
  verbatim — keep it user-safe.
- Authorization failures → `Forbidden`. Ownership / participant checks return
  it; see [auth-guidelines.md](./auth-guidelines.md).
- 5xx variants never expose internals: the client gets only the status + a
  generic message; the real error goes to logs.

## Common Mistakes

- **Don't** put secrets / SQL / internal detail in `BadRequest` or `Forbidden`
  strings — they reach the client unmodified.
- **Don't** add a 401 variant to `AppError`. Auth 401 is produced by the
  middleware and `verify_token` returning `StatusCode::UNAUTHORIZED`, not
  through `AppError`.

## Tests Required

Each variant's status + body is asserted in the `src/error.rs` unit tests
(`test_not_found_returns_404`, `test_bad_request_returns_400`,
`test_forbidden_returns_403`, `test_internal_msg_returns_500`,
`test_serialization_returns_500`). A new variant MUST add a matching test.
