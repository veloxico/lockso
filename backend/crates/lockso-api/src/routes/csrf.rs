use axum::{Json, Router, extract::State, routing::post};

use crate::extractors::AuthUser;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::services::session_service;

pub fn routes() -> Router<AppState> {
    Router::new().route("/generate", post(generate))
}

/// POST /v1/csrf-tokens/generate
async fn generate(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let token = session_service::generate_csrf_token(&state.db, auth.session_id).await?;
    Ok(Json(serde_json::json!({ "token": token })))
}
