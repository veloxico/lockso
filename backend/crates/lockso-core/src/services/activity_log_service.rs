use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::activity_log::{
    ActivityLogQuery, ActivityLogRow, ActivityLogView, PaginatedActivityLogs,
};

const DEFAULT_PER_PAGE: u32 = 50;
const MAX_PER_PAGE: u32 = 100;

/// Insert a new activity log entry.
///
/// Fire-and-forget: errors are logged but never propagated to callers.
pub async fn log_activity(
    pool: &PgPool,
    user_id: Option<Uuid>,
    action: &str,
    resource_type: Option<&str>,
    resource_id: Option<Uuid>,
    vault_id: Option<Uuid>,
    client_ip: Option<&str>,
    user_agent: Option<&str>,
    details: serde_json::Value,
) {
    let id = Uuid::now_v7();
    let result = sqlx::query(
        r#"INSERT INTO activity_logs
           (id, user_id, action, resource_type, resource_id, vault_id, client_ip, user_agent, details)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(vault_id)
    .bind(client_ip)
    .bind(user_agent)
    .bind(&details)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, action, "Failed to write activity log");
    }

    // Fire webhook notifications for this event (async, non-blocking).
    // We need the encryption key from AppState — pass pool and get key from env.
    // Using a simplified approach: only fire if webhooks table exists.
    let detail_str = details.to_string();
    let ip_str = client_ip.unwrap_or("-").to_string();
    let message = format!("IP: {ip_str}\nDetails: {detail_str}");
    let action_owned = action.to_string();
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        // Try to load encryption key from env (same as state.rs logic)
        if let Ok(key_hex) = std::env::var("LOCKSO_ENCRYPTION_KEY") {
            if let Ok(key) = hex::decode(&key_hex) {
                crate::services::webhook_service::fire_event(
                    pool_clone,
                    key,
                    &action_owned,
                    message,
                );
            }
        }
    });
}

/// List activity logs (admin: global, or filtered by vault_id for vault owners).
pub async fn list_activity(
    pool: &PgPool,
    query: &ActivityLogQuery,
    vault_id_filter: Option<Uuid>,
) -> Result<PaginatedActivityLogs, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(DEFAULT_PER_PAGE).min(MAX_PER_PAGE);
    let offset = (page.saturating_sub(1)).saturating_mul(per_page) as i64;
    let limit = per_page as i64;

    // Build dynamic WHERE clauses
    let mut conditions = Vec::new();
    let mut param_index = 1u32;

    // vault_id filter
    let vault_param_idx = if vault_id_filter.is_some() {
        let idx = param_index;
        conditions.push(format!("al.vault_id = ${idx}"));
        param_index += 1;
        Some(idx)
    } else {
        None
    };

    // user_id filter
    let user_param_idx = if query.user_id.is_some() {
        let idx = param_index;
        conditions.push(format!("al.user_id = ${idx}"));
        param_index += 1;
        Some(idx)
    } else {
        None
    };

    // action filter
    let action_param_idx = if query.action.is_some() {
        let idx = param_index;
        conditions.push(format!("al.action = ${idx}"));
        param_index += 1;
        Some(idx)
    } else {
        None
    };

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let limit_idx = param_index;
    let offset_idx = param_index + 1;

    let count_sql = format!(
        "SELECT COUNT(*) FROM activity_logs al {where_clause}"
    );
    let data_sql = format!(
        r#"SELECT al.id, al.user_id,
                  COALESCE(u.full_name, u.login) AS user_name,
                  al.action, al.resource_type, al.resource_id, al.vault_id,
                  al.client_ip, al.user_agent, al.details, al.created_at
           FROM activity_logs al
           LEFT JOIN users u ON u.id = al.user_id
           {where_clause}
           ORDER BY al.created_at DESC
           LIMIT ${limit_idx} OFFSET ${offset_idx}"#,
    );

    // Execute count query
    let total = {
        let mut q = sqlx::query_scalar::<_, i64>(&count_sql);
        if vault_param_idx.is_some() {
            if let Some(vid) = vault_id_filter {
                q = q.bind(vid);
            }
        }
        if user_param_idx.is_some() {
            if let Some(uid) = query.user_id {
                q = q.bind(uid);
            }
        }
        if action_param_idx.is_some() {
            if let Some(ref act) = query.action {
                q = q.bind(act);
            }
        }
        q.fetch_one(pool).await?
    };

    // Execute data query
    let rows = {
        let mut q = sqlx::query_as::<_, ActivityLogRow>(&data_sql);
        if vault_param_idx.is_some() {
            if let Some(vid) = vault_id_filter {
                q = q.bind(vid);
            }
        }
        if user_param_idx.is_some() {
            if let Some(uid) = query.user_id {
                q = q.bind(uid);
            }
        }
        if action_param_idx.is_some() {
            if let Some(ref act) = query.action {
                q = q.bind(act);
            }
        }
        q = q.bind(limit).bind(offset);
        q.fetch_all(pool).await?
    };

    let data = rows
        .into_iter()
        .map(|r| ActivityLogView {
            id: r.id,
            user_id: r.user_id,
            user_name: r.user_name,
            action: r.action,
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            vault_id: r.vault_id,
            client_ip: r.client_ip,
            details: r.details,
            created_at: r.created_at,
        })
        .collect();

    Ok(PaginatedActivityLogs {
        data,
        total,
        page,
        per_page,
    })
}
