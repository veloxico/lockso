use axum::{
    Extension, Json, Router,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
};

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::middleware::rate_limit::RateLimiter;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::services::activity_log_service;
use lockso_core::services::totp_service::{
    self, DisableTotpRequest, EnableTotpRequestV2, TotpSetupResponse, TotpStatus,
    VerifyTotpRequest,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/setup", post(setup))
        .route("/enable", post(enable))
        .route("/verify", post(verify))
        .route("/disable", post(disable))
}

/// GET /v1/2fa/status
///
/// Get current 2FA status.
async fn get_status(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<TotpStatus>, AppError> {
    let status = totp_service::get_totp_status(&state.db, auth.user_id).await?;
    Ok(Json(status))
}

/// POST /v1/2fa/setup
///
/// Begin 2FA setup — generates secret and QR code URI.
async fn setup(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<TotpSetupResponse>, AppError> {
    let response = totp_service::setup_totp(&state.db, auth.user_id).await?;
    Ok(Json(response))
}

/// POST /v1/2fa/enable
///
/// Confirm 2FA setup with verification code.
async fn enable(
    State(state): State<AppState>,
    Extension(rate_limiter): Extension<RateLimiter>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<EnableTotpRequestV2>,
) -> Result<Json<TotpStatus>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    // Rate limit: 5 attempts per 60 seconds per user
    rate_limiter
        .check(&format!("totp_enable:{}", auth.user_id), 5, 60)
        .await
        .map_err(|_| AppError::TooManyRequests)?;

    let status = totp_service::enable_totp(
        &state.db,
        &state.encryption_key,
        auth.user_id,
        &input.secret,
        &input.code,
        &input.recovery_codes,
    )
    .await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::TOTP_ENABLED,
        None, None, None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(status))
}

/// POST /v1/2fa/verify
///
/// Verify a TOTP code (used after login when 2FA is required).
async fn verify(
    State(state): State<AppState>,
    Extension(rate_limiter): Extension<RateLimiter>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<VerifyTotpRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    // Rate limit: 5 attempts per 60 seconds per user
    rate_limiter
        .check(&format!("totp_verify:{}", auth.user_id), 5, 60)
        .await
        .map_err(|_| AppError::TooManyRequests)?;

    let valid = totp_service::verify_totp_code(
        &state.db,
        &state.redis,
        &state.encryption_key,
        auth.user_id,
        &input.code,
    )
    .await?;

    if !valid {
        activity_log_service::log_activity(
            &state.db, Some(auth.user_id), ActivityAction::TOTP_FAILED,
            None, None, None,
            auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
            serde_json::json!({}),
        ).await;
        return Err(AppError::Validation("Invalid 2FA code".into()));
    }
    // Mark 2FA as satisfied for this session
    sqlx::query(
        "UPDATE sessions SET is_two_factor_auth_required = FALSE WHERE id = $1",
    )
    .bind(auth.session_id)
    .execute(&state.db)
    .await?;

    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::TOTP_VERIFIED,
        None, None, None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(serde_json::json!({ "verified": true })))
}

/// POST /v1/2fa/disable
///
/// Disable 2FA (requires valid code).
async fn disable(
    State(state): State<AppState>,
    Extension(rate_limiter): Extension<RateLimiter>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<DisableTotpRequest>,
) -> Result<Json<TotpStatus>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    // Rate limit: 5 attempts per 60 seconds per user
    rate_limiter
        .check(&format!("totp_disable:{}", auth.user_id), 5, 60)
        .await
        .map_err(|_| AppError::TooManyRequests)?;

    let status = totp_service::disable_totp(
        &state.db,
        &state.redis,
        &state.encryption_key,
        auth.user_id,
        &input.code,
    )
    .await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::TOTP_DISABLED,
        None, None, None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(status))
}
