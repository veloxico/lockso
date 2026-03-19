use axum::{
    Json, Router,
    extract::State,
    routing::get,
};

use crate::extractors::auth::AuthUser;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::services::health_report_service::{self, HealthReport};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_health_report))
}

/// GET /v1/health-report
async fn get_health_report(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<HealthReport>, AppError> {
    let report = health_report_service::generate_report(
        &state.db,
        &state.encryption_key,
        auth.user_id,
    )
    .await?;
    Ok(Json(report))
}
