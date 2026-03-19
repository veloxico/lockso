use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User group — a named collection of users for sharing.
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroup {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub creator_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lightweight group item for list views (with member count).
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroupListItem {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub member_count: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Full group view with members.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroupView {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub creator_id: Option<Uuid>,
    pub is_active: bool,
    pub members: Vec<GroupMember>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A member of a group.
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMember {
    pub id: Uuid,
    pub user_id: Uuid,
    pub login: String,
    pub full_name: String,
    pub email: Option<String>,
    pub added_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a group.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// Request to update a group.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request to add a member.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}
