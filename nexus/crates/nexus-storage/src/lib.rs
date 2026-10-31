use sqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path:: Path;
use std::time::{SystemTime, UNIX_EPOCH};

//// SQLite-based storage implementation
pub struct SqliteStorage {
    /// Database connection
    conn: Connection,
}

impl SqliteStorage {
    /// Open a new storage at the given path
    pub fn open(path: &Path) -> Result<Self, sqlite::Error> {
        let db_file = if path.is_dir() {
            path.join("nexus.db")
        } else {
            path.to_path_buf()
        };
        let conn = Connection::open(&db_file)?;

        // Initialize tables
        Self::initialize(&conn)?;

        Ok(Self { conn })
    }

    /// Initialize the database schema
    fn initialize(conn: &Connection) -> Result<(), sqlite::Error> {
        // Create clipboard_entries table
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

        // Create favorites table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorites (
                id INTEGER PRIMARY KEY,
                item_type TEXT NOT NULL,
                item_value TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )?;

        // Create settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )?;

        Ok(())
    }

    /// Get current unix epoch seconds
    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// Save a clipboard entry (or update timestamp and move to top if duplicate content)
    pub fn save_entry(&mut self, entry: &ClipboardEntry) -> Result<(), sqlite::Error> {
        let now = Self::now();

        // Check if an entry with this exact content already exists using parameterized query
        let mut check_stmt = self.conn.prepare("SELECT id FROM clipboard_entries WHERE content = ? LIMIT 1")?;
        check_stmt.bind(1, &entry.content[..])?;
        let mut existing_id: Option<i64> = None;
        if check_stmt.next()? == sqlite::State::Row {
            existing_id = Some(check_stmt.read::<i64>(0)?);
        }

        if let Some(id) = existing_id {
            // Content was copied again: immediately jump to top of history!
            let mut update_stmt = self.conn.prepare(
                "UPDATE clipboard_entries SET created_at = ?, last_used_at = ? WHERE id = ?"
            )?;
            update_stmt.bind(1, now)?;
            update_stmt.bind(2, now)?;
            update_stmt.bind(3, id)?;
            update_stmt.next()?;
        } else {
            // New entry: assign a unique ID that does not collide
            let mut next_id: i64 = 1;
            let mut id_stmt = self.conn.prepare("SELECT COALESCE(MAX(id), 0) + 1 FROM clipboard_entries")?;
            if id_stmt.next()? == sqlite::State::Row {
                next_id = id_stmt.read::<i64>(0)?;
            }
            let hash = if entry.content_hash.is_empty() {
                format!("{:x?}", entry.content)
            } else {
                entry.content_hash.clone()
            };
            let mut insert_stmt = self.conn.prepare(
                "INSERT INTO clipboard_entries (id, content_type, content, content_hash, created_at, last_used_at, is_pinned) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )?;
            insert_stmt.bind(1, next_id)?;
            insert_stmt.bind(2, entry.content_type.mime_type())?;
            insert_stmt.bind(3, &entry.content[..])?;
            insert_stmt.bind(4, &hash[..])?;
            insert_stmt.bind(5, now)?;
            insert_stmt.bind(6, now)?;
            insert_stmt.bind(7, if entry.is_pinned { 1 } else { 0 })?;
            insert_stmt.next()?;
        }

        Ok(())
    }

    /// Get database stats for quick change detection (count, max created_at)
    pub fn get_stats(&self) -> Result<(usize, i64), sqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*), COALESCE(MAX(created_at), 0) FROM clipboard_entries")?;
        if stmt.next()? == sqlite::State::Row {
            let count: i64 = stmt.read(0)?;
            let max_created: i64 = stmt.read(1)?;
            return Ok((count as usize, max_created));
        }
        Ok((0, 0))
    }

    /// Get all clipboard entries
    pub fn get_entries(&self) -> Result<Vec<ClipboardEntry>, sqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content, content_hash, created_at, last_used_at, is_pinned FROM clipboard_entries ORDER BY is_pinned DESC, created_at DESC",
        )?;

        let mut entries = Vec::new();

        let mut state = stmt.next()?;
        while state == sqlite::State::Row {
            let id: i64 = stmt.read(0)?;
            let content_type: String = stmt.read(1)?;
            let content: String = stmt.read(2)?;  // Read as String, handle empty
            let content_hash: String = stmt.read(3)?;
            let created_at: i64 = stmt.read(4)?;
            // last_used_at is optional - read as i64 and check for 0/sentinel
            let last_used_at_raw: i64 = stmt.read(5)?;
            let is_pinned: i64 = stmt.read(6)?;

            let last_used_at = if last_used_at_raw > 0 { Some(last_used_at_raw) } else { None };

            let entry_type = ContentType::from_mime(&content_type);

            entries.push(ClipboardEntry {
                id: id as u64,
                content_type: entry_type,
                content: if content.is_empty() { String::new() } else { content },
                content_hash,
                created_at,
                last_used_at,
                is_pinned: is_pinned == 1,
            });

            state = stmt.next()?;
        }

        Ok(entries)
    }

    /// Get pinned entries only
    pub fn get_pinned_entries(&self) -> Result<Vec<ClipboardEntry>, sqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content, content_hash, created_at, last_used_at, is_pinned FROM clipboard_entries WHERE is_pinned = 1 ORDER BY created_at DESC",
        )?;

        let mut entries = Vec::new();

        let mut state = stmt.next()?;
        while state == sqlite::State::Row {
            let id: i64 = stmt.read(0)?;
            let content_type: String = stmt.read(1)?;
            let content: String = stmt.read(2)?;  // Read as String, handle empty
            let content_hash: String = stmt.read(3)?;
            let created_at: i64 = stmt.read(4)?;
            // last_used_at is optional - read as i64 and check for 0/sentinel
            let last_used_at_raw: i64 = stmt.read(5)?;
            let is_pinned: i64 = stmt.read(6)?;

            let last_used_at = if last_used_at_raw > 0 { Some(last_used_at_raw) } else { None };

            let entry_type = ContentType::from_mime(&content_type);

            entries.push(ClipboardEntry {
                id: id as u64,
                content_type: entry_type,
                content: if content.is_empty() { String::new() } else { content },
                content_hash,
                created_at,
                last_used_at,
                is_pinned: is_pinned == 1,
            });

            state = stmt.next()?;
        }

        Ok(entries)
    }

    /// Search entries by content using LIKE
    pub fn search_entries(&self, query: &str) -> Result<Vec<ClipboardEntry>, sqlite::Error> {
        let pattern = format!("%{}%", query);
        let sql = "SELECT id, content_type, content, content_hash, created_at, last_used_at, is_pinned FROM clipboard_entries
             WHERE content LIKE ? OR content_hash LIKE ?
             ORDER BY is_pinned DESC, created_at DESC";

        let mut stmt = self.conn.prepare(sql)?;
        stmt.bind(1, &pattern[..])?;
        stmt.bind(2, &pattern[..])?;

        let mut entries = Vec::new();

        let mut state = stmt.next()?;
        while state == sqlite::State::Row {
            let id: i64 = stmt.read(0)?;
            let content_type: String = stmt.read(1)?;
            let content: String = stmt.read(2)?;  // Read as String, handle empty
            let content_hash: String = stmt.read(3)?;
            let created_at: i64 = stmt.read(4)?;
            // last_used_at is optional - read as i64 and check for 0/sentinel
            let last_used_at_raw: i64 = stmt.read(5)?;
            let is_pinned: i64 = stmt.read(6)?;

            let last_used_at = if last_used_at_raw > 0 { Some(last_used_at_raw) } else { None };

            let entry_type = ContentType::from_mime(&content_type);

            entries.push(ClipboardEntry {
                id: id as u64,
                content_type: entry_type,
                content: if content.is_empty() { String::new() } else { content },
                content_hash,
                created_at,
                last_used_at,
                is_pinned: is_pinned == 1,
            });

            state = stmt.next()?;
        }

        Ok(entries)
    }

    /// Delete an entry by ID
    pub fn delete_entry(&mut self, id: u64) -> Result<(), sqlite::Error> {
        let mut stmt = self.conn.prepare("DELETE FROM clipboard_entries WHERE id = ?")?;
        stmt.bind(1, id as i64)?;
        stmt.next()?;

        Ok(())
    }

    /// Mark an entry as used (update last_used_at)
    pub fn mark_used(&mut self, id: u64) -> Result<(), sqlite::Error> {
        let mut stmt = self.conn.prepare("UPDATE clipboard_entries SET last_used_at = ? WHERE id = ?")?;
        stmt.bind(1, Self::now())?;
        stmt.bind(2, id as i64)?;
        stmt.next()?;

        Ok(())
    }

    /// Clear all entries
    pub fn clear_all(&mut self) -> Result<(), sqlite::Error> {
        self.conn.execute("DELETE FROM clipboard_entries")?;

        Ok(())
    }

    /// Clear unpinned entries only
    pub fn clear_unpinned(&mut self) -> Result<(), sqlite::Error> {
        self.conn.execute("DELETE FROM clipboard_entries WHERE is_pinned = 0")?;

        Ok(())
    }

    /// Toggle pin status of an entry
    pub fn toggle_pin(&mut self, id: u64) -> Result<bool, sqlite::Error> {
        let mut update_stmt = self.conn.prepare("UPDATE clipboard_entries SET is_pinned = 1 - is_pinned WHERE id = ?")?;
        update_stmt.bind(1, id as i64)?;
        update_stmt.next()?;

        let mut stmt = self.conn.prepare("SELECT is_pinned FROM clipboard_entries WHERE id = ?")?;
        stmt.bind(1, id as i64)?;
        if stmt.next()? == sqlite::State::Row {
            let is_pinned: i64 = stmt.read(0)?;
            Ok(is_pinned == 1)
        } else {
            Ok(false)
        }
    }

    /// Save a setting
    pub fn save_setting(&mut self, key: &str, value: &str) -> Result<(), sqlite::Error> {
        let mut stmt = self.conn.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")?;
        stmt.bind(1, key)?;
        stmt.bind(2, value)?;
        stmt.next()?;

        Ok(())
    }

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, sqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT value FROM settings WHERE key = ?")?;
        stmt.bind(1, key)?;

        let state = stmt.next()?;
        if state == sqlite::State::Row {
            let value: String = stmt.read(0)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Get all settings
    pub fn get_all_settings(&self) -> Result<Vec<(String, String)>, sqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;

        let mut settings = Vec::new();

        let mut state = stmt.next()?;
        while state == sqlite::State::Row {
            let key: String = stmt.read(0)?;
            let value: String = stmt.read(1)?;
            settings.push((key, value));

            state = stmt.next()?;
        }

        Ok(settings)
    }
}

/// Clipboard entry for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    /// Unique identifier
    pub id: u64,
    /// Content type
    pub content_type: ContentType,
    /// The content
    pub content: String,
    /// Hash of the content
    pub content_hash: String,
    /// Created at timestamp (unix epoch)
    pub created_at: i64,
    /// Last used at timestamp (unix epoch, None if never used)
    pub last_used_at: Option<i64>,
    /// Whether pinned
    pub is_pinned: bool,
}

/// Content type enum
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

impl Default for ClipboardEntry {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            id: 0,
            content_type: ContentType::Text,
            content: String::new(),
            content_hash: String::new(),
            created_at: now,
            last_used_at: None,
            is_pinned: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage_init() {
        let dir = tempdir().unwrap();
        let storage = SqliteStorage::open(dir.path()).unwrap();
        let entries = storage.get_entries().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_save_and_retrieve_entry() {
        let dir = tempdir().unwrap();
        let mut storage = SqliteStorage::open(dir.path()).unwrap();

        let entry = ClipboardEntry {
            id: 1,
            content_type: ContentType::Text,
            content: "Hello, world!".to_string(),
            content_hash: "test-hash".to_string(),
            created_at: 1000,
            last_used_at: None,
            is_pinned: false,
        };

        storage.save_entry(&entry).unwrap();
        let entries = storage.get_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "Hello, world!");
    }

    #[test]
    fn test_search_entries() {
        let dir = tempdir().unwrap();
        let mut storage = SqliteStorage::open(dir.path()).unwrap();

        // Save a test entry
        let entry = ClipboardEntry {
            id: 1,
            content_type: ContentType::Text,
            content: "Hello world test".to_string(),
            content_hash: "hash1".to_string(),
            created_at: 1000,
            last_used_at: None,
            is_pinned: false,
        };

        storage.save_entry(&entry).unwrap();

        // Search for "world"
        let results = storage.search_entries("world").unwrap();
        assert!(!results.is_empty());
        assert!(
            results.iter().any(|e| e.content.contains("world"))
        );
    }

    #[test]
    fn test_delete_entry() {
        let dir = tempdir().unwrap();
        let mut storage = SqliteStorage::open(dir.path()).unwrap();

        // Save an entry
        let entry = ClipboardEntry {
            id: 1,
            content_type: ContentType::Text,
            content: "To be deleted".to_string(),
            content_hash: "hash-del".to_string(),
            created_at: 1000,
            last_used_at: None,
            is_pinned: false,
        };

        storage.save_entry(&entry).unwrap();

        // Delete it
        storage.delete_entry(1).unwrap();

        // Verify it's gone
        let entries = storage.get_entries().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_settings() {
        let dir = tempdir().unwrap();
        let mut storage = SqliteStorage::open(dir.path()).unwrap();

        // Save a setting
        storage.save_setting("test_key", "test_value").unwrap();

        // Retrieve it
        let value = storage.get_setting("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[test]
    fn test_apostrophe_and_special_chars() {
        let dir = tempdir().unwrap();
        let mut storage = SqliteStorage::open(dir.path()).unwrap();

        let special_text = "It's working! Don't worry, here's some emojis: 🔥🚀 and symbols: <>&'\"\\";
        let entry = ClipboardEntry {
            id: 0,
            content_type: ContentType::Text,
            content: special_text.to_string(),
            content_hash: "hash".to_string(),
            created_at: 1000,
            last_used_at: None,
            is_pinned: false,
        };

        storage.save_entry(&entry).unwrap();
        let entries = storage.get_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, special_text);

        // Search test
        let results = storage.search_entries("Don't").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, special_text);
    }
}