use sqlx::PgPool;
use uuid::Uuid;

use crate::encryption::decrypt_field;
use crate::error::AppError;
use crate::models::import_export::{
    ExportFormat, ExportResult, ImportFormat, ImportResult, PortableCustomField, PortableItem,
};
use crate::models::item::{CreateItem, CustomField, Item};
use crate::services::{item_service, sharing_service};

/// Maximum items per import to prevent abuse.
const MAX_IMPORT_ITEMS: usize = 5000;

/// Import items into a vault from a given format.
pub async fn import_items(
    pool: &PgPool,
    key: &[u8],
    vault_id: Uuid,
    user_id: Uuid,
    format: ImportFormat,
    data: &str,
    create_folders: bool,
) -> Result<ImportResult, AppError> {
    // Check write access
    sharing_service::require_write_access(pool, vault_id, user_id).await?;

    // Parse items from the input format
    let items = match format {
        ImportFormat::Csv => parse_csv(data)?,
        ImportFormat::Json => parse_lockso_json(data)?,
        ImportFormat::Passwork => parse_passwork_json(data)?,
        ImportFormat::Keepass => parse_keepass_xml(data)?,
        ImportFormat::Bitwarden => parse_bitwarden_json(data)?,
    };

    if items.len() > MAX_IMPORT_ITEMS {
        return Err(AppError::Validation(format!(
            "Import limited to {MAX_IMPORT_ITEMS} items per batch"
        )));
    }

    let mut imported = 0u32;
    let mut skipped = 0u32;
    let mut errors = Vec::new();

    for (i, portable) in items.iter().enumerate() {
        if portable.name.trim().is_empty() {
            skipped += 1;
            errors.push(format!("Row {}: empty name, skipped", i + 1));
            continue;
        }

        // Resolve folder path to folder_id
        let folder_id = if create_folders && !portable.folder_path.is_empty() {
            match resolve_folder_path(pool, vault_id, &portable.folder_path).await {
                Ok(id) => Some(id),
                Err(e) => {
                    errors.push(format!(
                        "Row {}: failed to create folder path {:?}: {}",
                        i + 1,
                        portable.folder_path,
                        e
                    ));
                    None
                }
            }
        } else {
            None
        };

        // Convert custom fields
        let customs: Vec<CustomField> = portable
            .custom_fields
            .iter()
            .map(|f| CustomField {
                name: f.name.clone(),
                value: f.value.clone(),
                field_type: if f.r#type.is_empty() {
                    "text".to_string()
                } else {
                    f.r#type.clone()
                },
            })
            .collect();

        let input = CreateItem {
            vault_id,
            folder_id,
            name: portable.name.clone(),
            login: Some(portable.login.clone()),
            password: Some(portable.password.clone()),
            url: Some(portable.url.clone()),
            description: Some(portable.description.clone()),
            tags: if portable.tags.is_empty() {
                None
            } else {
                Some(portable.tags.clone())
            },
            customs: if customs.is_empty() {
                None
            } else {
                Some(customs)
            },
            color_code: None,
        };

        match item_service::create_item(pool, key, user_id, input).await {
            Ok(_) => imported += 1,
            Err(e) => {
                skipped += 1;
                errors.push(format!("Row {}: {}", i + 1, e));
            }
        }
    }

    tracing::info!(
        vault_id = %vault_id,
        user_id = %user_id,
        format = ?format,
        imported = imported,
        skipped = skipped,
        "Import completed"
    );

    Ok(ImportResult {
        imported_count: imported,
        skipped_count: skipped,
        errors,
    })
}

/// Export all items from a vault in the given format.
pub async fn export_items(
    pool: &PgPool,
    key: &[u8],
    vault_id: Uuid,
    user_id: Uuid,
    format: ExportFormat,
) -> Result<ExportResult, AppError> {
    // Check read access
    sharing_service::check_vault_access(pool, vault_id, user_id).await?;

    // Fetch all items in the vault
    let items = sqlx::query_as::<_, Item>(
        "SELECT * FROM items WHERE vault_id = $1 ORDER BY created_at ASC",
    )
    .bind(vault_id)
    .fetch_all(pool)
    .await?;

    // Build folder path map: folder_id -> path segments
    let folder_paths = build_folder_path_map(pool, vault_id).await?;

    // Decrypt and convert to portable items
    let mut portables = Vec::with_capacity(items.len());
    for item in &items {
        let name = decrypt_field(key, &item.name_enc).unwrap_or_default();
        let login = decrypt_field(key, &item.login_enc).unwrap_or_default();
        let password = decrypt_field(key, &item.password_enc).unwrap_or_default();
        let url = decrypt_field(key, &item.url_enc).unwrap_or_default();
        let description = decrypt_field(key, &item.description_enc).unwrap_or_default();

        let customs_json = decrypt_field(key, &item.customs_enc).unwrap_or_default();
        let customs: Vec<CustomField> =
            serde_json::from_str(&customs_json).unwrap_or_default();

        let tags: Vec<String> =
            serde_json::from_value(item.tags.clone()).unwrap_or_default();

        let folder_path = item
            .folder_id
            .and_then(|fid| folder_paths.get(&fid).cloned())
            .unwrap_or_default();

        portables.push(PortableItem {
            name,
            login,
            password,
            url,
            description,
            tags,
            folder_path,
            custom_fields: customs
                .into_iter()
                .map(|f| PortableCustomField {
                    name: f.name,
                    value: f.value,
                    r#type: f.field_type,
                })
                .collect(),
        });
    }

    let item_count = portables.len() as u32;

    let (format_str, data) = match format {
        ExportFormat::Csv => ("csv".to_string(), export_csv(&portables)?),
        ExportFormat::Json => (
            "json".to_string(),
            serde_json::to_string_pretty(&portables)
                .map_err(|e| AppError::Internal(format!("JSON serialization failed: {e}")))?,
        ),
    };

    tracing::info!(
        vault_id = %vault_id,
        user_id = %user_id,
        format = format_str,
        count = item_count,
        "Export completed"
    );

    Ok(ExportResult {
        format: format_str,
        data,
        item_count,
    })
}

// ─── Parsers ───

/// Parse Lockso/generic CSV: Name,Login,Password,URL,Description,Tags,FolderPath
fn parse_csv(data: &str) -> Result<Vec<PortableItem>, AppError> {
    let mut items = Vec::new();
    let mut lines = data.lines();

    // Skip header if present
    let first_line = match lines.next() {
        Some(line) => line,
        None => return Ok(items),
    };

    let is_header = first_line.to_lowercase().contains("name")
        && (first_line.to_lowercase().contains("login")
            || first_line.to_lowercase().contains("password")
            || first_line.to_lowercase().contains("url"));

    let iter: Box<dyn Iterator<Item = &str>> = if is_header {
        Box::new(lines)
    } else {
        Box::new(std::iter::once(first_line).chain(lines))
    };

    for line in iter {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields = parse_csv_line(line);
        let get = |i: usize| fields.get(i).cloned().unwrap_or_default();

        let tags_str = get(5);
        let tags: Vec<String> = if tags_str.is_empty() {
            vec![]
        } else {
            tags_str.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
        };

        let folder_str = get(6);
        let folder_path: Vec<String> = if folder_str.is_empty() {
            vec![]
        } else {
            folder_str.split('/').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        };

        items.push(PortableItem {
            name: get(0),
            login: get(1),
            password: get(2),
            url: get(3),
            description: get(4),
            tags,
            folder_path,
            custom_fields: vec![],
        });
    }

    Ok(items)
}

/// Minimal CSV line parser handling quoted fields.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == ',' {
            fields.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(ch);
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Parse Lockso JSON format (array of PortableItem).
fn parse_lockso_json(data: &str) -> Result<Vec<PortableItem>, AppError> {
    serde_json::from_str(data)
        .map_err(|e| AppError::Validation(format!("Invalid JSON: {e}")))
}

/// Parse Passwork JSON export format.
/// Passwork exports: { "folders": [...], "passwords": [...] }
fn parse_passwork_json(data: &str) -> Result<Vec<PortableItem>, AppError> {
    let root: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| AppError::Validation(format!("Invalid Passwork JSON: {e}")))?;

    // Build folder ID -> path map from Passwork's folder structure
    let folder_map = build_passwork_folder_map(&root);

    let passwords = root
        .get("passwords")
        .or_else(|| root.get("items"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::Validation("No 'passwords' or 'items' array found".into()))?;

    let mut items = Vec::new();
    for pw in passwords {
        let name = pw.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let login = pw.get("login").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let password = pw
            .get("cryptedPassword")
            .or_else(|| pw.get("password"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = pw.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = pw.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let tags: Vec<String> = pw
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Resolve folder path from Passwork folder ID
        let folder_id = pw.get("folderId").or_else(|| pw.get("groupId")).and_then(|v| v.as_str());
        let folder_path = folder_id
            .and_then(|id| folder_map.get(id).cloned())
            .unwrap_or_default();

        // Custom fields
        let custom_fields: Vec<PortableCustomField> = pw
            .get("custom")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| {
                        Some(PortableCustomField {
                            name: f.get("name")?.as_str()?.to_string(),
                            value: f.get("value")?.as_str()?.to_string(),
                            r#type: f
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("text")
                                .to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        items.push(PortableItem {
            name,
            login,
            password,
            url,
            description,
            tags,
            folder_path,
            custom_fields,
        });
    }

    Ok(items)
}

/// Build Passwork folder ID -> path map.
fn build_passwork_folder_map(root: &serde_json::Value) -> std::collections::HashMap<String, Vec<String>> {
    let mut map = std::collections::HashMap::new();

    let folders = match root.get("folders").and_then(|v| v.as_array()) {
        Some(f) => f,
        None => return map,
    };

    // Build parent map
    let mut parent_map: std::collections::HashMap<String, (String, Option<String>)> =
        std::collections::HashMap::new();
    for folder in folders {
        let id = folder.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let name = folder.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let parent = folder
            .get("parentId")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        parent_map.insert(id, (name, parent));
    }

    // Resolve paths
    for id in parent_map.keys().cloned().collect::<Vec<_>>() {
        let mut path = Vec::new();
        let mut current = Some(id.clone());
        let mut depth = 0;
        while let Some(ref cid) = current {
            if depth > 20 {
                break; // Prevent infinite loops
            }
            if let Some((name, parent)) = parent_map.get(cid) {
                path.push(name.clone());
                current = parent.clone();
            } else {
                break;
            }
            depth += 1;
        }
        path.reverse();
        map.insert(id, path);
    }

    map
}

/// Parse KeePass XML (KDBX XML export).
fn parse_keepass_xml(data: &str) -> Result<Vec<PortableItem>, AppError> {
    // KeePass XML structure: <KeePassFile><Root><Group>...<Entry>...</Entry></Group></Root></KeePassFile>
    // We use a simple manual parser since we don't want to add an XML crate dependency.
    let mut items = Vec::new();
    let mut current_path: Vec<String> = Vec::new();

    // Simple state machine for XML parsing
    let mut pos = 0;
    let bytes = data.as_bytes();

    while pos < bytes.len() {
        if bytes[pos] == b'<' {
            let tag_end = match data[pos..].find('>') {
                Some(i) => pos + i + 1,
                None => break,
            };
            let tag = &data[pos + 1..tag_end - 1];

            if tag.starts_with("Group") && !tag.starts_with("Group/") {
                // Entering a group — look for <Name>
                if let Some(name) = extract_xml_child_value(data, tag_end, "Name") {
                    // Skip root-level groups like "Root" and "Recycle Bin"
                    if name != "Root" && !name.contains("Recycle") {
                        current_path.push(name);
                    }
                }
            } else if tag == "/Group" {
                current_path.pop();
            } else if tag.starts_with("Entry") && !tag.starts_with("Entry/") {
                // Parse entry
                if let Some(item) = parse_keepass_entry(data, tag_end, &current_path) {
                    items.push(item);
                }
            }

            pos = tag_end;
        } else {
            pos += 1;
        }
    }

    Ok(items)
}

fn extract_xml_child_value(data: &str, from: usize, tag_name: &str) -> Option<String> {
    let search_area = &data[from..std::cmp::min(from + 2000, data.len())];
    let open_tag = format!("<{tag_name}>");
    let close_tag = format!("</{tag_name}>");

    let start = search_area.find(&open_tag)? + open_tag.len();
    let end = search_area[start..].find(&close_tag)? + start;
    Some(unescape_xml(&search_area[start..end]))
}

fn parse_keepass_entry(data: &str, from: usize, folder_path: &[String]) -> Option<PortableItem> {
    // Find the end of this entry
    let entry_data = &data[from..std::cmp::min(from + 10000, data.len())];
    let entry_end = entry_data.find("</Entry>")?;
    let entry_str = &entry_data[..entry_end];

    let mut name = String::new();
    let mut login = String::new();
    let mut password = String::new();
    let mut url = String::new();
    let mut notes = String::new();

    // Parse <String> elements: <Key>...</Key><Value>...</Value>
    let mut search_pos = 0;
    while let Some(string_start) = entry_str[search_pos..].find("<String>") {
        let abs_start = search_pos + string_start;
        let string_end = match entry_str[abs_start..].find("</String>") {
            Some(e) => abs_start + e,
            None => break,
        };

        let block = &entry_str[abs_start..string_end];
        if let (Some(key), Some(value)) = (
            extract_inner_value(block, "Key"),
            extract_inner_value(block, "Value"),
        ) {
            match key.as_str() {
                "Title" => name = value,
                "UserName" => login = value,
                "Password" => password = value,
                "URL" => url = value,
                "Notes" => notes = value,
                _ => {}
            }
        }

        search_pos = string_end + 9;
    }

    if name.is_empty() && login.is_empty() && password.is_empty() {
        return None;
    }

    Some(PortableItem {
        name,
        login,
        password,
        url,
        description: notes,
        tags: vec![],
        folder_path: folder_path.to_vec(),
        custom_fields: vec![],
    })
}

fn extract_inner_value(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    // Also handle <Value Protected="True"> etc.
    let alt_open = format!("<{tag} ");

    let start = if let Some(i) = block.find(&open) {
        i + open.len()
    } else if let Some(i) = block.find(&alt_open) {
        let rest = &block[i..];
        rest.find('>')? + i + 1
    } else {
        return None;
    };

    let end = block[start..].find(&close)? + start;
    Some(unescape_xml(&block[start..end]))
}

/// Parse Bitwarden JSON export.
/// Bitwarden: { "items": [{ "type": 1, "name": "...", "login": { "username": "...", "password": "...", "uris": [...] }, "notes": "...", "folderId": "..." }], "folders": [...] }
fn parse_bitwarden_json(data: &str) -> Result<Vec<PortableItem>, AppError> {
    let root: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| AppError::Validation(format!("Invalid Bitwarden JSON: {e}")))?;

    // Build folder map
    let mut folder_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    if let Some(folders) = root.get("folders").and_then(|v| v.as_array()) {
        for f in folders {
            let id = f.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if !id.is_empty() {
                folder_map.insert(id.to_string(), name.to_string());
            }
        }
    }

    let bw_items = root
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::Validation("No 'items' array found".into()))?;

    let mut items = Vec::new();
    for bw in bw_items {
        // type 1 = login, type 2 = secure note, type 3 = card, type 4 = identity
        let item_type = bw.get("type").and_then(|v| v.as_u64()).unwrap_or(1);
        if item_type != 1 {
            continue; // Only import login items
        }

        let name = bw.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let notes = bw.get("notes").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let login_obj = bw.get("login");
        let username = login_obj
            .and_then(|l| l.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let password = login_obj
            .and_then(|l| l.get("password"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // First URI
        let url = login_obj
            .and_then(|l| l.get("uris"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|u| u.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Folder path
        let folder_id = bw.get("folderId").and_then(|v| v.as_str()).unwrap_or("");
        let folder_path = if folder_id.is_empty() {
            vec![]
        } else {
            folder_map
                .get(folder_id)
                .map(|name| name.split('/').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default()
        };

        // Custom fields from Bitwarden
        let custom_fields: Vec<PortableCustomField> = bw
            .get("fields")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| {
                        Some(PortableCustomField {
                            name: f.get("name")?.as_str()?.to_string(),
                            value: f.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            r#type: match f.get("type").and_then(|v| v.as_u64()).unwrap_or(0) {
                                0 => "text".to_string(),
                                1 => "hidden".to_string(),
                                2 => "boolean".to_string(),
                                _ => "text".to_string(),
                            },
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        items.push(PortableItem {
            name,
            login: username,
            password,
            url,
            description: notes,
            tags: vec![],
            folder_path,
            custom_fields,
        });
    }

    Ok(items)
}

// ─── Export helpers ───

fn export_csv(items: &[PortableItem]) -> Result<String, AppError> {
    let mut csv = String::from("Name,Login,Password,URL,Description,Tags,FolderPath\n");
    for item in items {
        csv.push_str(&csv_escape(&item.name));
        csv.push(',');
        csv.push_str(&csv_escape(&item.login));
        csv.push(',');
        csv.push_str(&csv_escape(&item.password));
        csv.push(',');
        csv.push_str(&csv_escape(&item.url));
        csv.push(',');
        csv.push_str(&csv_escape(&item.description));
        csv.push(',');
        csv.push_str(&csv_escape(&item.tags.join(",")));
        csv.push(',');
        csv.push_str(&csv_escape(&item.folder_path.join("/")));
        csv.push('\n');
    }
    Ok(csv)
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

// ─── Folder resolution ───

/// Resolve a folder path like ["Work", "SSH Keys"] to a folder_id,
/// creating folders as needed.
async fn resolve_folder_path(
    pool: &PgPool,
    vault_id: Uuid,
    path: &[String],
) -> Result<Uuid, AppError> {
    let mut parent_id: Option<Uuid> = None;

    for segment in path {
        if segment.trim().is_empty() {
            continue;
        }

        // Try to find existing folder
        let existing: Option<(Uuid,)> = if let Some(pid) = parent_id {
            sqlx::query_as(
                "SELECT id FROM folders WHERE vault_id = $1 AND parent_folder_id = $2 AND name = $3",
            )
            .bind(vault_id)
            .bind(pid)
            .bind(segment.trim())
            .fetch_optional(pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT id FROM folders WHERE vault_id = $1 AND parent_folder_id IS NULL AND name = $2",
            )
            .bind(vault_id)
            .bind(segment.trim())
            .fetch_optional(pool)
            .await?
        };

        if let Some((id,)) = existing {
            parent_id = Some(id);
        } else {
            // Create the folder
            let folder_id = Uuid::now_v7();
            let ancestor_ids: Vec<String> = if let Some(pid) = parent_id {
                // Get parent's ancestor_ids and append parent
                let parent_ancestors: Option<(serde_json::Value,)> = sqlx::query_as(
                    "SELECT ancestor_ids FROM folders WHERE id = $1",
                )
                .bind(pid)
                .fetch_optional(pool)
                .await?;

                let mut ancestors: Vec<String> = parent_ancestors
                    .and_then(|(v,)| serde_json::from_value(v).ok())
                    .unwrap_or_default();
                ancestors.push(pid.to_string());
                ancestors
            } else {
                vec![]
            };

            let ancestor_json = serde_json::to_value(&ancestor_ids).unwrap_or_default();

            sqlx::query(
                r#"INSERT INTO folders (id, name, vault_id, parent_folder_id, ancestor_ids)
                VALUES ($1, $2, $3, $4, $5)"#,
            )
            .bind(folder_id)
            .bind(segment.trim())
            .bind(vault_id)
            .bind(parent_id)
            .bind(&ancestor_json)
            .execute(pool)
            .await?;

            parent_id = Some(folder_id);
        }
    }

    parent_id.ok_or_else(|| AppError::Internal("Empty folder path".into()))
}

/// Build a map of folder_id -> path segments for export.
async fn build_folder_path_map(
    pool: &PgPool,
    vault_id: Uuid,
) -> Result<std::collections::HashMap<Uuid, Vec<String>>, AppError> {
    let folders: Vec<(Uuid, String, Option<Uuid>)> = sqlx::query_as(
        "SELECT id, name, parent_folder_id FROM folders WHERE vault_id = $1",
    )
    .bind(vault_id)
    .fetch_all(pool)
    .await?;

    let mut name_map: std::collections::HashMap<Uuid, (String, Option<Uuid>)> =
        std::collections::HashMap::new();
    for (id, name, parent) in &folders {
        name_map.insert(*id, (name.clone(), *parent));
    }

    let mut result: std::collections::HashMap<Uuid, Vec<String>> =
        std::collections::HashMap::new();
    for (id, _, _) in &folders {
        let mut path = Vec::new();
        let mut current = Some(*id);
        let mut depth = 0;
        while let Some(cid) = current {
            if depth > 20 {
                break;
            }
            if let Some((name, parent)) = name_map.get(&cid) {
                path.push(name.clone());
                current = *parent;
            } else {
                break;
            }
            depth += 1;
        }
        path.reverse();
        result.insert(*id, path);
    }

    Ok(result)
}
