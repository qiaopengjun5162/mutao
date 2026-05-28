pub mod ai;
pub mod auth_handler;
pub mod blockchain_handler;
pub mod cycles;
pub mod demands;
pub mod items;
pub mod upload;

use std::sync::Arc;

pub type SharedState = Arc<crate::AppState>;
