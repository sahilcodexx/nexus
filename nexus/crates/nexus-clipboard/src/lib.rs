use nexus_storage::{ClipboardEntry as StorageEntry, ContentType, SqliteStorage};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::mpsc;

/// Clipboard manager error types
#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("Storage error: {0}")]
    Storage(#[from] sqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Clipboard not available")]
    NotAvailable,
}

/// Clipboard event types
#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    /// New clipboard entry available
    NewEntry { entry: StorageEntry },
    /// Clipboard content changed
    Changed,
    /// History cleared
    Cleared,
}

/// Clipboard manager
pub struct ClipboardManager {
    storage: Arc<Mutex<SqliteStorage>>,
    event_sender: mpsc::UnboundedSender<ClipboardEvent>,
    private_mode: Arc<Mutex<bool>>,
    max_history: Arc<Mutex<usize>>,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new(db_path: PathBuf) -> Result<(Self, mpsc::UnboundedReceiver<ClipboardEvent>), ClipboardError> {
        let storage = SqliteStorage::open(&db_path)?;
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Ok((
            Self {
                storage: Arc::new(Mutex::new(storage)),
                event_sender,
                private_mode: Arc::new(Mutex::new(false)),
                max_history: Arc::new(Mutex::new(100)),
            },
            event_receiver,
        ))
    }

    /// Add a new clipboard entry
    pub fn add_entry(&self, content: String, content_type: ContentType) -> Result<u64, ClipboardError> {
        // Check private mode
        if *self.private_mode.lock().unwrap() {
            return Err(ClipboardError::NotAvailable);
        }

        // Check for duplicates
        let hash = Self::hash_content(&content);
        let mut storage = self.storage.lock().unwrap();
        let entries = storage.get_entries()?;
        
        if entries.iter().any(|e| e.content_hash == hash) {
            return Err(ClipboardError::NotAvailable);
        }

        // Check history limit
        let max_history = *self.max_history.lock().unwrap();
        if entries.len() >= max_history {
            // Remove oldest non-pinned entry
            if let Some(oldest) = entries.iter().rev().find(|e| !e.is_pinned) {
                storage.delete_entry(oldest.id)?;
            }
        }

        // Create new entry
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let entry = StorageEntry {
            id: entries.len() as u64 + 1,
            content_type,
            content: content.clone(),
            content_hash: hash,
            created_at: now,
            last_used_at: None,
            is_pinned: false,
        };

        storage.save_entry(&entry)?;

        // Send event
        let _ = self.event_sender.send(ClipboardEvent::NewEntry { entry: entry.clone() });

        Ok(entry.id)
    }

    /// Get all clipboard entries
    pub fn get_entries(&self) -> Result<Vec<StorageEntry>, ClipboardError> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.get_entries()?)
    }

    /// Search clipboard entries
    pub fn search(&self, query: &str) -> Result<Vec<StorageEntry>, ClipboardError> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.search_entries(query)?)
    }

    /// Delete an entry
    pub fn delete_entry(&self, id: u64) -> Result<(), ClipboardError> {
        let mut storage = self.storage.lock().unwrap();
        storage.delete_entry(id)?;
        Ok(())
    }

    /// Pin an entry
    pub fn pin_entry(&self, id: u64) -> Result<(), ClipboardError> {
        let mut storage = self.storage.lock().unwrap();
        let entries = storage.get_entries()?;
        if let Some(entry) = entries.iter().find(|e| e.id == id) {
            let mut updated = entry.clone();
            updated.is_pinned = true;
            storage.save_entry(&updated)?;
        }
        Ok(())
    }

    /// Unpin an entry
    pub fn unpin_entry(&self, id: u64) -> Result<(), ClipboardError> {
        let mut storage = self.storage.lock().unwrap();
        let entries = storage.get_entries()?;
        if let Some(entry) = entries.iter().find(|e| e.id == id) {
            let mut updated = entry.clone();
            updated.is_pinned = false;
            storage.save_entry(&updated)?;
        }
        Ok(())
    }

    /// Clear all entries
    pub fn clear_all(&self) -> Result<(), ClipboardError> {
        let mut storage = self.storage.lock().unwrap();
        storage.clear_all()?;
        let _ = self.event_sender.send(ClipboardEvent::Cleared);
        Ok(())
    }

    /// Clear unpinned entries only
    pub fn clear_unpinned(&self) -> Result<(), ClipboardError> {
        let mut storage = self.storage.lock().unwrap();
        storage.clear_unpinned()?;
        let _ = self.event_sender.send(ClipboardEvent::Changed);
        Ok(())
    }

    /// Toggle pin status of an entry
    pub fn toggle_pin(&self, id: u64) -> Result<bool, ClipboardError> {
        let mut storage = self.storage.lock().unwrap();
        let new_state = storage.toggle_pin(id)?;
        let _ = self.event_sender.send(ClipboardEvent::Changed);
        Ok(new_state)
    }

    /// Set private mode
    pub fn set_private_mode(&self, enabled: bool) {
        *self.private_mode.lock().unwrap() = enabled;
    }

    /// Set max history
    pub fn set_max_history(&self, max: usize) {
        *self.max_history.lock().unwrap() = max;
    }

    /// Hash content for duplicate detection
    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_clipboard_manager() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let (manager, _receiver) = ClipboardManager::new(db_path).unwrap();

        // Add an entry
        let id = manager
            .add_entry("Hello, world!".to_string(), ContentType::Text)
            .unwrap();

        // Get entries
        let entries = manager.get_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "Hello, world!");

        // Search
        let results = manager.search("world").unwrap();
        assert_eq!(results.len(), 1);

        // Delete
        manager.delete_entry(id).unwrap();
        let entries = manager.get_entries().unwrap();
        assert_eq!(entries.len(), 0);
    }
}