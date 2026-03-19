use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
};

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::settings::Settings;
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::services::{activity_log_service, settings_service, user_management_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_settings))
        .route("/{category}", axum::routing::put(update_category))
}

/// GET /v1/settings
///
/// Returns full settings. Requires admin/owner role.
async fn get_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Settings>, AppError> {
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let settings = settings_service::get_settings(&state.db).await?;
    Ok(Json(settings))
}

/// PUT /v1/settings/:category
///
/// Updates a single settings category. Requires admin/owner role.
async fn update_category(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(category): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> Result<Json<Settings>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let settings =
        settings_service::update_settings_category(&state.db, &category, value).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::SETTINGS_UPDATED,
        Some("settings"), None, None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"category": category}),
    ).await;
    Ok(Json(settings))
}
