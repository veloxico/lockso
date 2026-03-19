use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::encryption::{decrypt_field, encrypt_field};
use crate::error::AppError;
use crate::models::webhook::*;

/// Shared HTTP client for webhook delivery.
static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client")
});

// ─── CRUD ───

pub async fn list_webhooks(pool: &PgPool, key: &[u8]) -> Result<Vec<WebhookView>, AppError> {
    let rows = sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;

    let mut views = Vec::with_capacity(rows.len());
    for row in rows {
        let url = decrypt_field(key, &row.url_enc).unwrap_or_default();
        views.push(to_view(row, &url));
    }
    Ok(views)
}

pub async fn create_webhook(
    pool: &PgPool,
    key: &[u8],
    user_id: Uuid,
    input: CreateWebhook,
) -> Result<WebhookView, AppError> {
    // Validate
    if input.name.trim().is_empty() || input.name.len() > 100 {
        return Err(AppError::Validation("Name is required (max 100 chars)".into()));
    }
    if !WEBHOOK_PROVIDERS.contains(&input.provider.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid provider: {}. Supported: {:?}",
            input.provider, WEBHOOK_PROVIDERS
        )));
    }
    if input.url.trim().is_empty() {
        return Err(AppError::Validation("URL is required".into()));
    }
    // Validate events
    for event in &input.events {
        if !WEBHOOK_EVENTS.contains(&event.as_str()) {
            return Err(AppError::Validation(format!("Unknown event: {event}")));
        }
    }

    let url_enc = encrypt_field(key, &input.url)?;
    let events_json = serde_json::to_value(&input.events).unwrap_or_default();
    let id = Uuid::now_v7();

    sqlx::query(
        r#"INSERT INTO webhooks (id, name, provider, url_enc, events, is_enabled, creator_id, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, TRUE, $6, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(input.name.trim())
    .bind(&input.provider)
    .bind(&url_enc)
    .bind(&events_json)
    .bind(user_id)
    .execute(pool)
    .await?;

    let row = sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(to_view(row, &input.url))
}

pub async fn update_webhook(
    pool: &PgPool,
    key: &[u8],
    id: Uuid,
    input: UpdateWebhook,
) -> Result<WebhookView, AppError> {
    let row = sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Webhook not found".into()))?;

    let name = input.name.as_deref().unwrap_or(&row.name);
    let is_enabled = input.is_enabled.unwrap_or(row.is_enabled);
    let events_json = if let Some(events) = &input.events {
        for event in events {
            if !WEBHOOK_EVENTS.contains(&event.as_str()) {
                return Err(AppError::Validation(format!("Unknown event: {event}")));
            }
        }
        serde_json::to_value(events).unwrap_or_default()
    } else {
        row.events.clone()
    };

    let url_enc = if let Some(url) = &input.url {
        encrypt_field(key, url)?
    } else {
        row.url_enc.clone()
    };

    sqlx::query(
        "UPDATE webhooks SET name = $1, url_enc = $2, events = $3, is_enabled = $4, updated_at = NOW() WHERE id = $5",
    )
    .bind(name)
    .bind(&url_enc)
    .bind(&events_json)
    .bind(is_enabled)
    .bind(id)
    .execute(pool)
    .await?;

    let url = if let Some(url) = &input.url {
        url.clone()
    } else {
        decrypt_field(key, &row.url_enc).unwrap_or_default()
    };

    let updated = sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(to_view(updated, &url))
}

pub async fn delete_webhook(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM webhooks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Webhook not found".into()));
    }
    Ok(())
}

pub async fn test_webhook(pool: &PgPool, key: &[u8], id: Uuid) -> Result<(), AppError> {
    let row = sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Webhook not found".into()))?;

    let url = decrypt_field(key, &row.url_enc)?;

    deliver(&row.provider, &url, "🔔 Lockso Test", "This is a test webhook notification from Lockso.")
        .await
}

// ─── Event dispatch ───

/// Fire webhooks for a given event. Called from activity_log_service or route handlers.
/// Runs asynchronously — does NOT block the caller.
pub fn fire_event(pool: PgPool, key: Vec<u8>, event: &str, message: String) {
    let event = event.to_string();
    tokio::spawn(async move {
        if let Err(e) = dispatch_event(&pool, &key, &event, &message).await {
            tracing::warn!(event, error = %e, "Webhook dispatch failed");
        }
    });
}

async fn dispatch_event(
    pool: &PgPool,
    key: &[u8],
    event: &str,
    message: &str,
) -> Result<(), AppError> {
    // Fetch all enabled webhooks subscribed to this event
    let rows = sqlx::query_as::<_, Webhook>(
        "SELECT * FROM webhooks WHERE is_enabled = TRUE",
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let events: Vec<String> =
            serde_json::from_value(row.events.clone()).unwrap_or_default();
        if !events.iter().any(|e| e == event) {
            continue;
        }

        let url = match decrypt_field(key, &row.url_enc) {
            Ok(u) => u,
            Err(_) => continue,
        };

        let title = format!("🔔 {event}");
        if let Err(e) = deliver(&row.provider, &url, &title, message).await {
            tracing::warn!(
                webhook_id = %row.id,
                provider = row.provider,
                event,
                error = %e,
                "Webhook delivery failed"
            );
        }
    }

    Ok(())
}

// ─── Provider delivery ───

async fn deliver(provider: &str, url: &str, title: &str, message: &str) -> Result<(), AppError> {
    match provider {
        "telegram" => deliver_telegram(url, title, message).await,
        "slack" => deliver_slack(url, title, message).await,
        "discord" => deliver_discord(url, title, message).await,
        _ => deliver_custom(url, title, message).await,
    }
}

/// Telegram: url format = "bot_token|chat_id"
async fn deliver_telegram(url: &str, title: &str, message: &str) -> Result<(), AppError> {
    let (token, chat_id) = url
        .split_once('|')
        .ok_or_else(|| AppError::Validation("Telegram URL must be: bot_token|chat_id".into()))?;

    let api_url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let text = format!("*{title}*\n{message}");

    let resp = HTTP_CLIENT
        .post(&api_url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown",
            "disable_web_page_preview": true
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Telegram request failed: {e}")))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!("Telegram error: {text}")));
    }
    Ok(())
}

/// Slack: url = Incoming Webhook URL
async fn deliver_slack(url: &str, title: &str, message: &str) -> Result<(), AppError> {
    let resp = HTTP_CLIENT
        .post(url)
        .json(&serde_json::json!({
            "text": format!("*{title}*\n{message}")
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Slack request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal("Slack webhook failed".into()));
    }
    Ok(())
}

/// Discord: url = Discord Webhook URL
async fn deliver_discord(url: &str, title: &str, message: &str) -> Result<(), AppError> {
    let resp = HTTP_CLIENT
        .post(url)
        .json(&serde_json::json!({
            "embeds": [{
                "title": title,
                "description": message,
                "color": 5814783
            }]
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Discord request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal("Discord webhook failed".into()));
    }
    Ok(())
}

/// Custom: POST JSON payload to any URL
async fn deliver_custom(url: &str, title: &str, message: &str) -> Result<(), AppError> {
    let resp = HTTP_CLIENT
        .post(url)
        .json(&serde_json::json!({
            "event": title,
            "message": message,
            "timestamp": Utc::now().to_rfc3339()
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Webhook request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal("Custom webhook failed".into()));
    }
    Ok(())
}

// ─── Helpers ───

fn to_view(row: Webhook, url: &str) -> WebhookView {
    let events: Vec<String> = serde_json::from_value(row.events).unwrap_or_default();
    WebhookView {
        id: row.id,
        name: row.name,
        provider: row.provider,
        url_masked: mask_url(url),
        events,
        is_enabled: row.is_enabled,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn mask_url(url: &str) -> String {
    if url.contains('|') {
        // Telegram format: token|chat_id
        let parts: Vec<&str> = url.splitn(2, '|').collect();
        let token = parts[0];
        let chat = parts.get(1).unwrap_or(&"");
        let masked_token = if token.len() > 8 {
            format!("{}••••{}", &token[..4], &token[token.len() - 4..])
        } else {
            "••••••••".to_string()
        };
        format!("{masked_token}|{chat}")
    } else if url.len() > 20 {
        format!("{}••••••••", &url[..url.len().min(30)])
    } else {
        "••••••••".to_string()
    }
}
