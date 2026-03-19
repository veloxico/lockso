use anyhow::Result;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::permissions::{ALL_RESOURCE_PERMISSIONS, ALL_ROLE_PERMISSIONS, user_role_permissions};
use crate::models::enums::{DefaultPermissionAccess, DefaultUserRole, DefaultVaultType};
use crate::models::settings::{PasswordComplexity, SessionSettings, UserLockoutSettings};

/// Advisory lock ID for bootstrap — prevents race condition on first user.
pub const BOOTSTRAP_LOCK_ID: i64 = 0x4C4F434B534F_01; // "LOCKSO" + 01

// Use a generic executor so we can pass either &PgPool or &mut Transaction
pub type Tx<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

/// Check if the system has been bootstrapped (settings exist).
pub async fn is_bootstrapped(pool: &PgPool) -> Result<bool> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM settings")
        .fetch_one(pool)
        .await?;
    Ok(count.0 > 0)
}

/// Run bootstrap atomically using PostgreSQL advisory lock.
///
/// This ensures that even if two registration requests arrive simultaneously,
/// only one will run bootstrap. The second will see settings already exist.
pub async fn run_bootstrap_atomic(pool: &PgPool) -> Result<()> {
    // Acquire advisory lock (blocks until available, released at end of transaction)
    let mut tx = pool.begin().await?;

    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(BOOTSTRAP_LOCK_ID)
        .execute(&mut *tx)
        .await?;

    // Double-check after acquiring lock
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM settings")
        .fetch_one(&mut *tx)
        .await?;

    if count.0 > 0 {
        // Another request already ran bootstrap
        tracing::info!("Bootstrap already completed by another request");
        tx.commit().await?;
        return Ok(());
    }

    tracing::info!("Running bootstrap: creating default system data");

    create_default_settings(&mut tx).await?;
    let owner_role_id = create_default_roles(&mut tx).await?;
    create_default_resource_accesses(&mut tx).await?;
    create_default_vault_types(&mut tx, owner_role_id).await?;

    tx.commit().await?;
    tracing::info!("Bootstrap complete");
    Ok(())
}

/// Run bootstrap inside an existing transaction (caller holds the advisory lock).
pub async fn run_bootstrap_in_tx(tx: &mut Tx<'_>) -> Result<()> {
    tracing::info!("Running bootstrap: creating default system data");

    create_default_settings(tx).await?;
    let owner_role_id = create_default_roles(tx).await?;
    create_default_resource_accesses(tx).await?;
    create_default_vault_types(tx, owner_role_id).await?;

    tracing::info!("Bootstrap complete");
    Ok(())
}

/// Get the Owner role ID (used for first user registration).
pub async fn get_owner_role_id(pool: &PgPool) -> Result<Option<Uuid>> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM user_roles WHERE code = 'owner' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

/// Get the default role ID for new users.
pub async fn get_default_user_role_id(pool: &PgPool) -> Result<Uuid> {
    let row: (Uuid,) = sqlx::query_as(
        "SELECT id FROM user_roles WHERE code = 'user' LIMIT 1",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}



async fn create_default_settings(tx: &mut Tx<'_>) -> Result<()> {
    let session_settings = serde_json::to_value(SessionSettings::default())?;
    let auth_complexity = serde_json::to_value(PasswordComplexity::default())?;
    let master_complexity = serde_json::to_value(PasswordComplexity::master_default())?;
    let lockout = serde_json::to_value(UserLockoutSettings::default())?;

    sqlx::query(
        r#"INSERT INTO settings (
            id, session, email, sso, search, favicon, interface, notification,
            custom_banner, user_lockout, activity_log, browser_extension,
            auth_password_complexity, master_password_complexity, vault, task,
            "user", internal
        ) VALUES (
            $1, $2, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb,
            $3, '{}'::jsonb, '{}'::jsonb, $4, '{}'::jsonb, '{}'::jsonb,
            $5, $6, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb
        )"#,
    )
    .bind(Uuid::now_v7())
    .bind(&session_settings)
    .bind(json!({"defaultLanguage": "en", "defaultTimezone": "UTC"}))
    .bind(&lockout)
    .bind(&auth_complexity)
    .bind(&master_complexity)
    .execute(&mut **tx)
    .await?;

    tracing::info!("Created default settings");
    Ok(())
}

async fn create_default_roles(tx: &mut Tx<'_>) -> Result<Uuid> {
    let all_perms: serde_json::Value = ALL_ROLE_PERMISSIONS
        .iter()
        .map(|p| json!(p))
        .collect::<Vec<_>>()
        .into();

    let user_perms: serde_json::Value = user_role_permissions()
        .iter()
        .map(|p| json!(p))
        .collect::<Vec<_>>()
        .into();

    let default_auth = json!({
        "sessionTtl": 86400,
        "accessTokenTtl": 3600,
        "refreshTokenTtl": 2592000,
        "pinCodeTtl": 300,
        "isTwoFactorAuthRequired": false
    });

    let roles = [
        (DefaultUserRole::Owner, &all_perms),
        (DefaultUserRole::Admin, &all_perms),
        (DefaultUserRole::User, &user_perms),
    ];

    let mut owner_id = Uuid::nil();

    for (role, perms) in &roles {
        let id = Uuid::now_v7();
        if *role == DefaultUserRole::Owner {
            owner_id = id;
        }

        sqlx::query(
            "INSERT INTO user_roles (id, name, code, permissions, auth_settings, manageable_user_roles, offline_access)
             VALUES ($1, $2, $3, $4, $5, '[]'::jsonb, '{}'::jsonb)",
        )
        .bind(id)
        .bind(role.name())
        .bind(role.code())
        .bind(*perms)
        .bind(&default_auth)
        .execute(&mut **tx)
        .await?;
    }

    tracing::info!("Created default roles: Owner, Admin, User");
    Ok(owner_id)
}

async fn create_default_resource_accesses(tx: &mut Tx<'_>) -> Result<()> {
    let all_res_perms: Vec<serde_json::Value> = ALL_RESOURCE_PERMISSIONS
        .iter()
        .map(|p| json!(p))
        .collect();

    let levels = [
        (DefaultPermissionAccess::Admin, all_res_perms.clone()),
        (DefaultPermissionAccess::Manage, all_res_perms.clone()),
        (
            DefaultPermissionAccess::Write,
            ALL_RESOURCE_PERMISSIONS
                .iter()
                .filter(|p| !p.contains("accessList:edit") && !p.contains("Access:edit"))
                .map(|p| json!(p))
                .collect(),
        ),
        (
            DefaultPermissionAccess::Read,
            ALL_RESOURCE_PERMISSIONS
                .iter()
                .filter(|p| p.contains(":read") || p.contains(":view"))
                .map(|p| json!(p))
                .collect(),
        ),
        (DefaultPermissionAccess::Forbidden, vec![]),
    ];

    for (level, perms) in &levels {
        sqlx::query(
            "INSERT INTO resource_accesses (id, name, code, permissions, priority, is_access_override_allowed)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(Uuid::now_v7())
        .bind(level.name())
        .bind(level.code())
        .bind(json!(perms))
        .bind(level.priority())
        .bind(matches!(level, DefaultPermissionAccess::Forbidden))
        .execute(&mut **tx)
        .await?;
    }

    tracing::info!("Created default resource accesses: Admin, Manage, Write, Read, Forbidden");
    Ok(())
}

async fn create_default_vault_types(tx: &mut Tx<'_>, owner_role_id: Uuid) -> Result<()> {
    let types = [
        DefaultVaultType::Organization,
        DefaultVaultType::Personal,
        DefaultVaultType::PrivateShared,
    ];

    for vt in &types {
        sqlx::query(
            "INSERT INTO vault_types (id, name, code, allowed_roles)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::now_v7())
        .bind(vt.name())
        .bind(vt.code())
        .bind(json!([owner_role_id.to_string()]))
        .execute(&mut **tx)
        .await?;
    }

    tracing::info!("Created default vault types: Organization, Personal, PrivateShared");
    Ok(())
}
