use axum::{
    Extension, Json, Router,
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    routing::{delete, get, post},
};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::middleware::rate_limit::RateLimiter;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::models::session::{RefreshRequest, RefreshResponse, SessionView};
use lockso_core::services::{activity_log_service, auth_service, session_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_sessions).delete(delete_all_sessions))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/current/info", get(current_info))
        .route("/{id}", delete(delete_session))
}

/// POST /v1/sessions/refresh
async fn refresh(
    State(state): State<AppState>,
    Extension(rate_limiter): Extension<RateLimiter>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    let client_ip = crate::routes::auth::extract_client_ip(&headers, &addr, state.config.trust_proxy);

    // Rate limit refresh attempts per IP (10/min)
    rate_limiter
        .check(&format!("refresh:{client_ip}"), 10, 60)
        .await
        .map_err(|_| AppError::TooManyRequests)?;

    let response = auth_service::refresh_tokens(&state.db, input).await?;
    Ok(Json(response))
}

/// POST /v1/sessions/logout
async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    session_service::logout(&state.db, auth.session_id).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::LOGOUT,
        None, None, None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(serde_json::json!({"message": "logged out"})))
}

/// GET /v1/sessions/current/info
async fn current_info(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<SessionView>, AppError> {
    let info = session_service::get_session_info(&state.db, auth.session_id).await?;
    Ok(Json(info))
}

/// GET /v1/sessions
async fn list_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<SessionView>>, AppError> {
    let sessions =
        session_service::list_sessions(&state.db, auth.user_id, auth.session_id).await?;
    Ok(Json(sessions))
}

/// DELETE /v1/sessions/{id}
async fn delete_session(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    session_service::delete_session(&state.db, id, auth.user_id).await?;
    Ok(Json(serde_json::json!({"message": "session deleted"})))
}

/// DELETE /v1/sessions
async fn delete_all_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    let count =
        session_service::delete_other_sessions(&state.db, auth.user_id, auth.session_id).await?;
    Ok(Json(
        serde_json::json!({"message": "sessions deleted", "count": count}),
    ))
}

