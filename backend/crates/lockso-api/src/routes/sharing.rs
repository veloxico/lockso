use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::access_grant::{AccessGrantView, GrantAccessRequest};
use lockso_core::models::resource_access::ResourceAccess;
use lockso_core::models::vault_access::{
    ShareVaultRequest, UpdateAccessRequest, VaultMember,
};
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::services::{access_service, activity_log_service, sharing_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/access-levels", get(list_access_levels))
        .route(
            "/{vault_id}/members",
            get(list_members).post(share_vault),
        )
        .route(
            "/{vault_id}/members/{user_id}",
            axum::routing::put(update_access).delete(revoke_access),
        )
        // Group sharing
        .route("/{vault_id}/groups", axum::routing::post(share_vault_with_group))
        // Unified grants
        .route("/{vault_id}/grants", get(list_vault_grants))
        .route("/grants/{grant_id}", axum::routing::delete(revoke_grant))
        // Folder/item grants
        .route("/folders/{folder_id}/grants", get(list_folder_grants).post(grant_folder_access))
        .route("/items/{item_id}/grants", get(list_item_grants).post(grant_item_access))
}

/// GET /v1/sharing/access-levels
async fn list_access_levels(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<Vec<ResourceAccess>>, AppError> {
    let levels = sqlx::query_as::<_, ResourceAccess>(
        "SELECT * FROM resource_accesses ORDER BY priority DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(levels))
}

/// GET /v1/sharing/:vault_id/members
async fn list_members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(vault_id): Path<Uuid>,
) -> Result<Json<Vec<VaultMember>>, AppError> {
    sharing_service::check_vault_access(&state.db, vault_id, auth.user_id).await?;
    let members = sharing_service::list_vault_members(&state.db, vault_id).await?;
    Ok(Json(members))
}

/// POST /v1/sharing/:vault_id/members
async fn share_vault(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(vault_id): Path<Uuid>,
    Json(input): Json<ShareVaultRequest>,
) -> Result<Json<VaultMember>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let member = sharing_service::share_vault(
        &state.db,
        vault_id,
        auth.user_id,
        input.user_id,
        input.resource_access_id,
    )
    .await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::SHARING_GRANTED,
        Some("vault"), Some(vault_id), Some(vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"targetUserId": input.user_id.to_string()}),
    ).await;
    Ok(Json(member))
}

/// PUT /v1/sharing/:vault_id/members/:user_id
async fn update_access(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path((vault_id, user_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateAccessRequest>,
) -> Result<Json<VaultMember>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let member = sharing_service::update_member_access(
        &state.db,
        vault_id,
        auth.user_id,
        user_id,
        input.resource_access_id,
    )
    .await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::SHARING_UPDATED,
        Some("vault"), Some(vault_id), Some(vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"targetUserId": user_id.to_string()}),
    ).await;
    Ok(Json(member))
}

/// DELETE /v1/sharing/:vault_id/members/:user_id
async fn revoke_access(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path((vault_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    sharing_service::revoke_access(&state.db, vault_id, auth.user_id, user_id).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::SHARING_REVOKED,
        Some("vault"), Some(vault_id), Some(vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"targetUserId": user_id.to_string()}),
    ).await;
    Ok(())
}

// ─── Group sharing ───

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShareGroupRequest {
    group_id: Uuid,
    resource_access_id: Uuid,
}

/// POST /v1/sharing/:vault_id/groups
async fn share_vault_with_group(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(vault_id): Path<Uuid>,
    Json(input): Json<ShareGroupRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    sharing_service::require_vault_admin(&state.db, vault_id, auth.user_id).await?;

    let grant = access_service::grant_vault_access(
        &state.db,
        vault_id,
        "group",
        input.group_id,
        input.resource_access_id,
        auth.user_id,
    )
    .await?;

    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::SHARING_GRANTED,
        Some("vault"), Some(vault_id), Some(vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"targetGroupId": input.group_id.to_string()}),
    ).await;

    Ok(Json(serde_json::json!({"id": grant.id})))
}

// ─── Unified grants ───

/// GET /v1/sharing/:vault_id/grants
async fn list_vault_grants(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(vault_id): Path<Uuid>,
) -> Result<Json<Vec<AccessGrantView>>, AppError> {
    sharing_service::check_vault_access(&state.db, vault_id, auth.user_id).await?;
    let grants = access_service::list_vault_grants(&state.db, vault_id).await?;
    Ok(Json(grants))
}

/// DELETE /v1/sharing/grants/:grant_id
async fn revoke_grant(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(grant_id): Path<Uuid>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    // Look up the grant to find which vault it belongs to
    let grant_vault: Option<(Option<Uuid>, Option<Uuid>, Option<Uuid>)> = sqlx::query_as(
        "SELECT vault_id, folder_id, item_id FROM resource_access_grants WHERE id = $1",
    )
    .bind(grant_id)
    .fetch_optional(&state.db)
    .await?;

    let (vault_id_opt, folder_id_opt, item_id_opt) =
        grant_vault.ok_or(AppError::NotFound("grant not found".into()))?;

    // Resolve the vault that this grant belongs to
    let vault_id = if let Some(vid) = vault_id_opt {
        vid
    } else if let Some(fid) = folder_id_opt {
        let row: Option<(Uuid,)> =
            sqlx::query_as("SELECT vault_id FROM folders WHERE id = $1")
                .bind(fid)
                .fetch_optional(&state.db)
                .await?;
        row.ok_or(AppError::FolderNotFound)?.0
    } else if let Some(iid) = item_id_opt {
        let row: Option<(Uuid,)> =
            sqlx::query_as("SELECT vault_id FROM items WHERE id = $1")
                .bind(iid)
                .fetch_optional(&state.db)
                .await?;
        row.ok_or(AppError::ItemNotFound)?.0
    } else {
        return Err(AppError::Internal("grant has no resource".into()));
    };

    // Verify actor has admin access to the vault
    sharing_service::require_vault_admin(&state.db, vault_id, auth.user_id).await?;

    access_service::revoke_grant(&state.db, grant_id).await?;

    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::SHARING_REVOKED,
        Some("grant"), Some(grant_id), Some(vault_id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"grantId": grant_id.to_string()}),
    ).await;

    Ok(())
}

/// GET /v1/sharing/folders/:folder_id/grants
async fn list_folder_grants(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(folder_id): Path<Uuid>,
) -> Result<Json<Vec<AccessGrantView>>, AppError> {
    // Get the vault_id from the folder to check access
    let folder_info: Option<(Uuid,)> = sqlx::query_as(
        "SELECT vault_id FROM folders WHERE id = $1",
    )
    .bind(folder_id)
    .fetch_optional(&state.db)
    .await?;
    let (vault_id,) = folder_info.ok_or(AppError::FolderNotFound)?;
    sharing_service::check_vault_access(&state.db, vault_id, auth.user_id).await?;

    let grants = access_service::list_folder_grants(&state.db, folder_id).await?;
    Ok(Json(grants))
}

/// POST /v1/sharing/folders/:folder_id/grants
async fn grant_folder_access(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(folder_id): Path<Uuid>,
    Json(input): Json<GrantAccessRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    let folder_info: Option<(Uuid,)> = sqlx::query_as(
        "SELECT vault_id FROM folders WHERE id = $1",
    )
    .bind(folder_id)
    .fetch_optional(&state.db)
    .await?;
    let (vault_id,) = folder_info.ok_or(AppError::FolderNotFound)?;
    sharing_service::require_vault_admin(&state.db, vault_id, auth.user_id).await?;

    let grant = access_service::grant_folder_access(
        &state.db,
        folder_id,
        &input.grantee_type,
        input.grantee_id,
        input.resource_access_id,
        auth.user_id,
    )
    .await?;

    Ok(Json(serde_json::json!({"id": grant.id})))
}

/// GET /v1/sharing/items/:item_id/grants
async fn list_item_grants(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(item_id): Path<Uuid>,
) -> Result<Json<Vec<AccessGrantView>>, AppError> {
    let item_info: Option<(Uuid,)> = sqlx::query_as(
        "SELECT vault_id FROM items WHERE id = $1",
    )
    .bind(item_id)
    .fetch_optional(&state.db)
    .await?;
    let (vault_id,) = item_info.ok_or(AppError::ItemNotFound)?;
    sharing_service::check_vault_access(&state.db, vault_id, auth.user_id).await?;

    let grants = access_service::list_item_grants(&state.db, item_id).await?;
    Ok(Json(grants))
}

/// POST /v1/sharing/items/:item_id/grants
async fn grant_item_access(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
    Json(input): Json<GrantAccessRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    let item_info: Option<(Uuid,)> = sqlx::query_as(
        "SELECT vault_id FROM items WHERE id = $1",
    )
    .bind(item_id)
    .fetch_optional(&state.db)
    .await?;
    let (vault_id,) = item_info.ok_or(AppError::ItemNotFound)?;
    sharing_service::require_vault_admin(&state.db, vault_id, auth.user_id).await?;

    let grant = access_service::grant_item_access(
        &state.db,
        item_id,
        &input.grantee_type,
        input.grantee_id,
        input.resource_access_id,
        auth.user_id,
    )
    .await?;

    Ok(Json(serde_json::json!({"id": grant.id})))
}
