use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::activity_log::{ActivityLogQuery, PaginatedActivityLogs};
use lockso_core::services::{activity_log_service, sharing_service, user_management_service};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(list_global_activity))
}

pub fn vault_activity_routes() -> Router<AppState> {
    Router::new().route("/", get(list_vault_activity))
}

/// GET /v1/activity?page=&perPage=&userId=&action=
/// Admin-only: global activity log.
async fn list_global_activity(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ActivityLogQuery>,
) -> Result<Json<PaginatedActivityLogs>, AppError> {
    user_management_service::require_admin(&state.db, auth.user_id).await?;
    let result = activity_log_service::list_activity(&state.db, &query, None).await?;
    Ok(Json(result))
}

/// GET /v1/vaults/:vault_id/activity?page=&perPage=
/// Vault owner or admin: per-vault activity.
async fn list_vault_activity(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(vault_id): Path<Uuid>,
    Query(query): Query<ActivityLogQuery>,
) -> Result<Json<PaginatedActivityLogs>, AppError> {
    // Require at least manage-level access to the vault
    sharing_service::require_vault_admin(&state.db, vault_id, auth.user_id).await?;
    let result = activity_log_service::list_activity(&state.db, &query, Some(vault_id)).await?;
    Ok(Json(result))
}
