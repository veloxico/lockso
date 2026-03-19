use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::api_key::*;

/// Generate a cryptographically random API key.
fn generate_raw_key() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    format!("lk_{}", hex::encode(bytes))
}

/// SHA-256 hash of a raw key.
fn hash_key(raw: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

/// List all API keys for a user.
pub async fn list_keys(pool: &PgPool, user_id: Uuid) -> Result<Vec<ApiKeyView>, AppError> {
    let rows = sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(to_view).collect())
}

/// Create a new API key. Returns the raw key (shown once).
pub async fn create_key(
    pool: &PgPool,
    user_id: Uuid,
    input: CreateApiKey,
) -> Result<ApiKeyCreated, AppError> {
    if input.name.trim().is_empty() || input.name.len() > 100 {
        return Err(AppError::Validation("Name is required (max 100 chars)".into()));
    }
    if !VALID_PERMISSIONS.contains(&input.permission.as_str()) {
        return Err(AppError::Validation(format!(
            "Permission must be one of: {:?}",
            VALID_PERMISSIONS
        )));
    }

    // Check max keys per user (limit 20)
    let count: (i64,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM api_keys WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if count.0 >= 20 {
        return Err(AppError::Validation("Maximum 20 API keys per user".into()));
    }

    let raw_key = generate_raw_key();
    let key_hash = hash_key(&raw_key);
    let key_prefix = raw_key[..8.min(raw_key.len())].to_string();
    let id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO api_keys (id, name, key_hash, key_prefix, user_id, permission, vault_id, expires_at, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(id)
    .bind(input.name.trim())
    .bind(&key_hash)
    .bind(&key_prefix)
    .bind(user_id)
    .bind(&input.permission)
    .bind(input.vault_id)
    .bind(input.expires_at)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(ApiKeyCreated {
        id,
        name: input.name.trim().to_string(),
        key: raw_key,
        key_prefix,
        permission: input.permission,
        vault_id: input.vault_id,
        expires_at: input.expires_at,
        created_at: now,
    })
}

/// Delete an API key.
pub async fn delete_key(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM api_keys WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("API key not found".into()));
    }
    Ok(())
}

/// Validate an API key from the `Authorization: Bearer lk_...` header.
/// Returns (user_id, permission, vault_id) if valid.
pub async fn validate_api_key(
    pool: &PgPool,
    raw_key: &str,
) -> Result<(Uuid, String, Option<Uuid>), AppError> {
    let key_hash = hash_key(raw_key);

    let key = sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE key_hash = $1",
    )
    .bind(&key_hash)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !key.is_enabled {
        return Err(AppError::Unauthorized);
    }

    // Check expiry
    if let Some(expires) = key.expires_at {
        if Utc::now() > expires {
            return Err(AppError::Unauthorized);
        }
    }

    // Update last_used_at (fire and forget)
    let pool2 = pool.clone();
    let key_id = key.id;
    tokio::spawn(async move {
        let _ = sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key_id)
            .execute(&pool2)
            .await;
    });

    Ok((key.user_id, key.permission, key.vault_id))
}

fn to_view(key: ApiKey) -> ApiKeyView {
    ApiKeyView {
        id: key.id,
        name: key.name,
        key_prefix: key.key_prefix,
        user_id: key.user_id,
        permission: key.permission,
        vault_id: key.vault_id,
        expires_at: key.expires_at,
        last_used_at: key.last_used_at,
        is_enabled: key.is_enabled,
        created_at: key.created_at,
    }
}
