use anyhow::Result;
use axum::{Router, extract::DefaultBodyLimit, serve};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use axum::http::HeaderValue;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod extractors;
mod health;
mod helpers;
mod middleware;
mod routes;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,lockso=debug".into()))
        .with(fmt::layer().json())
        .init();

    let state = state::AppState::init().await?;

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        env = ?if state.config.env.is_production() { "production" } else { "development" },
        "Lockso starting"
    );

    // Spawn background tasks
    tokio::spawn(background_tasks(state.clone()));

    let cors = build_cors(&state.config.app_url);

    let rate_limiter = middleware::rate_limit::RateLimiter::new(state.redis.clone());

    let app = Router::new()
        .nest("/v1", api_routes())
        // 55 MB body limit (50 MB file + multipart overhead)
        .layer(DefaultBodyLimit::max(55 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        // Security headers
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ))
        .layer(cors)
        .layer(axum::Extension(rate_limiter))
        .with_state(state);

    let bind = std::env::var("LOCKSO_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = TcpListener::bind(&bind).await?;
    tracing::info!(bind = %bind, "Listening");

    let app = app.into_make_service_with_connect_info::<SocketAddr>();
    serve(listener, app).await?;

    Ok(())
}

fn api_routes() -> Router<state::AppState> {
    Router::new()
        .nest("/app", health::routes())
        .nest("/users", routes::auth::routes())
        .nest("/sessions", routes::sessions::routes())
        .nest("/csrf-tokens", routes::csrf::routes())
        .nest("/vaults", routes::vaults::routes())
        .nest("/vaults", routes::import_export::routes())
        .nest("/vault-types", routes::vaults::vault_type_routes())
        .nest("/folders", routes::folders::routes())
        .nest("/items", routes::items::routes())
        .nest("/trash", routes::trash::routes())
        .nest("/health-report", routes::health_report::routes())
        .nest("/sends", routes::sends::routes())
        .nest("/public/sends", routes::sends::public_routes())
        .merge(routes::attachments::routes())
        .nest("/settings", routes::settings::routes())
        .nest("/email", routes::email::routes())
        .nest("/groups", routes::groups::routes())
        .nest("/sharing", routes::sharing::routes())
        .nest("/activity", routes::activity::routes())
        .nest("/vaults/{vault_id}/activity", routes::activity::vault_activity_routes())
        .nest("/2fa", routes::totp::routes())
        .nest("/webauthn", routes::webauthn::routes())
        .nest("/users", routes::users::routes())
        .nest("/webhooks", routes::webhooks::routes())
        .nest("/api-keys", routes::api_keys::routes())
}

fn build_cors(app_url: &str) -> CorsLayer {
    use axum::http::{HeaderName, Method, HeaderValue};

    let origin = app_url
        .parse::<HeaderValue>()
        .expect("APP_URL must be a valid header value");

    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-csrf-token"),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

/// Background tasks: session cleanup, CSRF cleanup.
async fn background_tasks(state: state::AppState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
    loop {
        interval.tick().await;

        // Clean expired sessions
        match sqlx::query("DELETE FROM sessions WHERE refresh_token_expired_at < NOW()")
            .execute(&state.db)
            .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    tracing::info!(count = result.rows_affected(), "Cleaned expired sessions");
                }
            }
            Err(e) => tracing::warn!(error = %e, "Failed to clean expired sessions"),
        }

        // Clean old activity logs (retain 90 days)
        match sqlx::query("DELETE FROM activity_logs WHERE created_at < NOW() - INTERVAL '90 days'")
            .execute(&state.db)
            .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    tracing::info!(count = result.rows_affected(), "Cleaned old activity logs");
                }
            }
            Err(e) => tracing::warn!(error = %e, "Failed to clean old activity logs"),
        }

        // Clean expired CSRF tokens
        match sqlx::query("DELETE FROM csrf_tokens WHERE expired_at < NOW()")
            .execute(&state.db)
            .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    tracing::debug!(count = result.rows_affected(), "Cleaned expired CSRF tokens");
                }
            }
            Err(e) => tracing::warn!(error = %e, "Failed to clean expired CSRF tokens"),
        }

        // Clean expired sends
        match lockso_core::services::send_service::cleanup_expired_sends(&state.db).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!(count = count, "Cleaned expired sends");
                }
            }
            Err(e) => tracing::warn!(error = %e, "Failed to clean expired sends"),
        }

        // Auto-purge expired trash items
        match lockso_core::services::settings_service::get_trash_settings(&state.db).await {
            Ok(trash_settings) => {
                if trash_settings.auto_empty_enabled {
                    match lockso_core::services::item_service::auto_purge_trash(
                        &state.db,
                        Some(&state.storage),
                        trash_settings.retention_days,
                    ).await {
                        Ok(count) => {
                            if count > 0 {
                                tracing::info!(count = count, "Auto-purged expired trash items");
                            }
                        }
                        Err(e) => tracing::warn!(error = %e, "Failed to auto-purge trash"),
                    }
                }
            }
            Err(e) => tracing::warn!(error = %e, "Failed to read trash settings for auto-purge"),
        }
    }
}
