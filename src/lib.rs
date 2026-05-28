pub mod api_doc;
pub mod auth;
pub mod blockchain;
pub mod error;
pub mod handlers;
pub mod matcher;
pub mod models;
pub mod store;
pub mod ws;

/// 全局应用状态，供所有 handler 共享
pub struct AppState {
    pub store: store::Store,
    pub ws_hub: ws::WsHub,
    pub chain_manager: blockchain::ChainManager,
}
