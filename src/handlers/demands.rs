use axum::{extract::State, response::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Demand;

use super::SharedState;

#[derive(Deserialize)]
pub struct CreateDemandReq {
    pub user_id: Uuid,
    pub offer_item_id: Uuid,
    pub offer_tags: Vec<String>,
    pub target_tags: Vec<String>,
}

pub async fn create_demand(
    State(s): State<SharedState>,
    Json(req): Json<CreateDemandReq>,
) -> Result<Json<Demand>, AppError> {
    if req.offer_tags.is_empty() {
        return Err(AppError::BadRequest("offer_tags 不能为空".into()));
    }
    if req.target_tags.is_empty() {
        return Err(AppError::BadRequest("target_tags 不能为空".into()));
    }

    let demand = Demand {
        id: Uuid::new_v4(),
        user_id: req.user_id,
        offer_item_id: req.offer_item_id,
        offer_tags: req.offer_tags,
        target_tags: req.target_tags,
        created_at: chrono::Utc::now(),
    };

    s.store.create_demand(&demand).await?;

    Ok(Json(demand))
}

pub async fn list_demands(State(s): State<SharedState>) -> Result<Json<Vec<Demand>>, AppError> {
    Ok(Json(s.store.list_demands().await?))
}
