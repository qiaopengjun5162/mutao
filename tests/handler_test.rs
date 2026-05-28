use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::{Router, routing::get, routing::post};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;
use tower::util::ServiceExt;
use uuid::Uuid;

use mutao::blockchain::ChainManager;
use mutao::handlers::{SharedState, auth_handler, cycles, demands, items};
use mutao::store::Store;
use mutao::ws::WsHub;

async fn setup_app() -> Router {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/mutao".into());
    let pool = PgPool::connect(&database_url).await.unwrap();

    let state: SharedState = Arc::new(mutao::AppState {
        store: Store::new(pool),
        ws_hub: WsHub::new(),
        chain_manager: ChainManager::new(),
    });

    Router::new()
        .route("/api/auth/register", post(auth_handler::register))
        .route("/api/auth/login", post(auth_handler::login))
        .route(
            "/api/items",
            post(items::create_item).get(items::list_items),
        )
        .route("/api/items/:id", get(items::get_item))
        .route("/api/items/:id/match", post(cycles::match_item))
        .route(
            "/api/items/:id/status",
            axum::routing::patch(items::update_item_status),
        )
        .route("/api/cycles/:id/confirm", post(cycles::confirm_swap))
        .route(
            "/api/demands",
            post(demands::create_demand).get(demands::list_demands),
        )
        .route("/api/cycles", get(cycles::list_cycles))
        .route("/api/health", get(items::health))
        .with_state(state)
}

async fn register_user(app: &Router, username: &str) -> Value {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "username": username, "password": "testpass123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

async fn create_item(app: &Router, owner_id: Uuid, title: &str) -> Value {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner_id": owner_id,
                        "title": title,
                        "tags": ["测试"],
                        "value_tier": 3
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

// ---- Health ----

#[tokio::test]
async fn test_health_endpoint() {
    let app = setup_app().await;
    let resp = app
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["name"], "木桃 Mutao");
}

// ---- Items ----

#[tokio::test]
async fn test_create_item_success() {
    let app = setup_app().await;
    let user = register_user(&app, &format!("item_{}", Uuid::new_v4().as_simple())).await;
    let owner_id: Uuid = serde_json::from_value(user["user_id"].clone()).unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner_id": owner_id,
                        "title": "机械键盘",
                        "tags": ["键盘", "外设"],
                        "value_tier": 3
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let item: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(item["title"], "机械键盘");
    assert_eq!(item["value_tier"], 3);
}

#[tokio::test]
async fn test_create_item_empty_title() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner_id": Uuid::new_v4(),
                        "title": "   ",
                        "tags": ["键盘"],
                        "value_tier": 3
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("title"));
}

#[tokio::test]
async fn test_create_item_empty_tags() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner_id": Uuid::new_v4(),
                        "title": "键盘",
                        "tags": [],
                        "value_tier": 3
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("tags"));
}

#[tokio::test]
async fn test_create_item_invalid_value_tier() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "owner_id": Uuid::new_v4(),
                        "title": "键盘",
                        "tags": ["外设"],
                        "value_tier": 0
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_item_not_found() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/items/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_items() {
    let app = setup_app().await;
    let resp = app
        .oneshot(Request::builder().uri("/api/items").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let items: Vec<Value> = serde_json::from_slice(&body).unwrap();
    let _ = items.len();
}

// ---- Demands ----

#[tokio::test]
async fn test_create_demand_success() {
    let app = setup_app().await;
    let user = register_user(&app, &format!("demand_{}", Uuid::new_v4().as_simple())).await;
    let user_id: Uuid = serde_json::from_value(user["user_id"].clone()).unwrap();
    let item = create_item(&app, user_id, "书籍").await;
    let item_id: Uuid = serde_json::from_value(item["id"].clone()).unwrap();

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/demands")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "user_id": user_id,
                        "offer_item_id": item_id,
                        "offer_tags": ["书籍"],
                        "target_tags": ["键盘"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let demand: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(demand["offer_tags"], json!(["书籍"]));
    assert_eq!(demand["target_tags"], json!(["键盘"]));
}

#[tokio::test]
async fn test_create_demand_empty_offer_tags() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/demands")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "user_id": Uuid::new_v4(),
                        "offer_item_id": Uuid::new_v4(),
                        "offer_tags": [],
                        "target_tags": ["键盘"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_demand_empty_target_tags() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/demands")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "user_id": Uuid::new_v4(),
                        "offer_item_id": Uuid::new_v4(),
                        "offer_tags": ["书籍"],
                        "target_tags": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_demands() {
    let app = setup_app().await;
    let resp = app
        .oneshot(Request::builder().uri("/api/demands").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ---- Cycles ----

#[tokio::test]
async fn test_list_cycles() {
    let app = setup_app().await;
    let resp = app
        .oneshot(Request::builder().uri("/api/cycles").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_confirm_swap_not_found() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/cycles/{}/confirm", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---- Auth ----

#[tokio::test]
async fn test_register_and_login() {
    let app = setup_app().await;
    let username = format!("test_{}", Uuid::new_v4().as_simple());

    // 注册
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "username": username, "password": "testpass123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let reg: Value = serde_json::from_slice(&body).unwrap();
    assert!(!reg["token"].as_str().unwrap().is_empty());

    // 登录
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "username": username, "password": "testpass123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let login: Value = serde_json::from_slice(&body).unwrap();
    assert!(!login["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_register_empty_username() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "username": "   ", "password": "testpass123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_short_password() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "username": "newuser", "password": "123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_wrong_password() {
    let app = setup_app().await;
    let username = format!("test_{}", Uuid::new_v4().as_simple());

    // 先注册
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "username": username, "password": "correctpass" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // 用错误密码登录
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "username": username, "password": "wrongpass" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
