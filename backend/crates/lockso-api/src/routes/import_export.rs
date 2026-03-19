use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::post,
};
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::import_export::{
    ExportRequest, ExportResult, ImportRequest, ImportResult,
};
use lockso_core::services::import_export_service;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{vault_id}/import", post(import_items))
        .route("/{vault_id}/export", post(export_items))
}

/// POST /v1/vaults/:vault_id/import
///
/// Import items from CSV, JSON, Passwork, KeePass, or Bitwarden format.
async fn import_items(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(vault_id): Path<Uuid>,
    Json(input): Json<ImportRequest>,
) -> Result<Json<ImportResult>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    let result = import_export_service::import_items(
        &state.db,
        &state.encryption_key,
        vault_id,
        auth.user_id,
        input.format,
        &input.data,
        input.create_folders,
    )
    .await?;

    Ok(Json(result))
}

/// POST /v1/vaults/:vault_id/export
///
/// Export all items from a vault in CSV or JSON format.
async fn export_items(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(vault_id): Path<Uuid>,
    Json(input): Json<ExportRequest>,
) -> Result<Json<ExportResult>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    let result = import_export_service::export_items(
        &state.db,
        &state.encryption_key,
        vault_id,
        auth.user_id,
        input.format,
    )
    .await?;

    Ok(Json(result))
}
