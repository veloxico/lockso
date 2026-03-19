use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Folder entity — organizes items within a vault.
#[derive(Debug, Clone, FromRow)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub vault_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub ancestor_ids: serde_json::Value,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Folder view for API responses.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderView {
    pub id: Uuid,
    pub name: String,
    pub vault_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub position: i32,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Folder tree node — includes children for building tree.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderTreeNode {
    pub id: Uuid,
    pub name: String,
    pub parent_folder_id: Option<Uuid>,
    pub position: i32,
    pub item_count: i64,
    pub children: Vec<FolderTreeNode>,
}

/// Create folder DTO.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolder {
    pub name: String,
    pub vault_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub position: Option<i32>,
}

/// Update folder DTO.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFolder {
    pub name: Option<String>,
    pub position: Option<i32>,
}

/// Move folder DTO.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveFolder {
    pub parent_folder_id: Option<Uuid>,
}
