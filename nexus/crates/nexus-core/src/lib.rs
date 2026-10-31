use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

/// Represents a clipboard entry in the history
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipboardEntry {
    /// Unique identifier for the entry
    pub id: u64,
    /// Type of content (text, url, etc.)
    pub content_type: ContentType,
    /// The actual content
    pub content: String,
    /// Hash of the content for duplicate detection
    pub content_hash: String,
    /// Timestamp when entry was created (unix epoch seconds)
    pub created_at: i64,
    /// Timestamp when entry was last used (unix epoch seconds, None if never used)
    pub last_used_at: Option<i64>,
    /// Whether the entry is pinned/pinned
    pub is_pinned: bool,
}

/// Content type for clipboard entries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    /// Plain text
    Text,
    /// URL
    Url,
    /// Image / Screenshot
    Image,
    /// Other MIME type
    Other(String),
}

impl ContentType {
    /// Parse from a MIME type string
    pub fn from_mime(mime: &str) -> Self {
        match mime {
            "text/plain" => ContentType::Text,
            "text/uri-list" | "uri" => ContentType::Url,
            "image/png" | "image/jpeg" | "image/webp" | "image/bmp" | "image" => ContentType::Image,
            other => ContentType::Other(other.to_string()),
        }
    }

    /// Get the MIME type display name
    pub fn mime_type(&self) -> &str {
        match self {
            ContentType::Text => "text/plain",
            ContentType::Url => "text/uri-list",
            ContentType::Image => "image/png",
            ContentType::Other(mime) => mime,
        }
    }
}

/// Settings for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Whether private mode is enabled
    pub private_mode: bool,
    /// Maximum number of clipboard entries to keep
    pub max_history: usize,
    /// Retention period in days (None for indefinite)
    pub retention_days: Option<i64>,
    /// Whether to automatically exclude sensitive applications
    pub auto_exclude_sensitive: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            private_mode: false,
            max_history: 100,
            retention_days: None,
            auto_exclude_sensitive: false,
        }
    }
}

/// Error types for nexus-core
#[derive(thiserror::Error, Debug)]
pub enum NexusError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] sqlite::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Entry not found")]
    EntryNotFound,
    #[error("Duplicate entry")]
    DuplicateEntry,
    #[error("Invalid content type")]
    InvalidContentType,
    #[error("Settings error: {0}")]
    SettingsError(String),
}

/// Trait for clipboard storage operations
pub trait ClipboardStorage: Send + Sync {
    /// Save a clipboard entry
    fn save_entry(&self, entry: &ClipboardEntry) -> Result<(), NexusError>;
    /// Get all clipboard entries
    fn get_entries(&self) -> Result<Vec<ClipboardEntry>, NexusError>;
    /// Get pinned entries
    fn get_pinned_entries(&self) -> Result<Vec<ClipboardEntry>, NexusError>;
    /// Search entries by content
    fn search_entries(&self, query: &str) -> Result<Vec<ClipboardEntry>, NexusError>;
    /// Delete an entry by ID
    fn delete_entry(&self, id: u64) -> Result<(), NexusError>;
    /// Update last_used_at timestamp
    fn mark_used(&self, id: u64) -> Result<(), NexusError>;
    /// Clear all entries
    fn clear_all(&self) -> Result<(), NexusError>;
}

/// Initialize the SQLite database schema
pub fn initialize_database(db_path: &Path) -> Result<(), NexusError> {
    let actual_path = if db_path.is_dir() {
        db_path.join("nexus.db")
    } else {
        db_path.to_path_buf()
    };
    let conn = sqlite::open(&actual_path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS clipboard_entries (
            id INTEGER PRIMARY KEY,
            content_type TEXT NOT NULL,
            content TEXT,
            content_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            last_used_at INTEGER,
            is_pinned INTEGER DEFAULT 0
        )",
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS favorites (
            id INTEGER PRIMARY KEY,
            item_type TEXT NOT NULL,
            item_value TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
    )?;
    
    info!("Database initialized at {:?}", db_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(!settings.private_mode);
        assert_eq!(settings.max_history, 100);
        assert!(settings.retention_days.is_none());
        assert!(!settings.auto_exclude_sensitive);
    }

    #[test]
    fn test_content_type_from_mime() {
        assert_eq!(ContentType::from_mime("text/plain"), ContentType::Text);
        assert_eq!(ContentType::from_mime("text/uri-list"), ContentType::Url);
        assert_eq!(ContentType::from_mime("image/png"), ContentType::Image);
        assert!(matches!(ContentType::from_mime("application/pdf"), ContentType::Other(_)));
    }

    #[test]
    fn test_database_initialization() {
        let dir = tempdir().unwrap();
        let result = initialize_database(dir.path());
        assert!(result.is_ok());
    }
}