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
use lockso_core::models::webhook::{CreateWebhook, UpdateWebhook, WebhookView};
use lockso_core::services::{user_management_service, webhook_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_webhooks).post(create_webhook))
        .route(
            "/{id}",
            axum::routing::put(update_webhook).delete(delete_webhook),
        )
        .route("/{id}/test", axum::routing::post(test_webhook))
}

/// GET /v1/webhooks
async fn list_webhooks(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<WebhookView>>, AppError> {
    user_management_service::require_admin(&state.db, auth.user_id).await?;
    let webhooks = webhook_service::list_webhooks(&state.db, &state.encryption_key).await?;
    Ok(Json(webhooks))
}

/// POST /v1/webhooks
async fn create_webhook(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<CreateWebhook>,
) -> Result<Json<WebhookView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let webhook = webhook_service::create_webhook(
        &state.db,
        &state.encryption_key,
        auth.user_id,
        input,
    )
    .await?;

    Ok(Json(webhook))
}

/// PUT /v1/webhooks/:id
async fn update_webhook(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateWebhook>,
) -> Result<Json<WebhookView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let webhook = webhook_service::update_webhook(
        &state.db,
        &state.encryption_key,
        id,
        input,
    )
    .await?;

    Ok(Json(webhook))
}

/// DELETE /v1/webhooks/:id
async fn delete_webhook(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;
    webhook_service::delete_webhook(&state.db, id).await
}

/// POST /v1/webhooks/:id/test
async fn test_webhook(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;
    webhook_service::test_webhook(&state.db, &state.encryption_key, id).await
}
