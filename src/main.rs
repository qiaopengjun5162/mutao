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

use mutao::auth;
use mutao::error::AppError;
use mutao::matcher;
use mutao::models::{Demand, Item, ItemStatus, SwapCycle, User};
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
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/items", post(create_item).get(list_items))
        .route("/api/items/:id", get(get_item))
        .route("/api/items/:id/match", post(match_item))
        .route("/api/items/:id/status", axum::routing::patch(update_item_status))
        .route("/api/cycles/:id/confirm", post(confirm_swap))
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

async fn confirm_swap(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let cycles = s.store.list_cycles().await?;
    let cycle = cycles
        .iter()
        .find(|c| c.id == id)
        .ok_or(AppError::NotFound)?;

    // 将交换环中所有物品标记为 Completed
    for leg in &cycle.swaps {
        s.store
            .update_item_status(leg.offer_item_id, &ItemStatus::Completed)
            .await
            .ok();
    }

    tracing::info!("交换环 {} 已确认，{} 个物品完成交换", id, cycle.swaps.len());

    Ok(Json(serde_json::json!({
        "cycle_id": id,
        "status": "confirmed",
        "items_completed": cycle.swaps.len()
    })))
}

#[derive(Deserialize)]
struct UpdateStatusReq {
    status: ItemStatus,
}

async fn update_item_status(
    State(s): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<UpdateStatusReq>,
) -> Result<Json<Item>, AppError> {
    let item = s
        .store
        .get_item(id)
        .await?
        .ok_or(AppError::NotFound)?;

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

// ---- Auth ----

#[derive(Deserialize)]
struct RegisterReq {
    username: String,
    password: String,
}

/// 用户注册：创建新用户，返回 JWT token
async fn register(
    State(s): State<SharedState>,
    Json(req): Json<RegisterReq>,
) -> Result<Json<auth::LoginRes>, AppError> {
    // 验证输入
    if req.username.trim().is_empty() {
        return Err(AppError::BadRequest("用户名不能为空".into()));
    }
    if req.password.len() < 6 {
        return Err(AppError::BadRequest("密码长度至少 6 位".into()));
    }

    // 检查用户名是否已存在
    if s.store.get_user_by_username(&req.username).await?.is_some() {
        return Err(AppError::BadRequest("用户名已存在".into()));
    }

    // 创建用户
    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::InternalMsg(format!("密码加密失败: {e}")))?;

    let user = User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        password_hash,
        created_at: chrono::Utc::now(),
    };

    s.store.create_user(&user).await?;

    // 生成 token
    let token = auth::create_token(user.id, &user.username)
        .map_err(|_| AppError::InternalMsg("生成 token 失败".into()))?;

    tracing::info!("用户 {} 注册成功", user.username);

    Ok(Json(auth::LoginRes {
        token,
        user_id: user.id,
        username: user.username,
    }))
}

/// 用户登录：验证凭据，返回 JWT token
async fn login(
    State(s): State<SharedState>,
    Json(req): Json<auth::LoginReq>,
) -> Result<Json<auth::LoginRes>, AppError> {
    // 查找用户
    let user = s
        .store
        .get_user_by_username(&req.username)
        .await?
        .ok_or(AppError::BadRequest("用户名或密码错误".into()))?;

    // 验证密码
    let valid = bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|e| AppError::InternalMsg(format!("密码验证失败: {e}")))?;

    if !valid {
        return Err(AppError::BadRequest("用户名或密码错误".into()));
    }

    // 生成 token
    let token = auth::create_token(user.id, &user.username)
        .map_err(|_| AppError::InternalMsg("生成 token 失败".into()))?;

    tracing::info!("用户 {} 登录成功", user.username);

    Ok(Json(auth::LoginRes {
        token,
        user_id: user.id,
        username: user.username,
    }))
}
