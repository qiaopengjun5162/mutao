use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::routing::{get, patch, post};
use axum::{Router, middleware};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;
use tower::util::ServiceExt;
use uuid::Uuid;

use mutao::auth;
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

    // 与生产一致：写操作走中间件鉴权，浏览类 GET 公开
    let protected = Router::new()
        .route("/api/items", post(items::create_item))
        .route("/api/items/:id/match", post(cycles::match_item))
        .route("/api/items/:id/status", patch(items::update_item_status))
        .route("/api/cycles/:id/confirm", post(cycles::confirm_swap))
        .route("/api/demands", post(demands::create_demand))
        .route_layer(middleware::from_fn(auth::auth_middleware));

    let public = Router::new()
        .route("/api/auth/register", post(auth_handler::register))
        .route("/api/auth/login", post(auth_handler::login))
        .route("/api/items", get(items::list_items))
        .route("/api/items/:id", get(items::get_item))
        .route("/api/demands", get(demands::list_demands))
        .route("/api/cycles", get(cycles::list_cycles))
        .route("/api/health", get(items::health));

    public.merge(protected).with_state(state)
}

async fn body_json(resp: axum::response::Response) -> (StatusCode, Value) {
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// 注册用户，返回 (token, user_id)
async fn register_user(app: &Router, username: &str) -> (String, Uuid) {
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
    let (_, body) = body_json(resp).await;
    let token = body["token"].as_str().unwrap().to_string();
    let user_id = serde_json::from_value(body["user_id"].clone()).unwrap();
    (token, user_id)
}

async fn post_auth(app: &Router, uri: &str, token: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    body_json(resp).await
}

async fn patch_auth(app: &Router, uri: &str, token: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    body_json(resp).await
}

async fn create_item(app: &Router, token: &str, title: &str) -> Value {
    let (_, item) = post_auth(
        app,
        "/api/items",
        token,
        json!({ "title": title, "tags": ["测试"], "value_tier": 3 }),
    )
    .await;
    item
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
    let (_, json) = body_json(resp).await;
    assert_eq!(json["status"], "ok");
    assert_eq!(json["name"], "木桃 Mutao");
}

// ---- Items ----

#[tokio::test]
async fn test_create_item_success() {
    let app = setup_app().await;
    let (token, owner_id) =
        register_user(&app, &format!("item_{}", Uuid::new_v4().as_simple())).await;

    let (status, item) = post_auth(
        &app,
        "/api/items",
        &token,
        json!({ "title": "机械键盘", "tags": ["键盘", "外设"], "value_tier": 3 }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(item["title"], "机械键盘");
    assert_eq!(item["value_tier"], 3);
    // owner_id 必须来自 token，而非客户端
    assert_eq!(item["owner_id"], json!(owner_id));
}

#[tokio::test]
async fn test_create_item_unauthorized() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "title": "无 token", "tags": ["x"], "value_tier": 3 }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_item_empty_title() {
    let app = setup_app().await;
    let (token, _) = register_user(&app, &format!("et_{}", Uuid::new_v4().as_simple())).await;
    let (status, json) = post_auth(
        &app,
        "/api/items",
        &token,
        json!({ "title": "   ", "tags": ["键盘"], "value_tier": 3 }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("title"));
}

#[tokio::test]
async fn test_create_item_empty_tags() {
    let app = setup_app().await;
    let (token, _) = register_user(&app, &format!("etag_{}", Uuid::new_v4().as_simple())).await;
    let (status, json) = post_auth(
        &app,
        "/api/items",
        &token,
        json!({ "title": "键盘", "tags": [], "value_tier": 3 }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("tags"));
}

#[tokio::test]
async fn test_create_item_invalid_value_tier() {
    let app = setup_app().await;
    let (token, _) = register_user(&app, &format!("vt_{}", Uuid::new_v4().as_simple())).await;
    let (status, _) = post_auth(
        &app,
        "/api/items",
        &token,
        json!({ "title": "键盘", "tags": ["外设"], "value_tier": 0 }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_status_forbidden_for_non_owner() {
    let app = setup_app().await;
    let (token_a, _) = register_user(&app, &format!("owner_{}", Uuid::new_v4().as_simple())).await;
    let (token_b, _) = register_user(&app, &format!("other_{}", Uuid::new_v4().as_simple())).await;

    let item = create_item(&app, &token_a, "A 的物品").await;
    let item_id = item["id"].as_str().unwrap();

    // 用户 B 试图修改用户 A 的物品状态
    let (status, _) = patch_auth(
        &app,
        &format!("/api/items/{item_id}/status"),
        &token_b,
        json!({ "status": "Matching" }),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
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
    let (_, items) = body_json(resp).await;
    assert!(items.is_array());
}

// ---- Demands ----

#[tokio::test]
async fn test_create_demand_success() {
    let app = setup_app().await;
    let (token, _) = register_user(&app, &format!("demand_{}", Uuid::new_v4().as_simple())).await;
    let item = create_item(&app, &token, "书籍").await;
    let item_id: Uuid = serde_json::from_value(item["id"].clone()).unwrap();

    let (status, demand) = post_auth(
        &app,
        "/api/demands",
        &token,
        json!({
            "offer_item_id": item_id,
            "offer_tags": ["书籍"],
            "target_tags": ["键盘"]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(demand["offer_tags"], json!(["书籍"]));
    assert_eq!(demand["target_tags"], json!(["键盘"]));
}

#[tokio::test]
async fn test_create_demand_unauthorized() {
    let app = setup_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/demands")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "offer_item_id": Uuid::new_v4(),
                        "offer_tags": ["书籍"],
                        "target_tags": ["键盘"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_demand_empty_offer_tags() {
    let app = setup_app().await;
    let (token, _) = register_user(&app, &format!("eo_{}", Uuid::new_v4().as_simple())).await;
    let (status, _) = post_auth(
        &app,
        "/api/demands",
        &token,
        json!({
            "offer_item_id": Uuid::new_v4(),
            "offer_tags": [],
            "target_tags": ["键盘"]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_demand_empty_target_tags() {
    let app = setup_app().await;
    let (token, _) = register_user(&app, &format!("et2_{}", Uuid::new_v4().as_simple())).await;
    let (status, _) = post_auth(
        &app,
        "/api/demands",
        &token,
        json!({
            "offer_item_id": Uuid::new_v4(),
            "offer_tags": ["书籍"],
            "target_tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
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
async fn test_confirm_swap_unauthorized() {
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

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_confirm_swap_not_found() {
    let app = setup_app().await;
    let (token, _) = register_user(&app, &format!("cs_{}", Uuid::new_v4().as_simple())).await;
    let (status, _) = post_auth(
        &app,
        &format!("/api/cycles/{}/confirm", Uuid::new_v4()),
        &token,
        json!({}),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
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
    let (_, reg) = body_json(resp).await;
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
    let (_, login) = body_json(resp).await;
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
