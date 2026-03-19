use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::models::attachment::AttachmentView;
use lockso_core::services::{activity_log_service, attachment_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/items/{item_id}/attachments", get(list_attachments).post(upload_attachment))
        .route("/attachments/{id}", get(download_attachment).delete(delete_attachment))
}

/// GET /v1/items/:item_id/attachments
async fn list_attachments(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(item_id): Path<Uuid>,
) -> Result<Json<Vec<AttachmentView>>, AppError> {
    let attachments = attachment_service::list_attachments(
        &state.db,
        &state.encryption_key,
        item_id,
        auth.user_id,
    )
    .await?;
    Ok(Json(attachments))
}

/// POST /v1/items/:item_id/attachments (multipart/form-data)
///
/// Expects a single file field named "file".
async fn upload_attachment(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<AttachmentView>), AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    let mut file_name: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            file_name = field.file_name().map(|s| s.to_string());
            mime_type = field.content_type().map(|s| s.to_string());

            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read file: {e}")))?;

            // Check size early
            if data.len() > attachment_service::MAX_FILE_SIZE {
                return Err(AppError::PayloadTooLarge(format!(
                    "File exceeds maximum size of {} MB",
                    attachment_service::MAX_FILE_SIZE / (1024 * 1024)
                )));
            }

            file_data = Some(data.to_vec());
        }
    }

    let data = file_data.ok_or_else(|| AppError::Validation("No file uploaded".into()))?;
    let name = file_name.unwrap_or_else(|| "untitled".to_string());
    let mime = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());

    let view = attachment_service::upload_attachment(
        &state.db,
        &state.storage,
        &state.encryption_key,
        item_id,
        auth.user_id,
        &name,
        &mime,
        &data,
    )
    .await?;

    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::ATTACHMENT_UPLOADED,
        Some("attachment"), Some(view.id), None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"itemId": item_id.to_string()}),
    ).await;
    Ok((StatusCode::CREATED, Json(view)))
}

/// GET /v1/attachments/:id — download attachment as binary.
async fn download_attachment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let (data, file_name, mime_type) = attachment_service::download_attachment(
        &state.db,
        &state.storage,
        &state.encryption_key,
        id,
        auth.user_id,
    )
    .await?;

    // Build content-disposition header with sanitized filename
    let safe_name = sanitize_filename(&file_name);
    let disposition = format!("attachment; filename=\"{safe_name}\"");

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime_type).unwrap_or_else(|_| {
            HeaderValue::from_static("application/octet-stream")
        }),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition).unwrap_or_else(|_| {
            HeaderValue::from_static("attachment")
        }),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&data.len().to_string()).unwrap_or_else(|_| {
            HeaderValue::from_static("0")
        }),
    );
    // Prevent caching of decrypted attachments
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    // Prevent MIME sniffing — browser must respect the Content-Type header
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    Ok((headers, Body::from(data)).into_response())
}

/// DELETE /v1/attachments/:id
async fn delete_attachment(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    attachment_service::delete_attachment(
        &state.db,
        &state.storage,
        id,
        auth.user_id,
    )
    .await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::ATTACHMENT_DELETED,
        Some("attachment"), Some(id), None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(StatusCode::NO_CONTENT)
}

/// Sanitize filename for Content-Disposition header.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '"' | '\\' | '/' | '\0' | '\r' | '\n'))
        .collect::<String>()
        .chars()
        .take(255)
        .collect()
}
