use axum::{
    Json, Router,
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    routing::{delete, get, post},
};
use std::net::SocketAddr;
use uuid::Uuid;

use axum::Extension;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::middleware::rate_limit::RateLimiter;
use crate::routes::auth::extract_client_ip;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::models::send::{
    CreateSend, CreateSendResponse, SendAccessView, SendListEntry, SendPassphraseRequest,
    SendPublicMeta,
};
use lockso_core::services::{activity_log_service, send_service};

/// Authenticated routes: /v1/sends
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_sends).post(create_send))
        .route("/{id}", delete(delete_send))
}

/// Public routes: /v1/public/sends (no auth required)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/{access_id}", get(get_send_meta))
        .route("/{access_id}/access", post(access_send))
}

/// GET /v1/sends
async fn list_sends(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<SendListEntry>>, AppError> {
    let sends = send_service::list_sends(&state.db, auth.user_id).await?;
    Ok(Json(sends))
}

/// POST /v1/sends
async fn create_send(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<CreateSend>,
) -> Result<Json<CreateSendResponse>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let resp = send_service::create_send(&state.db, auth.user_id, input).await?;
    activity_log_service::log_activity(
        &state.db,
        Some(auth.user_id),
        ActivityAction::SEND_CREATED,
        Some("send"),
        Some(resp.id),
        None,
        auth.session.client_ip.as_deref(),
        auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    )
    .await;
    Ok(Json(resp))
}

/// DELETE /v1/sends/:id
async fn delete_send(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    send_service::delete_send(&state.db, id, auth.user_id).await?;
    activity_log_service::log_activity(
        &state.db,
        Some(auth.user_id),
        ActivityAction::SEND_DELETED,
        Some("send"),
        Some(id),
        None,
        auth.session.client_ip.as_deref(),
        auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    )
    .await;
    Ok(())
}

/// GET /v1/public/sends/:access_id — metadata (no auth)
async fn get_send_meta(
    State(state): State<AppState>,
    Path(access_id): Path<String>,
) -> Result<Json<SendPublicMeta>, AppError> {
    let meta = send_service::get_send_meta(&state.db, &access_id).await?;
    Ok(Json(meta))
}

/// POST /v1/public/sends/:access_id/access — get ciphertext (no auth)
async fn access_send(
    State(state): State<AppState>,
    Extension(rate_limiter): Extension<RateLimiter>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(access_id): Path<String>,
    body: Option<Json<SendPassphraseRequest>>,
) -> Result<Json<SendAccessView>, AppError> {
    // Rate limit: 5 attempts per minute per IP+access_id
    let client_ip = extract_client_ip(&headers, &addr, state.config.trust_proxy);
    rate_limiter
        .check(&format!("send_access:{client_ip}:{access_id}"), 5, 60)
        .await
        .map_err(|_| AppError::TooManyRequests)?;

    let passphrase = body.as_ref().map(|b| b.passphrase.as_str());
    let view = send_service::access_send(&state.db, &access_id, passphrase).await?;
    activity_log_service::log_activity(
        &state.db,
        None,
        ActivityAction::SEND_ACCESSED,
        Some("send"),
        None,
        None,
        None,
        None,
        serde_json::json!({}),
    )
    .await;
    Ok(Json(view))
}
