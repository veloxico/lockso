use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::session::Session;
use lockso_core::services::session_service;

/// Authenticated user extractor.
///
/// Extracts and validates the access token from the `Authorization: Bearer <token>` header.
/// Makes the session and user_id available to handlers.
///
/// **Security**: Blocks access if 2FA verification is still pending, except for
/// 2FA-related endpoints (verify, WebAuthn authenticate).
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub session: Session,
}

/// Exact paths allowed when 2FA is pending.
const TWO_FA_ALLOWED_EXACT: &[&str] = &[
    "/v1/2fa/verify",
    "/v1/2fa/status",
    "/v1/webauthn/authenticate/begin",
    "/v1/webauthn/authenticate/finish",
    "/v1/csrf-tokens/generate",
];

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        if token.is_empty() {
            return Err(AppError::Unauthorized);
        }

        let session = session_service::validate_access_token(&state.db, token).await?;

        // Block access if 2FA is still pending (except for 2FA endpoints)
        if session.is_two_factor_auth_required {
            let path = parts.uri.path();
            let allowed = TWO_FA_ALLOWED_EXACT.iter().any(|p| path == *p);
            if !allowed {
                return Err(AppError::TwoFactorRequired);
            }
        }

        Ok(AuthUser {
            user_id: session.user_id,
            session_id: session.id,
            session,
        })
    }
}
