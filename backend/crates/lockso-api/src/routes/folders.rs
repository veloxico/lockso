use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post, put},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::folder::{
    CreateFolder, FolderTreeNode, FolderView, MoveFolder, UpdateFolder,
};
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::services::{activity_log_service, folder_service};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VaultQuery {
    vault_id: Uuid,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_folders).post(create_folder))
        .route("/tree", get(get_folder_tree))
        .route("/{id}", put(update_folder).delete(delete_folder))
        .route("/{id}/move", post(move_folder))
}

/// GET /v1/folders?vaultId=...
async fn list_folders(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<VaultQuery>,
) -> Result<Json<Vec<FolderView>>, AppError> {
    let folders = folder_service::list_folders(&state.db, q.vault_id, auth.user_id).await?;
    Ok(Json(folders))
}

/// GET /v1/folders/tree?vaultId=...
async fn get_folder_tree(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<VaultQuery>,
) -> Result<Json<Vec<FolderTreeNode>>, AppError> {
    let tree = folder_service::get_folder_tree(&state.db, q.vault_id, auth.user_id).await?;
    Ok(Json(tree))
}

/// POST /v1/folders
async fn create_folder(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<CreateFolder>,
) -> Result<Json<FolderView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let folder = folder_service::create_folder(&state.db, auth.user_id, input).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::FOLDER_CREATED,
        Some("folder"), Some(folder.id), Some(folder.vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(folder))
}

/// PUT /v1/folders/:id
async fn update_folder(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateFolder>,
) -> Result<Json<FolderView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let folder = folder_service::update_folder(&state.db, id, auth.user_id, input).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::FOLDER_UPDATED,
        Some("folder"), Some(id), Some(folder.vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(folder))
}

/// DELETE /v1/folders/:id
async fn delete_folder(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    folder_service::delete_folder(&state.db, id, auth.user_id).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::FOLDER_DELETED,
        Some("folder"), Some(id), None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(())
}

/// POST /v1/folders/:id/move
async fn move_folder(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<MoveFolder>,
) -> Result<Json<FolderView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let folder = folder_service::move_folder(&state.db, id, auth.user_id, input).await?;
    Ok(Json(folder))
}
