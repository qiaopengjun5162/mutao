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

fn build_router(pool: PgPool) -> Router {
    let state: SharedState = Arc::new(mutao::AppState {
        store: Store::new(pool),
        ws_hub: WsHub::new(),
        chain_manager: ChainManager::new(),
    });

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

async fn post_json(app: Router, uri: &str, body: Value) -> (Router, StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (app, status, json)
}

async fn post_auth(
    app: Router,
    uri: &str,
    token: &str,
    body: Value,
) -> (Router, StatusCode, Value) {
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
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (app, status, json)
}

async fn patch_auth(
    app: Router,
    uri: &str,
    token: &str,
    body: Value,
) -> (Router, StatusCode, Value) {
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
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (app, status, json)
}

async fn get_json(app: Router, uri: &str) -> (Router, StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (app, status, json)
}

/// 注册用户，返回 (app, token, user_id)
async fn register(app: Router, username: &str) -> (Router, String, Uuid) {
    let (app, _, body) = post_json(
        app,
        "/api/auth/register",
        json!({ "username": username, "password": "testpass123" }),
    )
    .await;
    let token = body["token"].as_str().unwrap().to_string();
    let user_id = serde_json::from_value(body["user_id"].clone()).unwrap();
    (app, token, user_id)
}

// ---- E2E: Full swap flow ----

#[sqlx::test]
async fn e2e_full_swap_flow(pool: PgPool) {
    let app = build_router(pool);
    let prefix = Uuid::new_v4().as_simple().to_string();

    // 1. Register two users
    let (app, token_a, _user_a_id) = register(app, &format!("alice_{prefix}")).await;
    let (app, token_b, _user_b_id) = register(app, &format!("bob_{prefix}")).await;

    // 2. User A creates an item (book)
    let (app, status, item_a) = post_auth(
        app,
        "/api/items",
        &token_a,
        json!({
            "title": "Rust 编程之道",
            "tags": ["书籍", "编程"],
            "value_tier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item_a_id: Uuid = serde_json::from_value(item_a["id"].clone()).unwrap();

    // 3. User B creates an item (keyboard)
    let (app, status, item_b) = post_auth(
        app,
        "/api/items",
        &token_b,
        json!({
            "title": "机械键盘",
            "tags": ["键盘", "外设"],
            "value_tier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item_b_id: Uuid = serde_json::from_value(item_b["id"].clone()).unwrap();

    // 4. User A wants to trade book for keyboard
    let (app, status, _) = post_auth(
        app,
        "/api/demands",
        &token_a,
        json!({
            "offer_item_id": item_a_id,
            "offer_tags": ["书籍"],
            "target_tags": ["键盘"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 5. User B wants to trade keyboard for book
    let (app, status, _) = post_auth(
        app,
        "/api/demands",
        &token_b,
        json!({
            "offer_item_id": item_b_id,
            "offer_tags": ["键盘"],
            "target_tags": ["书籍"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 6. Trigger matching on item A (owner = user A)
    let (app, status, cycles) = post_auth(
        app,
        &format!("/api/items/{item_a_id}/match"),
        &token_a,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !cycles.as_array().unwrap().is_empty(),
        "should find at least one cycle"
    );

    // 7. List cycles and confirm the one containing our items
    let (app, status, cycle_list) = get_json(app, "/api/cycles").await;
    assert_eq!(status, StatusCode::OK);

    let cycle = cycle_list
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["swaps"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s["offer_item_id"].as_str() == Some(&item_a_id.to_string()))
        })
        .expect("should find cycle containing our items");

    let cycle_id = cycle["id"].as_str().unwrap();
    // User A 是该环参与者，可确认
    let (app, status, confirm_resp) = post_auth(
        app,
        &format!("/api/cycles/{cycle_id}/confirm"),
        &token_a,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(confirm_resp["status"], "confirmed");

    // 8. Verify items are now Completed
    let (app, status, item_a_final) = get_json(app, &format!("/api/items/{item_a_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(item_a_final["status"], "Completed");

    let (_, status, item_b_final) = get_json(app, &format!("/api/items/{item_b_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(item_b_final["status"], "Completed");
}

// ---- E2E: Confirm swap rejects non-participant ----

#[sqlx::test]
async fn e2e_confirm_swap_forbidden_for_outsider(pool: PgPool) {
    let app = build_router(pool);
    let prefix = Uuid::new_v4().as_simple().to_string();

    let (app, token_a, _) = register(app, &format!("pa_{prefix}")).await;
    let (app, token_b, _) = register(app, &format!("pb_{prefix}")).await;
    let (app, token_c, _) = register(app, &format!("pc_{prefix}")).await;

    let (app, _, item_a) = post_auth(
        app,
        "/api/items",
        &token_a,
        json!({ "title": "书", "tags": ["书籍"], "value_tier": 3 }),
    )
    .await;
    let item_a_id: Uuid = serde_json::from_value(item_a["id"].clone()).unwrap();

    let (app, _, item_b) = post_auth(
        app,
        "/api/items",
        &token_b,
        json!({ "title": "键盘", "tags": ["键盘"], "value_tier": 3 }),
    )
    .await;
    let item_b_id: Uuid = serde_json::from_value(item_b["id"].clone()).unwrap();

    let (app, _, _) = post_auth(
        app,
        "/api/demands",
        &token_a,
        json!({ "offer_item_id": item_a_id, "offer_tags": ["书籍"], "target_tags": ["键盘"] }),
    )
    .await;
    let (app, _, _) = post_auth(
        app,
        "/api/demands",
        &token_b,
        json!({ "offer_item_id": item_b_id, "offer_tags": ["键盘"], "target_tags": ["书籍"] }),
    )
    .await;

    let (app, _, _) = post_auth(
        app,
        &format!("/api/items/{item_a_id}/match"),
        &token_a,
        json!({}),
    )
    .await;

    let (app, _, cycle_list) = get_json(app, "/api/cycles").await;
    let cycle = cycle_list
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["swaps"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s["offer_item_id"].as_str() == Some(&item_a_id.to_string()))
        })
        .expect("cycle should exist");
    let cycle_id = cycle["id"].as_str().unwrap();

    // User C 不是参与者 → 403
    let (_, status, _) = post_auth(
        app,
        &format!("/api/cycles/{cycle_id}/confirm"),
        &token_c,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ---- E2E: Duplicate registration ----

#[sqlx::test]
async fn e2e_duplicate_registration(pool: PgPool) {
    let app = build_router(pool);
    let username = format!("dup_{}", Uuid::new_v4().as_simple());

    let (app, status, _) = post_json(
        app,
        "/api/auth/register",
        json!({ "username": username, "password": "testpass123" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, status, body) = post_json(
        app,
        "/api/auth/register",
        json!({ "username": username, "password": "testpass123" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("已存在"));
}

// ---- E2E: Login with wrong password ----

#[sqlx::test]
async fn e2e_login_wrong_password(pool: PgPool) {
    let app = build_router(pool);
    let username = format!("wrong_{}", Uuid::new_v4().as_simple());

    let (app, _, _) = register(app, &username).await;

    let (_, status, body) = post_json(
        app,
        "/api/auth/login",
        json!({ "username": username, "password": "wrongpassword" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("错误"));
}

// ---- E2E: Item status transitions ----

#[sqlx::test]
async fn e2e_item_status_transitions(pool: PgPool) {
    let app = build_router(pool);
    let (app, token, _) = register(app, &format!("status_{}", Uuid::new_v4().as_simple())).await;

    let (app, _, item) = post_auth(
        app,
        "/api/items",
        &token,
        json!({
            "title": "测试物品",
            "tags": ["测试"],
            "value_tier": 2
        }),
    )
    .await;
    let item_id: Uuid = serde_json::from_value(item["id"].clone()).unwrap();
    assert_eq!(item["status"], "Idle");

    // Idle → Matching
    let (app, status, updated) = patch_auth(
        app,
        &format!("/api/items/{item_id}/status"),
        &token,
        json!({ "status": "Matching" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "Matching");

    // Matching → Completed
    let (app, status, updated) = patch_auth(
        app,
        &format!("/api/items/{item_id}/status"),
        &token,
        json!({ "status": "Completed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "Completed");

    // Completed → Archived
    let (app, status, updated) = patch_auth(
        app,
        &format!("/api/items/{item_id}/status"),
        &token,
        json!({ "status": "Archived" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "Archived");

    // Archived → Idle (should fail)
    let (_, status, _) = patch_auth(
        app,
        &format!("/api/items/{item_id}/status"),
        &token,
        json!({ "status": "Idle" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ---- E2E: No cycle found ----

#[sqlx::test]
async fn e2e_no_cycle_found(pool: PgPool) {
    let app = build_router(pool);
    let prefix = Uuid::new_v4().as_simple().to_string();
    let (app, token, _) = register(app, &format!("nocycle_{prefix}")).await;

    // Use a unique tag that won't match any existing demands
    let unique_tag = format!("unique_nocycle_{prefix}");
    let (app, _, item) = post_auth(
        app,
        "/api/items",
        &token,
        json!({
            "title": "独物品",
            "tags": [unique_tag],
            "value_tier": 5
        }),
    )
    .await;
    let item_id: Uuid = serde_json::from_value(item["id"].clone()).unwrap();

    // Create a demand for this item but with a target that nobody offers
    let unique_target = format!("unique_target_{prefix}");
    let (app, _, _) = post_auth(
        app,
        "/api/demands",
        &token,
        json!({
            "offer_item_id": item_id,
            "offer_tags": [unique_tag],
            "target_tags": [unique_target]
        }),
    )
    .await;

    let (_, status, cycles) = post_auth(
        app,
        &format!("/api/items/{item_id}/match"),
        &token,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        cycles.as_array().unwrap().is_empty(),
        "should not find any cycle with unique tags"
    );
}
