use axum::{Extension, extract::State, response::Json};
use uuid::Uuid;

use crate::auth::Claims;
use crate::error::AppError;
use crate::matcher;
use crate::models::{ItemStatus, SwapCycle};
use crate::ws::WsNotification;

use super::SharedState;

pub async fn match_item(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<SwapCycle>>, AppError> {
    let item = s.store.get_item(id).await?.ok_or(AppError::NotFound)?;

    if item.owner_id != claims.sub {
        return Err(AppError::Forbidden("无权为他人物品发起匹配".into()));
    }

    let demands = s.store.list_demands().await?;

    if demands.len() < 2 {
        return Ok(Json(vec![]));
    }

    let cycles = matcher::Matcher::find_cycles(&demands, 4);

    for cycle in &cycles {
        s.store.save_cycle(cycle).await.ok();
    }

    if !cycles.is_empty() {
        s.ws_hub.notify(&WsNotification {
            event: "match_found".into(),
            data: serde_json::json!({
                "item_id": id,
                "cycles_count": cycles.len(),
            }),
        });
    }

    Ok(Json(cycles))
}

pub async fn list_cycles(State(s): State<SharedState>) -> Result<Json<Vec<SwapCycle>>, AppError> {
    Ok(Json(s.store.list_cycles().await?))
}

pub async fn confirm_swap(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, AppError> {
    let cycles = s.store.list_cycles().await?;
    let cycle = cycles.iter().find(|c| c.id == id).ok_or(AppError::NotFound)?;

    if !cycle.swaps.iter().any(|leg| leg.from_user_id == claims.sub) {
        return Err(AppError::Forbidden("只有交换环参与者可以确认".into()));
    }

    for leg in &cycle.swaps {
        s.store.update_item_status(leg.offer_item_id, &ItemStatus::Completed).await.ok();
    }

    tracing::info!("交换环 {} 已确认，{} 个物品完成交换", id, cycle.swaps.len());

    s.ws_hub.notify(&WsNotification {
        event: "swap_confirmed".into(),
        data: serde_json::json!({
            "cycle_id": id,
            "items_completed": cycle.swaps.len(),
        }),
    });

    Ok(Json(serde_json::json!({
        "cycle_id": id,
        "status": "confirmed",
        "items_completed": cycle.swaps.len()
    })))
}
