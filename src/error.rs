use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("资源不存在")]
    NotFound,

    #[error("{0}")]
    BadRequest(String),

    #[error("服务器内部错误")]
    Internal(#[from] sqlx::Error),

    #[error("{0}")]
    InternalMsg(String),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            // 内部错误不暴露细节给客户端，只记日志
            AppError::Internal(_) | AppError::InternalMsg(_) | AppError::Serialization(_) => {
                tracing::error!("内部错误: {self}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "服务器内部错误".to_string(),
                )
            }
        };

        (status, Json(json!({"error": message}))).into_response()
    }
}
