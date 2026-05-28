use axum::extract::{Multipart, Path};
use axum::response::Json;
use uuid::Uuid;

use crate::error::AppError;

use super::SharedState;

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
const UPLOAD_DIR: &str = "uploads";

pub async fn upload_image(
    axum::extract::State(s): axum::extract::State<SharedState>,
    Path(item_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    if !s.store.item_exists(item_id).await? {
        return Err(AppError::NotFound);
    }

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("读取上传文件失败: {e}")))?
        .ok_or(AppError::BadRequest("没有上传文件".into()))?;

    let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

    if !content_type.starts_with("image/") {
        return Err(AppError::BadRequest("只支持图片文件".into()));
    }

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("读取文件内容失败: {e}")))?;

    if data.len() > MAX_FILE_SIZE {
        return Err(AppError::BadRequest("文件大小不能超过 5MB".into()));
    }

    let ext = match content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => return Err(AppError::BadRequest("不支持的图片格式".into())),
    };

    std::fs::create_dir_all(UPLOAD_DIR)
        .map_err(|e| AppError::InternalMsg(format!("创建上传目录失败: {e}")))?;

    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let filepath = format!("{}/{}", UPLOAD_DIR, filename);
    std::fs::write(&filepath, &data)
        .map_err(|e| AppError::InternalMsg(format!("保存文件失败: {e}")))?;

    let image_url = format!("/uploads/{}", filename);
    s.store
        .update_item_image(item_id, &image_url)
        .await
        .map_err(|e| AppError::InternalMsg(format!("更新物品图片失败: {e}")))?;

    tracing::info!("物品 {} 上传图片: {}", item_id, image_url);

    Ok(Json(
        serde_json::json!({ "image_url": image_url, "size": data.len() }),
    ))
}
