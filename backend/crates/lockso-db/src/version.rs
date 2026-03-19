use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

/// Application version record stored in `_lockso_version`.
#[derive(Debug, FromRow)]
pub struct AppVersion {
    pub app_version: String,
    pub db_schema_version: i64,
    pub updated_at: DateTime<Utc>,
}

/// Read the current version from the database.
pub async fn get_version(pool: &PgPool) -> Result<Option<AppVersion>> {
    let row = sqlx::query_as::<_, AppVersion>(
        "SELECT app_version, db_schema_version, updated_at
         FROM _lockso_version
         ORDER BY updated_at DESC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Upsert the version record after migration.
pub async fn set_version(pool: &PgPool, app_version: &str, schema_version: i64) -> Result<()> {
    sqlx::query(
        "INSERT INTO _lockso_version (app_version, db_schema_version, updated_at)
         VALUES ($1, $2, NOW())
         ON CONFLICT (app_version) DO UPDATE
         SET db_schema_version = $2, updated_at = NOW()",
    )
    .bind(app_version)
    .bind(schema_version)
    .execute(pool)
    .await?;
    Ok(())
}
