use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 生产环境 JWT_SECRET 的最小长度要求（字节）
const MIN_SECRET_LEN: usize = 32;

/// 仅用于非生产环境的内置默认密钥（生产必须显式设置 JWT_SECRET）
const DEV_FALLBACK_SECRET: &str = "mutao-secret-change-in-production";

/// JWT 密钥，优先从环境变量读取，非生产环境兜底用内置值。
///
/// 生产环境下 `ensure_secret_for_env` 已在启动时保证 `JWT_SECRET` 存在且足够强，
/// 因此此处的兜底分支只会在 dev/test 命中。
fn jwt_secret() -> &'static [u8] {
    static SECRET: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    SECRET
        .get_or_init(|| {
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| DEV_FALLBACK_SECRET.into())
                .into_bytes()
        })
        .as_slice()
}

/// 启动时校验密钥配置：生产环境必须显式设置足够强的 `JWT_SECRET`，
/// 否则拒绝启动；非生产环境缺失时给出告警并使用内置默认值。
///
/// 由 `APP_ENV=production` 触发严格模式。
pub fn ensure_secret_for_env() -> Result<(), String> {
    let is_production = std::env::var("APP_ENV")
        .map(|v| v.eq_ignore_ascii_case("production"))
        .unwrap_or(false);

    match std::env::var("JWT_SECRET") {
        Ok(secret) => {
            if is_production && secret.len() < MIN_SECRET_LEN {
                return Err(format!(
                    "生产环境 JWT_SECRET 太弱：要求至少 {MIN_SECRET_LEN} 字节，当前仅 {} 字节",
                    secret.len()
                ));
            }
        }
        Err(_) => {
            if is_production {
                return Err(
                    "生产环境（APP_ENV=production）必须设置 JWT_SECRET，拒绝以内置弱密钥启动"
                        .to_string(),
                );
            }
            tracing::warn!(
                "JWT_SECRET 未设置，回退到内置开发默认密钥——仅供本地/测试使用，切勿用于生产"
            );
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: usize,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginRes {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
}

pub fn create_token(user_id: Uuid, username: &str) -> Result<String, StatusCode> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn verify_token(token: &str) -> Result<Claims, StatusCode> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| StatusCode::UNAUTHORIZED)
}

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());

    let token = auth_header
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_token(token)?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_token() {
        let user_id = Uuid::new_v4();
        let username = "testuser";
        let token = create_token(user_id, username).unwrap();

        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
    }

    #[test]
    fn test_verify_invalid_token() {
        let result = verify_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_serialization() {
        let user_id = Uuid::new_v4();
        let token = create_token(user_id, "alice").unwrap();
        let claims = verify_token(&token).unwrap();

        // token 应该包含正确的用户信息
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, "alice");
        // exp 应该在未来
        assert!(claims.exp > chrono::Utc::now().timestamp() as usize);
    }

    // 下列测试改写进程级环境变量；依赖 cargo-nextest 的「每测试独立进程」隔离，
    // 不要用 `cargo test`（共享进程会相互污染 env）。
    #[test]
    fn test_ensure_secret_production_missing_fails() {
        std::env::set_var("APP_ENV", "production");
        std::env::remove_var("JWT_SECRET");
        assert!(ensure_secret_for_env().is_err());
    }

    #[test]
    fn test_ensure_secret_production_short_fails() {
        std::env::set_var("APP_ENV", "production");
        std::env::set_var("JWT_SECRET", "too-short");
        assert!(ensure_secret_for_env().is_err());
    }

    #[test]
    fn test_ensure_secret_production_strong_ok() {
        std::env::set_var("APP_ENV", "production");
        std::env::set_var(
            "JWT_SECRET",
            "a-sufficiently-long-production-secret-key-0123456789",
        );
        assert!(ensure_secret_for_env().is_ok());
    }

    #[test]
    fn test_ensure_secret_dev_missing_ok() {
        std::env::set_var("APP_ENV", "development");
        std::env::remove_var("JWT_SECRET");
        assert!(ensure_secret_for_env().is_ok());
    }
}
