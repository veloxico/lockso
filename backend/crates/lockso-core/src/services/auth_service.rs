use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::password_validator::validate_password;
use crate::error::AppError;
use crate::models::enums::ClientType;
use crate::models::session::{LoginRequest, LoginResponse, RefreshRequest, RefreshResponse};
use crate::models::settings::PasswordComplexity;
use crate::models::user::{CreateUser, User, UserView};
use crate::services::bootstrap;
use lockso_crypto::argon2::{Argon2Config, hash_password, verify_password};
use lockso_crypto::random::generate_token;
use lockso_crypto::search_hash::hash_token;

/// Maximum password length to prevent Argon2id DoS.
const MAX_PASSWORD_LENGTH: usize = 256;

/// Maximum sessions per user before oldest is evicted.
const MAX_SESSIONS_PER_USER: i64 = 50;

/// Register a new user.
///
/// If this is the first user, runs bootstrap (under advisory lock) and assigns Owner role.
/// Otherwise assigns the default User role.
pub async fn register(pool: &PgPool, input: CreateUser) -> Result<User, AppError> {
    // ─── Input validation ───
    validate_login(&input.login)?;
    validate_email(input.email.as_deref())?;
    validate_full_name(input.full_name.as_deref())?;

    if input.password.len() > MAX_PASSWORD_LENGTH {
        return Err(AppError::Validation(format!(
            "Password must not exceed {MAX_PASSWORD_LENGTH} characters"
        )));
    }

    // ─── Password complexity validation ───
    let complexity = load_password_complexity(pool).await;
    validate_password(&input.password, &complexity)
        .map_err(|e| AppError::PasswordComplexityFailed(e.violations.join("; ")))?;

    // ─── Uniqueness checks ───
    let exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE login = $1")
            .bind(&input.login)
            .fetch_optional(pool)
            .await?;

    if exists.is_some() {
        return Err(AppError::LoginAlreadyTaken);
    }

    if let Some(ref email) = input.email {
        let email_exists: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(pool)
                .await?;

        if email_exists.is_some() {
            return Err(AppError::EmailAlreadyTaken);
        }
    }

    // ─── Bootstrap + role assignment + INSERT in one atomic operation ───
    // Hash password BEFORE entering the transaction (Argon2id is CPU-heavy,
    // we don't want to hold the advisory lock during hashing)
    let config = Argon2Config::default();
    let password_hash = hash_password(&input.password, &config)
        .map_err(|_| AppError::Internal("password hashing failed".into()))?;

    let user_id = Uuid::now_v7();
    let now = Utc::now();

    // Start a transaction with advisory lock to prevent dual-Owner race:
    // bootstrap + COUNT(users) + INSERT must be atomic.
    let mut tx = pool.begin().await?;

    // Advisory lock — same key as bootstrap, serializes all first registrations
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(bootstrap::BOOTSTRAP_LOCK_ID)
        .execute(&mut *tx)
        .await?;

    // Bootstrap if needed (idempotent inside lock)
    let settings_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM settings")
        .fetch_one(&mut *tx)
        .await?;
    if settings_count.0 == 0 {
        bootstrap::run_bootstrap_in_tx(&mut tx).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    // Determine role based on actual user count — inside the same lock
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await?;

    let role_id = if user_count.0 == 0 {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM user_roles WHERE code = 'owner' LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await?;
        row.map(|r| r.0)
            .ok_or_else(|| AppError::Internal("Owner role not found after bootstrap".into()))?
    } else {
        let row: (Uuid,) = sqlx::query_as(
            "SELECT id FROM user_roles WHERE code = 'user' LIMIT 1",
        )
        .fetch_one(&mut *tx)
        .await?;
        row.0
    };

    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (
            id, login, password_hash, email, full_name,
            master_key_hash, keys_public, keys_private_encrypted,
            signup_type, role_id, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'Default', $9, $10, $10)
        RETURNING *"#,
    )
    .bind(user_id)
    .bind(&input.login)
    .bind(&password_hash)
    .bind(&input.email)
    .bind(input.full_name.as_deref().unwrap_or(""))
    .bind(&input.master_key_hash)
    .bind(&input.keys_public)
    .bind(&input.keys_private_encrypted)
    .bind(role_id)
    .bind(now)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(user_id = %user.id, login = %user.login, "User registered");
    Ok(user)
}

/// Authenticate a user with login and password.
///
/// Returns the same error for missing user, blocked user, and wrong password
/// to prevent user enumeration.
pub async fn login(
    pool: &PgPool,
    input: LoginRequest,
    client_ip: Option<String>,
    user_agent: Option<String>,
) -> Result<LoginResponse, AppError> {
    // Prevent Argon2id DoS with oversized passwords
    if input.password.len() > MAX_PASSWORD_LENGTH {
        return Err(AppError::InvalidLoginOrPassword);
    }

    // Validate client_type
    let client_type = ClientType::from_str(&input.client_type)
        .ok_or_else(|| AppError::Validation("Invalid client type".into()))?;

    // Truncate user_agent to prevent storage abuse
    let user_agent = user_agent.map(|ua| truncate_string(ua, 512));

    // Find user by login
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE login = $1")
        .bind(&input.login)
        .fetch_optional(pool)
        .await?;

    // Constant-time-ish: always verify even if user not found to prevent timing oracle
    let (user, valid) = match user {
        Some(u) => {
            if u.is_blocked {
                // Return same error as invalid credentials to prevent enumeration
                return Err(AppError::InvalidLoginOrPassword);
            }
            let v = verify_password(&input.password, &u.password_hash)
                .unwrap_or(false);
            (Some(u), v)
        }
        None => {
            // Perform dummy hash to equalize timing
            let _ = verify_password(
                &input.password,
                "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            );
            (None, false)
        }
    };

    if !valid {
        return Err(AppError::InvalidLoginOrPassword);
    }

    let user = user.expect("user is Some when valid is true");

    // Generate tokens
    let access_token = generate_token()
        .map_err(|_| AppError::Internal("token generation failed".into()))?;
    let refresh_token = generate_token()
        .map_err(|_| AppError::Internal("token generation failed".into()))?;

    // Read session TTL from role's auth_settings (not user's personal settings)
    let role_auth: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT auth_settings FROM user_roles WHERE id = $1",
    )
    .bind(user.role_id)
    .fetch_optional(pool)
    .await?;

    let access_ttl_secs: i64 = role_auth
        .as_ref()
        .and_then(|(v,)| v.get("accessTokenTtl")?.as_i64())
        .unwrap_or(3600);
    let refresh_ttl_secs: i64 = role_auth
        .as_ref()
        .and_then(|(v,)| v.get("refreshTokenTtl")?.as_i64())
        .unwrap_or(2_592_000);

    let now = Utc::now();
    let access_expired_at = now + Duration::seconds(access_ttl_secs);
    let refresh_expired_at = now + Duration::seconds(refresh_ttl_secs);

    // Store session with token hashes (never store raw tokens)
    let session_id = Uuid::now_v7();
    let access_hash = hash_token(&access_token);
    let refresh_hash = hash_token(&refresh_token);

    // Enforce session count limit — evict oldest if at limit
    evict_excess_sessions(pool, user.id, MAX_SESSIONS_PER_USER - 1).await?;

    // Check if 2FA is enabled — must be done before session insert
    let is_two_factor_required = user
        .auth_settings
        .get("totpEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Also check if user has any WebAuthn credentials (FIDO2 as 2FA)
    let has_webauthn: bool = if !is_two_factor_required {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM webauthn_credentials WHERE user_id = $1",
        )
        .bind(user.id)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));
        count.0 > 0
    } else {
        false
    };
    let requires_2fa = is_two_factor_required || has_webauthn;

    sqlx::query(
        r#"INSERT INTO sessions (
            id, user_id, access_token_hash, refresh_token_hash,
            auth_method, client_type, client_ip, user_agent,
            is_two_factor_auth_required,
            access_token_expired_at, refresh_token_expired_at,
            last_activity_at, created_at
        ) VALUES ($1, $2, $3, $4, 'Local', $5, $6, $7, $8, $9, $10, $11, $11)"#,
    )
    .bind(session_id)
    .bind(user.id)
    .bind(&access_hash)
    .bind(&refresh_hash)
    .bind(client_type.as_str())
    .bind(&client_ip)
    .bind(&user_agent)
    .bind(requires_2fa)
    .bind(access_expired_at)
    .bind(refresh_expired_at)
    .bind(now)
    .execute(pool)
    .await?;

    // Update last_login_at
    sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
        .bind(now)
        .bind(user.id)
        .execute(pool)
        .await?;

    let is_master_key_required = user.master_key_hash.is_some();

    tracing::info!(
        user_id = %user.id,
        session_id = %session_id,
        client_type = %client_type.as_str(),
        "User logged in"
    );

    Ok(LoginResponse {
        access_token,
        refresh_token,
        access_token_expired_at: access_expired_at,
        refresh_token_expired_at: refresh_expired_at,
        user: UserView::from(user),
        is_two_factor_auth_required: requires_2fa,
        is_master_key_required,
    })
}

/// Refresh an access token using a valid refresh token.
pub async fn refresh_tokens(
    pool: &PgPool,
    input: RefreshRequest,
) -> Result<RefreshResponse, AppError> {
    let refresh_hash = hash_token(&input.refresh_token);

    // Find session by refresh token hash and check expiry in single query
    let session: Option<(Uuid, Uuid, bool)> = sqlx::query_as(
        "SELECT id, user_id, refresh_token_expired_at < NOW() AS expired FROM sessions WHERE refresh_token_hash = $1",
    )
    .bind(&refresh_hash)
    .fetch_optional(pool)
    .await?;

    let (session_id, user_id, expired) = session.ok_or(AppError::SessionNotFound)?;

    if expired {
        // Delete expired session
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(session_id)
            .execute(pool)
            .await?;
        return Err(AppError::RefreshTokenExpired);
    }

    // Check if the user is blocked — prevent token refresh for blocked users
    let is_blocked: Option<(bool,)> =
        sqlx::query_as("SELECT is_blocked FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    match is_blocked {
        Some((true,)) => {
            sqlx::query("DELETE FROM sessions WHERE id = $1")
                .bind(session_id)
                .execute(pool)
                .await?;
            return Err(AppError::Unauthorized);
        }
        None => return Err(AppError::Unauthorized), // User deleted
        _ => {}
    }

    // Read TTLs from user's role auth_settings
    let user_auth: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT ur.auth_settings FROM users u JOIN user_roles ur ON u.role_id = ur.id WHERE u.id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let access_ttl_secs: i64 = user_auth
        .as_ref()
        .and_then(|(v,)| v.get("accessTokenTtl")?.as_i64())
        .unwrap_or(3600);
    let refresh_ttl_secs: i64 = user_auth
        .as_ref()
        .and_then(|(v,)| v.get("refreshTokenTtl")?.as_i64())
        .unwrap_or(2_592_000);

    // Generate new tokens (rotate both)
    let new_access = generate_token()
        .map_err(|_| AppError::Internal("token generation failed".into()))?;
    let new_refresh = generate_token()
        .map_err(|_| AppError::Internal("token generation failed".into()))?;

    let now = Utc::now();
    let access_expired_at = now + Duration::seconds(access_ttl_secs);
    let refresh_expired_at = now + Duration::seconds(refresh_ttl_secs);

    let new_access_hash = hash_token(&new_access);
    let new_refresh_hash = hash_token(&new_refresh);

    // Update session with new tokens atomically
    let result = sqlx::query(
        r#"UPDATE sessions SET
            access_token_hash = $1,
            refresh_token_hash = $2,
            access_token_expired_at = $3,
            refresh_token_expired_at = $4,
            last_activity_at = $5
        WHERE id = $6 AND refresh_token_hash = $7"#,
    )
    .bind(&new_access_hash)
    .bind(&new_refresh_hash)
    .bind(access_expired_at)
    .bind(refresh_expired_at)
    .bind(now)
    .bind(session_id)
    .bind(&refresh_hash) // Verify old token still matches (prevents race)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        // Concurrent refresh detected — session was already refreshed
        return Err(AppError::SessionNotFound);
    }

    Ok(RefreshResponse {
        access_token: new_access,
        refresh_token: new_refresh,
        access_token_expired_at: access_expired_at,
        refresh_token_expired_at: refresh_expired_at,
    })
}

// ─── Input validation helpers ───

fn validate_login(login: &str) -> Result<(), AppError> {
    if login.is_empty() || login.len() < 2 {
        return Err(AppError::Validation("Login must be at least 2 characters".into()));
    }
    if login.len() > 100 {
        return Err(AppError::Validation("Login must not exceed 100 characters".into()));
    }
    if !login.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
        return Err(AppError::Validation(
            "Login may only contain letters, digits, underscores, hyphens, and dots".into(),
        ));
    }
    Ok(())
}

fn validate_email(email: Option<&str>) -> Result<(), AppError> {
    if let Some(email) = email {
        if email.len() > 255 {
            return Err(AppError::Validation("Email must not exceed 255 characters".into()));
        }
        // Basic email validation: contains @ with text on both sides
        if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
            return Err(AppError::Validation("Invalid email format".into()));
        }
        let parts: Vec<&str> = email.splitn(2, '@').collect();
        if parts.len() != 2 || parts[1].is_empty() || !parts[1].contains('.') {
            return Err(AppError::Validation("Invalid email format".into()));
        }
    }
    Ok(())
}

fn validate_full_name(name: Option<&str>) -> Result<(), AppError> {
    if let Some(name) = name {
        if name.len() > 255 {
            return Err(AppError::Validation("Full name must not exceed 255 characters".into()));
        }
    }
    Ok(())
}

/// Load password complexity rules from settings, fallback to defaults.
async fn load_password_complexity(pool: &PgPool) -> PasswordComplexity {
    let result: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT auth_password_complexity FROM settings LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    result
        .and_then(|(v,)| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Evict oldest sessions if user has more than `max_allowed`.
async fn evict_excess_sessions(pool: &PgPool, user_id: Uuid, max_allowed: i64) -> Result<(), AppError> {
    sqlx::query(
        r#"DELETE FROM sessions WHERE id IN (
            SELECT id FROM sessions
            WHERE user_id = $1
            ORDER BY last_activity_at DESC
            OFFSET $2
        )"#,
    )
    .bind(user_id)
    .bind(max_allowed)
    .execute(pool)
    .await?;
    Ok(())
}

/// Truncate a string to at most `max_bytes` bytes, ensuring valid UTF-8.
/// Never panics on multi-byte characters.
fn truncate_string(s: String, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s;
    }
    // Find the last valid char boundary at or before max_bytes
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}
