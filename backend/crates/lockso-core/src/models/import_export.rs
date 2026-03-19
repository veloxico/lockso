use serde::{Deserialize, Serialize};

/// Supported import formats.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportFormat {
    Csv,
    Json,
    Passwork,
    Keepass,
    Bitwarden,
}

/// Supported export formats.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Json,
}

/// A single item in portable (decrypted) form for import/export.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableItem {
    pub name: String,
    #[serde(default)]
    pub login: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Folder path segments, e.g. ["Work", "SSH Keys"]
    #[serde(default)]
    pub folder_path: Vec<String>,
    #[serde(default)]
    pub custom_fields: Vec<PortableCustomField>,
}

/// Custom field in portable form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableCustomField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub r#type: String,
}

/// Import request body.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    pub format: ImportFormat,
    pub data: String,
    /// If true, create folders based on folder_path.
    #[serde(default = "default_true")]
    pub create_folders: bool,
}

fn default_true() -> bool {
    true
}

/// Import result summary.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported_count: u32,
    pub skipped_count: u32,
    pub errors: Vec<String>,
}

/// Export request body.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub format: ExportFormat,
}

/// Export result.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub format: String,
    pub data: String,
    pub item_count: u32,
}
