use axum::response::Json;
use serde::Deserialize;

use crate::error::AppError;

#[derive(Deserialize)]
pub struct AnalyzeReq {
    pub description: String,
}

#[derive(serde::Serialize)]
pub struct AnalyzeRes {
    pub tags: Vec<String>,
    pub value_tier: u8,
    pub method: String,
}

pub async fn analyze_item(Json(req): Json<AnalyzeReq>) -> Result<Json<AnalyzeRes>, AppError> {
    if req.description.trim().is_empty() {
        return Err(AppError::BadRequest("description 不能为空".into()));
    }

    let output = tokio::process::Command::new("python3")
        .arg("scalpel/scalpel.py")
        .arg("--description")
        .arg(&req.description)
        .arg("--llm")
        .output()
        .await
        .map_err(|e| AppError::InternalMsg(format!("调用 scalpel 失败: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("scalpel 执行失败: {}", stderr);
        return Err(AppError::InternalMsg("标签提取失败".into()));
    }

    let result: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::InternalMsg(format!("解析 scalpel 输出失败: {e}")))?;

    Ok(Json(AnalyzeRes {
        tags: result["tags"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        value_tier: result["value_tier"].as_u64().unwrap_or(1) as u8,
        method: result["method"].as_str().unwrap_or("unknown").to_string(),
    }))
}
