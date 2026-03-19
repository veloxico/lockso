use axum::{
    Extension, Json, Router,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    routing::post,
};
use std::net::SocketAddr;

use crate::middleware::rate_limit::RateLimiter;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::session::{LoginRequest, LoginResponse};
use lockso_core::models::user::{CreateUser, UserView};
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::services::{activity_log_service, auth_service, settings_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

/// POST /v1/users/login
async fn login(
    State(state): State<AppState>,
    Extension(rate_limiter): Extension<RateLimiter>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let client_ip = extract_client_ip(&headers, &addr, state.config.trust_proxy);

    // Rate limit login attempts per IP using lockout settings from DB
    let lockout = settings_service::get_lockout_settings(&state.db)
        .await
        .unwrap_or_default();

    rate_limiter
        .check_login_with_settings(
            &client_ip,
            lockout.enabled,
            lockout.max_attempts,
            lockout.window_seconds,
        )
        .await
        .map_err(|_| AppError::TooManyRequests)?;

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let ua = user_agent.clone();
    let result = auth_service::login(&state.db, input, Some(client_ip.clone()), user_agent).await;

    match &result {
        Ok(resp) => {
            activity_log_service::log_activity(
                &state.db, Some(resp.user.id), ActivityAction::LOGIN,
                None, None, None, Some(&client_ip), ua.as_deref(), serde_json::json!({}),
            ).await;
        }
        Err(_) => {
            activity_log_service::log_activity(
                &state.db, None, ActivityAction::LOGIN_FAILED,
                None, None, None, Some(&client_ip), ua.as_deref(), serde_json::json!({}),
            ).await;
        }
    }

    Ok(Json(result?))
}

/// POST /v1/users/register
async fn register(
    State(state): State<AppState>,
    Extension(rate_limiter): Extension<RateLimiter>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(input): Json<CreateUser>,
) -> Result<Json<UserView>, AppError> {
    let client_ip = extract_client_ip(&headers, &addr, state.config.trust_proxy);

    // Rate limit registration attempts per IP
    rate_limiter
        .check(&format!("register:{client_ip}"), 3, 300)
        .await
        .map_err(|_| AppError::TooManyRequests)?;

    // Check if registration is allowed:
    // - Always allowed if no users exist yet (first user / owner setup)
    // - Otherwise, requires "user.allowRegistration" setting to be true
    let user_count: (i64,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap_or((1,));
    if user_count.0 > 0 {
        let settings = lockso_core::services::settings_service::get_settings(&state.db).await?;
        let allow = settings
            .user
            .get("allowRegistration")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !allow {
            return Err(AppError::Forbidden);
        }
    }

    let user = auth_service::register(&state.db, input).await?;
    activity_log_service::log_activity(
        &state.db, Some(user.id), ActivityAction::REGISTER,
        Some("user"), Some(user.id), None, Some(&client_ip), None, serde_json::json!({}),
    ).await;
    Ok(Json(UserView::from(user)))
}

/// Extract client IP from X-Forwarded-For (only if trust_proxy is true) or socket address.
///
/// When `trust_proxy` is false, the X-Forwarded-For header is ignored to prevent
/// IP spoofing attacks that could bypass rate limiting.
pub fn extract_client_ip(headers: &HeaderMap, addr: &SocketAddr, trust_proxy: bool) -> String {
    if trust_proxy {
        if let Some(forwarded) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
        {
            if !forwarded.is_empty() {
                return forwarded;
            }
        }
    }
    addr.ip().to_string()
}
