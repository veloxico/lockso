//! Email sending service with support for 7 providers:
//! SMTP, SendGrid, Amazon SES, Resend, Mailgun, Postmark, Mandrill.
//!
//! Provider configuration is stored encrypted in the database.
//! The service decrypts config at send-time — no secrets in memory long-term.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::encryption::{decrypt_field, encrypt_field};
use crate::error::AppError;
use crate::models::email::*;

/// Shared HTTP client for email API providers (avoids per-request TLS handshake).
static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client")
    });

// ─── Settings CRUD ──────────────────────────────────────────────────────────

/// Get current email settings (with masked secrets).
pub async fn get_settings(pool: &PgPool, key: &[u8]) -> Result<Option<EmailSettingsView>, AppError> {
    let row = sqlx::query_as::<_, EmailSettings>("SELECT * FROM email_settings LIMIT 1")
        .fetch_optional(pool)
        .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let config_json = decrypt_field(key, &row.config_enc)?;
    let config: serde_json::Value =
        serde_json::from_str(&config_json).unwrap_or(serde_json::Value::Null);

    Ok(Some(EmailSettingsView {
        provider: row.provider,
        is_enabled: row.is_enabled,
        from_name: row.from_name,
        from_email: row.from_email,
        config: mask_secrets(&config),
        updated_at: row.updated_at,
    }))
}

/// Upsert email settings.
pub async fn update_settings(
    pool: &PgPool,
    key: &[u8],
    input: UpdateEmailSettings,
) -> Result<EmailSettingsView, AppError> {
    // Validate provider
    if EmailProvider::from_str(&input.provider).is_none() {
        return Err(AppError::Validation(format!(
            "Unsupported email provider: {}",
            input.provider
        )));
    }

    let config_json = serde_json::to_string(&input.config).unwrap_or_default();
    let config_enc = encrypt_field(key, &config_json)?;
    let now = Utc::now();

    // Check if settings exist
    let existing: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM email_settings LIMIT 1")
            .fetch_optional(pool)
            .await?;

    if let Some((id,)) = existing {
        sqlx::query(
            r#"UPDATE email_settings SET
                provider = $1, is_enabled = $2, from_name = $3, from_email = $4,
                config_enc = $5, updated_at = $6
            WHERE id = $7"#,
        )
        .bind(&input.provider)
        .bind(input.is_enabled)
        .bind(&input.from_name)
        .bind(&input.from_email)
        .bind(&config_enc)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            r#"INSERT INTO email_settings (
                provider, is_enabled, from_name, from_email, config_enc, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $6)"#,
        )
        .bind(&input.provider)
        .bind(input.is_enabled)
        .bind(&input.from_name)
        .bind(&input.from_email)
        .bind(&config_enc)
        .bind(now)
        .execute(pool)
        .await?;
    }

    Ok(EmailSettingsView {
        provider: input.provider,
        is_enabled: input.is_enabled,
        from_name: input.from_name,
        from_email: input.from_email,
        config: mask_secrets(&input.config),
        updated_at: now,
    })
}

// ─── Send Email ─────────────────────────────────────────────────────────────

/// Send an email using the configured provider.
pub async fn send_email(
    pool: &PgPool,
    key: &[u8],
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), AppError> {
    let row = sqlx::query_as::<_, EmailSettings>("SELECT * FROM email_settings LIMIT 1")
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::Validation("Email is not configured".into()))?;

    if !row.is_enabled {
        return Err(AppError::Validation("Email sending is disabled".into()));
    }

    let config_json = decrypt_field(key, &row.config_enc)?;
    let provider = EmailProvider::from_str(&row.provider)
        .ok_or(AppError::Validation("Unknown email provider".into()))?;

    let from = format!("{} <{}>", row.from_name, row.from_email);

    match provider {
        EmailProvider::Smtp => send_smtp(&config_json, &from, to, subject, html_body).await,
        EmailProvider::Sendgrid => {
            send_api_provider("https://api.sendgrid.com/v3/mail/send", &config_json, &row.from_email, &row.from_name, to, subject, html_body, "sendgrid").await
        }
        EmailProvider::Ses => send_ses(&config_json, &row.from_email, &row.from_name, to, subject, html_body).await,
        EmailProvider::Resend => {
            send_api_provider("https://api.resend.com/emails", &config_json, &row.from_email, &row.from_name, to, subject, html_body, "resend").await
        }
        EmailProvider::Mailgun => send_mailgun(&config_json, &row.from_email, &row.from_name, to, subject, html_body).await,
        EmailProvider::Postmark => send_postmark(&config_json, &row.from_email, &row.from_name, to, subject, html_body).await,
        EmailProvider::Mandrill => send_mandrill(&config_json, &row.from_email, &row.from_name, to, subject, html_body).await,
    }
}

/// Send a test email.
pub async fn send_test_email(
    pool: &PgPool,
    key: &[u8],
    to: &str,
) -> Result<(), AppError> {
    // Basic email format validation to prevent misuse as a spam relay
    if !to.contains('@') || to.len() > 254 || to.contains(' ') {
        return Err(AppError::Validation("Invalid email address".into()));
    }

    send_email(
        pool,
        key,
        to,
        "Lockso — Test Email",
        "<h2>Lockso Email Configuration</h2><p>If you received this message, your email settings are configured correctly.</p>",
    )
    .await
}

// ─── Provider implementations ───────────────────────────────────────────────

/// Send via SMTP using lettre.
async fn send_smtp(
    config_json: &str,
    from: &str,
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), AppError> {
    use lettre::{
        AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
        message::{Message, header::ContentType},
        transport::smtp::authentication::Credentials,
    };

    let config: SmtpConfig = serde_json::from_str(config_json)
        .map_err(|e| AppError::Validation(format!("Invalid SMTP config: {e}")))?;

    let email = Message::builder()
        .from(from.parse().map_err(|_| AppError::Validation("Invalid from address".into()))?)
        .to(to.parse().map_err(|_| AppError::Validation("Invalid to address".into()))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .map_err(|e| AppError::Validation(format!("Failed to build email: {e}")))?;

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let transport = if config.use_tls {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
    }
    .map_err(|e| AppError::Validation(format!("SMTP connection error: {e}")))?
    .port(config.port)
    .credentials(creds)
    .build();

    transport
        .send(email)
        .await
        .map_err(|e| AppError::Validation(format!("SMTP send failed: {e}")))?;

    Ok(())
}

/// Send via SendGrid or Resend (JSON-body HTTP APIs with similar structure).
async fn send_api_provider(
    url: &str,
    config_json: &str,
    from_email: &str,
    from_name: &str,
    to: &str,
    subject: &str,
    html_body: &str,
    provider: &str,
) -> Result<(), AppError> {
    let config: ApiKeyConfig = serde_json::from_str(config_json)
        .map_err(|e| AppError::Validation(format!("Invalid {provider} config: {e}")))?;

    let client = HTTP_CLIENT.clone();

    let body = if provider == "sendgrid" {
        serde_json::json!({
            "personalizations": [{"to": [{"email": to}]}],
            "from": {"email": from_email, "name": from_name},
            "subject": subject,
            "content": [{"type": "text/html", "value": html_body}]
        })
    } else {
        // Resend format
        serde_json::json!({
            "from": format!("{from_name} <{from_email}>"),
            "to": [to],
            "subject": subject,
            "html": html_body
        })
    };

    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(provider, error = %e, "Email provider request failed");
            AppError::Validation(format!("{provider}: failed to send email"))
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::error!(provider, %status, %text, "Email provider returned error");
        return Err(AppError::Validation(format!(
            "{provider}: failed to send email (status {status})"
        )));
    }

    Ok(())
}

/// Send via Mailgun (multipart form POST).
async fn send_mailgun(
    config_json: &str,
    from_email: &str,
    from_name: &str,
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), AppError> {
    let config: ApiKeyConfig = serde_json::from_str(config_json)
        .map_err(|e| AppError::Validation(format!("Invalid Mailgun config: {e}")))?;

    let base_url = if config.eu_region {
        "https://api.eu.mailgun.net"
    } else {
        "https://api.mailgun.net"
    };

    let url = format!("{base_url}/v3/{}/messages", config.domain);
    let from = format!("{from_name} <{from_email}>");

    let client = HTTP_CLIENT.clone();
    let resp = client
        .post(&url)
        .basic_auth("api", Some(&config.api_key))
        .form(&[
            ("from", from.as_str()),
            ("to", to),
            ("subject", subject),
            ("html", html_body),
        ])
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Mailgun request failed");
            AppError::Validation("Mailgun: failed to send email".into())
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::error!(%status, %text, "Mailgun returned error");
        return Err(AppError::Validation(format!(
            "Mailgun: failed to send email (status {status})"
        )));
    }

    Ok(())
}

/// Send via Postmark (JSON API).
async fn send_postmark(
    config_json: &str,
    from_email: &str,
    _from_name: &str,
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), AppError> {
    let config: ApiKeyConfig = serde_json::from_str(config_json)
        .map_err(|e| AppError::Validation(format!("Invalid Postmark config: {e}")))?;

    let client = HTTP_CLIENT.clone();
    let resp = client
        .post("https://api.postmarkapp.com/email")
        .header("X-Postmark-Server-Token", &config.api_key)
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "From": from_email,
            "To": to,
            "Subject": subject,
            "HtmlBody": html_body,
            "MessageStream": "outbound"
        }))
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Postmark request failed");
            AppError::Validation("Postmark: failed to send email".into())
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::error!(%status, %text, "Postmark returned error");
        return Err(AppError::Validation(format!(
            "Postmark: failed to send email (status {status})"
        )));
    }

    Ok(())
}

/// Send via Mandrill (Mailchimp Transactional).
async fn send_mandrill(
    config_json: &str,
    from_email: &str,
    from_name: &str,
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), AppError> {
    let config: ApiKeyConfig = serde_json::from_str(config_json)
        .map_err(|e| AppError::Validation(format!("Invalid Mandrill config: {e}")))?;

    let client = HTTP_CLIENT.clone();
    let resp = client
        .post("https://mandrillapp.com/api/1.0/messages/send.json")
        .json(&serde_json::json!({
            "key": config.api_key,
            "message": {
                "from_email": from_email,
                "from_name": from_name,
                "to": [{"email": to, "type": "to"}],
                "subject": subject,
                "html": html_body
            }
        }))
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Mandrill request failed");
            AppError::Validation("Mandrill: failed to send email".into())
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        // Do not log the response body — Mandrill may echo the API key back
        let _text = resp.text().await.unwrap_or_default();
        tracing::error!(%status, "Mandrill returned error");
        return Err(AppError::Validation(format!(
            "Mandrill: failed to send email (status {status})"
        )));
    }

    Ok(())
}

/// Send via Amazon SES (using reqwest with SigV4 — simplified).
async fn send_ses(
    config_json: &str,
    from_email: &str,
    from_name: &str,
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), AppError> {
    // For SES, we use SMTP transport through lettre with SES SMTP credentials.
    // SES SMTP is the simplest integration — users provide their SES SMTP username/password.
    let config: SesConfig = serde_json::from_str(config_json)
        .map_err(|e| AppError::Validation(format!("Invalid SES config: {e}")))?;

    let from = format!("{from_name} <{from_email}>");
    let smtp_host = format!("email-smtp.{}.amazonaws.com", config.region);

    // SES access_key_id is the SMTP username, secret_access_key is used to derive SMTP password
    send_smtp(
        &serde_json::to_string(&SmtpConfig {
            host: smtp_host,
            port: 587,
            username: config.access_key_id,
            password: config.secret_access_key,
            use_tls: false, // STARTTLS
        })
        .unwrap_or_default(),
        &from,
        to,
        subject,
        html_body,
    )
    .await
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Mask sensitive fields in config for API responses.
fn mask_secrets(config: &serde_json::Value) -> serde_json::Value {
    const SENSITIVE: &[&str] = &[
        "password",
        "apiKey",
        "api_key",
        "secretAccessKey",
        "secret_access_key",
    ];

    match config {
        serde_json::Value::Object(map) => {
            let mut masked = serde_json::Map::new();
            for (k, v) in map {
                if SENSITIVE.iter().any(|s| k.eq_ignore_ascii_case(s)) {
                    // Fully redact sensitive values — never reveal any suffix
                    if let Some(s) = v.as_str() {
                        let indicator = if s.is_empty() { "(empty)" } else { "••••••••" };
                        masked.insert(k.clone(), serde_json::Value::String(indicator.into()));
                    } else {
                        masked.insert(k.clone(), serde_json::Value::String("••••••••".into()));
                    }
                } else {
                    masked.insert(k.clone(), v.clone());
                }
            }
            serde_json::Value::Object(masked)
        }
        other => other.clone(),
    }
}
