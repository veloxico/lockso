use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::vault_access::VaultMember;
use crate::services::access_service;

/// Check if a user can access a vault (creator OR shared member, not Forbidden).
///
/// Delegates to access_service which also checks group-based grants.
pub async fn check_vault_access(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<String, AppError> {
    access_service::check_vault_access(pool, vault_id, user_id).await
}

/// Check write-level access (owner, admin, manage, or write).
pub async fn require_write_access(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    access_service::require_write_access(pool, vault_id, user_id).await
}

/// Check admin-level access (owner or admin).
pub async fn require_vault_admin(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    access_service::require_vault_admin(pool, vault_id, user_id).await
}

/// List all members of a vault (from the unified grants table).
pub async fn list_vault_members(
    pool: &PgPool,
    vault_id: Uuid,
) -> Result<Vec<VaultMember>, AppError> {
    // Query resource_access_grants for vault-level user grants
    let members = sqlx::query_as::<_, VaultMember>(
        r#"SELECT
            rag.id,
            rag.user_id,
            u.login,
            u.full_name,
            u.email,
            ra.code AS access_code,
            ra.name AS access_name,
            rag.resource_access_id,
            rag.granted_by AS "granted_by!: Uuid",
            rag.created_at
        FROM resource_access_grants rag
        JOIN users u ON rag.user_id = u.id
        JOIN resource_accesses ra ON rag.resource_access_id = ra.id
        WHERE rag.vault_id = $1 AND rag.user_id IS NOT NULL
        ORDER BY rag.created_at ASC"#,
    )
    .bind(vault_id)
    .fetch_all(pool)
    .await?;

    Ok(members)
}

/// Share a vault with a user.
///
/// Requires vault admin access. Cannot share with yourself or the vault creator.
pub async fn share_vault(
    pool: &PgPool,
    vault_id: Uuid,
    actor_id: Uuid,
    target_user_id: Uuid,
    resource_access_id: Uuid,
) -> Result<VaultMember, AppError> {
    // Validate that the vault exists and actor has admin access
    require_vault_admin(pool, vault_id, actor_id).await?;

    // Cannot share with vault creator (they always have owner access)
    let creator: Option<(Option<Uuid>,)> =
        sqlx::query_as("SELECT creator_id FROM vaults WHERE id = $1")
            .bind(vault_id)
            .fetch_optional(pool)
            .await?;
    if creator.and_then(|c| c.0) == Some(target_user_id) {
        return Err(AppError::Validation(
            "Cannot share vault with its creator".into(),
        ));
    }

    // Use access_service to create the grant
    let _grant = access_service::grant_vault_access(
        pool,
        vault_id,
        "user",
        target_user_id,
        resource_access_id,
        actor_id,
    )
    .await?;

    // Also write to legacy vault_user_accesses for backward compat
    let access_id = uuid::Uuid::now_v7();
    let _ = sqlx::query(
        r#"INSERT INTO vault_user_accesses (id, vault_id, user_id, resource_access_id, granted_by)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (vault_id, user_id)
        DO UPDATE SET resource_access_id = $4, granted_by = $5, updated_at = NOW()"#,
    )
    .bind(access_id)
    .bind(vault_id)
    .bind(target_user_id)
    .bind(resource_access_id)
    .bind(actor_id)
    .execute(pool)
    .await;

    tracing::info!(
        vault_id = %vault_id,
        actor = %actor_id,
        target = %target_user_id,
        "Vault shared"
    );

    // Return the member record
    let member = sqlx::query_as::<_, VaultMember>(
        r#"SELECT
            rag.id,
            rag.user_id,
            u.login,
            u.full_name,
            u.email,
            ra.code AS access_code,
            ra.name AS access_name,
            rag.resource_access_id,
            rag.granted_by AS "granted_by!: Uuid",
            rag.created_at
        FROM resource_access_grants rag
        JOIN users u ON rag.user_id = u.id
        JOIN resource_accesses ra ON rag.resource_access_id = ra.id
        WHERE rag.vault_id = $1 AND rag.user_id = $2"#,
    )
    .bind(vault_id)
    .bind(target_user_id)
    .fetch_one(pool)
    .await?;

    Ok(member)
}

/// Update a member's access level.
pub async fn update_member_access(
    pool: &PgPool,
    vault_id: Uuid,
    actor_id: Uuid,
    target_user_id: Uuid,
    resource_access_id: Uuid,
) -> Result<VaultMember, AppError> {
    require_vault_admin(pool, vault_id, actor_id).await?;

    // Update in unified grants table
    let result = sqlx::query(
        r#"UPDATE resource_access_grants
        SET resource_access_id = $1, granted_by = $2, updated_at = NOW()
        WHERE vault_id = $3 AND user_id = $4"#,
    )
    .bind(resource_access_id)
    .bind(actor_id)
    .bind(vault_id)
    .bind(target_user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Member not found".into()));
    }

    // Also update legacy table
    let _ = sqlx::query(
        r#"UPDATE vault_user_accesses
        SET resource_access_id = $1, granted_by = $2, updated_at = NOW()
        WHERE vault_id = $3 AND user_id = $4"#,
    )
    .bind(resource_access_id)
    .bind(actor_id)
    .bind(vault_id)
    .bind(target_user_id)
    .execute(pool)
    .await;

    let member = sqlx::query_as::<_, VaultMember>(
        r#"SELECT
            rag.id,
            rag.user_id,
            u.login,
            u.full_name,
            u.email,
            ra.code AS access_code,
            ra.name AS access_name,
            rag.resource_access_id,
            rag.granted_by AS "granted_by!: Uuid",
            rag.created_at
        FROM resource_access_grants rag
        JOIN users u ON rag.user_id = u.id
        JOIN resource_accesses ra ON rag.resource_access_id = ra.id
        WHERE rag.vault_id = $1 AND rag.user_id = $2"#,
    )
    .bind(vault_id)
    .bind(target_user_id)
    .fetch_one(pool)
    .await?;

    Ok(member)
}

/// Remove a user's access to a vault.
pub async fn revoke_access(
    pool: &PgPool,
    vault_id: Uuid,
    actor_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    require_vault_admin(pool, vault_id, actor_id).await?;

    let result = sqlx::query(
        "DELETE FROM resource_access_grants WHERE vault_id = $1 AND user_id = $2",
    )
    .bind(vault_id)
    .bind(target_user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Member not found".into()));
    }

    // Also remove from legacy table
    let _ = sqlx::query(
        "DELETE FROM vault_user_accesses WHERE vault_id = $1 AND user_id = $2",
    )
    .bind(vault_id)
    .bind(target_user_id)
    .execute(pool)
    .await;

    tracing::info!(
        vault_id = %vault_id,
        actor = %actor_id,
        target = %target_user_id,
        "Vault access revoked"
    );

    Ok(())
}
