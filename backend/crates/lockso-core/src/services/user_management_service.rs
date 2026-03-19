use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::user::{User, UserView};
use crate::models::user_role::{UserRole, UserRoleView};

/// Admin user list item with role info.
#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserListItem {
    pub id: Uuid,
    pub login: String,
    pub email: Option<String>,
    pub full_name: String,
    pub signup_type: String,
    pub role_id: Uuid,
    pub role_name: String,
    pub role_code: String,
    pub is_blocked: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Update user role request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRole {
    pub role_id: Uuid,
}

/// Block/unblock request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUserBlocked {
    pub is_blocked: bool,
}

/// Admin create user request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminCreateUser {
    pub login: String,
    pub password: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role_id: Option<Uuid>,
}

/// Get a single user by ID.
pub async fn get_user(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

/// List all users with their role info (admin only).
pub async fn list_users(pool: &PgPool) -> Result<Vec<AdminUserListItem>, AppError> {
    let rows: Vec<AdminUserListItem> = sqlx::query_as::<_, AdminUserListItem>(
        r#"SELECT
            u.id,
            u.login,
            u.email,
            u.full_name,
            u.signup_type,
            u.role_id,
            ur.name AS role_name,
            ur.code AS role_code,
            u.is_blocked,
            u.last_login_at,
            u.created_at
        FROM users u
        JOIN user_roles ur ON u.role_id = ur.id
        ORDER BY u.created_at ASC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// List all available roles.
pub async fn list_roles(pool: &PgPool) -> Result<Vec<UserRoleView>, AppError> {
    let roles = sqlx::query_as::<_, UserRole>(
        "SELECT * FROM user_roles ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(roles.into_iter().map(UserRoleView::from).collect())
}

/// Update a user's role (admin only).
///
/// Cannot change the owner's role. Cannot assign owner role to non-owners.
pub async fn update_user_role(
    pool: &PgPool,
    actor_id: Uuid,
    target_user_id: Uuid,
    new_role_id: Uuid,
) -> Result<UserView, AppError> {
    // Prevent self-role-change
    if actor_id == target_user_id {
        return Err(AppError::Validation(
            "Cannot change your own role".into(),
        ));
    }

    // Verify target user exists
    let target = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(target_user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::UserNotFound)?;

    // Check target is not owner (owner role is immutable)
    let target_role: Option<(String,)> =
        sqlx::query_as("SELECT code FROM user_roles WHERE id = $1")
            .bind(target.role_id)
            .fetch_optional(pool)
            .await?;
    if target_role.as_ref().map(|(c,)| c.as_str()) == Some("owner") {
        return Err(AppError::Forbidden);
    }

    // Check new role is not owner
    let new_role: Option<(String,)> =
        sqlx::query_as("SELECT code FROM user_roles WHERE id = $1")
            .bind(new_role_id)
            .fetch_optional(pool)
            .await?;
    match new_role.as_ref().map(|(c,)| c.as_str()) {
        None => return Err(AppError::Validation("Role not found".into())),
        Some("owner") => {
            return Err(AppError::Validation(
                "Cannot assign owner role".into(),
            ));
        }
        _ => {}
    }

    let updated = sqlx::query_as::<_, User>(
        "UPDATE users SET role_id = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(new_role_id)
    .bind(target_user_id)
    .fetch_one(pool)
    .await?;

    tracing::info!(
        actor_id = %actor_id,
        target_id = %target_user_id,
        new_role_id = %new_role_id,
        "User role updated"
    );

    Ok(UserView::from(updated))
}

/// Block or unblock a user (admin only).
///
/// Cannot block the owner or yourself.
pub async fn set_user_blocked(
    pool: &PgPool,
    actor_id: Uuid,
    target_user_id: Uuid,
    is_blocked: bool,
) -> Result<UserView, AppError> {
    if actor_id == target_user_id {
        return Err(AppError::Validation("Cannot block yourself".into()));
    }

    // Check target is not owner
    let target_role_code: Option<(String,)> = sqlx::query_as(
        "SELECT ur.code FROM users u JOIN user_roles ur ON u.role_id = ur.id WHERE u.id = $1",
    )
    .bind(target_user_id)
    .fetch_optional(pool)
    .await?;

    match target_role_code.as_ref().map(|(c,)| c.as_str()) {
        None => return Err(AppError::UserNotFound),
        Some("owner") => return Err(AppError::Forbidden),
        _ => {}
    }

    let updated = sqlx::query_as::<_, User>(
        "UPDATE users SET is_blocked = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(is_blocked)
    .bind(target_user_id)
    .fetch_one(pool)
    .await?;

    // Invalidate all sessions when blocking a user
    if is_blocked {
        let deleted = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(target_user_id)
            .execute(pool)
            .await?;
        tracing::info!(
            target_id = %target_user_id,
            sessions_deleted = deleted.rows_affected(),
            "Blocked user sessions invalidated"
        );
    }

    tracing::info!(
        actor_id = %actor_id,
        target_id = %target_user_id,
        is_blocked = is_blocked,
        "User block status updated"
    );

    Ok(UserView::from(updated))
}

/// Delete a user (admin only).
///
/// Cannot delete the owner or yourself. Cascading deletes will handle sessions, etc.
pub async fn delete_user(
    pool: &PgPool,
    actor_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    if actor_id == target_user_id {
        return Err(AppError::Validation("Cannot delete yourself".into()));
    }

    // Check target is not owner
    let target_role_code: Option<(String,)> = sqlx::query_as(
        "SELECT ur.code FROM users u JOIN user_roles ur ON u.role_id = ur.id WHERE u.id = $1",
    )
    .bind(target_user_id)
    .fetch_optional(pool)
    .await?;

    match target_role_code.as_ref().map(|(c,)| c.as_str()) {
        None => return Err(AppError::UserNotFound),
        Some("owner") => return Err(AppError::Forbidden),
        _ => {}
    }

    // Delete sessions first, then user
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(target_user_id)
        .execute(pool)
        .await?;

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(target_user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::UserNotFound);
    }

    tracing::info!(
        actor_id = %actor_id,
        target_id = %target_user_id,
        "User deleted"
    );

    Ok(())
}

/// Admin-level user creation.
///
/// Creates a new user with the specified role. If no role is provided, assigns the default "user" role.
pub async fn admin_create_user(pool: &PgPool, input: AdminCreateUser) -> Result<UserView, AppError> {
    use crate::models::user::CreateUser;
    use crate::services::auth_service;

    // Register using the standard flow (validates login, password, uniqueness, etc.)
    let create_input = CreateUser {
        login: input.login,
        password: input.password,
        email: input.email,
        full_name: input.full_name,
        master_key_hash: None,
        keys_public: None,
        keys_private_encrypted: None,
    };

    let user = auth_service::register(pool, create_input).await?;

    // If a specific role was requested, update it
    if let Some(role_id) = input.role_id {
        // Validate role exists and is not owner
        let role: Option<(String,)> =
            sqlx::query_as("SELECT code FROM user_roles WHERE id = $1")
                .bind(role_id)
                .fetch_optional(pool)
                .await?;

        match role.as_ref().map(|(c,)| c.as_str()) {
            Some("owner") => return Err(AppError::Validation("Cannot assign owner role".into())),
            None => return Err(AppError::Validation("Invalid role ID".into())),
            _ => {}
        }

        sqlx::query("UPDATE users SET role_id = $1 WHERE id = $2")
            .bind(role_id)
            .bind(user.id)
            .execute(pool)
            .await?;
    }

    // Return the user view
    let view: UserView = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(pool)
        .await?
        .into();

    Ok(view)
}

/// Check if a user has admin-level permissions by role code.
pub async fn require_admin(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    let role_code: Option<(String,)> = sqlx::query_as(
        "SELECT ur.code FROM users u JOIN user_roles ur ON u.role_id = ur.id WHERE u.id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match role_code.as_ref().map(|(c,)| c.as_str()) {
        Some("owner") | Some("admin") => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}
