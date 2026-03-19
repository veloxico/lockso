use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::encryption::{decrypt_field, encrypt_field};
use crate::error::AppError;
use crate::models::item::{
    CreateItem, CustomField, Item, ItemListEntry, ItemView, MoveItem, SearchRequest, TrashListEntry, UpdateItem,
};
use crate::models::snapshot::{Snapshot, SnapshotListEntry, SnapshotView};
use crate::services::{attachment_service, vault_service};
use lockso_crypto::search_hash::blind_search_hash;

/// Max item name length.
const MAX_NAME_LENGTH: usize = 500;
/// Max recent views per user.
const MAX_RECENT_VIEWS: i64 = 50;
/// Max total search results across all vaults.
const MAX_SEARCH_RESULTS: usize = 200;
/// Max snapshots to keep per item (prune oldest beyond this).
const MAX_SNAPSHOTS_PER_ITEM: i64 = 50;

/// List items in a vault (optionally filtered by folder).
/// Returns list entries with decrypted name/login/url but NOT password.
pub async fn list_items(
    pool: &PgPool,
    key: &[u8],
    vault_id: Uuid,
    folder_id: Option<Uuid>,
    user_id: Uuid,
) -> Result<Vec<ItemListEntry>, AppError> {
    verify_vault_access(pool, vault_id, user_id).await?;

    let items = if let Some(fid) = folder_id {
        sqlx::query_as::<_, Item>(
            "SELECT * FROM items WHERE vault_id = $1 AND folder_id = $2 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(vault_id)
        .bind(fid)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Item>(
            "SELECT * FROM items WHERE vault_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(vault_id)
        .fetch_all(pool)
        .await?
    };

    // Check favorites for this user
    let fav_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT item_id FROM favorites WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let fav_set: std::collections::HashSet<Uuid> =
        fav_ids.into_iter().map(|(id,)| id).collect();

    let mut entries = Vec::with_capacity(items.len());
    for item in items {
        let name = decrypt_field(key, &item.name_enc)?;
        let login = decrypt_field(key, &item.login_enc)?;
        let url = decrypt_field(key, &item.url_enc)?;
        let tags: Vec<String> =
            serde_json::from_value(item.tags.clone()).unwrap_or_default();

        entries.push(ItemListEntry {
            id: item.id,
            vault_id: item.vault_id,
            folder_id: item.folder_id,
            name,
            login,
            url,
            tags,
            color_code: item.color_code,
            is_favorite: fav_set.contains(&item.id),
            created_at: item.created_at,
            updated_at: item.updated_at,
        });
    }

    Ok(entries)
}

/// Get a single item with all decrypted fields.
/// Also records a recent view.
pub async fn get_item(
    pool: &PgPool,
    key: &[u8],
    item_id: Uuid,
    user_id: Uuid,
) -> Result<ItemView, AppError> {
    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1 AND deleted_at IS NULL")
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ItemNotFound)?;

    verify_vault_access(pool, item.vault_id, user_id).await?;

    // Record recent view (upsert)
    record_recent_view(pool, user_id, item_id).await?;

    decrypt_item_to_view(key, &item, pool, user_id).await
}

/// Create a new item (password) and auto-create snapshot.
pub async fn create_item(
    pool: &PgPool,
    key: &[u8],
    user_id: Uuid,
    input: CreateItem,
) -> Result<ItemView, AppError> {
    verify_vault_write_access(pool, input.vault_id, user_id).await?;

    // Validate
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Item name is required".into()));
    }
    if input.name.len() > MAX_NAME_LENGTH {
        return Err(AppError::Validation(format!(
            "Item name must not exceed {MAX_NAME_LENGTH} characters"
        )));
    }

    // Validate folder belongs to vault
    if let Some(folder_id) = input.folder_id {
        let folder_exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM folders WHERE id = $1 AND vault_id = $2",
        )
        .bind(folder_id)
        .bind(input.vault_id)
        .fetch_optional(pool)
        .await?;
        if folder_exists.is_none() {
            return Err(AppError::FolderNotFound);
        }
    }

    // Encrypt fields
    let name_enc = encrypt_field(key, input.name.trim())?;
    let login_enc = encrypt_field(key, input.login.as_deref().unwrap_or(""))?;
    let password_enc = encrypt_field(key, input.password.as_deref().unwrap_or(""))?;
    let url_enc = encrypt_field(key, input.url.as_deref().unwrap_or(""))?;
    let description_enc = encrypt_field(key, input.description.as_deref().unwrap_or(""))?;

    let customs = input.customs.as_deref().unwrap_or(&[]);
    let customs_json = serde_json::to_string(customs).unwrap_or_default();
    let customs_enc = encrypt_field(key, &customs_json)?;

    let tags: Vec<String> = input.tags.unwrap_or_default();
    let tags_json = serde_json::to_value(&tags).unwrap_or_default();

    // Build search hashes
    let vault_salt = vault_service::get_vault_salt(pool, input.vault_id).await?;
    let search_hashes = build_search_hashes(
        input.name.trim(),
        input.login.as_deref().unwrap_or(""),
        input.url.as_deref().unwrap_or(""),
        &vault_salt,
    );

    let item_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO items (
            id, vault_id, folder_id, creator_id,
            name_enc, login_enc, password_enc, url_enc, description_enc, customs_enc,
            tags, search_hashes, color_code, password_changed_at, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14, $14)"#,
    )
    .bind(item_id)
    .bind(input.vault_id)
    .bind(input.folder_id)
    .bind(user_id)
    .bind(&name_enc)
    .bind(&login_enc)
    .bind(&password_enc)
    .bind(&url_enc)
    .bind(&description_enc)
    .bind(&customs_enc)
    .bind(&tags_json)
    .bind(&search_hashes)
    .bind(input.color_code.unwrap_or(0))
    .bind(now)
    .execute(pool)
    .await?;

    // Auto-create snapshot
    create_snapshot(pool, item_id, input.vault_id, user_id, &name_enc, &login_enc, &password_enc, &url_enc, &description_enc, &customs_enc, &tags_json).await?;

    tracing::info!(item_id = %item_id, vault_id = %input.vault_id, "Item created");

    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_one(pool)
        .await?;

    decrypt_item_to_view(key, &item, pool, user_id).await
}

/// Update an item and auto-create snapshot.
pub async fn update_item(
    pool: &PgPool,
    key: &[u8],
    item_id: Uuid,
    user_id: Uuid,
    input: UpdateItem,
) -> Result<ItemView, AppError> {
    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1 AND deleted_at IS NULL")
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ItemNotFound)?;

    verify_vault_write_access(pool, item.vault_id, user_id).await?;

    // Decrypt current values as fallback
    let current_name = decrypt_field(key, &item.name_enc)?;
    let current_login = decrypt_field(key, &item.login_enc)?;
    let current_password = decrypt_field(key, &item.password_enc)?;
    let current_url = decrypt_field(key, &item.url_enc)?;
    let current_description = decrypt_field(key, &item.description_enc)?;
    let current_customs_json = decrypt_field(key, &item.customs_enc)?;

    let new_name = input.name.as_deref().unwrap_or(&current_name);
    let new_login = input.login.as_deref().unwrap_or(&current_login);
    let new_password = input.password.as_deref().unwrap_or(&current_password);
    let new_url = input.url.as_deref().unwrap_or(&current_url);
    let new_description = input.description.as_deref().unwrap_or(&current_description);

    if new_name.trim().is_empty() {
        return Err(AppError::Validation("Item name is required".into()));
    }
    if new_name.len() > MAX_NAME_LENGTH {
        return Err(AppError::Validation(format!(
            "Item name must not exceed {MAX_NAME_LENGTH} characters"
        )));
    }

    let new_customs_json = if let Some(ref customs) = input.customs {
        serde_json::to_string(customs).unwrap_or_default()
    } else {
        current_customs_json
    };

    let new_tags: Vec<String> = input.tags.unwrap_or_else(|| {
        serde_json::from_value(item.tags.clone()).unwrap_or_default()
    });

    // Validate folder if being moved
    let new_folder_id = if input.folder_id.is_some() {
        if let Some(fid) = input.folder_id {
            let exists: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM folders WHERE id = $1 AND vault_id = $2",
            )
            .bind(fid)
            .bind(item.vault_id)
            .fetch_optional(pool)
            .await?;
            if exists.is_none() {
                return Err(AppError::FolderNotFound);
            }
        }
        input.folder_id
    } else {
        item.folder_id
    };

    // Re-encrypt
    let name_enc = encrypt_field(key, new_name.trim())?;
    let login_enc = encrypt_field(key, new_login)?;
    let password_enc = encrypt_field(key, new_password)?;
    let url_enc = encrypt_field(key, new_url)?;
    let description_enc = encrypt_field(key, new_description)?;
    let customs_enc = encrypt_field(key, &new_customs_json)?;
    let tags_json = serde_json::to_value(&new_tags).unwrap_or_default();

    // Update search hashes
    let vault_salt = vault_service::get_vault_salt(pool, item.vault_id).await?;
    let search_hashes = build_search_hashes(new_name.trim(), new_login, new_url, &vault_salt);

    let now = Utc::now();

    // Track password change timestamp
    let password_actually_changed = input.password.is_some() && new_password != current_password;
    let password_changed_at = if password_actually_changed {
        now
    } else {
        item.password_changed_at
    };

    sqlx::query(
        r#"UPDATE items SET
            name_enc = $1, login_enc = $2, password_enc = $3, url_enc = $4,
            description_enc = $5, customs_enc = $6, tags = $7, search_hashes = $8,
            color_code = $9, folder_id = $10, password_changed_at = $11, updated_at = $12
        WHERE id = $13"#,
    )
    .bind(&name_enc)
    .bind(&login_enc)
    .bind(&password_enc)
    .bind(&url_enc)
    .bind(&description_enc)
    .bind(&customs_enc)
    .bind(&tags_json)
    .bind(&search_hashes)
    .bind(input.color_code.unwrap_or(item.color_code))
    .bind(new_folder_id)
    .bind(password_changed_at)
    .bind(now)
    .bind(item_id)
    .execute(pool)
    .await?;

    // Auto-create snapshot
    create_snapshot(pool, item_id, item.vault_id, user_id, &name_enc, &login_enc, &password_enc, &url_enc, &description_enc, &customs_enc, &tags_json).await?;

    let updated = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_one(pool)
        .await?;

    decrypt_item_to_view(key, &updated, pool, user_id).await
}

/// Soft-delete an item (move to trash).
pub async fn delete_item(
    pool: &PgPool,
    _storage: Option<&lockso_db::storage::FileStorage>,
    item_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, AppError> {
    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1 AND deleted_at IS NULL")
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ItemNotFound)?;

    verify_vault_write_access(pool, item.vault_id, user_id).await?;

    sqlx::query("UPDATE items SET deleted_at = NOW(), deleted_by = $1 WHERE id = $2")
        .bind(user_id)
        .bind(item_id)
        .execute(pool)
        .await?;

    // Remove from favorites and recent views so trashed items don't appear there
    sqlx::query("DELETE FROM favorites WHERE item_id = $1")
        .bind(item_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM recent_views WHERE item_id = $1")
        .bind(item_id)
        .execute(pool)
        .await
        .ok();

    tracing::info!(item_id = %item_id, vault_id = %item.vault_id, "Item moved to trash");
    Ok(item.vault_id)
}

// ─── Trash ───

/// List all trashed items across vaults the user has access to.
pub async fn list_trash(
    pool: &PgPool,
    key: &[u8],
    user_id: Uuid,
) -> Result<Vec<TrashListEntry>, AppError> {
    // Items in trash across accessible vaults, joined with vault name
    let rows: Vec<(Uuid, Uuid, Option<Uuid>, String, String, String, i16, String, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
        r#"SELECT i.id, i.vault_id, i.folder_id, i.name_enc, i.login_enc, i.url_enc,
                  i.color_code, v.name AS vault_name, i.deleted_at, i.created_at
           FROM items i
           JOIN vaults v ON v.id = i.vault_id
           WHERE i.deleted_at IS NOT NULL
           AND (
               v.creator_id = $1
               OR EXISTS (
                   SELECT 1 FROM vault_user_accesses vua
                   JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                   WHERE vua.vault_id = v.id AND vua.user_id = $1 AND ra.code IN ('write', 'admin', 'manage')
               )
           )
           ORDER BY i.deleted_at DESC
           LIMIT 500"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut entries = Vec::with_capacity(rows.len());
    for (id, vault_id, folder_id, name_enc, login_enc, url_enc, color_code, vault_name, deleted_at, created_at) in rows {
        let name = decrypt_field(key, &name_enc)?;
        let login = decrypt_field(key, &login_enc)?;
        let url = decrypt_field(key, &url_enc)?;
        entries.push(TrashListEntry {
            id,
            vault_id,
            folder_id,
            name,
            login,
            url,
            color_code,
            vault_name,
            deleted_at,
            created_at,
        });
    }
    Ok(entries)
}

/// Get trash item count for badge.
pub async fn trash_count(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<i64, AppError> {
    let (count,): (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM items i
           JOIN vaults v ON v.id = i.vault_id
           WHERE i.deleted_at IS NOT NULL
           AND (
               v.creator_id = $1
               OR EXISTS (
                   SELECT 1 FROM vault_user_accesses vua
                   JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                   WHERE vua.vault_id = v.id AND vua.user_id = $1 AND ra.code IN ('write', 'admin', 'manage')
               )
           )"#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// Restore an item from trash.
pub async fn restore_item(
    pool: &PgPool,
    item_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, AppError> {
    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1 AND deleted_at IS NOT NULL")
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ItemNotFound)?;

    verify_vault_write_access(pool, item.vault_id, user_id).await?;

    // If original folder was deleted, move to vault root
    let folder_id = if let Some(fid) = item.folder_id {
        let exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM folders WHERE id = $1")
            .bind(fid)
            .fetch_optional(pool)
            .await?;
        if exists.is_some() { Some(fid) } else { None }
    } else {
        None
    };

    sqlx::query("UPDATE items SET deleted_at = NULL, deleted_by = NULL, folder_id = $1 WHERE id = $2")
        .bind(folder_id)
        .bind(item_id)
        .execute(pool)
        .await?;

    tracing::info!(item_id = %item_id, "Item restored from trash");
    Ok(item.vault_id)
}

/// Permanently delete a trashed item.
pub async fn permanent_delete_item(
    pool: &PgPool,
    storage: Option<&lockso_db::storage::FileStorage>,
    item_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, AppError> {
    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1 AND deleted_at IS NOT NULL")
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ItemNotFound)?;

    verify_vault_write_access(pool, item.vault_id, user_id).await?;

    // Delete S3 attachments
    if let Some(storage) = storage {
        attachment_service::delete_all_for_item(pool, storage, item_id).await.ok();
    }

    sqlx::query("DELETE FROM items WHERE id = $1")
        .bind(item_id)
        .execute(pool)
        .await?;

    tracing::info!(item_id = %item_id, "Item permanently deleted");
    Ok(item.vault_id)
}

/// Empty all trash for the user.
pub async fn empty_trash(
    pool: &PgPool,
    storage: Option<&lockso_db::storage::FileStorage>,
    user_id: Uuid,
) -> Result<u64, AppError> {
    // Get all trashed item IDs the user can write to
    let items: Vec<(Uuid,)> = sqlx::query_as(
        r#"SELECT i.id FROM items i
           JOIN vaults v ON v.id = i.vault_id
           WHERE i.deleted_at IS NOT NULL
           AND (
               v.creator_id = $1
               OR EXISTS (
                   SELECT 1 FROM vault_user_accesses vua
                   JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                   WHERE vua.vault_id = v.id AND vua.user_id = $1 AND ra.code IN ('write', 'admin', 'manage')
               )
           )"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let count = items.len() as u64;

    for (item_id,) in &items {
        if let Some(storage) = storage {
            attachment_service::delete_all_for_item(pool, storage, *item_id).await.ok();
        }
        sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(item_id)
            .execute(pool)
            .await
            .ok();
    }

    tracing::info!(count = count, "Trash emptied");
    Ok(count)
}

/// Auto-purge expired trash items (called by background task).
pub async fn auto_purge_trash(
    pool: &PgPool,
    storage: Option<&lockso_db::storage::FileStorage>,
    retention_days: u32,
) -> Result<u64, AppError> {
    let items: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM items WHERE deleted_at IS NOT NULL AND deleted_at < NOW() - make_interval(days => $1)",
    )
    .bind(retention_days as i32)
    .fetch_all(pool)
    .await?;

    let count = items.len() as u64;

    for (item_id,) in &items {
        if let Some(storage) = storage {
            attachment_service::delete_all_for_item(pool, storage, *item_id).await.ok();
        }
        sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(item_id)
            .execute(pool)
            .await
            .ok();
    }

    if count > 0 {
        tracing::info!(count = count, retention_days = retention_days, "Auto-purged expired trash items");
    }
    Ok(count)
}

/// Move an item to a different folder or vault.
pub async fn move_item(
    pool: &PgPool,
    key: &[u8],
    item_id: Uuid,
    user_id: Uuid,
    input: MoveItem,
) -> Result<ItemView, AppError> {
    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1 AND deleted_at IS NULL")
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ItemNotFound)?;

    verify_vault_write_access(pool, item.vault_id, user_id).await?;

    let target_vault_id = input.vault_id.unwrap_or(item.vault_id);
    if target_vault_id != item.vault_id {
        verify_vault_write_access(pool, target_vault_id, user_id).await?;
    }

    // Validate folder belongs to target vault
    if let Some(folder_id) = input.folder_id {
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM folders WHERE id = $1 AND vault_id = $2",
        )
        .bind(folder_id)
        .bind(target_vault_id)
        .fetch_optional(pool)
        .await?;
        if exists.is_none() {
            return Err(AppError::FolderNotFound);
        }
    }

    // If vault changed, re-hash search hashes with new vault salt
    let now = Utc::now();
    if target_vault_id != item.vault_id {
        let new_name = decrypt_field(key, &item.name_enc)?;
        let new_login = decrypt_field(key, &item.login_enc)?;
        let new_url = decrypt_field(key, &item.url_enc)?;
        let new_salt = vault_service::get_vault_salt(pool, target_vault_id).await?;
        let new_hashes = build_search_hashes(&new_name, &new_login, &new_url, &new_salt);

        sqlx::query(
            "UPDATE items SET vault_id = $1, folder_id = $2, search_hashes = $3, updated_at = $4 WHERE id = $5",
        )
        .bind(target_vault_id)
        .bind(input.folder_id)
        .bind(&new_hashes)
        .bind(now)
        .bind(item_id)
        .execute(pool)
        .await?;

        // Update snapshots to point to the new vault
        sqlx::query("UPDATE snapshots SET vault_id = $1 WHERE item_id = $2")
            .bind(target_vault_id)
            .bind(item_id)
            .execute(pool)
            .await?;

        // Update attachments to point to the new vault
        sqlx::query("UPDATE attachments SET vault_id = $1 WHERE item_id = $2")
            .bind(target_vault_id)
            .bind(item_id)
            .execute(pool)
            .await?;
    } else {
        sqlx::query("UPDATE items SET folder_id = $1, updated_at = $2 WHERE id = $3")
            .bind(input.folder_id)
            .bind(now)
            .bind(item_id)
            .execute(pool)
            .await?;
    }

    let updated = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_one(pool)
        .await?;

    decrypt_item_to_view(key, &updated, pool, user_id).await
}

/// Blind search across vaults.
pub async fn search_items(
    pool: &PgPool,
    key: &[u8],
    user_id: Uuid,
    input: SearchRequest,
) -> Result<Vec<ItemListEntry>, AppError> {
    if input.query.trim().is_empty() {
        return Ok(vec![]);
    }

    // Get vaults user can access (owned + shared)
    let vaults: Vec<(Uuid, String)> = if let Some(vault_id) = input.vault_id {
        sqlx::query_as(
            r#"SELECT v.id, v.salt FROM vaults v
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
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"SELECT v.id, v.salt FROM vaults v
            WHERE v.creator_id = $1
            OR EXISTS (
                SELECT 1 FROM vault_user_accesses vua
                JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                WHERE vua.vault_id = v.id AND vua.user_id = $1 AND ra.code != 'forbidden'
            )"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
    };

    let mut all_entries = Vec::new();

    // Pre-fetch favorites once (not per vault)
    let fav_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT item_id FROM favorites WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let fav_set: std::collections::HashSet<Uuid> =
        fav_ids.into_iter().map(|(id,)| id).collect();

    for (vault_id, salt) in &vaults {
        if all_entries.len() >= MAX_SEARCH_RESULTS {
            break;
        }

        let query_hash = blind_search_hash(input.query.trim(), salt);

        // Search items where any search_hash value matches
        let items = sqlx::query_as::<_, Item>(
            r#"SELECT * FROM items
            WHERE vault_id = $1
            AND deleted_at IS NULL
            AND (
                search_hashes @> jsonb_build_object('name', $2)
                OR search_hashes @> jsonb_build_object('login', $2)
                OR search_hashes @> jsonb_build_object('url', $2)
            )
            ORDER BY updated_at DESC
            LIMIT 50"#,
        )
        .bind(vault_id)
        .bind(&query_hash)
        .fetch_all(pool)
        .await?;

        for item in items {
            let name = decrypt_field(key, &item.name_enc)?;
            let login = decrypt_field(key, &item.login_enc)?;
            let url = decrypt_field(key, &item.url_enc)?;
            let tags: Vec<String> =
                serde_json::from_value(item.tags.clone()).unwrap_or_default();

            all_entries.push(ItemListEntry {
                id: item.id,
                vault_id: item.vault_id,
                folder_id: item.folder_id,
                name,
                login,
                url,
                tags,
                color_code: item.color_code,
                is_favorite: fav_set.contains(&item.id),
                created_at: item.created_at,
                updated_at: item.updated_at,
            });
        }
    }

    all_entries.truncate(MAX_SEARCH_RESULTS);
    Ok(all_entries)
}

// ─── Snapshots ───

/// List snapshots for an item.
pub async fn list_snapshots(
    pool: &PgPool,
    key: &[u8],
    item_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<SnapshotListEntry>, AppError> {
    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1 AND deleted_at IS NULL")
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ItemNotFound)?;

    verify_vault_access(pool, item.vault_id, user_id).await?;

    let snapshots = sqlx::query_as::<_, Snapshot>(
        "SELECT * FROM snapshots WHERE item_id = $1 ORDER BY created_at DESC",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;

    let mut entries = Vec::with_capacity(snapshots.len());
    for s in snapshots {
        entries.push(SnapshotListEntry {
            id: s.id,
            item_id: s.item_id,
            name: decrypt_field(key, &s.name_enc)?,
            login: decrypt_field(key, &s.login_enc)?,
            created_by: s.created_by,
            created_at: s.created_at,
        });
    }

    Ok(entries)
}

/// Get a single snapshot with full decrypted content.
pub async fn get_snapshot(
    pool: &PgPool,
    key: &[u8],
    item_id: Uuid,
    snapshot_id: Uuid,
    user_id: Uuid,
) -> Result<SnapshotView, AppError> {
    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1 AND deleted_at IS NULL")
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ItemNotFound)?;

    verify_vault_access(pool, item.vault_id, user_id).await?;

    let s = sqlx::query_as::<_, Snapshot>(
        "SELECT * FROM snapshots WHERE id = $1 AND item_id = $2",
    )
    .bind(snapshot_id)
    .bind(item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::SnapshotNotFound)?;

    let customs_json = decrypt_field(key, &s.customs_enc)?;
    let customs: Vec<CustomField> = if customs_json.is_empty() {
        vec![]
    } else {
        serde_json::from_str(&customs_json).unwrap_or_default()
    };

    Ok(SnapshotView {
        id: s.id,
        item_id: s.item_id,
        name: decrypt_field(key, &s.name_enc)?,
        login: decrypt_field(key, &s.login_enc)?,
        password: decrypt_field(key, &s.password_enc)?,
        url: decrypt_field(key, &s.url_enc)?,
        description: decrypt_field(key, &s.description_enc)?,
        customs,
        tags: serde_json::from_value(s.tags).unwrap_or_default(),
        created_by: s.created_by,
        created_at: s.created_at,
    })
}

/// Revert item to a snapshot state.
pub async fn revert_to_snapshot(
    pool: &PgPool,
    key: &[u8],
    item_id: Uuid,
    snapshot_id: Uuid,
    user_id: Uuid,
) -> Result<ItemView, AppError> {
    let snapshot = get_snapshot(pool, key, item_id, snapshot_id, user_id).await?;

    // Build update from snapshot data
    let update = UpdateItem {
        name: Some(snapshot.name),
        login: Some(snapshot.login),
        password: Some(snapshot.password),
        url: Some(snapshot.url),
        description: Some(snapshot.description),
        customs: Some(snapshot.customs),
        tags: Some(snapshot.tags),
        color_code: None,
        folder_id: None,
    };

    update_item(pool, key, item_id, user_id, update).await
}

// ─── Favorites ───

/// Toggle favorite for an item.
pub async fn toggle_favorite(
    pool: &PgPool,
    user_id: Uuid,
    item_id: Uuid,
) -> Result<bool, AppError> {
    // Check item exists and user has access
    let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = $1 AND deleted_at IS NULL")
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::ItemNotFound)?;

    verify_vault_access(pool, item.vault_id, user_id).await?;

    // Check if already favorited
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM favorites WHERE user_id = $1 AND item_id = $2",
    )
    .bind(user_id)
    .bind(item_id)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        // Remove favorite
        sqlx::query("DELETE FROM favorites WHERE user_id = $1 AND item_id = $2")
            .bind(user_id)
            .bind(item_id)
            .execute(pool)
            .await?;
        Ok(false) // Was favorite, now removed
    } else {
        // Add as favorite — ON CONFLICT handles race condition
        sqlx::query(
            r#"INSERT INTO favorites (id, user_id, item_id, sort_order, created_at)
            VALUES ($1, $2, $3, 0, NOW())
            ON CONFLICT (user_id, item_id) DO NOTHING"#,
        )
        .bind(Uuid::now_v7())
        .bind(user_id)
        .bind(item_id)
        .execute(pool)
        .await?;
        Ok(true) // Added as favorite
    }
}

/// Get recent items for a user.
pub async fn get_recent_items(
    pool: &PgPool,
    key: &[u8],
    user_id: Uuid,
) -> Result<Vec<ItemListEntry>, AppError> {
    let recent: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT item_id FROM recent_views WHERE user_id = $1 ORDER BY viewed_at DESC LIMIT 20",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let fav_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT item_id FROM favorites WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let fav_set: std::collections::HashSet<Uuid> =
        fav_ids.into_iter().map(|(id,)| id).collect();

    // Batch-fetch recent items, filtered by current vault access (avoids N+1)
    let item_ids: Vec<Uuid> = recent.iter().map(|(id,)| *id).collect();
    let items = sqlx::query_as::<_, Item>(
        r#"SELECT i.* FROM items i
        JOIN vaults v ON v.id = i.vault_id
        WHERE i.id = ANY($1) AND i.deleted_at IS NULL AND (
            v.creator_id = $2
            OR EXISTS (
                SELECT 1 FROM vault_user_accesses vua
                JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                WHERE vua.vault_id = v.id AND vua.user_id = $2 AND ra.code != 'forbidden'
            )
        )"#,
    )
    .bind(&item_ids)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    // Build a map for O(1) lookup, preserving recent order
    let item_map: std::collections::HashMap<Uuid, &Item> =
        items.iter().map(|i| (i.id, i)).collect();

    let mut entries = Vec::new();
    for (item_id,) in recent {
        let item = item_map.get(&item_id).copied();

        if let Some(item) = item {
            let name = decrypt_field(key, &item.name_enc)?;
            let login = decrypt_field(key, &item.login_enc)?;
            let url = decrypt_field(key, &item.url_enc)?;
            let tags: Vec<String> =
                serde_json::from_value(item.tags.clone()).unwrap_or_default();

            entries.push(ItemListEntry {
                id: item.id,
                vault_id: item.vault_id,
                folder_id: item.folder_id,
                name,
                login,
                url,
                tags,
                color_code: item.color_code,
                is_favorite: fav_set.contains(&item.id),
                created_at: item.created_at,
                updated_at: item.updated_at,
            });
        }
    }

    Ok(entries)
}

// ─── Internal helpers ───

/// Verify user has at least read access to the vault (owner or any non-forbidden shared access).
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

async fn decrypt_item_to_view(
    key: &[u8],
    item: &Item,
    pool: &PgPool,
    user_id: Uuid,
) -> Result<ItemView, AppError> {
    let customs_json = decrypt_field(key, &item.customs_enc)?;
    let customs: Vec<CustomField> = if customs_json.is_empty() {
        vec![]
    } else {
        serde_json::from_str(&customs_json).unwrap_or_default()
    };

    let is_favorite: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM favorites WHERE user_id = $1 AND item_id = $2",
    )
    .bind(user_id)
    .bind(item.id)
    .fetch_optional(pool)
    .await?;

    Ok(ItemView {
        id: item.id,
        vault_id: item.vault_id,
        folder_id: item.folder_id,
        creator_id: item.creator_id,
        name: decrypt_field(key, &item.name_enc)?,
        login: decrypt_field(key, &item.login_enc)?,
        password: decrypt_field(key, &item.password_enc)?,
        url: decrypt_field(key, &item.url_enc)?,
        description: decrypt_field(key, &item.description_enc)?,
        customs,
        tags: serde_json::from_value(item.tags.clone()).unwrap_or_default(),
        color_code: item.color_code,
        is_favorite: is_favorite.is_some(),
        password_changed_at: item.password_changed_at,
        created_at: item.created_at,
        updated_at: item.updated_at,
    })
}

fn build_search_hashes(
    name: &str,
    login: &str,
    url: &str,
    salt: &str,
) -> serde_json::Value {
    let mut hashes = serde_json::Map::new();

    if !name.is_empty() {
        hashes.insert(
            "name".to_string(),
            serde_json::Value::String(blind_search_hash(name, salt)),
        );
    }
    if !login.is_empty() {
        hashes.insert(
            "login".to_string(),
            serde_json::Value::String(blind_search_hash(login, salt)),
        );
    }
    if !url.is_empty() {
        hashes.insert(
            "url".to_string(),
            serde_json::Value::String(blind_search_hash(url, salt)),
        );
    }

    serde_json::Value::Object(hashes)
}

async fn create_snapshot(
    pool: &PgPool,
    item_id: Uuid,
    vault_id: Uuid,
    user_id: Uuid,
    name_enc: &str,
    login_enc: &str,
    password_enc: &str,
    url_enc: &str,
    description_enc: &str,
    customs_enc: &str,
    tags: &serde_json::Value,
) -> Result<(), AppError> {
    sqlx::query(
        r#"INSERT INTO snapshots (
            id, item_id, vault_id, name_enc, login_enc, password_enc,
            url_enc, description_enc, customs_enc, tags, created_by, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())"#,
    )
    .bind(Uuid::now_v7())
    .bind(item_id)
    .bind(vault_id)
    .bind(name_enc)
    .bind(login_enc)
    .bind(password_enc)
    .bind(url_enc)
    .bind(description_enc)
    .bind(customs_enc)
    .bind(tags)
    .bind(user_id)
    .execute(pool)
    .await?;

    // Prune old snapshots beyond the limit
    sqlx::query(
        r#"DELETE FROM snapshots WHERE id IN (
            SELECT id FROM snapshots WHERE item_id = $1
            ORDER BY created_at DESC
            OFFSET $2
        )"#,
    )
    .bind(item_id)
    .bind(MAX_SNAPSHOTS_PER_ITEM)
    .execute(pool)
    .await
    .ok(); // Best-effort pruning

    Ok(())
}

async fn record_recent_view(pool: &PgPool, user_id: Uuid, item_id: Uuid) -> Result<(), AppError> {
    // Upsert recent view
    sqlx::query(
        r#"INSERT INTO recent_views (id, user_id, item_id, viewed_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (user_id, item_id) DO UPDATE SET viewed_at = NOW()"#,
    )
    .bind(Uuid::now_v7())
    .bind(user_id)
    .bind(item_id)
    .execute(pool)
    .await?;

    // Trim old entries beyond limit
    sqlx::query(
        r#"DELETE FROM recent_views WHERE id IN (
            SELECT id FROM recent_views
            WHERE user_id = $1
            ORDER BY viewed_at DESC
            OFFSET $2
        )"#,
    )
    .bind(user_id)
    .bind(MAX_RECENT_VIEWS)
    .execute(pool)
    .await?;

    Ok(())
}
