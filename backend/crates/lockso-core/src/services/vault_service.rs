use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::vault::{CreateVault, UpdateVault, Vault, VaultListItem, VaultView};
use crate::services::{attachment_service, sharing_service};
use lockso_crypto::random::secure_random_bytes;
use lockso_db::storage::FileStorage;

/// Max vault name length.
const MAX_NAME_LENGTH: usize = 255;

/// Generate a random salt for blind search hashing (32 hex chars).
fn generate_vault_salt() -> Result<String, AppError> {
    let bytes = secure_random_bytes(16)
        .map_err(|_| AppError::Internal("salt generation failed".into()))?;
    Ok(hex::encode(bytes))
}

/// List all vaults accessible to a user (owned + shared, excluding Forbidden).
pub async fn list_vaults(pool: &PgPool, user_id: Uuid) -> Result<Vec<VaultListItem>, AppError> {
    let vaults = sqlx::query_as::<_, VaultListItem>(
        r#"SELECT
            v.id, v.name, v.description, v.vault_type_id, v.color_code, v.created_at,
            COALESCE(ic.cnt, 0) AS item_count,
            COALESCE(fc.cnt, 0) AS folder_count
        FROM vaults v
        LEFT JOIN (SELECT vault_id, COUNT(*) AS cnt FROM items GROUP BY vault_id) ic ON ic.vault_id = v.id
        LEFT JOIN (SELECT vault_id, COUNT(*) AS cnt FROM folders GROUP BY vault_id) fc ON fc.vault_id = v.id
        WHERE v.creator_id = $1
           OR v.id IN (
               SELECT rag.vault_id FROM resource_access_grants rag
               JOIN resource_accesses ra ON rag.resource_access_id = ra.id
               WHERE rag.vault_id IS NOT NULL
                 AND ra.code != 'forbidden'
                 AND (
                     rag.user_id = $1
                     OR rag.group_id IN (
                         SELECT group_id FROM user_group_members WHERE user_id = $1
                     )
                 )
           )
        ORDER BY v.name"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(vaults)
}

/// Get a single vault by ID with counts.
/// Checks both ownership and shared access.
pub async fn get_vault(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<VaultView, AppError> {
    // Verify access (throws VaultNotFound if no access)
    let _access = sharing_service::check_vault_access(pool, vault_id, user_id).await?;

    let vault = sqlx::query_as::<_, Vault>("SELECT * FROM vaults WHERE id = $1")
        .bind(vault_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::VaultNotFound)?;

    let item_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM items WHERE vault_id = $1")
            .bind(vault_id)
            .fetch_one(pool)
            .await?;

    let folder_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM folders WHERE vault_id = $1")
            .bind(vault_id)
            .fetch_one(pool)
            .await?;

    Ok(VaultView {
        id: vault.id,
        name: vault.name,
        description: vault.description,
        vault_type_id: vault.vault_type_id,
        creator_id: vault.creator_id,
        color_code: vault.color_code,
        item_count: item_count.0,
        folder_count: folder_count.0,
        created_at: vault.created_at,
        updated_at: vault.updated_at,
    })
}

/// Create a new vault.
pub async fn create_vault(
    pool: &PgPool,
    user_id: Uuid,
    input: CreateVault,
) -> Result<VaultView, AppError> {
    // Validate name
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Vault name is required".into()));
    }
    if input.name.len() > MAX_NAME_LENGTH {
        return Err(AppError::Validation(format!(
            "Vault name must not exceed {MAX_NAME_LENGTH} characters"
        )));
    }

    // Resolve vault type: accept UUID, "default" (→ organization), or type code string
    let vault_type_id: Uuid = match input.vault_type_id.as_str() {
        "default" | "organization" | "personal" | "private_shared" => {
            let code = if input.vault_type_id == "default" {
                "organization"
            } else {
                &input.vault_type_id
            };
            let row: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM vault_types WHERE code = $1 LIMIT 1",
            )
            .bind(code)
            .fetch_optional(pool)
            .await?;
            row.map(|r| r.0)
                .ok_or_else(|| AppError::Internal(format!("Vault type '{code}' not found")))?
        }
        _ => {
            input
                .vault_type_id
                .parse::<Uuid>()
                .map_err(|_| AppError::Validation("Invalid vault type ID".into()))?
        }
    };

    // Verify vault type exists
    let type_exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM vault_types WHERE id = $1")
            .bind(vault_type_id)
            .fetch_optional(pool)
            .await?;
    if type_exists.is_none() {
        return Err(AppError::Validation("Invalid vault type".into()));
    }

    // Check name uniqueness for this user
    let name_exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM vaults WHERE name = $1 AND creator_id = $2",
    )
    .bind(input.name.trim())
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    if name_exists.is_some() {
        return Err(AppError::VaultNameTaken);
    }

    let vault_id = Uuid::now_v7();
    let salt = generate_vault_salt()?;
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO vaults (id, name, description, vault_type_id, creator_id, salt, color_code, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)"#,
    )
    .bind(vault_id)
    .bind(input.name.trim())
    .bind(input.description.as_deref().unwrap_or(""))
    .bind(vault_type_id)
    .bind(user_id)
    .bind(&salt)
    .bind(input.color_code.unwrap_or(0))
    .bind(now)
    .execute(pool)
    .await?;

    tracing::info!(vault_id = %vault_id, user_id = %user_id, "Vault created");

    Ok(VaultView {
        id: vault_id,
        name: input.name.trim().to_string(),
        description: input.description.unwrap_or_default(),
        vault_type_id,
        creator_id: Some(user_id),
        color_code: input.color_code.unwrap_or(0),
        item_count: 0,
        folder_count: 0,
        created_at: now,
        updated_at: now,
    })
}

/// Update a vault. Requires write access or higher.
pub async fn update_vault(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
    input: UpdateVault,
) -> Result<VaultView, AppError> {
    // Verify write access
    sharing_service::require_write_access(pool, vault_id, user_id).await?;

    let vault = sqlx::query_as::<_, Vault>("SELECT * FROM vaults WHERE id = $1")
        .bind(vault_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::VaultNotFound)?;

    let new_name = input.name.as_deref().unwrap_or(&vault.name);
    let new_description = input.description.as_deref().unwrap_or(&vault.description);
    let new_color = input.color_code.unwrap_or(vault.color_code);

    if let Some(ref name) = input.name {
        if name.trim().is_empty() {
            return Err(AppError::Validation("Vault name is required".into()));
        }
        if name.len() > MAX_NAME_LENGTH {
            return Err(AppError::Validation(format!(
                "Vault name must not exceed {MAX_NAME_LENGTH} characters"
            )));
        }
        // Check uniqueness per creator (excluding self)
        let name_exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM vaults WHERE name = $1 AND creator_id = $2 AND id != $3",
        )
        .bind(name.trim())
        .bind(vault.creator_id)
        .bind(vault_id)
        .fetch_optional(pool)
        .await?;
        if name_exists.is_some() {
            return Err(AppError::VaultNameTaken);
        }
    }

    let now = Utc::now();
    sqlx::query("UPDATE vaults SET name = $1, description = $2, color_code = $3, updated_at = $4 WHERE id = $5")
        .bind(new_name.trim())
        .bind(new_description)
        .bind(new_color)
        .bind(now)
        .bind(vault_id)
        .execute(pool)
        .await?;

    get_vault(pool, vault_id, user_id).await
}

/// Delete a vault and all its contents (cascading).
/// Only the vault creator (owner) can delete a vault.
/// Cleans up S3 attachment objects before DB cascade to prevent orphans.
pub async fn delete_vault(
    pool: &PgPool,
    storage: &FileStorage,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    // Verify the creator owns this vault before doing any cleanup
    let exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM vaults WHERE id = $1 AND creator_id = $2")
            .bind(vault_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    if exists.is_none() {
        return Err(AppError::VaultNotFound);
    }

    // Clean up S3 attachment objects BEFORE cascade delete removes DB records
    attachment_service::delete_all_for_vault(pool, storage, vault_id).await.ok();

    // Now delete the vault — items, folders, snapshots, etc. cascade
    sqlx::query("DELETE FROM vaults WHERE id = $1 AND creator_id = $2")
        .bind(vault_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    tracing::info!(vault_id = %vault_id, user_id = %user_id, "Vault deleted");
    Ok(())
}

/// Get the vault salt for blind search hashing.
pub async fn get_vault_salt(
    pool: &PgPool,
    vault_id: Uuid,
) -> Result<String, AppError> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT salt FROM vaults WHERE id = $1")
            .bind(vault_id)
            .fetch_optional(pool)
            .await?;

    row.map(|r| r.0).ok_or(AppError::VaultNotFound)
}
