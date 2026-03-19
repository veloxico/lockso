use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::session::{Session, SessionView};
use crate::services::settings_service;
use lockso_crypto::search_hash::hash_token;

/// Minimum interval between last_activity_at updates (60 seconds).
/// Prevents a write on every single authenticated request.
const ACTIVITY_UPDATE_DEBOUNCE_SECS: i64 = 60;

/// Validate an access token and return the session.
///
/// Checks:
/// 1. Session exists with this access token hash
/// 2. Access token has not expired
/// 3. For Web clients: inactivity TTL not exceeded
pub async fn validate_access_token(
    pool: &PgPool,
    access_token: &str,
) -> Result<Session, AppError> {
    let token_hash = hash_token(access_token);

    let session = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE access_token_hash = $1",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::SessionNotFound)?;

    let now = Utc::now();

    // Check access token expiry
    if session.access_token_expired_at < now {
        return Err(AppError::AccessTokenExpired);
    }

    // Check if the user is blocked
    let is_blocked: Option<(bool,)> =
        sqlx::query_as("SELECT is_blocked FROM users WHERE id = $1")
            .bind(session.user_id)
            .fetch_optional(pool)
            .await?;
    match is_blocked {
        Some((true,)) => {
            // Delete the session and reject
            sqlx::query("DELETE FROM sessions WHERE id = $1")
                .bind(session.id)
                .execute(pool)
                .await?;
            return Err(AppError::Unauthorized);
        }
        None => return Err(AppError::Unauthorized), // User deleted
        _ => {}
    }

    // Check inactivity TTL for Web clients
    if session.client_type == "Web" {
        let inactivity_ttl_secs: i64 = settings_service::get_session_settings(pool)
            .await
            .map(|s| s.inactivity_ttl)
            .unwrap_or(1800); // fallback 30 min
        if inactivity_ttl_secs > 0 {
            let inactive_since = now - session.last_activity_at;
            if inactive_since.num_seconds() > inactivity_ttl_secs {
                // Delete expired session
                sqlx::query("DELETE FROM sessions WHERE id = $1")
                    .bind(session.id)
                    .execute(pool)
                    .await?;
                return Err(AppError::AccessTokenExpired);
            }
        }
    }

    // Debounced last_activity_at update — only if more than DEBOUNCE seconds passed
    let since_last = (now - session.last_activity_at).num_seconds();
    if since_last >= ACTIVITY_UPDATE_DEBOUNCE_SECS {
        sqlx::query("UPDATE sessions SET last_activity_at = $1 WHERE id = $2")
            .bind(now)
            .bind(session.id)
            .execute(pool)
            .await?;
    }

    Ok(session)
}

/// List all sessions for a user.
pub async fn list_sessions(
    pool: &PgPool,
    user_id: Uuid,
    current_session_id: Uuid,
) -> Result<Vec<SessionView>, AppError> {
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let views = sessions
        .into_iter()
        .map(|s| {
            let is_current = s.id == current_session_id;
            SessionView {
                id: s.id,
                auth_method: s.auth_method,
                client_type: s.client_type,
                client_ip: s.client_ip,
                user_agent: s.user_agent,
                access_token_expired_at: s.access_token_expired_at,
                last_activity_at: s.last_activity_at,
                is_current,
                created_at: s.created_at,
            }
        })
        .collect();

    Ok(views)
}

/// Get current session info.
pub async fn get_session_info(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<SessionView, AppError> {
    let session = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::SessionNotFound)?;

    Ok(SessionView {
        id: session.id,
        auth_method: session.auth_method,
        client_type: session.client_type,
        client_ip: session.client_ip,
        user_agent: session.user_agent,
        access_token_expired_at: session.access_token_expired_at,
        last_activity_at: session.last_activity_at,
        is_current: true,
        created_at: session.created_at,
    })
}

/// Delete a specific session.
pub async fn delete_session(
    pool: &PgPool,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "DELETE FROM sessions WHERE id = $1 AND user_id = $2",
    )
    .bind(session_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::SessionNotFound);
    }

    Ok(())
}

/// Delete all sessions for a user except the current one.
pub async fn delete_other_sessions(
    pool: &PgPool,
    user_id: Uuid,
    current_session_id: Uuid,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "DELETE FROM sessions WHERE user_id = $1 AND id != $2",
    )
    .bind(user_id)
    .bind(current_session_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Logout — delete the current session.
pub async fn logout(pool: &PgPool, session_id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Generate and store a CSRF token for a session.
///
/// Also cleans up expired tokens to prevent table bloat.
pub async fn generate_csrf_token(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<String, AppError> {
    let token = lockso_crypto::random::generate_token()
        .map_err(|_| AppError::Internal("csrf token generation failed".into()))?;

    let token_hash = hash_token(&token);

    // Default CSRF TTL: 1 hour
    let expired_at = Utc::now() + chrono::Duration::seconds(3600);

    sqlx::query(
        "INSERT INTO csrf_tokens (id, token_hash, session_id, expired_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(&token_hash)
    .bind(session_id)
    .bind(expired_at)
    .execute(pool)
    .await?;

    // Cleanup: delete expired tokens periodically to prevent table bloat.
    // Runs when the last hex char of the token hash is '0' (~6.25% probability).
    if token_hash.as_bytes().last().is_some_and(|b| *b == b'0') {
        sqlx::query("DELETE FROM csrf_tokens WHERE expired_at < NOW()")
            .execute(pool)
            .await
            .ok(); // Best-effort, don't fail the request
    }

    Ok(token)
}

/// Validate and consume a CSRF token (single-use).
///
/// The token is deleted after successful validation to prevent replay.
pub async fn validate_csrf_token(
    pool: &PgPool,
    token: &str,
    session_id: Uuid,
) -> Result<(), AppError> {
    let token_hash = hash_token(token);

    // Delete and return in one query — atomic single-use
    let result = sqlx::query(
        "DELETE FROM csrf_tokens WHERE token_hash = $1 AND session_id = $2 AND expired_at > NOW()",
    )
    .bind(&token_hash)
    .bind(session_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::CsrfTokenInvalid);
    }

    Ok(())
}
