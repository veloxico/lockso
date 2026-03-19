use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::access_grant::{AccessGrant, AccessGrantView};
use crate::services::group_service;

/// Effective access result for a user on a resource.
#[derive(Debug, Clone)]
pub struct EffectiveAccess {
    pub priority: i32,
    pub code: String,
}

// ─── Vault-level resolution ───

/// Resolve a user's effective access to a vault.
///
/// Checks: vault creator (→ "owner" / priority 100), then MAX of all direct +
/// group-based grants from resource_access_grants.
pub async fn resolve_vault_access(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<EffectiveAccess, AppError> {
    // 1. Check if creator (implicit owner)
    let is_creator: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM vaults WHERE id = $1 AND creator_id = $2")
            .bind(vault_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    if is_creator.is_some() {
        return Ok(EffectiveAccess {
            priority: 100,
            code: "owner".to_string(),
        });
    }

    // 2. Get user's group IDs
    let group_ids = group_service::get_user_group_ids(pool, user_id).await?;

    // 3. Find best grant (direct user OR any group)
    let best: Option<(i32, String)> = if group_ids.is_empty() {
        sqlx::query_as(
            r#"SELECT ra.priority, ra.code
            FROM resource_access_grants rag
            JOIN resource_accesses ra ON rag.resource_access_id = ra.id
            WHERE rag.vault_id = $1 AND rag.user_id = $2
            ORDER BY ra.priority DESC
            LIMIT 1"#,
        )
        .bind(vault_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"SELECT ra.priority, ra.code
            FROM resource_access_grants rag
            JOIN resource_accesses ra ON rag.resource_access_id = ra.id
            WHERE rag.vault_id = $1
              AND (rag.user_id = $2 OR rag.group_id = ANY($3))
            ORDER BY ra.priority DESC
            LIMIT 1"#,
        )
        .bind(vault_id)
        .bind(user_id)
        .bind(&group_ids)
        .fetch_optional(pool)
        .await?
    };

    match best {
        Some((priority, code)) if code != "forbidden" => Ok(EffectiveAccess { priority, code }),
        _ => Err(AppError::VaultNotFound),
    }
}

/// Check vault access — returns the access code string.
/// Drop-in replacement for sharing_service::check_vault_access.
pub async fn check_vault_access(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<String, AppError> {
    let access = resolve_vault_access(pool, vault_id, user_id).await?;
    Ok(access.code)
}

/// Require at least write-level access to a vault.
pub async fn require_write_access(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let code = check_vault_access(pool, vault_id, user_id).await?;
    match code.as_str() {
        "owner" | "admin" | "manage" | "write" => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}

/// Require admin-level access to a vault.
pub async fn require_vault_admin(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let code = check_vault_access(pool, vault_id, user_id).await?;
    match code.as_str() {
        "owner" | "admin" => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}

// ─── Folder-level resolution ───

/// Resolve a user's effective access to a folder.
///
/// Looks for the most specific grant: folder-level first, then walks up
/// ancestor folders, then falls back to vault-level.
pub async fn resolve_folder_access(
    pool: &PgPool,
    folder_id: Uuid,
    user_id: Uuid,
) -> Result<EffectiveAccess, AppError> {
    // Get folder info (vault_id, ancestor_ids)
    let folder_info: Option<(Uuid, serde_json::Value)> = sqlx::query_as(
        "SELECT vault_id, ancestor_ids FROM folders WHERE id = $1",
    )
    .bind(folder_id)
    .fetch_optional(pool)
    .await?;

    let (vault_id, ancestor_ids_json) =
        folder_info.ok_or(AppError::FolderNotFound)?;

    let group_ids = group_service::get_user_group_ids(pool, user_id).await?;

    // Check this folder specifically
    if let Some(access) =
        find_grant_for_resource(pool, None, Some(folder_id), None, user_id, &group_ids).await?
    {
        return Ok(access);
    }

    // Walk up ancestors (from closest to root)
    if let Some(ancestors) = ancestor_ids_json.as_array() {
        // ancestor_ids is ordered from root to parent; we reverse to check closest first
        for ancestor_val in ancestors.iter().rev() {
            if let Some(ancestor_str) = ancestor_val.as_str() {
                if let Ok(ancestor_id) = Uuid::parse_str(ancestor_str) {
                    if let Some(access) = find_grant_for_resource(
                        pool, None, Some(ancestor_id), None, user_id, &group_ids,
                    )
                    .await?
                    {
                        return Ok(access);
                    }
                }
            }
        }
    }

    // Fall back to vault-level
    resolve_vault_access(pool, vault_id, user_id).await
}

// ─── Item-level resolution ───

/// Resolve a user's effective access to an item.
///
/// Checks: item-level → folder-level (with ancestors) → vault-level.
pub async fn resolve_item_access(
    pool: &PgPool,
    item_id: Uuid,
    user_id: Uuid,
) -> Result<EffectiveAccess, AppError> {
    let item_info: Option<(Uuid, Option<Uuid>)> = sqlx::query_as(
        "SELECT vault_id, folder_id FROM items WHERE id = $1",
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await?;

    let (vault_id, folder_id) = item_info.ok_or(AppError::ItemNotFound)?;

    let group_ids = group_service::get_user_group_ids(pool, user_id).await?;

    // Check item-level grant
    if let Some(access) =
        find_grant_for_resource(pool, None, None, Some(item_id), user_id, &group_ids).await?
    {
        return Ok(access);
    }

    // Check folder chain
    if let Some(fid) = folder_id {
        // We can reuse resolve_folder_access, but to avoid double group_id fetch,
        // we inline the logic here
        let folder_info: Option<(serde_json::Value,)> = sqlx::query_as(
            "SELECT ancestor_ids FROM folders WHERE id = $1",
        )
        .bind(fid)
        .fetch_optional(pool)
        .await?;

        if let Some((ancestor_ids_json,)) = folder_info {
            // Check the direct folder
            if let Some(access) =
                find_grant_for_resource(pool, None, Some(fid), None, user_id, &group_ids).await?
            {
                return Ok(access);
            }

            // Walk up ancestors
            if let Some(ancestors) = ancestor_ids_json.as_array() {
                for ancestor_val in ancestors.iter().rev() {
                    if let Some(ancestor_str) = ancestor_val.as_str() {
                        if let Ok(ancestor_id) = Uuid::parse_str(ancestor_str) {
                            if let Some(access) = find_grant_for_resource(
                                pool, None, Some(ancestor_id), None, user_id, &group_ids,
                            )
                            .await?
                            {
                                return Ok(access);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fall back to vault-level
    resolve_vault_access(pool, vault_id, user_id).await
}

// ─── Grant management ───

/// Grant access on a vault to a user or group.
pub async fn grant_vault_access(
    pool: &PgPool,
    vault_id: Uuid,
    grantee_type: &str,
    grantee_id: Uuid,
    resource_access_id: Uuid,
    granted_by: Uuid,
) -> Result<AccessGrant, AppError> {
    validate_grantee(pool, grantee_type, grantee_id).await?;
    validate_access_level(pool, resource_access_id).await?;

    let (user_id, group_id) = split_grantee(grantee_type, grantee_id);

    let id = Uuid::now_v7();
    let grant = sqlx::query_as::<_, AccessGrant>(
        r#"INSERT INTO resource_access_grants (id, vault_id, user_id, group_id, resource_access_id, granted_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (vault_id, folder_id, item_id, user_id, group_id)
        DO UPDATE SET resource_access_id = $5, granted_by = $6, updated_at = NOW()
        RETURNING *"#,
    )
    .bind(id)
    .bind(vault_id)
    .bind(user_id)
    .bind(group_id)
    .bind(resource_access_id)
    .bind(granted_by)
    .fetch_one(pool)
    .await?;

    Ok(grant)
}

/// Grant access on a folder to a user or group.
pub async fn grant_folder_access(
    pool: &PgPool,
    folder_id: Uuid,
    grantee_type: &str,
    grantee_id: Uuid,
    resource_access_id: Uuid,
    granted_by: Uuid,
) -> Result<AccessGrant, AppError> {
    validate_grantee(pool, grantee_type, grantee_id).await?;
    validate_access_level(pool, resource_access_id).await?;

    let (user_id, group_id) = split_grantee(grantee_type, grantee_id);

    let id = Uuid::now_v7();
    let grant = sqlx::query_as::<_, AccessGrant>(
        r#"INSERT INTO resource_access_grants (id, folder_id, user_id, group_id, resource_access_id, granted_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (vault_id, folder_id, item_id, user_id, group_id)
        DO UPDATE SET resource_access_id = $5, granted_by = $6, updated_at = NOW()
        RETURNING *"#,
    )
    .bind(id)
    .bind(folder_id)
    .bind(user_id)
    .bind(group_id)
    .bind(resource_access_id)
    .bind(granted_by)
    .fetch_one(pool)
    .await?;

    Ok(grant)
}

/// Grant access on an item to a user or group.
pub async fn grant_item_access(
    pool: &PgPool,
    item_id: Uuid,
    grantee_type: &str,
    grantee_id: Uuid,
    resource_access_id: Uuid,
    granted_by: Uuid,
) -> Result<AccessGrant, AppError> {
    validate_grantee(pool, grantee_type, grantee_id).await?;
    validate_access_level(pool, resource_access_id).await?;

    let (user_id, group_id) = split_grantee(grantee_type, grantee_id);

    let id = Uuid::now_v7();
    let grant = sqlx::query_as::<_, AccessGrant>(
        r#"INSERT INTO resource_access_grants (id, item_id, user_id, group_id, resource_access_id, granted_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (vault_id, folder_id, item_id, user_id, group_id)
        DO UPDATE SET resource_access_id = $5, granted_by = $6, updated_at = NOW()
        RETURNING *"#,
    )
    .bind(id)
    .bind(item_id)
    .bind(user_id)
    .bind(group_id)
    .bind(resource_access_id)
    .bind(granted_by)
    .fetch_one(pool)
    .await?;

    Ok(grant)
}

/// Revoke a specific grant by ID.
pub async fn revoke_grant(pool: &PgPool, grant_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM resource_access_grants WHERE id = $1")
        .bind(grant_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("grant not found".into()));
    }

    Ok(())
}

/// List all grants for a vault (user + group grants at vault level).
pub async fn list_vault_grants(
    pool: &PgPool,
    vault_id: Uuid,
) -> Result<Vec<AccessGrantView>, AppError> {
    let grants = sqlx::query_as::<_, AccessGrantView>(
        r#"SELECT
            rag.id, rag.vault_id, rag.folder_id, rag.item_id,
            rag.user_id, rag.group_id,
            COALESCE(u.full_name, u.login, g.name, '') AS grantee_name,
            CASE WHEN rag.user_id IS NOT NULL THEN 'user' ELSE 'group' END AS grantee_type,
            ra.code AS access_code, ra.name AS access_name,
            rag.resource_access_id, rag.granted_by, rag.created_at
        FROM resource_access_grants rag
        JOIN resource_accesses ra ON rag.resource_access_id = ra.id
        LEFT JOIN users u ON rag.user_id = u.id
        LEFT JOIN user_groups g ON rag.group_id = g.id
        WHERE rag.vault_id = $1
        ORDER BY rag.created_at ASC"#,
    )
    .bind(vault_id)
    .fetch_all(pool)
    .await?;

    Ok(grants)
}

/// List grants for a folder.
pub async fn list_folder_grants(
    pool: &PgPool,
    folder_id: Uuid,
) -> Result<Vec<AccessGrantView>, AppError> {
    let grants = sqlx::query_as::<_, AccessGrantView>(
        r#"SELECT
            rag.id, rag.vault_id, rag.folder_id, rag.item_id,
            rag.user_id, rag.group_id,
            COALESCE(u.full_name, u.login, g.name, '') AS grantee_name,
            CASE WHEN rag.user_id IS NOT NULL THEN 'user' ELSE 'group' END AS grantee_type,
            ra.code AS access_code, ra.name AS access_name,
            rag.resource_access_id, rag.granted_by, rag.created_at
        FROM resource_access_grants rag
        JOIN resource_accesses ra ON rag.resource_access_id = ra.id
        LEFT JOIN users u ON rag.user_id = u.id
        LEFT JOIN user_groups g ON rag.group_id = g.id
        WHERE rag.folder_id = $1
        ORDER BY rag.created_at ASC"#,
    )
    .bind(folder_id)
    .fetch_all(pool)
    .await?;

    Ok(grants)
}

/// List grants for an item.
pub async fn list_item_grants(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<AccessGrantView>, AppError> {
    let grants = sqlx::query_as::<_, AccessGrantView>(
        r#"SELECT
            rag.id, rag.vault_id, rag.folder_id, rag.item_id,
            rag.user_id, rag.group_id,
            COALESCE(u.full_name, u.login, g.name, '') AS grantee_name,
            CASE WHEN rag.user_id IS NOT NULL THEN 'user' ELSE 'group' END AS grantee_type,
            ra.code AS access_code, ra.name AS access_name,
            rag.resource_access_id, rag.granted_by, rag.created_at
        FROM resource_access_grants rag
        JOIN resource_accesses ra ON rag.resource_access_id = ra.id
        LEFT JOIN users u ON rag.user_id = u.id
        LEFT JOIN user_groups g ON rag.group_id = g.id
        WHERE rag.item_id = $1
        ORDER BY rag.created_at ASC"#,
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;

    Ok(grants)
}

// ─── Helpers ───

/// Find the best grant for a specific resource (returns None if no grant found).
async fn find_grant_for_resource(
    pool: &PgPool,
    vault_id: Option<Uuid>,
    folder_id: Option<Uuid>,
    item_id: Option<Uuid>,
    user_id: Uuid,
    group_ids: &[Uuid],
) -> Result<Option<EffectiveAccess>, AppError> {
    let best: Option<(i32, String)> = if group_ids.is_empty() {
        sqlx::query_as(
            r#"SELECT ra.priority, ra.code
            FROM resource_access_grants rag
            JOIN resource_accesses ra ON rag.resource_access_id = ra.id
            WHERE rag.vault_id IS NOT DISTINCT FROM $1
              AND rag.folder_id IS NOT DISTINCT FROM $2
              AND rag.item_id IS NOT DISTINCT FROM $3
              AND rag.user_id = $4
            ORDER BY ra.priority DESC
            LIMIT 1"#,
        )
        .bind(vault_id)
        .bind(folder_id)
        .bind(item_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"SELECT ra.priority, ra.code
            FROM resource_access_grants rag
            JOIN resource_accesses ra ON rag.resource_access_id = ra.id
            WHERE rag.vault_id IS NOT DISTINCT FROM $1
              AND rag.folder_id IS NOT DISTINCT FROM $2
              AND rag.item_id IS NOT DISTINCT FROM $3
              AND (rag.user_id = $4 OR rag.group_id = ANY($5))
            ORDER BY ra.priority DESC
            LIMIT 1"#,
        )
        .bind(vault_id)
        .bind(folder_id)
        .bind(item_id)
        .bind(user_id)
        .bind(group_ids)
        .fetch_optional(pool)
        .await?
    };

    Ok(best.map(|(priority, code)| EffectiveAccess { priority, code }))
}

fn split_grantee(grantee_type: &str, grantee_id: Uuid) -> (Option<Uuid>, Option<Uuid>) {
    match grantee_type {
        "user" => (Some(grantee_id), None),
        "group" => (None, Some(grantee_id)),
        _ => (Some(grantee_id), None),
    }
}

async fn validate_grantee(pool: &PgPool, grantee_type: &str, grantee_id: Uuid) -> Result<(), AppError> {
    match grantee_type {
        "user" => {
            let exists: Option<(Uuid,)> =
                sqlx::query_as("SELECT id FROM users WHERE id = $1")
                    .bind(grantee_id)
                    .fetch_optional(pool)
                    .await?;
            if exists.is_none() {
                return Err(AppError::UserNotFound);
            }
        }
        "group" => {
            let exists: Option<(Uuid,)> =
                sqlx::query_as("SELECT id FROM user_groups WHERE id = $1")
                    .bind(grantee_id)
                    .fetch_optional(pool)
                    .await?;
            if exists.is_none() {
                return Err(AppError::NotFound("group not found".into()));
            }
        }
        _ => {
            return Err(AppError::Validation("granteeType must be 'user' or 'group'".into()));
        }
    }
    Ok(())
}

async fn validate_access_level(pool: &PgPool, resource_access_id: Uuid) -> Result<(), AppError> {
    let exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM resource_accesses WHERE id = $1")
            .bind(resource_access_id)
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        return Err(AppError::Validation("invalid access level".into()));
    }
    Ok(())
}
