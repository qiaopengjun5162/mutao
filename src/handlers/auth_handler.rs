use axum::{extract::State, response::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth;
use crate::error::AppError;
use crate::models::User;

use super::SharedState;

#[derive(Deserialize)]
pub struct RegisterReq {
    pub username: String,
    pub password: String,
}

pub async fn register(
    State(s): State<SharedState>,
    Json(req): Json<RegisterReq>,
) -> Result<Json<auth::LoginRes>, AppError> {
    if req.username.trim().is_empty() {
        return Err(AppError::BadRequest("用户名不能为空".into()));
    }
    if req.password.len() < 6 {
        return Err(AppError::BadRequest("密码长度至少 6 位".into()));
    }

    if s.store.get_user_by_username(&req.username).await?.is_some() {
        return Err(AppError::BadRequest("用户名已存在".into()));
    }

    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::InternalMsg(format!("密码加密失败: {e}")))?;

    let user = User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        password_hash,
        created_at: chrono::Utc::now(),
    };

    s.store.create_user(&user).await?;

    let token = auth::create_token(user.id, &user.username)
        .map_err(|_| AppError::InternalMsg("生成 token 失败".into()))?;

    tracing::info!("用户 {} 注册成功", user.username);

    Ok(Json(auth::LoginRes {
        token,
        user_id: user.id,
        username: user.username,
    }))
}

pub async fn login(
    State(s): State<SharedState>,
    Json(req): Json<auth::LoginReq>,
) -> Result<Json<auth::LoginRes>, AppError> {
    let user = s
        .store
        .get_user_by_username(&req.username)
        .await?
        .ok_or(AppError::BadRequest("用户名或密码错误".into()))?;

    let valid = bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|e| AppError::InternalMsg(format!("密码验证失败: {e}")))?;

    if !valid {
        return Err(AppError::BadRequest("用户名或密码错误".into()));
    }

    let token = auth::create_token(user.id, &user.username)
        .map_err(|_| AppError::InternalMsg("生成 token 失败".into()))?;

    tracing::info!("用户 {} 登录成功", user.username);

    Ok(Json(auth::LoginRes {
        token,
        user_id: user.id,
        username: user.username,
    }))
}
