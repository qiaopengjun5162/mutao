use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT 密钥，优先从环境变量读取，兜底用内置值（生产环境必须设置 JWT_SECRET）
fn jwt_secret() -> &'static [u8] {
    static SECRET: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    SECRET
        .get_or_init(|| {
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "mutao-secret-change-in-production".into())
                .into_bytes()
        })
        .as_slice()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
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
}
