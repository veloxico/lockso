use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
};
use serde::Serialize;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::models::item::TrashListEntry;
use lockso_core::services::{activity_log_service, item_service};

#[derive(Serialize)]
struct TrashCountResponse {
    count: i64,
}

#[derive(Serialize)]
struct EmptyTrashResponse {
    deleted: u64,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_trash).delete(empty_trash))
        .route("/count", get(trash_count))
        .route("/{id}/restore", post(restore_item))
        .route("/{id}", delete(permanent_delete))
}

/// GET /v1/trash
async fn list_trash(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<TrashListEntry>>, AppError> {
    let items = item_service::list_trash(&state.db, &state.encryption_key, auth.user_id).await?;
    Ok(Json(items))
}

/// GET /v1/trash/count
async fn trash_count(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<TrashCountResponse>, AppError> {
    let count = item_service::trash_count(&state.db, auth.user_id).await?;
    Ok(Json(TrashCountResponse { count }))
}

/// POST /v1/trash/:id/restore
async fn restore_item(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let vault_id = item_service::restore_item(&state.db, id, auth.user_id).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::ITEM_RESTORED,
        Some("item"), Some(id), Some(vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /v1/trash/:id — permanent delete
async fn permanent_delete(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let vault_id = item_service::permanent_delete_item(&state.db, Some(&state.storage), id, auth.user_id).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::ITEM_PURGED,
        Some("item"), Some(id), Some(vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /v1/trash — empty all trash
async fn empty_trash(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Result<Json<EmptyTrashResponse>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let deleted = item_service::empty_trash(&state.db, Some(&state.storage), auth.user_id).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::TRASH_EMPTIED,
        None, None, None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"count": deleted}),
    ).await;
    Ok(Json(EmptyTrashResponse { deleted }))
}
