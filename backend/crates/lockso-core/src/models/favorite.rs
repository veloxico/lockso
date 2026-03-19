use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// Favorite entry — per-user bookmark on an item.
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub item_id: Uuid,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}
