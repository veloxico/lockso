use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health-check", get(health_check))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub is_bootstrapped: bool,
    pub database: ServiceStatus,
    pub redis: ServiceStatus,
    pub storage: ServiceStatus,
}

#[derive(Serialize, ToSchema)]
pub struct ServiceStatus {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ServiceStatus {
    fn ok() -> Self {
        Self { status: "ok", error: None }
    }
    fn err(e: impl std::fmt::Display, is_production: bool) -> Self {
        // Log full error server-side, return generic message to client in production
        tracing::warn!(error = %e, "Health check dependency failed");
        let error_msg = if is_production {
            "unavailable".to_string()
        } else {
            e.to_string()
        };
        Self { status: "error", error: Some(error_msg) }
    }
}

/// GET /v1/app/health-check
#[utoipa::path(
    get,
    path = "/v1/app/health-check",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    ),
    tag = "app"
)]
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let is_prod = state.config.env.is_production();

    let database = match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
    {
        Ok(_) => ServiceStatus::ok(),
        Err(e) => ServiceStatus::err(e, is_prod),
    };

    let redis = match redis::cmd("PING")
        .query_async::<String>(&mut state.redis.clone())
        .await
    {
        Ok(_) => ServiceStatus::ok(),
        Err(e) => ServiceStatus::err(e, is_prod),
    };

    let storage = match state.storage.health_check().await {
        Ok(_) => ServiceStatus::ok(),
        Err(e) => ServiceStatus::err(e, is_prod),
    };

    // Check if system is bootstrapped (settings exist = bootstrap has run)
    let is_bootstrapped = if database.status == "ok" {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM settings")
            .fetch_one(&state.db)
            .await
            .unwrap_or(0)
            > 0
    } else {
        false
    };

    let overall = if database.status == "ok" && redis.status == "ok" && storage.status == "ok" {
        "healthy"
    } else {
        "degraded"
    };

    Json(HealthResponse {
        status: overall,
        version: env!("CARGO_PKG_VERSION"),
        is_bootstrapped,
        database,
        redis,
        storage,
    })
}
