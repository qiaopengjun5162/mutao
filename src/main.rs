use axum::{Router, routing::get, routing::post};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use mutao::blockchain::ChainManager;
use mutao::blockchain::ethereum::{EthereumAdapter, EthereumConfig};
use mutao::handlers::{SharedState, ai, auth_handler, blockchain_handler, cycles, demands, items};
use mutao::store::Store;
use mutao::ws::WsHub;

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

    let app = Router::new()
        .route("/api/auth/register", post(auth_handler::register))
        .route("/api/auth/login", post(auth_handler::login))
        .route(
            "/api/items",
            post(items::create_item).get(items::list_items),
        )
        .route("/api/items/analyze", post(ai::analyze_item))
        .route("/api/items/:id", get(items::get_item))
        .route("/api/items/:id/match", post(cycles::match_item))
        .route(
            "/api/items/:id/status",
            axum::routing::patch(items::update_item_status),
        )
        .route(
            "/api/items/:id/attest",
            post(blockchain_handler::attest_item),
        )
        .route(
            "/api/items/:id/history",
            get(blockchain_handler::item_history),
        )
        .route("/api/cycles/:id/confirm", post(cycles::confirm_swap))
        .route(
            "/api/demands",
            post(demands::create_demand).get(demands::list_demands),
        )
        .route("/api/cycles", get(cycles::list_cycles))
        .route("/api/health", get(items::health))
        .route("/api/ws", get(mutao::ws::ws_handler))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("木桃 Mutao 运行在 http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
