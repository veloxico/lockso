use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::group::{
    GroupMember, UserGroup, UserGroupListItem, UserGroupView,
};

const MAX_NAME_LENGTH: usize = 255;

/// List all groups with member counts.
pub async fn list_groups(pool: &PgPool) -> Result<Vec<UserGroupListItem>, AppError> {
    let groups = sqlx::query_as::<_, UserGroupListItem>(
        r#"SELECT
            g.id, g.name, g.description, g.is_active, g.created_at,
            COALESCE(mc.cnt, 0) AS member_count
        FROM user_groups g
        LEFT JOIN (
            SELECT group_id, COUNT(*) AS cnt FROM user_group_members GROUP BY group_id
        ) mc ON mc.group_id = g.id
        ORDER BY g.name"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(groups)
}

/// Get a single group with its members.
pub async fn get_group(pool: &PgPool, group_id: Uuid) -> Result<UserGroupView, AppError> {
    let group = sqlx::query_as::<_, UserGroup>("SELECT * FROM user_groups WHERE id = $1")
        .bind(group_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".into()))?;

    let members = sqlx::query_as::<_, GroupMember>(
        r#"SELECT
            ugm.id, ugm.user_id, u.login, u.full_name, u.email,
            ugm.added_by, ugm.created_at
        FROM user_group_members ugm
        JOIN users u ON ugm.user_id = u.id
        WHERE ugm.group_id = $1
        ORDER BY u.login"#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;

    Ok(UserGroupView {
        id: group.id,
        name: group.name,
        description: group.description,
        creator_id: group.creator_id,
        is_active: group.is_active,
        members,
        created_at: group.created_at,
        updated_at: group.updated_at,
    })
}

/// Create a new group.
pub async fn create_group(
    pool: &PgPool,
    creator_id: Uuid,
    name: &str,
    description: &str,
) -> Result<UserGroup, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("group name is required".into()));
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(AppError::Validation("group name too long".into()));
    }

    // Check uniqueness (case-insensitive)
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM user_groups WHERE LOWER(name) = LOWER($1)",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    if exists.is_some() {
        return Err(AppError::Validation("group name already taken".into()));
    }

    let id = Uuid::now_v7();
    let group = sqlx::query_as::<_, UserGroup>(
        r#"INSERT INTO user_groups (id, name, description, creator_id)
        VALUES ($1, $2, $3, $4)
        RETURNING *"#,
    )
    .bind(id)
    .bind(name)
    .bind(description.trim())
    .bind(creator_id)
    .fetch_one(pool)
    .await?;

    tracing::info!(group_id = %id, name = %name, "Group created");
    Ok(group)
}

/// Update a group.
pub async fn update_group(
    pool: &PgPool,
    group_id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<UserGroup, AppError> {
    // Verify group exists
    let mut group = sqlx::query_as::<_, UserGroup>("SELECT * FROM user_groups WHERE id = $1")
        .bind(group_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("group not found".into()))?;

    if let Some(new_name) = name {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err(AppError::Validation("group name is required".into()));
        }
        if new_name.len() > MAX_NAME_LENGTH {
            return Err(AppError::Validation("group name too long".into()));
        }

        // Check uniqueness (excluding current group)
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM user_groups WHERE LOWER(name) = LOWER($1) AND id != $2",
        )
        .bind(new_name)
        .bind(group_id)
        .fetch_optional(pool)
        .await?;

        if exists.is_some() {
            return Err(AppError::Validation("group name already taken".into()));
        }

        group.name = new_name.to_string();
    }

    if let Some(new_desc) = description {
        group.description = new_desc.trim().to_string();
    }

    let updated = sqlx::query_as::<_, UserGroup>(
        r#"UPDATE user_groups SET name = $2, description = $3, updated_at = NOW()
        WHERE id = $1 RETURNING *"#,
    )
    .bind(group_id)
    .bind(&group.name)
    .bind(&group.description)
    .fetch_one(pool)
    .await?;

    Ok(updated)
}

/// Delete a group.
pub async fn delete_group(pool: &PgPool, group_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM user_groups WHERE id = $1")
        .bind(group_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("group not found".into()));
    }

    tracing::info!(group_id = %group_id, "Group deleted");
    Ok(())
}

/// Add a user to a group.
pub async fn add_member(
    pool: &PgPool,
    group_id: Uuid,
    user_id: Uuid,
    added_by: Uuid,
) -> Result<GroupMember, AppError> {
    // Verify group exists
    let exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM user_groups WHERE id = $1")
            .bind(group_id)
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        return Err(AppError::NotFound("group not found".into()));
    }

    // Verify user exists
    let user_exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    if user_exists.is_none() {
        return Err(AppError::UserNotFound);
    }

    let id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO user_group_members (id, group_id, user_id, added_by)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (group_id, user_id) DO NOTHING"#,
    )
    .bind(id)
    .bind(group_id)
    .bind(user_id)
    .bind(added_by)
    .execute(pool)
    .await?;

    let member = sqlx::query_as::<_, GroupMember>(
        r#"SELECT
            ugm.id, ugm.user_id, u.login, u.full_name, u.email,
            ugm.added_by, ugm.created_at
        FROM user_group_members ugm
        JOIN users u ON ugm.user_id = u.id
        WHERE ugm.group_id = $1 AND ugm.user_id = $2"#,
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    tracing::info!(group_id = %group_id, user_id = %user_id, "Member added to group");
    Ok(member)
}

/// Remove a user from a group.
pub async fn remove_member(
    pool: &PgPool,
    group_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "DELETE FROM user_group_members WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("member not found".into()));
    }

    tracing::info!(group_id = %group_id, user_id = %user_id, "Member removed from group");
    Ok(())
}

/// Get all group IDs a user belongs to.
pub async fn get_user_group_ids(pool: &PgPool, user_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT group_id FROM user_group_members WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}
