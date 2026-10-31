use std::collections::HashMap;

/// Represents a GIF entry
#[derive(Debug, Clone)]
pub struct GifEntry {
    /// Unique identifier
    pub id: String,
    /// URL to the GIF
    pub url: String,
    /// Preview thumbnail URL
    pub preview_url: Option<String>,
    /// Title/description
    pub title: String,
    /// Rating (pg, pg-13, r, etc.)
    pub rating: String,
    /// Whether it's a favorite
    pub is_favorite: bool,
    /// When it was added
    pub created_at: std::time::Instant,
}

/// GIF provider trait for different GIF sources
pub trait GifProvider: Send + Sync {
    /// Search for GIFs by query
    fn search(&self, query: &str, limit: usize) -> Vec<GifEntry>;
    /// Get a GIF by ID
    fn get_by_id(&self, id: &str) -> Option<GifEntry>;
    /// Get trending GIFs
    fn trending(&self, limit: usize) -> Vec<GifEntry>;
}

/// In-memory GIF provider for testing
#[derive(Debug)]
pub struct InMemoryGifProvider {
    /// Internal GIF storage
    gifs: HashMap<String, GifEntry>,
}

impl InMemoryGifProvider {
    /// Create a new empty provider
    pub fn new() -> Self {
        Self {
            gifs: HashMap::new(),
        }
    }

    /// Add a GIF to the provider
    pub fn add(&mut self, gif: GifEntry) {
        self.gifs.insert(gif.id.clone(), gif);
    }
}

impl GifProvider for InMemoryGifProvider {
    fn search(&self, query: &str, limit: usize) -> Vec<GifEntry> {
        self.gifs
            .values()
            .filter(|gif| {
                gif.title.to_lowercase().contains(&query.to_lowercase())
                    || gif.rating.to_lowercase().contains(&query.to_lowercase())
            })
            .take(limit)
            .cloned()
            .collect()
    }

    fn get_by_id(&self, id: &str) -> Option<GifEntry> {
        self.gifs.get(id).cloned()
    }

    fn trending(&self, limit: usize) -> Vec<GifEntry> {
        self.gifs.values().take(limit).cloned().collect()
    }
}

/// GIF provider enum for supporting multiple sources
#[derive(Debug)]
pub enum GifProviderEnum {
    /// In-memory provider (for testing)
    InMemory(InMemoryGifProvider),
    /// Placeholder for external providers (Tenor, etc.)
    External(ExternalProvider),
}

/// External GIF provider (future implementation)
#[derive(Debug)]
pub struct ExternalProvider {
    /// API key for the external service
    pub api_key: String,
    /// Base URL for the API
    pub base_url: String,
}

impl ExternalProvider {
    /// Create a new external provider
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
        }
    }
}

impl GifProvider for ExternalProvider {
    fn search(&self, _query: &str, _limit: usize) -> Vec<GifEntry> {
        // Placeholder - would make actual API call
        vec![]
    }

    fn get_by_id(&self, _id: &str) -> Option<GifEntry> {
        // Placeholder - would make actual API call
        None
    }

    fn trending(&self, _limit: usize) -> Vec<GifEntry> {
        // Placeholder - would make actual API call
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_provider() {
        let mut provider = InMemoryGifProvider::new();
        provider.add(GifEntry {
            id: "1".to_string(),
            url: "https://example.com/test.gif".to_string(),
            preview_url: Some("https://example.com/test-preview.gif".to_string()),
            title: "Test GIF".to_string(),
            rating: "pg".to_string(),
            is_favorite: false,
            created_at: std::time::Instant::now(),
        });

        let results = provider.search("test", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test GIF");
    }

    #[test]
    fn test_get_by_id() {
        let mut provider = InMemoryGifProvider::new();
        provider.add(GifEntry {
            id: "test-id".to_string(),
            url: "https://example.com/test.gif".to_string(),
            preview_url: None,
            title: "Test".to_string(),
            rating: "pg".to_string(),
            is_favorite: false,
            created_at: std::time::Instant::now(),
        });

        let gif = provider.get_by_id("test-id");
        assert!(gif.is_some());
        assert_eq!(gif.unwrap().title, "Test");
    }
}