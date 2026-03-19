use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::encryption::decrypt_field;
use crate::error::AppError;
use sqlx::FromRow;

/// Password age threshold in days.
const PASSWORD_AGE_THRESHOLD_DAYS: i64 = 90;

/// Strength score threshold (0-4). Items with score <= this are "weak".
const WEAK_THRESHOLD: u8 = 2;

/// Health report for all accessible vaults.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    /// Total number of items analyzed.
    pub total_items: usize,
    /// Summary counts.
    pub weak_count: usize,
    pub reused_count: usize,
    pub old_count: usize,
    pub breached_count: usize,
    /// Overall score 0-100.
    pub score: u8,
    /// Per-item health entries (sorted: worst first).
    pub items: Vec<HealthItem>,
    /// Reuse groups: list of groups where the same password is used.
    pub reuse_groups: Vec<ReuseGroup>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthItem {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub name: String,
    pub login: String,
    pub url: String,
    pub vault_name: String,
    pub color_code: i16,
    pub strength: u8,
    pub is_weak: bool,
    pub is_reused: bool,
    pub is_old: bool,
    pub is_breached: bool,
    pub breach_count: u64,
    pub password_age_days: i64,
    pub password_changed_at: DateTime<Utc>,
    /// SHA-256 prefix (first 10 chars) for client-side dedup.
    /// NOT enough to reverse the password.
    pub password_hash_prefix: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReuseGroup {
    pub item_ids: Vec<Uuid>,
    pub count: usize,
}

/// Generate a health report across all accessible vaults.
pub async fn generate_report(
    pool: &PgPool,
    key: &[u8],
    user_id: Uuid,
) -> Result<HealthReport, AppError> {
    let now = Utc::now();

    // Internal struct for the joined query
    #[derive(FromRow)]
    struct ItemRow {
        id: Uuid,
        vault_id: Uuid,
        name_enc: String,
        login_enc: String,
        password_enc: String,
        url_enc: String,
        color_code: i16,
        password_changed_at: DateTime<Utc>,
        vault_name: String,
    }

    // Fetch all non-trashed items across accessible vaults
    let rows: Vec<ItemRow> = sqlx::query_as(
        r#"SELECT i.id, i.vault_id, i.name_enc, i.login_enc, i.password_enc, i.url_enc,
                  i.color_code, i.password_changed_at, v.name AS vault_name
           FROM items i
           JOIN vaults v ON v.id = i.vault_id
           WHERE i.deleted_at IS NULL
           AND (
               v.creator_id = $1
               OR EXISTS (
                   SELECT 1 FROM vault_user_accesses vua
                   JOIN resource_accesses ra ON ra.id = vua.resource_access_id
                   WHERE vua.vault_id = v.id AND vua.user_id = $1 AND ra.code != 'forbidden'
               )
           )
           ORDER BY i.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to fetch items for health report");
        AppError::Internal("failed to fetch items".into())
    })?;

    let total_items = rows.len();
    if total_items == 0 {
        return Ok(HealthReport {
            total_items: 0,
            weak_count: 0,
            reused_count: 0,
            old_count: 0,
            breached_count: 0,
            score: 100,
            items: vec![],
            reuse_groups: vec![],
        });
    }

    // Decrypt all passwords and compute SHA-1 hashes for HIBP
    struct DecryptedItem {
        index: usize,
        name: String,
        login: String,
        url: String,
        password: String,
        sha1_hex: String,
    }

    let mut decrypted = Vec::with_capacity(total_items);
    // Map: SHA-1 prefix (5 chars) -> list of (index, full SHA-1 suffix)
    let mut hibp_prefixes: HashMap<String, Vec<(usize, String)>> = HashMap::new();

    for (i, row) in rows.iter().enumerate() {
        let password = decrypt_field(key, &row.password_enc)?;
        let name = decrypt_field(key, &row.name_enc)?;
        let login = decrypt_field(key, &row.login_enc)?;
        let url = decrypt_field(key, &row.url_enc)?;

        let sha1_hex = if !password.is_empty() {
            let hash = sha1_hex_upper(&password);
            let prefix = hash[..5].to_string();
            let suffix = hash[5..].to_string();
            hibp_prefixes.entry(prefix).or_default().push((i, suffix));
            hash
        } else {
            String::new()
        };

        decrypted.push(DecryptedItem {
            index: i,
            name,
            login,
            url,
            password,
            sha1_hex,
        });
    }

    // Batch HIBP k-anonymity lookups (one HTTP request per unique SHA-1 prefix)
    let breach_counts = check_hibp_batch(&hibp_prefixes).await;

    let mut health_items = Vec::with_capacity(total_items);
    let mut password_hashes: HashMap<String, Vec<Uuid>> = HashMap::new();

    for dec in &decrypted {
        let row = &rows[dec.index];
        let strength = password_strength(&dec.password);
        let age_days = (now - row.password_changed_at).num_days();
        let is_old = age_days >= PASSWORD_AGE_THRESHOLD_DAYS;
        let is_weak = !dec.password.is_empty() && strength <= WEAK_THRESHOLD;

        // SHA-256 for reuse detection
        let pw_hash = sha256_hex(&dec.password);
        let pw_hash_prefix = pw_hash[..10].to_string();

        if !dec.password.is_empty() {
            password_hashes.entry(pw_hash).or_default().push(row.id);
        }

        // Breach info
        let breach_count = breach_counts.get(&dec.sha1_hex).copied().unwrap_or(0);
        let is_breached = breach_count > 0;

        health_items.push(HealthItem {
            id: row.id,
            vault_id: row.vault_id,
            name: dec.name.clone(),
            login: dec.login.clone(),
            url: dec.url.clone(),
            vault_name: row.vault_name.clone(),
            color_code: row.color_code,
            strength,
            is_weak,
            is_reused: false, // Will be set below
            is_old,
            is_breached,
            breach_count,
            password_age_days: age_days,
            password_changed_at: row.password_changed_at,
            password_hash_prefix: pw_hash_prefix,
        });
    }

    // Mark reused passwords
    let mut reuse_groups = Vec::new();
    let mut reused_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

    for (_, ids) in &password_hashes {
        if ids.len() > 1 {
            for id in ids {
                reused_ids.insert(*id);
            }
            reuse_groups.push(ReuseGroup {
                item_ids: ids.clone(),
                count: ids.len(),
            });
        }
    }

    for item in &mut health_items {
        if reused_ids.contains(&item.id) {
            item.is_reused = true;
        }
    }

    let weak_count = health_items.iter().filter(|i| i.is_weak).count();
    let reused_count = reused_ids.len();
    let old_count = health_items.iter().filter(|i| i.is_old).count();
    let breached_count = health_items.iter().filter(|i| i.is_breached).count();

    // Calculate overall score (0-100).
    // Score = percentage of items with NO issues (simple and intuitive).
    let score = if total_items == 0 {
        100
    } else {
        let items_with_issues = health_items
            .iter()
            .filter(|i| i.is_breached || i.is_weak || i.is_reused || i.is_old)
            .count();
        (((total_items - items_with_issues) as f64 / total_items as f64) * 100.0) as u8
    };

    // Sort: items with issues first, then by severity (breached > weak > reused > old)
    health_items.sort_by(|a, b| {
        let a_issues =
            (a.is_breached as u8) * 4 + (a.is_weak as u8) * 2 + (a.is_reused as u8) + (a.is_old as u8);
        let b_issues =
            (b.is_breached as u8) * 4 + (b.is_weak as u8) * 2 + (b.is_reused as u8) + (b.is_old as u8);
        b_issues.cmp(&a_issues).then(a.strength.cmp(&b.strength))
    });

    // Sort reuse groups by count (largest first)
    reuse_groups.sort_by(|a, b| b.count.cmp(&a.count));

    Ok(HealthReport {
        total_items,
        weak_count,
        reused_count,
        old_count,
        breached_count,
        score,
        items: health_items,
        reuse_groups,
    })
}

/// Batch check HIBP k-anonymity API for all unique SHA-1 prefixes.
/// Returns a map of full SHA-1 hex -> breach count.
async fn check_hibp_batch(
    prefixes: &HashMap<String, Vec<(usize, String)>>,
) -> HashMap<String, u64> {
    let mut result: HashMap<String, u64> = HashMap::new();

    if prefixes.is_empty() {
        return result;
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Lockso-PasswordManager")
        .build()
        .unwrap_or_default();

    // Process prefixes concurrently (max 10 at a time to be respectful)
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(10));
    let mut handles = Vec::new();

    for (prefix, items) in prefixes {
        let client = client.clone();
        let prefix = prefix.clone();
        let items = items.clone();
        let sem = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.ok()?;
            let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);
            let resp = client
                .get(&url)
                .header("Add-Padding", "true")
                .send()
                .await
                .ok()?;

            if !resp.status().is_success() {
                return None;
            }

            let text = resp.text().await.ok()?;

            // Parse HIBP response: each line is "SUFFIX:COUNT"
            let mut matches: Vec<(String, u64)> = Vec::new();
            for (_, suffix) in &items {
                for line in text.lines() {
                    if let Some((hash_suffix, count_str)) = line.split_once(':') {
                        if hash_suffix.trim() == suffix.as_str() {
                            if let Ok(count) = count_str.trim().parse::<u64>() {
                                if count > 0 {
                                    let full_hash = format!("{}{}", prefix, suffix);
                                    matches.push((full_hash, count));
                                }
                            }
                            break;
                        }
                    }
                }
            }

            Some(matches)
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Ok(Some(matches)) = handle.await {
            for (hash, count) in matches {
                result.insert(hash, count);
            }
        }
    }

    result
}

/// Calculate password strength (0-4), matching frontend logic.
fn password_strength(password: &str) -> u8 {
    if password.is_empty() {
        return 0;
    }
    let mut score: u8 = 0;
    if password.len() >= 8 {
        score += 1;
    }
    if password.len() >= 16 {
        score += 1;
    }
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    if has_upper && has_lower {
        score += 1;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }
    if password.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }
    score.min(4)
}

/// SHA-1 hex digest (uppercase, for HIBP k-anonymity).
fn sha1_hex_upper(input: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode_upper(result)
}

/// SHA-256 hex digest.
fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
