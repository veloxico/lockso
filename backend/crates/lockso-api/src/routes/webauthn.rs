use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{delete, get, post},
};
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::middleware::rate_limit::RateLimiter;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::webauthn::*;
use lockso_core::services::webauthn_service;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register/begin", post(begin_registration))
        .route("/register/finish", post(finish_registration))
        .route("/authenticate/begin", post(begin_authentication))
        .route("/authenticate/finish", post(finish_authentication))
        .route("/credentials", get(list_credentials))
        .route("/credentials/{id}", delete(delete_credential))
        .route("/credentials/{id}/name", axum::routing::put(rename_credential))
}

/// POST /v1/webauthn/register/begin
async fn begin_registration(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Result<Json<RegistrationOptionsResponse>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    let user = lockso_core::services::user_management_service::get_user(&state.db, auth.user_id)
        .await?
        .ok_or(AppError::UserNotFound)?;

    let rp_id = extract_rp_id(&state.config.app_url);

    let options = webauthn_service::begin_registration(
        &state.db,
        auth.user_id,
        &user.login,
        &user.full_name,
        &rp_id,
        "Lockso",
        auth.session_id,
    )
    .await?;

    Ok(Json(options))
}

/// POST /v1/webauthn/register/finish
async fn finish_registration(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(response): Json<RegistrationResponse>,
) -> Result<Json<WebAuthnCredentialView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    let rp_id = extract_rp_id(&state.config.app_url);

    let cred = webauthn_service::finish_registration(
        &state.db,
        auth.user_id,
        auth.session_id,
        &rp_id,
        response,
        state.config.env.is_production(),
    )
    .await?;

    Ok(Json(cred))
}

/// POST /v1/webauthn/authenticate/begin
async fn begin_authentication(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Result<Json<AuthenticationOptionsResponse>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    let rp_id = extract_rp_id(&state.config.app_url);

    let options = webauthn_service::begin_authentication(
        &state.db,
        auth.user_id,
        &rp_id,
        auth.session_id,
    )
    .await?;

    Ok(Json(options))
}

/// POST /v1/webauthn/authenticate/finish
async fn finish_authentication(
    State(state): State<AppState>,
    Extension(rate_limiter): Extension<RateLimiter>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(response): Json<AuthenticationResponse>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    // Rate limit: 5 attempts per 60 seconds per user
    rate_limiter
        .check(&format!("webauthn_auth:{}", auth.user_id), 5, 60)
        .await
        .map_err(|_| AppError::TooManyRequests)?;

    let rp_id = extract_rp_id(&state.config.app_url);

    webauthn_service::finish_authentication(
        &state.db,
        auth.user_id,
        auth.session_id,
        &rp_id,
        response,
    )
    .await?;

    // Mark 2FA as satisfied for this session
    sqlx::query(
        "UPDATE sessions SET is_two_factor_auth_required = FALSE WHERE id = $1",
    )
    .bind(auth.session_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"verified": true})))
}

/// GET /v1/webauthn/credentials
async fn list_credentials(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<WebAuthnCredentialView>>, AppError> {
    let creds = webauthn_service::list_credentials(&state.db, auth.user_id).await?;
    Ok(Json(creds))
}

/// DELETE /v1/webauthn/credentials/{id}
async fn delete_credential(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    webauthn_service::delete_credential(&state.db, id, auth.user_id).await?;
    Ok(Json(serde_json::json!({"message": "Credential deleted"})))
}

/// PUT /v1/webauthn/credentials/{id}/name
async fn rename_credential(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;

    let name = body["name"]
        .as_str()
        .ok_or(AppError::Validation("name is required".into()))?;

    webauthn_service::rename_credential(&state.db, id, auth.user_id, name).await?;
    Ok(Json(serde_json::json!({"message": "Credential renamed"})))
}

/// Extract the RP ID (hostname) from the app URL.
fn extract_rp_id(app_url: &str) -> String {
    url::Url::parse(app_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "localhost".to_string())
}
