use nexus_core::ClipboardEntry;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// IPC message types for communication between core and UI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IpMessage {
    /// Get clipboard history
    GetHistory,
    /// Search clipboard history
    SearchHistory {
        query: String,
    },
    /// Copy a clipboard entry
    CopyEntry {
        id: u64,
    },
    /// Delete a clipboard entry
    DeleteEntry {
        id: u64,
    },
    /// Pin/unpin a clipboard entry
    PinEntry {
        id: u64,
    },
    /// Clear all clipboard history
    ClearHistory,
    /// Get application settings
    GetSettings,
    /// Update application settings
    UpdateSettings {
        settings: AppSettings,
    },
}

/// Application settings sent via IPC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Whether private mode is enabled
    pub private_mode: bool,
    /// Maximum number of clipboard entries
    pub max_history: usize,
    /// Retention period in days
    pub retention_days: Option<i64>,
}

/// IPC response message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IpResponse {
    /// History response
    History(Vec<ClipboardEntry>),
    /// Search results
    SearchResults(Vec<ClipboardEntry>),
    /// Settings response
    Settings(AppSettings),
    /// Error response
    Error(String),
}

/// Local IPC socket handler
pub struct IpSocket {
    /// Socket path
    path: std::path::PathBuf,
    /// Whether the socket is listening
    listening: bool,
}

impl IpSocket {
    /// Create a new IPC socket at the given path
    pub fn new(socket_path: &Path) -> Self {
        Self {
            path: socket_path.to_path_buf(),
            listening: false,
        }
    }

    /// Start listening for IPC connections
    pub fn listen(&mut self) -> Result<(), std::io::Error> {
        // Create the socket file
        if !self.path.exists() {
            // Remove old socket if it exists
            let _ = std::fs::remove_file(&self.path);
        }

        // Initialize the Unix domain socket
        // This is a simplified implementation
        // In production, would use tokio::net:: unix::AsyncSocket or similar
        self.listening = true;
        Ok(())
    }

    /// Handle an incoming IPC message
    pub fn handle_message(&self, _message: &[u8]) -> Vec<u8> {
        // Placeholder - would parse and route messages
        vec![]
    }

    /// Stop listening
    pub fn stop(&mut self) {
        self.listening = false;
        // Clean up socket file
        let _ = std::fs::remove_file(&self.path);
    }

    /// Check if socket is listening
    pub fn is_listening(&self) -> bool {
        self.listening
    }
}

/// IPC message builder
pub struct IpMessageBuilder;

impl IpMessageBuilder {
    /// Build a GetHistory message
    pub fn get_history() -> Vec<u8> {
        serde_json::to_vec(&IpMessage::GetHistory).unwrap_or_default()
    }

    /// Build a SearchHistory message
    pub fn search_history(query: &str) -> Vec<u8> {
        serde_json::to_vec(&IpMessage::SearchHistory {
            query: query.to_string(),
        })
        .unwrap_or_default()
    }

    /// Build a CopyEntry message
    pub fn copy_entry(id: u64) -> Vec<u8> {
        serde_json::to_vec(&IpMessage::CopyEntry { id }).unwrap_or_default()
    }

    /// Build a DeleteEntry message
    pub fn delete_entry(id: u64) -> Vec<u8> {
        serde_json::to_vec(&IpMessage::DeleteEntry { id }).unwrap_or_default()
    }

    /// Build a PinEntry message
    pub fn pin_entry(id: u64) -> Vec<u8> {
        serde_json::to_vec(&IpMessage::PinEntry { id }).unwrap_or_default()
    }

    /// Build a ClearHistory message
    pub fn clear_history() -> Vec<u8> {
        serde_json::to_vec(&IpMessage::ClearHistory).unwrap_or_default()
    }

    /// Build a GetSettings message
    pub fn get_settings() -> Vec<u8> {
        serde_json::to_vec(&IpMessage::GetSettings).unwrap_or_default()
    }

    /// Build an UpdateSettings message
    pub fn update_settings(settings: &AppSettings) -> Vec<u8> {
        serde_json::to_vec(&IpMessage::UpdateSettings {
            settings: settings.clone(),
        })
        .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_socket_creation() {
        let dir = tempdir().unwrap();
        let socket = IpSocket::new(&dir.path().join("test.sock"));
        assert!(!socket.is_listening());
    }

    #[test]
    fn test_message_builder() {
        let history_msg = IpMessageBuilder::get_history();
        let parsed: IpMessage = serde_json::from_slice(&history_msg).unwrap();
        assert_eq!(parsed, IpMessage::GetHistory);
    }

    #[test]
    fn test_update_settings() {
        let settings = AppSettings {
            private_mode: true,
            max_history: 50,
            retention_days: Some(30),
        };
        let msg = IpMessageBuilder::update_settings(&settings);
        let parsed: IpMessage = serde_json::from_slice(&msg).unwrap();
        match parsed {
            IpMessage::UpdateSettings { settings } => {
                assert_eq!(settings.private_mode, true);
                assert_eq!(settings.max_history, 50);
            }
            _ => panic!("Expected UpdateSettings message"),
        }
    }
}
