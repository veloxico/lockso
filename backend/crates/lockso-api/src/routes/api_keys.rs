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
use lockso_core::models::api_key::{ApiKeyCreated, ApiKeyView, CreateApiKey};
use lockso_core::services::api_key_service;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_keys).post(create_key))
        .route("/{id}", axum::routing::delete(delete_key))
}

/// GET /v1/api-keys
async fn list_keys(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ApiKeyView>>, AppError> {
    let keys = api_key_service::list_keys(&state.db, auth.user_id).await?;
    Ok(Json(keys))
}

/// POST /v1/api-keys
async fn create_key(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<CreateApiKey>,
) -> Result<Json<ApiKeyCreated>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let key = api_key_service::create_key(&state.db, auth.user_id, input).await?;
    Ok(Json(key))
}

/// DELETE /v1/api-keys/:id
async fn delete_key(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    api_key_service::delete_key(&state.db, id, auth.user_id).await
}
