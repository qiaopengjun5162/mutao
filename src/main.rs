use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

use mutao::error::AppError;
use mutao::matcher;
use mutao::models::{Demand, Item, ItemStatus, SwapCycle};
use mutao::store::Store;

struct AppState {
    store: Store,
}

type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/mutao".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("无法连接 PostgreSQL");

    tracing::info!("数据库连接成功");

    let state = Arc::new(AppState {
        store: Store::new(pool),
    });

    let app = Router::new()
        .route("/api/items", post(create_item).get(list_items))
        .route("/api/items/:id", get(get_item))
        .route("/api/items/:id/match", post(match_item))
        .route("/api/demands", post(create_demand).get(list_demands))
        .route("/api/cycles", get(list_cycles))
        .route("/api/health", get(health))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("木桃 Mutao 运行在 http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status":"ok","name":"木桃 Mutao"}))
}

// ---- Item ----

#[derive(Deserialize)]
struct CreateItemReq {
    owner_id: Uuid,
    title: String,
    description: Option<String>,
    image_url: Option<String>,
    tags: Vec<String>,
    value_tier: u8,
}

async fn create_item(
    State(s): State<SharedState>,
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
        owner_id: req.owner_id,
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

async fn get_item(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Item>, AppError> {
    s.store
        .get_item(id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

async fn list_items(State(s): State<SharedState>) -> Result<Json<Vec<Item>>, AppError> {
    Ok(Json(s.store.list_items().await?))
}

// ---- Demand ----

#[derive(Deserialize)]
struct CreateDemandReq {
    user_id: Uuid,
    offer_item_id: Uuid,
    offer_tags: Vec<String>,
    target_tags: Vec<String>,
}

async fn create_demand(
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

async fn list_demands(State(s): State<SharedState>) -> Result<Json<Vec<Demand>>, AppError> {
    Ok(Json(s.store.list_demands().await?))
}

// ---- Matching ----

async fn match_item(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Vec<SwapCycle>>, AppError> {
    if !s.store.item_exists(id).await? {
        return Err(AppError::NotFound);
    }

    let demands = s.store.list_demands().await?;

    if demands.len() < 2 {
        return Ok(Json(vec![]));
    }

    let cycles = matcher::Matcher::find_cycles(&demands, 4);

    for cycle in &cycles {
        s.store.save_cycle(cycle).await.ok();
    }

    Ok(Json(cycles))
}

async fn list_cycles(State(s): State<SharedState>) -> Result<Json<Vec<SwapCycle>>, AppError> {
    Ok(Json(s.store.list_cycles().await?))
}
