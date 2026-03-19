use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::item::{
    CreateItem, ItemListEntry, ItemView, MoveItem, SearchRequest, UpdateItem,
};
use lockso_core::models::snapshot::{SnapshotListEntry, SnapshotView};
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::services::{activity_log_service, item_service};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemsQuery {
    vault_id: Uuid,
    folder_id: Option<Uuid>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FavoriteResponse {
    is_favorite: bool,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_items).post(create_item))
        .route("/search", post(search_items))
        .route("/recent", get(get_recent_items))
        .route("/{id}", get(get_item).put(update_item).delete(delete_item))
        .route("/{id}/move", post(move_item))
        .route("/{id}/favorite", post(toggle_favorite))
        .route("/{id}/snapshots", get(list_snapshots))
        .route("/{id}/snapshots/{snapshot_id}", get(get_snapshot))
        .route("/{id}/revert-to-snapshot", post(revert_to_snapshot))
}

/// GET /v1/items?vaultId=...&folderId=...
async fn list_items(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ItemsQuery>,
) -> Result<Json<Vec<ItemListEntry>>, AppError> {
    let items = item_service::list_items(
        &state.db,
        &state.encryption_key,
        q.vault_id,
        q.folder_id,
        auth.user_id,
    )
    .await?;
    Ok(Json(items))
}

/// GET /v1/items/:id
async fn get_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ItemView>, AppError> {
    let item =
        item_service::get_item(&state.db, &state.encryption_key, id, auth.user_id).await?;
    Ok(Json(item))
}

/// POST /v1/items
async fn create_item(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<CreateItem>,
) -> Result<Json<ItemView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let item = item_service::create_item(
        &state.db,
        &state.encryption_key,
        auth.user_id,
        input,
    )
    .await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::ITEM_CREATED,
        Some("item"), Some(item.id), Some(item.vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(item))
}

/// PUT /v1/items/:id
async fn update_item(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateItem>,
) -> Result<Json<ItemView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let item = item_service::update_item(
        &state.db,
        &state.encryption_key,
        id,
        auth.user_id,
        input,
    )
    .await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::ITEM_UPDATED,
        Some("item"), Some(id), Some(item.vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(item))
}

/// DELETE /v1/items/:id — soft-deletes (moves to trash)
async fn delete_item(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let vault_id = item_service::delete_item(&state.db, Some(&state.storage), id, auth.user_id).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::ITEM_TRASHED,
        Some("item"), Some(id), Some(vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /v1/items/:id/move
async fn move_item(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<MoveItem>,
) -> Result<Json<ItemView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let item = item_service::move_item(
        &state.db,
        &state.encryption_key,
        id,
        auth.user_id,
        input,
    )
    .await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::ITEM_MOVED,
        Some("item"), Some(id), Some(item.vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(item))
}

/// POST /v1/items/search
async fn search_items(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<SearchRequest>,
) -> Result<Json<Vec<ItemListEntry>>, AppError> {
    let items = item_service::search_items(
        &state.db,
        &state.encryption_key,
        auth.user_id,
        input,
    )
    .await?;
    Ok(Json(items))
}

/// GET /v1/items/recent
async fn get_recent_items(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ItemListEntry>>, AppError> {
    let items =
        item_service::get_recent_items(&state.db, &state.encryption_key, auth.user_id).await?;
    Ok(Json(items))
}

/// POST /v1/items/:id/favorite
async fn toggle_favorite(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<FavoriteResponse>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let is_favorite = item_service::toggle_favorite(&state.db, auth.user_id, id).await?;
    Ok(Json(FavoriteResponse { is_favorite }))
}

/// GET /v1/items/:id/snapshots
async fn list_snapshots(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SnapshotListEntry>>, AppError> {
    let snapshots =
        item_service::list_snapshots(&state.db, &state.encryption_key, id, auth.user_id).await?;
    Ok(Json(snapshots))
}

/// GET /v1/items/:id/snapshots/:snapshot_id
async fn get_snapshot(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, snapshot_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<SnapshotView>, AppError> {
    let snapshot = item_service::get_snapshot(
        &state.db,
        &state.encryption_key,
        id,
        snapshot_id,
        auth.user_id,
    )
    .await?;
    Ok(Json(snapshot))
}

/// POST /v1/items/:id/revert-to-snapshot
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RevertRequest {
    snapshot_id: Uuid,
}

async fn revert_to_snapshot(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<RevertRequest>,
) -> Result<Json<ItemView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let item = item_service::revert_to_snapshot(
        &state.db,
        &state.encryption_key,
        id,
        input.snapshot_id,
        auth.user_id,
    )
    .await?;
    Ok(Json(item))
}
