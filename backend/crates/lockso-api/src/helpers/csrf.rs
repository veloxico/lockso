use axum::http::HeaderMap;

use crate::extractors::auth::AuthUser;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::services::session_service;

/// Validate CSRF token from X-CSRF-Token header on state-changing requests.
pub async fn validate_csrf(
    state: &AppState,
    auth: &AuthUser,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    let csrf_token = headers
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::CsrfTokenInvalid)?;

    session_service::validate_csrf_token(&state.db, csrf_token, auth.session_id).await
}
