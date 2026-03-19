use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::folder::{
    CreateFolder, Folder, FolderTreeNode, FolderView, MoveFolder, UpdateFolder,
};

/// Max folder name length.
const MAX_NAME_LENGTH: usize = 255;
/// Max folder nesting depth.
const MAX_DEPTH: usize = 10;

/// List folders in a vault as a flat list with item counts.
pub async fn list_folders(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<FolderView>, AppError> {
    // Verify vault ownership
    verify_vault_access(pool, vault_id, user_id).await?;

    let rows = sqlx::query_as::<_, (Uuid, String, Uuid, Option<Uuid>, i32, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, i64)>(
        r#"SELECT
            f.id, f.name, f.vault_id, f.parent_folder_id, f.position, f.created_at, f.updated_at,
            COALESCE(ic.cnt, 0) AS item_count
        FROM folders f
        LEFT JOIN (SELECT folder_id, COUNT(*) AS cnt FROM items GROUP BY folder_id) ic ON ic.folder_id = f.id
        WHERE f.vault_id = $1
        ORDER BY f.position, f.name"#,
    )
    .bind(vault_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, name, vid, pid, pos, ca, ua, ic)| FolderView {
            id,
            name,
            vault_id: vid,
            parent_folder_id: pid,
            position: pos,
            item_count: ic,
            created_at: ca,
            updated_at: ua,
        })
        .collect())
}

/// Get folder tree for a vault.
pub async fn get_folder_tree(
    pool: &PgPool,
    vault_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<FolderTreeNode>, AppError> {
    let folders = list_folders(pool, vault_id, user_id).await?;
    Ok(build_tree(&folders, None))
}

fn build_tree(folders: &[FolderView], parent_id: Option<Uuid>) -> Vec<FolderTreeNode> {
    folders
        .iter()
        .filter(|f| f.parent_folder_id == parent_id)
        .map(|f| FolderTreeNode {
            id: f.id,
            name: f.name.clone(),
            parent_folder_id: f.parent_folder_id,
            position: f.position,
            item_count: f.item_count,
            children: build_tree(folders, Some(f.id)),
        })
        .collect()
}

/// Create a folder.
pub async fn create_folder(
    pool: &PgPool,
    user_id: Uuid,
    input: CreateFolder,
) -> Result<FolderView, AppError> {
    // Verify vault write access
    verify_vault_write_access(pool, input.vault_id, user_id).await?;

    // Validate name
    validate_name(&input.name)?;

    // Check name uniqueness within same parent
    let name_exists: Option<(Uuid,)> = if let Some(parent_id) = input.parent_folder_id {
        sqlx::query_as(
            "SELECT id FROM folders WHERE name = $1 AND vault_id = $2 AND parent_folder_id = $3",
        )
        .bind(input.name.trim())
        .bind(input.vault_id)
        .bind(parent_id)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id FROM folders WHERE name = $1 AND vault_id = $2 AND parent_folder_id IS NULL",
        )
        .bind(input.name.trim())
        .bind(input.vault_id)
        .fetch_optional(pool)
        .await?
    };

    if name_exists.is_some() {
        return Err(AppError::FolderNameTaken);
    }

    // Validate parent exists and compute ancestor_ids
    let ancestor_ids = if let Some(parent_id) = input.parent_folder_id {
        let parent = sqlx::query_as::<_, Folder>(
            "SELECT * FROM folders WHERE id = $1 AND vault_id = $2",
        )
        .bind(parent_id)
        .bind(input.vault_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::FolderNotFound)?;

        let mut ancestors: Vec<Uuid> = serde_json::from_value(parent.ancestor_ids)
            .unwrap_or_default();
        ancestors.push(parent_id);

        if ancestors.len() >= MAX_DEPTH {
            return Err(AppError::Validation(format!(
                "Maximum folder nesting depth is {MAX_DEPTH}"
            )));
        }

        serde_json::to_value(&ancestors).unwrap_or_default()
    } else {
        serde_json::Value::Array(vec![])
    };

    let folder_id = Uuid::now_v7();
    let now = Utc::now();
    let position = input.position.unwrap_or(0);

    sqlx::query(
        r#"INSERT INTO folders (id, name, vault_id, parent_folder_id, ancestor_ids, position, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7)"#,
    )
    .bind(folder_id)
    .bind(input.name.trim())
    .bind(input.vault_id)
    .bind(input.parent_folder_id)
    .bind(&ancestor_ids)
    .bind(position)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(FolderView {
        id: folder_id,
        name: input.name.trim().to_string(),
        vault_id: input.vault_id,
        parent_folder_id: input.parent_folder_id,
        position,
        item_count: 0,
        created_at: now,
        updated_at: now,
    })
}

/// Update a folder.
pub async fn update_folder(
    pool: &PgPool,
    folder_id: Uuid,
    user_id: Uuid,
    input: UpdateFolder,
) -> Result<FolderView, AppError> {
    let folder = get_folder_checked(pool, folder_id, user_id).await?;

    let new_name = input.name.as_deref().unwrap_or(&folder.name);
    let new_position = input.position.unwrap_or(folder.position);

    if let Some(ref name) = input.name {
        validate_name(name)?;

        // Check uniqueness excluding self
        let name_exists: Option<(Uuid,)> = if let Some(parent_id) = folder.parent_folder_id {
            sqlx::query_as(
                "SELECT id FROM folders WHERE name = $1 AND vault_id = $2 AND parent_folder_id = $3 AND id != $4",
            )
            .bind(name.trim())
            .bind(folder.vault_id)
            .bind(parent_id)
            .bind(folder_id)
            .fetch_optional(pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT id FROM folders WHERE name = $1 AND vault_id = $2 AND parent_folder_id IS NULL AND id != $3",
            )
            .bind(name.trim())
            .bind(folder.vault_id)
            .bind(folder_id)
            .fetch_optional(pool)
            .await?
        };

        if name_exists.is_some() {
            return Err(AppError::FolderNameTaken);
        }
    }

    let now = Utc::now();
    sqlx::query("UPDATE folders SET name = $1, position = $2, updated_at = $3 WHERE id = $4")
        .bind(new_name.trim())
        .bind(new_position)
        .bind(now)
        .bind(folder_id)
        .execute(pool)
        .await?;

    // Return updated folder with item count
    let item_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM items WHERE folder_id = $1")
            .bind(folder_id)
            .fetch_one(pool)
            .await?;

    Ok(FolderView {
        id: folder_id,
        name: new_name.trim().to_string(),
        vault_id: folder.vault_id,
        parent_folder_id: folder.parent_folder_id,
        position: new_position,
        item_count: item_count.0,
        created_at: folder.created_at,
        updated_at: now,
    })
}

/// Move a folder to a new parent.
pub async fn move_folder(
    pool: &PgPool,
    folder_id: Uuid,
    user_id: Uuid,
    input: MoveFolder,
) -> Result<FolderView, AppError> {
    let folder = get_folder_checked(pool, folder_id, user_id).await?;

    // Prevent moving folder into itself or its descendants
    if let Some(new_parent_id) = input.parent_folder_id {
        if new_parent_id == folder_id {
            return Err(AppError::Validation("Cannot move folder into itself".into()));
        }

        let parent = sqlx::query_as::<_, Folder>(
            "SELECT * FROM folders WHERE id = $1 AND vault_id = $2",
        )
        .bind(new_parent_id)
        .bind(folder.vault_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::FolderNotFound)?;

        // Check if new parent is a descendant of this folder
        let parent_ancestors: Vec<Uuid> =
            serde_json::from_value(parent.ancestor_ids).unwrap_or_default();
        if parent_ancestors.contains(&folder_id) {
            return Err(AppError::Validation(
                "Cannot move folder into its own descendant".into(),
            ));
        }

        // Compute new ancestors
        let mut new_ancestors = parent_ancestors;
        new_ancestors.push(new_parent_id);

        if new_ancestors.len() >= MAX_DEPTH {
            return Err(AppError::Validation(format!(
                "Maximum folder nesting depth is {MAX_DEPTH}"
            )));
        }

        let ancestor_json = serde_json::to_value(&new_ancestors).unwrap_or_default();
        let now = Utc::now();

        sqlx::query(
            "UPDATE folders SET parent_folder_id = $1, ancestor_ids = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(new_parent_id)
        .bind(&ancestor_json)
        .bind(now)
        .bind(folder_id)
        .execute(pool)
        .await?;
    } else {
        // Move to root
        let now = Utc::now();
        sqlx::query(
            "UPDATE folders SET parent_folder_id = NULL, ancestor_ids = '[]'::jsonb, updated_at = $1 WHERE id = $2",
        )
        .bind(now)
        .bind(folder_id)
        .execute(pool)
        .await?;
    }

    // Update descendant ancestor_ids recursively
    update_descendant_ancestors(pool, folder_id, folder.vault_id).await?;

    // Fetch updated folder
    let updated = sqlx::query_as::<_, Folder>("SELECT * FROM folders WHERE id = $1")
        .bind(folder_id)
        .fetch_one(pool)
        .await?;

    let item_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM items WHERE folder_id = $1")
            .bind(folder_id)
            .fetch_one(pool)
            .await?;

    Ok(FolderView {
        id: updated.id,
        name: updated.name,
        vault_id: updated.vault_id,
        parent_folder_id: updated.parent_folder_id,
        position: updated.position,
        item_count: item_count.0,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    })
}

/// Delete a folder (cascading deletes items and subfolders via DB constraints).
pub async fn delete_folder(
    pool: &PgPool,
    folder_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let folder = get_folder_checked(pool, folder_id, user_id).await?;

    sqlx::query("DELETE FROM folders WHERE id = $1")
        .bind(folder_id)
        .execute(pool)
        .await?;

    tracing::info!(folder_id = %folder_id, vault_id = %folder.vault_id, "Folder deleted");
    Ok(())
}

// ─── Helpers ───

fn validate_name(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("Folder name is required".into()));
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(AppError::Validation(format!(
            "Folder name must not exceed {MAX_NAME_LENGTH} characters"
        )));
    }
    Ok(())
}

/// Verify user has at least read access to the vault.
async fn verify_vault_access(pool: &PgPool, vault_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let exists: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT v.id FROM vaults v
        WHERE v.id = $1 AND (
            v.creator_id = $2
            OR EXISTS (
                SELECT 1 FROM vault_user_accesses vua
                JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                WHERE vua.vault_id = v.id AND vua.user_id = $2 AND ra.code != 'forbidden'
            )
        )"#,
    )
    .bind(vault_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if exists.is_none() {
        return Err(AppError::VaultNotFound);
    }
    Ok(())
}

/// Verify user has write access to the vault (owner, or shared with 'write'/'admin' access).
async fn verify_vault_write_access(pool: &PgPool, vault_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let exists: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT v.id FROM vaults v
        WHERE v.id = $1 AND (
            v.creator_id = $2
            OR EXISTS (
                SELECT 1 FROM vault_user_accesses vua
                JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                WHERE vua.vault_id = v.id AND vua.user_id = $2
                AND ra.code IN ('write', 'admin', 'manage')
            )
        )"#,
    )
    .bind(vault_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if exists.is_none() {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

async fn get_folder_checked(
    pool: &PgPool,
    folder_id: Uuid,
    user_id: Uuid,
) -> Result<Folder, AppError> {
    let folder = sqlx::query_as::<_, Folder>("SELECT * FROM folders WHERE id = $1")
        .bind(folder_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::FolderNotFound)?;

    // Verify vault write access (used by update, delete, move — all write ops)
    verify_vault_write_access(pool, folder.vault_id, user_id).await?;

    Ok(folder)
}

/// Recursively update ancestor_ids for all descendants of a folder.
async fn update_descendant_ancestors(
    pool: &PgPool,
    folder_id: Uuid,
    vault_id: Uuid,
) -> Result<(), AppError> {
    let folder = sqlx::query_as::<_, Folder>("SELECT * FROM folders WHERE id = $1")
        .bind(folder_id)
        .fetch_one(pool)
        .await?;

    let my_ancestors: Vec<Uuid> =
        serde_json::from_value(folder.ancestor_ids).unwrap_or_default();

    let children = sqlx::query_as::<_, Folder>(
        "SELECT * FROM folders WHERE parent_folder_id = $1 AND vault_id = $2",
    )
    .bind(folder_id)
    .bind(vault_id)
    .fetch_all(pool)
    .await?;

    for child in children {
        let mut child_ancestors = my_ancestors.clone();
        child_ancestors.push(folder_id);
        let ancestor_json = serde_json::to_value(&child_ancestors).unwrap_or_default();

        sqlx::query("UPDATE folders SET ancestor_ids = $1 WHERE id = $2")
            .bind(&ancestor_json)
            .bind(child.id)
            .execute(pool)
            .await?;

        // Recurse
        Box::pin(update_descendant_ancestors(pool, child.id, vault_id)).await?;
    }

    Ok(())
}
