use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::send::{
    CreateSend, CreateSendResponse, Send, SendAccessView, SendListEntry, SendPublicMeta,
};

const MAX_CIPHERTEXT_LEN: usize = 100_000; // ~75KB of plaintext
const MAX_TTL_HOURS: i32 = 168; // 7 days
const MIN_TTL_HOURS: i32 = 1;
const MAX_MAX_VIEWS: i16 = 100;
const MIN_MAX_VIEWS: i16 = 1;

/// Create a new send.
pub async fn create_send(
    pool: &PgPool,
    creator_id: Uuid,
    input: CreateSend,
) -> Result<CreateSendResponse, AppError> {
    // Validate
    if input.ciphertext_b64.is_empty() {
        return Err(AppError::Validation("Ciphertext is required".into()));
    }
    if input.ciphertext_b64.len() > MAX_CIPHERTEXT_LEN {
        return Err(AppError::Validation("Payload too large".into()));
    }

    let ttl_hours = input.ttl_hours.unwrap_or(24).clamp(MIN_TTL_HOURS, MAX_TTL_HOURS);
    let max_views = input.max_views.unwrap_or(1).clamp(MIN_MAX_VIEWS, MAX_MAX_VIEWS);

    let access_id = lockso_crypto::random::secure_random_base64(24)
        .map_err(|_| AppError::Internal("failed to generate access id".into()))?;

    let passphrase_hash = if let Some(ref passphrase) = input.passphrase {
        if passphrase.is_empty() {
            None
        } else {
            let config = lockso_crypto::argon2::Argon2Config {
                memory_kib: 4096, // lighter config for send passphrases
                iterations: 2,
                parallelism: 1,
                output_len: 32,
            };
            Some(
                lockso_crypto::argon2::hash_password(passphrase, &config)
                    .map_err(|_| AppError::Internal("failed to hash passphrase".into()))?,
            )
        }
    } else {
        None
    };

    let id = Uuid::now_v7();
    let expires_at = Utc::now() + chrono::Duration::hours(ttl_hours as i64);

    sqlx::query(
        r#"INSERT INTO sends (id, creator_id, access_id, ciphertext_b64, passphrase_hash,
                              max_views, view_count, expires_at, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, 0, $7, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(creator_id)
    .bind(&access_id)
    .bind(&input.ciphertext_b64)
    .bind(&passphrase_hash)
    .bind(max_views)
    .bind(expires_at)
    .execute(pool)
    .await?;

    tracing::info!(send_id = %id, "Send created");

    Ok(CreateSendResponse { id, access_id })
}

/// List all sends for a user.
pub async fn list_sends(
    pool: &PgPool,
    creator_id: Uuid,
) -> Result<Vec<SendListEntry>, AppError> {
    let sends = sqlx::query_as::<_, Send>(
        "SELECT * FROM sends WHERE creator_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(creator_id)
    .fetch_all(pool)
    .await?;

    let now = Utc::now();
    let entries = sends
        .into_iter()
        .map(|s| SendListEntry {
            id: s.id,
            access_id: s.access_id,
            has_passphrase: s.passphrase_hash.is_some(),
            max_views: s.max_views,
            view_count: s.view_count,
            expires_at: s.expires_at,
            is_expired: s.expires_at < now,
            is_consumed: s.view_count >= s.max_views,
            created_at: s.created_at,
        })
        .collect();

    Ok(entries)
}

/// Delete (revoke) a send.
pub async fn delete_send(
    pool: &PgPool,
    send_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE sends SET deleted_at = NOW() WHERE id = $1 AND creator_id = $2 AND deleted_at IS NULL",
    )
    .bind(send_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::SendNotFound);
    }
    Ok(())
}

/// Get public metadata for a send (before passphrase check).
pub async fn get_send_meta(
    pool: &PgPool,
    access_id: &str,
) -> Result<SendPublicMeta, AppError> {
    let send = sqlx::query_as::<_, Send>(
        "SELECT * FROM sends WHERE access_id = $1 AND deleted_at IS NULL",
    )
    .bind(access_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::SendNotFound)?;

    let now = Utc::now();
    if send.expires_at < now {
        return Err(AppError::SendNotFound);
    }
    if send.view_count >= send.max_views {
        return Err(AppError::SendNotFound);
    }

    Ok(SendPublicMeta {
        has_passphrase: send.passphrase_hash.is_some(),
    })
}

/// Access a send (atomically increment view count, return ciphertext).
pub async fn access_send(
    pool: &PgPool,
    access_id: &str,
    passphrase: Option<&str>,
) -> Result<SendAccessView, AppError> {
    let send = sqlx::query_as::<_, Send>(
        "SELECT * FROM sends WHERE access_id = $1 AND deleted_at IS NULL",
    )
    .bind(access_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::SendNotFound)?;

    let now = Utc::now();
    if send.expires_at < now {
        return Err(AppError::SendNotFound);
    }
    if send.view_count >= send.max_views {
        return Err(AppError::SendNotFound);
    }

    // Verify passphrase if required
    if let Some(ref hash) = send.passphrase_hash {
        let pw = passphrase.ok_or(AppError::Validation(
            "Passphrase required".into(),
        ))?;
        let valid = lockso_crypto::argon2::verify_password(pw, hash)
            .map_err(|_| AppError::Internal("passphrase verification failed".into()))?;
        if !valid {
            return Err(AppError::Validation("Incorrect passphrase".into()));
        }
    }

    // Atomically increment view count and soft-delete if max reached
    sqlx::query(
        r#"UPDATE sends SET view_count = view_count + 1, updated_at = NOW(),
           deleted_at = CASE WHEN view_count + 1 >= max_views THEN NOW() ELSE deleted_at END
           WHERE id = $1"#,
    )
    .bind(send.id)
    .execute(pool)
    .await?;

    Ok(SendAccessView {
        ciphertext_b64: send.ciphertext_b64,
    })
}

/// Cleanup expired sends (background task).
pub async fn cleanup_expired_sends(pool: &PgPool) -> Result<u64, AppError> {
    let result = sqlx::query(
        "DELETE FROM sends WHERE (expires_at < NOW() OR deleted_at IS NOT NULL) AND created_at < NOW() - INTERVAL '1 day'",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
