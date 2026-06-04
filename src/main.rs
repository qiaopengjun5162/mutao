use axum::{Router, middleware, routing::get, routing::patch, routing::post};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use mutao::api_doc::ApiDoc;
use mutao::auth;
use mutao::blockchain::ChainManager;
use mutao::blockchain::ethereum::{EthereumAdapter, EthereumConfig};
use mutao::handlers::{
    SharedState, ai, auth_handler, blockchain_handler, cycles, demands, items, upload,
};
use mutao::store::Store;
use mutao::ws::WsHub;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    // 启动即校验密钥配置：生产环境缺失/过弱的 JWT_SECRET 直接拒绝启动
    auth::ensure_secret_for_env()?;

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/mutao".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("无法连接 PostgreSQL");

    tracing::info!("数据库连接成功");

    let mut chain_manager = ChainManager::new();
    let eth_rpc = std::env::var("ETH_RPC_URL").unwrap_or_default();
    let eth_contract = std::env::var("ETH_CONTRACT_ADDRESS").unwrap_or_default();
    if !eth_rpc.is_empty() && !eth_contract.is_empty() {
        chain_manager.register(Box::new(EthereumAdapter::new(EthereumConfig {
            rpc_url: eth_rpc,
            contract_address: eth_contract,
            chain_id: 1,
        })));
        tracing::info!("以太坊存证已启用");
    }

    let state: SharedState = Arc::new(mutao::AppState {
        store: Store::new(pool),
        ws_hub: WsHub::new(),
        chain_manager,
    });

    // 受保护路由：必须携带有效 JWT，否则中间件返回 401
    let protected = Router::new()
        .route("/api/items", post(items::create_item))
        .route("/api/items/analyze", post(ai::analyze_item))
        .route("/api/items/:id/match", post(cycles::match_item))
        .route("/api/items/:id/image", post(upload::upload_image))
        .route("/api/items/:id/status", patch(items::update_item_status))
        .route(
            "/api/items/:id/attest",
            post(blockchain_handler::attest_item),
        )
        .route("/api/cycles/:id/confirm", post(cycles::confirm_swap))
        .route("/api/demands", post(demands::create_demand))
        .route_layer(middleware::from_fn(auth::auth_middleware));

    // 公开路由：匿名可访问（浏览类 GET、认证、健康检查、文档）
    // WebSocket 在 handler 内通过 ?token= 自行鉴权
    let public = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/auth/register", post(auth_handler::register))
        .route("/api/auth/login", post(auth_handler::login))
        .route("/api/items", get(items::list_items))
        .route("/api/items/:id", get(items::get_item))
        .route(
            "/api/items/:id/history",
            get(blockchain_handler::item_history),
        )
        .route("/api/demands", get(demands::list_demands))
        .route("/api/cycles", get(cycles::list_cycles))
        .route("/api/health", get(items::health))
        .route("/api/ws", get(mutao::ws::ws_handler));

    let app = public
        .merge(protected)
        .with_state(state)
        .nest_service("/uploads", tower_http::services::ServeDir::new("uploads"));

    let addr = "0.0.0.0:3000";
    tracing::info!("木桃 Mutao 运行在 http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
