use axum::{extract::State, response::Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::blockchain::{ChainType, SwapProof};
use crate::error::AppError;

use super::SharedState;

#[derive(Deserialize)]
pub struct AttestReq {
    pub from_user: String,
    pub to_user: String,
    pub item_name: String,
    pub message: Option<String>,
}

pub async fn attest_item(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<AttestReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let item = s.store.get_item(id).await?.ok_or(AppError::NotFound)?;

    let proof = SwapProof {
        item_id: id.to_string(),
        from_user: req.from_user,
        to_user: req.to_user,
        item_name: req.item_name,
        message: req.message.unwrap_or_default(),
        swap_count: 0,
    };

    let record = s
        .chain_manager
        .record_swap(&ChainType::Ethereum, &proof)
        .await
        .map_err(|e| AppError::InternalMsg(format!("链上存证失败: {e}")))?;

    tracing::info!("物品 {} 存证成功, tx={}", item.title, record.tx_hash);

    Ok(Json(serde_json::json!({
        "item_id": id,
        "chain": record.chain,
        "tx_hash": record.tx_hash,
        "status": record.status,
    })))
}

pub async fn item_history(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !s.store.item_exists(id).await? {
        return Err(AppError::NotFound);
    }

    let history = s
        .chain_manager
        .get_history(&ChainType::Ethereum, &id.to_string())
        .await
        .map_err(|e| AppError::InternalMsg(format!("查询链上历史失败: {e}")))?;

    Ok(Json(serde_json::json!({
        "item_id": id,
        "chain": "Ethereum",
        "records": history,
    })))
}
