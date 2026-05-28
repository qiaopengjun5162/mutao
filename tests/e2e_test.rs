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

fn build_router(pool: PgPool) -> Router {
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

async fn setup_pool() -> PgPool {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/mutao".into());
    PgPool::connect(&database_url).await.unwrap()
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
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (app, status, json)
}

async fn patch_json(app: Router, uri: &str, body: Value) -> (Router, StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
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
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (app, status, json)
}

async fn register(app: Router, username: &str) -> (Router, Value) {
    let (app, _, body) = post_json(
        app,
        "/api/auth/register",
        json!({ "username": username, "password": "testpass123" }),
    )
    .await;
    (app, body)
}

// ---- E2E: Full swap flow ----

#[tokio::test]
#[ignore] // requires isolated database - run with: cargo test --test e2e_test -- --ignored
async fn e2e_full_swap_flow() {
    let pool = setup_pool().await;
    let app = build_router(pool);
    let prefix = Uuid::new_v4().as_simple().to_string();

    // 1. Register two users
    let (app, user_a) = register(app, &format!("alice_{prefix}")).await;
    let (app, user_b) = register(app, &format!("bob_{prefix}")).await;
    let user_a_id: Uuid = serde_json::from_value(user_a["user_id"].clone()).unwrap();
    let user_b_id: Uuid = serde_json::from_value(user_b["user_id"].clone()).unwrap();

    // 2. User A creates an item (book)
    let (app, status, item_a) = post_json(
        app,
        "/api/items",
        json!({
            "owner_id": user_a_id,
            "title": "Rust 编程之道",
            "tags": ["书籍", "编程"],
            "value_tier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item_a_id: Uuid = serde_json::from_value(item_a["id"].clone()).unwrap();

    // 3. User B creates an item (keyboard)
    let (app, status, item_b) = post_json(
        app,
        "/api/items",
        json!({
            "owner_id": user_b_id,
            "title": "机械键盘",
            "tags": ["键盘", "外设"],
            "value_tier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item_b_id: Uuid = serde_json::from_value(item_b["id"].clone()).unwrap();

    // 4. User A wants to trade book for keyboard
    let (app, status, _) = post_json(
        app,
        "/api/demands",
        json!({
            "user_id": user_a_id,
            "offer_item_id": item_a_id,
            "offer_tags": ["书籍"],
            "target_tags": ["键盘"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 5. User B wants to trade keyboard for book
    let (app, status, _) = post_json(
        app,
        "/api/demands",
        json!({
            "user_id": user_b_id,
            "offer_item_id": item_b_id,
            "offer_tags": ["键盘"],
            "target_tags": ["书籍"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 6. Trigger matching on item A
    let (app, status, cycles) =
        post_json(app, &format!("/api/items/{item_a_id}/match"), json!({})).await;
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
    let (app, status, confirm_resp) =
        post_json(app, &format!("/api/cycles/{cycle_id}/confirm"), json!({})).await;
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

// ---- E2E: Duplicate registration ----

#[tokio::test]
async fn e2e_duplicate_registration() {
    let pool = setup_pool().await;
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

#[tokio::test]
async fn e2e_login_wrong_password() {
    let pool = setup_pool().await;
    let app = build_router(pool);
    let username = format!("wrong_{}", Uuid::new_v4().as_simple());

    let (app, _) = register(app, &username).await;

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

#[tokio::test]
async fn e2e_item_status_transitions() {
    let pool = setup_pool().await;
    let app = build_router(pool);
    let (app, user) = register(app, &format!("status_{}", Uuid::new_v4().as_simple())).await;
    let user_id: Uuid = serde_json::from_value(user["user_id"].clone()).unwrap();

    let (app, _, item) = post_json(
        app,
        "/api/items",
        json!({
            "owner_id": user_id,
            "title": "测试物品",
            "tags": ["测试"],
            "value_tier": 2
        }),
    )
    .await;
    let item_id: Uuid = serde_json::from_value(item["id"].clone()).unwrap();
    assert_eq!(item["status"], "Idle");

    // Idle → Matching
    let (app, status, updated) = patch_json(
        app,
        &format!("/api/items/{item_id}/status"),
        json!({ "status": "Matching" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "Matching");

    // Matching → Completed
    let (app, status, updated) = patch_json(
        app,
        &format!("/api/items/{item_id}/status"),
        json!({ "status": "Completed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "Completed");

    // Completed → Archived
    let (app, status, updated) = patch_json(
        app,
        &format!("/api/items/{item_id}/status"),
        json!({ "status": "Archived" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "Archived");

    // Archived → Idle (should fail)
    let (_, status, _) = patch_json(
        app,
        &format!("/api/items/{item_id}/status"),
        json!({ "status": "Idle" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ---- E2E: No cycle found ----

#[tokio::test]
#[ignore] // requires isolated database - run with: cargo test --test e2e_test -- --ignored
async fn e2e_no_cycle_found() {
    let pool = setup_pool().await;
    let app = build_router(pool);
    let prefix = Uuid::new_v4().as_simple().to_string();
    let (app, user) = register(app, &format!("nocycle_{prefix}")).await;
    let user_id: Uuid = serde_json::from_value(user["user_id"].clone()).unwrap();

    // Use a unique tag that won't match any existing demands
    let unique_tag = format!("unique_nocycle_{prefix}");
    let (app, _, item) = post_json(
        app,
        "/api/items",
        json!({
            "owner_id": user_id,
            "title": "独物品",
            "tags": [unique_tag],
            "value_tier": 5
        }),
    )
    .await;
    let item_id: Uuid = serde_json::from_value(item["id"].clone()).unwrap();

    // Create a demand for this item but with a target that nobody offers
    let unique_target = format!("unique_target_{prefix}");
    let (app, _, _) = post_json(
        app,
        "/api/demands",
        json!({
            "user_id": user_id,
            "offer_item_id": item_id,
            "offer_tags": [unique_tag],
            "target_tags": [unique_target]
        }),
    )
    .await;

    let (_, status, cycles) =
        post_json(app, &format!("/api/items/{item_id}/match"), json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        cycles.as_array().unwrap().is_empty(),
        "should not find any cycle with unique tags"
    );
}
