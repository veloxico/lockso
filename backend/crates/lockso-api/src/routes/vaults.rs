use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
};
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::vault::{CreateVault, UpdateVault, VaultListItem, VaultView};
use lockso_core::models::vault_type::VaultTypeView;
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::services::{activity_log_service, vault_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_vaults).post(create_vault))
        .route("/{id}", get(get_vault).put(update_vault).delete(delete_vault))
}

pub fn vault_type_routes() -> Router<AppState> {
    Router::new().route("/", get(list_vault_types))
}

/// GET /v1/vault-types
async fn list_vault_types(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<Vec<VaultTypeView>>, AppError> {
    let types = sqlx::query_as::<_, VaultTypeView>(
        "SELECT id, name, code FROM vault_types ORDER BY name",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(types))
}

/// GET /v1/vaults
async fn list_vaults(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<VaultListItem>>, AppError> {
    let vaults = vault_service::list_vaults(&state.db, auth.user_id).await?;
    Ok(Json(vaults))
}

/// GET /v1/vaults/:id
async fn get_vault(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<VaultView>, AppError> {
    let vault = vault_service::get_vault(&state.db, id, auth.user_id).await?;
    Ok(Json(vault))
}

/// POST /v1/vaults
async fn create_vault(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<CreateVault>,
) -> Result<Json<VaultView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let vault = vault_service::create_vault(&state.db, auth.user_id, input).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::VAULT_CREATED,
        Some("vault"), Some(vault.id), Some(vault.id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(vault))
}

/// PUT /v1/vaults/:id
async fn update_vault(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateVault>,
) -> Result<Json<VaultView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let vault = vault_service::update_vault(&state.db, id, auth.user_id, input).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::VAULT_UPDATED,
        Some("vault"), Some(id), Some(id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(vault))
}

/// DELETE /v1/vaults/:id
async fn delete_vault(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    vault_service::delete_vault(&state.db, &state.storage, id, auth.user_id).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::VAULT_DELETED,
        Some("vault"), Some(id), Some(id),
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(())
}
