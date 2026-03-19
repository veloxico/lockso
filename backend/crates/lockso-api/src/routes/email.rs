use axum::{Json, Router, extract::State, http::HeaderMap, routing::get};

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::email::{EmailSettingsView, SendTestEmail, UpdateEmailSettings};
use lockso_core::services::{email_service, user_management_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_email_settings).put(update_email_settings))
        .route("/test", axum::routing::post(test_email))
}

/// GET /v1/email — get current email config (secrets masked).
async fn get_email_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Option<EmailSettingsView>>, AppError> {
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let settings = email_service::get_settings(&state.db, &state.encryption_key).await?;
    Ok(Json(settings))
}

/// PUT /v1/email — update email config.
async fn update_email_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<UpdateEmailSettings>,
) -> Result<Json<EmailSettingsView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let result =
        email_service::update_settings(&state.db, &state.encryption_key, input).await?;
    Ok(Json(result))
}

/// POST /v1/email/test — send a test email.
async fn test_email(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<SendTestEmail>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    email_service::send_test_email(&state.db, &state.encryption_key, &input.to).await?;
    Ok(Json(serde_json::json!({"message": "Test email sent"})))
}

