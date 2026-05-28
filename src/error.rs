use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn test_not_found_returns_404() {
        let resp = AppError::NotFound.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "资源不存在");
    }

    #[tokio::test]
    async fn test_bad_request_returns_400() {
        let resp = AppError::BadRequest("title 不能为空".into()).into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "title 不能为空");
    }

    #[tokio::test]
    async fn test_internal_msg_returns_500() {
        let resp = AppError::InternalMsg("密码加密失败".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "服务器内部错误");
    }

    #[tokio::test]
    async fn test_serialization_returns_500() {
        let err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
        let resp = AppError::Serialization(err).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "服务器内部错误");
    }
}
