use axum::{Extension, extract::State, response::Json};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::Claims;
use crate::error::AppError;
use crate::models::{Item, ItemStatus};

use super::SharedState;

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status":"ok","name":"木桃 Mutao"}))
}

#[derive(Deserialize, ToSchema)]
pub struct CreateItemReq {
    pub title: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub tags: Vec<String>,
    pub value_tier: u8,
}

pub async fn create_item(
    State(s): State<SharedState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateItemReq>,
) -> Result<Json<Item>, AppError> {
    if req.title.trim().is_empty() {
        return Err(AppError::BadRequest("title 不能为空".into()));
    }
    if req.tags.is_empty() {
        return Err(AppError::BadRequest("tags 不能为空".into()));
    }
    if req.value_tier == 0 || req.value_tier > 5 {
        return Err(AppError::BadRequest("value_tier 必须在 1-5 之间".into()));
    }

    let item = Item {
        id: Uuid::new_v4(),
        owner_id: claims.sub,
        title: req.title,
        description: req.description.unwrap_or_default(),
        image_url: req.image_url.unwrap_or_default(),
        tags: req.tags,
        value_tier: req.value_tier,
        status: ItemStatus::Idle,
        created_at: chrono::Utc::now(),
    };

    s.store.create_item(&item).await?;

    Ok(Json(item))
}

pub async fn get_item(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Item>, AppError> {
    s.store.get_item(id).await?.map(Json).ok_or(AppError::NotFound)
}

pub async fn list_items(State(s): State<SharedState>) -> Result<Json<Vec<Item>>, AppError> {
    Ok(Json(s.store.list_items().await?))
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateStatusReq {
    pub status: ItemStatus,
}

pub async fn update_item_status(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateStatusReq>,
) -> Result<Json<Item>, AppError> {
    let item = s.store.get_item(id).await?.ok_or(AppError::NotFound)?;

    if item.owner_id != claims.sub {
        return Err(AppError::Forbidden("无权修改他人物品".into()));
    }

    if !item.status.can_transition_to(&req.status) {
        return Err(AppError::BadRequest(format!(
            "不能从 {:?} 转换到 {:?}",
            item.status, req.status
        )));
    }

    s.store.update_item_status(id, &req.status).await?;

    let mut updated = item;
    updated.status = req.status;
    Ok(Json(updated))
}
