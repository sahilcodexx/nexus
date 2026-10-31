use std::collections::HashMap;

/// Represents a single emoji with its properties
#[derive(Debug, Clone, PartialEq)]
pub struct Emoji {
    /// The actual emoji character
    pub character: String,
    /// Unicode code point name
    pub unicode_name: String,
    /// Category the emoji belongs to
    pub category: EmojiCategory,
    /// Whether skin tone variations are available
    pub has_skin_tone: bool,
    /// Shortcodes (like :smile:)
    pub shortcodes: Vec<String>,
}

/// Emoji category for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmojiCategory {
    SmileysAndEmotions,
    PeopleAndBody,
    AnimalsAndNature,
    FoodAndDrink,
    TravelAndPlaces,
    Activities,
    Objects,
    Symbols,
    Flags,
}

/// Local emoji dataset container
#[derive(Debug)]
pub struct EmojiDataset {
    /// All emojis indexed by character
    emojis: HashMap<String, Emoji>,
    /// All emojis indexed by category
    by_category: HashMap<EmojiCategory, Vec<Emoji>>,
}

impl EmojiDataset {
    /// Create a new empty dataset
    pub fn new() -> Self {
        Self {
            emojis: HashMap::new(),
            by_category: HashMap::new(),
        }
    }

    /// Add an emoji to the dataset
    pub fn add(&mut self, emoji: Emoji) {
        let char_key = emoji.character.clone();
        self.emojis.insert(char_key.clone(), emoji.clone());

        self.by_category
            .entry(emoji.category.clone())
            .or_default()
            .push(emoji);
    }

    /// Get an emoji by its character
    pub fn get(&self, character: &str) -> Option<&Emoji> {
        self.emojis.get(character)
    }

    /// Get emojis by category
    pub fn get_by_category(&self, category: &EmojiCategory) -> Option<&Vec<Emoji>> {
        self.by_category.get(category)
    }

    /// Search emojis by query text
    pub fn search(&self, query: &str) -> Vec<&Emoji> {
        self.emojis
            .values()
            .filter(|e| {
                e.unicode_name.to_lowercase().contains(&query.to_lowercase())
                    || e
                        .shortcodes
                        .iter()
                        .any(|s| s.to_lowercase().contains(&query.to_lowercase()))
            })
            .collect()
    }

    /// Check if an emoji exists
    pub fn contains(&self, character: &str) -> bool {
        self.emojis.contains_key(character)
    }

    /// Get all categories
    pub fn categories(&self) -> Vec<&EmojiCategory> {
        self.by_category.keys().collect()
    }
}

/// Create a common emoji dataset with frequently used emojis
pub fn create_common_emoji_dataset() -> EmojiDataset {
    let mut dataset = EmojiDataset::new();

    // Smileys and emotions
    dataset.add(Emoji {
        character: "😀".to_string(),
        unicode_name: "GRINNING FACE".to_string(),
        category: EmojiCategory::SmileysAndEmotions,
        has_skin_tone: false,
        shortcodes: vec!["smile", "grin"].into_iter().map(|s| s.to_string()).collect(),
    });

    dataset.add(Emoji {
        character: "😂".to_string(),
        unicode_name: "FACE WITH TEARS OF JOY".to_string(),
        category: EmojiCategory::SmileysAndEmotions,
        has_skin_tone: false,
        shortcodes: vec!["rofl", "laugh"].into_iter().map(|s| s.to_string()).collect(),
    });

    // People
    dataset.add(Emoji {
        character: "👋".to_string(),
        unicode_name: "WAVING HAND".to_string(),
        category: EmojiCategory::PeopleAndBody,
        has_skin_tone: false,
        shortcodes: vec!["wave"].into_iter().map(|s| s.to_string()).collect(),
    });

    dataset
}

/// Common emoji dataset
/// Get the common emoji dataset
pub fn get_emoji_dataset() -> EmojiDataset {
    create_common_emoji_dataset()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_creation() {
        let dataset = EmojiDataset::new();
        assert!(dataset.emojis.is_empty());
        assert!(dataset.by_category.is_empty());
    }

    #[test]
    fn test_emoji_add_and_get() {
        let dataset = get_emoji_dataset();
        let emoji = dataset.emojis.get("😀");
        assert!(emoji.is_some());
        assert_eq!(emoji.unwrap().category, EmojiCategory::SmileysAndEmotions);
    }

    #[test]
    fn test_emoji_search() {
        let dataset = get_emoji_dataset();
        let results = dataset.search("joy");
        assert!(!results.is_empty());
        assert!(
            results
                .iter()
                .any(|e| e.unicode_name.contains("JOY"))
        );
    }
}