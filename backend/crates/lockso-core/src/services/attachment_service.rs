use sqlx::PgPool;
use uuid::Uuid;

use crate::encryption::{decrypt_field, encrypt_field};
use crate::error::AppError;
use crate::models::attachment::{Attachment, AttachmentView};
use lockso_db::storage::FileStorage;

/// Max file size: 50 MB.
pub const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
/// Max attachments per item.
const MAX_ATTACHMENTS_PER_ITEM: i64 = 20;

/// List attachments for an item (decrypted filenames).
pub async fn list_attachments(
    pool: &PgPool,
    key: &[u8],
    item_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<AttachmentView>, AppError> {
    verify_item_access(pool, item_id, user_id).await?;

    let rows = sqlx::query_as::<_, Attachment>(
        "SELECT * FROM attachments WHERE item_id = $1 ORDER BY created_at ASC",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;

    let mut views = Vec::with_capacity(rows.len());
    for a in rows {
        views.push(AttachmentView {
            id: a.id,
            item_id: a.item_id,
            name: decrypt_field(key, &a.name_enc)?,
            size_bytes: a.size_bytes,
            mime_type: a.mime_type,
            uploader_id: a.uploader_id,
            created_at: a.created_at,
        });
    }

    Ok(views)
}

/// Upload a file attachment.
///
/// The caller provides the raw file bytes (NOT encrypted).
/// This function encrypts the data before uploading to S3.
pub async fn upload_attachment(
    pool: &PgPool,
    storage: &FileStorage,
    key: &[u8],
    item_id: Uuid,
    user_id: Uuid,
    file_name: &str,
    mime_type: &str,
    data: &[u8],
) -> Result<AttachmentView, AppError> {
    if data.is_empty() {
        return Err(AppError::Validation("File is empty".into()));
    }
    if data.len() > MAX_FILE_SIZE {
        return Err(AppError::PayloadTooLarge(format!(
            "File exceeds maximum size of {} MB",
            MAX_FILE_SIZE / (1024 * 1024)
        )));
    }
    if file_name.trim().is_empty() {
        return Err(AppError::Validation("File name is required".into()));
    }

    let vault_id = verify_item_write_access(pool, item_id, user_id).await?;

    // Check attachment count limit
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM attachments WHERE item_id = $1")
            .bind(item_id)
            .fetch_one(pool)
            .await?;
    if count.0 >= MAX_ATTACHMENTS_PER_ITEM {
        return Err(AppError::Validation(format!(
            "Maximum {} attachments per item",
            MAX_ATTACHMENTS_PER_ITEM
        )));
    }

    // Encrypt file data with AES-256-GCM
    let encrypted_data = lockso_crypto::aes_gcm::encrypt(key, data)
        .map_err(|e| AppError::Internal(format!("Encryption failed: {e}")))?;

    // Generate opaque S3 key
    let attachment_id = Uuid::now_v7();
    let storage_key = format!("attachments/{vault_id}/{item_id}/{attachment_id}");

    // Upload to S3
    storage
        .put(&storage_key, &encrypted_data)
        .await
        .map_err(|e| AppError::Internal(format!("S3 upload failed: {e}")))?;

    // Encrypt filename
    let name_enc = encrypt_field(key, file_name.trim())?;

    // Sanitize mime type
    let safe_mime = if mime_type.len() > 255 || mime_type.is_empty() {
        "application/octet-stream"
    } else {
        mime_type
    };

    // Insert DB record
    let result = sqlx::query(
        r#"INSERT INTO attachments (id, item_id, vault_id, uploader_id, name_enc, storage_key, size_bytes, mime_type, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())"#,
    )
    .bind(attachment_id)
    .bind(item_id)
    .bind(vault_id)
    .bind(user_id)
    .bind(&name_enc)
    .bind(&storage_key)
    .bind(data.len() as i64)
    .bind(safe_mime)
    .execute(pool)
    .await;

    // If DB insert fails, clean up S3
    if let Err(e) = result {
        let _ = storage.delete(&storage_key).await;
        return Err(e.into());
    }

    tracing::info!(
        attachment_id = %attachment_id,
        item_id = %item_id,
        size = data.len(),
        "Attachment uploaded"
    );

    let row = sqlx::query_as::<_, Attachment>("SELECT * FROM attachments WHERE id = $1")
        .bind(attachment_id)
        .fetch_one(pool)
        .await?;

    Ok(AttachmentView {
        id: row.id,
        item_id: row.item_id,
        name: decrypt_field(key, &row.name_enc)?,
        size_bytes: row.size_bytes,
        mime_type: row.mime_type,
        uploader_id: row.uploader_id,
        created_at: row.created_at,
    })
}

/// Download an attachment — returns decrypted bytes plus filename and mime type.
pub async fn download_attachment(
    pool: &PgPool,
    storage: &FileStorage,
    key: &[u8],
    attachment_id: Uuid,
    user_id: Uuid,
) -> Result<(Vec<u8>, String, String), AppError> {
    let attachment = sqlx::query_as::<_, Attachment>(
        "SELECT * FROM attachments WHERE id = $1",
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::AttachmentNotFound)?;

    verify_item_access(pool, attachment.item_id, user_id).await?;

    // Download encrypted data from S3
    let encrypted_data = storage
        .get(&attachment.storage_key)
        .await
        .map_err(|e| AppError::Internal(format!("S3 download failed: {e}")))?;

    // Decrypt
    let data = lockso_crypto::aes_gcm::decrypt(key, &encrypted_data)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {e}")))?;

    let file_name = decrypt_field(key, &attachment.name_enc)?;

    Ok((data, file_name, attachment.mime_type))
}

/// Delete an attachment (DB record + S3 object).
pub async fn delete_attachment(
    pool: &PgPool,
    storage: &FileStorage,
    attachment_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let attachment = sqlx::query_as::<_, Attachment>(
        "SELECT * FROM attachments WHERE id = $1",
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::AttachmentNotFound)?;

    verify_item_write_access(pool, attachment.item_id, user_id).await?;

    // Delete from DB first
    sqlx::query("DELETE FROM attachments WHERE id = $1")
        .bind(attachment_id)
        .execute(pool)
        .await?;

    // Delete from S3 (best-effort — data is orphaned if this fails, but
    // it's encrypted and will be cleaned up eventually)
    if let Err(e) = storage.delete(&attachment.storage_key).await {
        tracing::warn!(
            attachment_id = %attachment_id,
            storage_key = %attachment.storage_key,
            error = %e,
            "Failed to delete attachment from S3"
        );
    }

    tracing::info!(
        attachment_id = %attachment_id,
        item_id = %attachment.item_id,
        "Attachment deleted"
    );

    Ok(())
}

/// Delete all attachments for an item (used when deleting items).
pub async fn delete_all_for_item(
    pool: &PgPool,
    storage: &FileStorage,
    item_id: Uuid,
) -> Result<(), AppError> {
    let attachments = sqlx::query_as::<_, Attachment>(
        "SELECT * FROM attachments WHERE item_id = $1",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?;

    // Delete all DB records
    sqlx::query("DELETE FROM attachments WHERE item_id = $1")
        .bind(item_id)
        .execute(pool)
        .await?;

    // Best-effort S3 cleanup
    for a in attachments {
        if let Err(e) = storage.delete(&a.storage_key).await {
            tracing::warn!(
                storage_key = %a.storage_key,
                error = %e,
                "Failed to delete attachment from S3"
            );
        }
    }

    Ok(())
}

/// Delete all attachments for a vault (used when deleting vaults).
/// Must be called BEFORE deleting the vault to prevent S3 orphans.
pub async fn delete_all_for_vault(
    pool: &PgPool,
    storage: &FileStorage,
    vault_id: Uuid,
) -> Result<(), AppError> {
    let attachments = sqlx::query_as::<_, Attachment>(
        "SELECT * FROM attachments WHERE vault_id = $1",
    )
    .bind(vault_id)
    .fetch_all(pool)
    .await?;

    if attachments.is_empty() {
        return Ok(());
    }

    // Delete all DB records
    sqlx::query("DELETE FROM attachments WHERE vault_id = $1")
        .bind(vault_id)
        .execute(pool)
        .await?;

    // Best-effort S3 cleanup
    for a in attachments {
        if let Err(e) = storage.delete(&a.storage_key).await {
            tracing::warn!(
                storage_key = %a.storage_key,
                error = %e,
                "Failed to delete attachment from S3 during vault deletion"
            );
        }
    }

    tracing::info!(vault_id = %vault_id, "All vault attachments deleted from S3");
    Ok(())
}

// ─── Internal helpers ───

/// Verify user has access to the item's vault. Returns vault_id on success.
async fn verify_item_access(pool: &PgPool, item_id: Uuid, user_id: Uuid) -> Result<Uuid, AppError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT i.vault_id FROM items i
        JOIN vaults v ON v.id = i.vault_id
        WHERE i.id = $1 AND (
            v.creator_id = $2
            OR EXISTS (
                SELECT 1 FROM vault_user_accesses vua
                JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                WHERE vua.vault_id = v.id AND vua.user_id = $2 AND ra.code != 'forbidden'
            )
        )"#,
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((vault_id,)) => Ok(vault_id),
        None => Err(AppError::ItemNotFound),
    }
}

/// Verify user has write access to the item's vault. Returns vault_id on success.
async fn verify_item_write_access(pool: &PgPool, item_id: Uuid, user_id: Uuid) -> Result<Uuid, AppError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT i.vault_id FROM items i
        JOIN vaults v ON v.id = i.vault_id
        WHERE i.id = $1 AND (
            v.creator_id = $2
            OR EXISTS (
                SELECT 1 FROM vault_user_accesses vua
                JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                WHERE vua.vault_id = v.id AND vua.user_id = $2
                AND ra.code IN ('write', 'admin', 'manage')
            )
        )"#,
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((vault_id,)) => Ok(vault_id),
        None => Err(AppError::Forbidden),
    }
}
