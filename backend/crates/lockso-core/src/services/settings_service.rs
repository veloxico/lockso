use serde_json::Value;
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::settings::{PasswordComplexity, SessionSettings, Settings, TrashSettings, UserLockoutSettings};

/// Allowed settings categories for partial update.
const ALLOWED_CATEGORIES: &[&str] = &[
    "session",
    "email",
    "interface",
    "notification",
    "user_lockout",
    "auth_password_complexity",
    "master_password_complexity",
    "browser_extension",
    "favicon",
    "search",
    "vault",
    "user",
    "trash",
];

/// Get the singleton settings row.
pub async fn get_settings(pool: &PgPool) -> Result<Settings, AppError> {
    let settings = sqlx::query_as::<_, Settings>("SELECT * FROM settings LIMIT 1")
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Internal("Settings not initialized".into()))?;

    Ok(settings)
}

/// Update a single settings category by column name.
///
/// Validates that the category name is allowed (prevents SQL injection)
/// and that the new value is valid JSON.
pub async fn update_settings_category(
    pool: &PgPool,
    category: &str,
    value: Value,
) -> Result<Settings, AppError> {
    // Validate category name against allowlist
    if !ALLOWED_CATEGORIES.contains(&category) {
        return Err(AppError::Validation(format!(
            "Invalid settings category: {category}"
        )));
    }

    // Validate typed settings where we have strong types
    match category {
        "session" => {
            serde_json::from_value::<SessionSettings>(value.clone())
                .map_err(|e| AppError::Validation(format!("Invalid session settings: {e}")))?;
        }
        "auth_password_complexity" | "master_password_complexity" => {
            serde_json::from_value::<PasswordComplexity>(value.clone())
                .map_err(|e| AppError::Validation(format!("Invalid password complexity: {e}")))?;
        }
        "user_lockout" => {
            serde_json::from_value::<UserLockoutSettings>(value.clone())
                .map_err(|e| AppError::Validation(format!("Invalid lockout settings: {e}")))?;
        }
        "trash" => {
            serde_json::from_value::<TrashSettings>(value.clone())
                .map_err(|e| AppError::Validation(format!("Invalid trash settings: {e}")))?;
        }
        _ => {
            // Other categories: accept any valid JSON object
            if !value.is_object() {
                return Err(AppError::Validation(
                    "Settings value must be a JSON object".into(),
                ));
            }
        }
    }

    // Use a quoted column name to handle "user" (reserved keyword)
    // Safe because category is validated against allowlist
    let query = format!(
        r#"UPDATE settings SET "{category}" = $1, updated_at = NOW()
           RETURNING *"#
    );

    let settings = sqlx::query_as::<_, Settings>(&query)
        .bind(&value)
        .fetch_one(pool)
        .await?;

    tracing::info!(category = category, "Settings updated");
    Ok(settings)
}

/// Get a typed settings value by category.
pub async fn get_session_settings(pool: &PgPool) -> Result<SessionSettings, AppError> {
    let settings = get_settings(pool).await?;
    serde_json::from_value(settings.session)
        .map_err(|_| AppError::Internal("Failed to parse session settings".into()))
}

pub async fn get_password_complexity(pool: &PgPool) -> Result<PasswordComplexity, AppError> {
    let settings = get_settings(pool).await?;
    serde_json::from_value(settings.auth_password_complexity)
        .map_err(|_| AppError::Internal("Failed to parse password complexity".into()))
}

pub async fn get_lockout_settings(pool: &PgPool) -> Result<UserLockoutSettings, AppError> {
    let settings = get_settings(pool).await?;
    serde_json::from_value(settings.user_lockout)
        .map_err(|_| AppError::Internal("Failed to parse lockout settings".into()))
}

pub async fn get_trash_settings(pool: &PgPool) -> Result<TrashSettings, AppError> {
    let settings = get_settings(pool).await?;
    serde_json::from_value(settings.trash)
        .map_err(|_| AppError::Internal("Failed to parse trash settings".into()))
}
