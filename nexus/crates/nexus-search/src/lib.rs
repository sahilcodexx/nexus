use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Content type for clipboard entries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Url,
    Other(String),
}

/// Search query and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub whole_word: bool,
    pub case_sensitive: bool,
}

/// Search results containing matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults<T> {
    pub matches: Vec<T>,
    pub has_more: bool,
    pub total_count: usize,
}

/// Search trait for different search strategies
pub trait SearchStrategy<T>: Send + Sync {
    fn search(&self, items: &[T], query: &SearchQuery) -> SearchResults<T>;
}

/// Clipboard entry search strategy
pub struct ClipboardSearchStrategy;

impl SearchStrategy<ClipboardEntry> for ClipboardSearchStrategy {
    fn search(&self, entries: &[ClipboardEntry], query: &SearchQuery) -> SearchResults<ClipboardEntry> {
        let mut matches = Vec::new();
        let query_lower = if query.case_sensitive {
            query.query.clone()
        } else {
            query.query.to_lowercase()
        };

        for entry in entries {
            let mut matched = false;

            // Search in content
            let content_matches = if query.whole_word {
                // Whole word match
                let content_lower = entry.content.to_lowercase();
                let words: HashSet<&str> = content_lower.split_whitespace().collect();
                words.contains(&query_lower.as_str())
            } else {
                // Substring match
                entry.content.to_lowercase().contains(&query_lower)
                    || entry.content_hash.to_lowercase().contains(&query_lower)
            };

            if content_matches {
                matched = true;
            }

            // Search in content type
            let type_matches = match &entry.content_type {
                ContentType::Text => query.query.to_lowercase().contains("text")
                    || query.query.to_lowercase().contains("plain"),
                ContentType::Url => query.query.to_lowercase().contains("url")
                    || query.query.to_lowercase().contains("link"),
                ContentType::Other(_) => false,
            };

            if type_matches && !matched {
                matched = true;
            }

            if matched {
                matches.push(entry.clone());
            }
        }

        let total = matches.len();
        let has_more = total < entries.len();

        SearchResults {
            matches,
            has_more,
            total_count: total,
        }
    }
}

/// Emoji search strategy
pub struct EmojiSearchStrategy;

impl SearchStrategy<Emoji> for EmojiSearchStrategy {
    fn search(&self, emojis: &[Emoji], query: &SearchQuery) -> SearchResults<Emoji> {
        let mut matches = Vec::new();
        let query_lower = if query.case_sensitive {
            query.query.clone()
        } else {
            query.query.to_lowercase()
        };

        for emoji in emojis {
            let mut matched = false;

            // Search in unicode name
            if emoji
                .unicode_name
                .to_lowercase()
                .contains(&query_lower)
            {
                matched = true;
            }

            // Search in shortcodes
            if !matched {
                for code in &emoji.shortcodes {
                    if code.to_lowercase().contains(&query_lower) {
                        matched = true;
                        break;
                    }
                }
            }

            // Search in category
            if !matched {
                let category_name = format!("{:?}", emoji.category).to_lowercase();
                if category_name.contains(&query_lower) {
                    matched = true;
                }
            }

            if matched {
                matches.push(emoji.clone());
            }
        }

        let total = matches.len();
        let has_more = total < emojis.len();

        SearchResults {
            matches,
            has_more,
            total_count: total,
        }
    }
}

/// Symbol search strategy
pub struct SymbolSearchStrategy;

impl SearchStrategy<Symbol> for SymbolSearchStrategy {
    fn search(&self, symbols: &[Symbol], query: &SearchQuery) -> SearchResults<Symbol> {
        let mut matches = Vec::new();
        let query_lower = if query.case_sensitive {
            query.query.clone()
        } else {
            query.query.to_lowercase()
        };

        for symbol in symbols {
            let mut matched = false;

            // Search in the symbol value
            if symbol.value.to_lowercase().contains(&query_lower) {
                matched = true;
            }

            // Search in category
            if !matched {
                let category_name = format!("{:?}", symbol.category).to_lowercase();
                if category_name.contains(&query_lower) {
                    matched = true;
                }
            }

            if matched {
                matches.push(symbol.clone());
            }
        }

        let total = matches.len();
        let has_more = total < symbols.len();

        SearchResults {
            matches,
            has_more,
            total_count: total,
        }
    }
}

// Clipboard entry type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: u64,
    pub content_type: ContentType,
    pub content: String,
    pub content_hash: String,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub is_pinned: bool,
}

/// Emoji type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Emoji {
    pub character: String,
    pub unicode_name: String,
    pub category: EmojiCategory,
    pub has_skin_tone: bool,
    pub shortcodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EmojiCategory {
    SmileysAndEmotions,
    AnimalsAndNature,
    FoodAndDrink,
    PeopleAndBody,
    NatureAndObjects,
    Symbols,
    TravelAndPlaces,
}

/// Symbol type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    pub value: String,
    pub category: SymbolCategory,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymbolCategory {
    Mathematical,
    Currency,
    Arrows,
    GreekAlphabet,
    Technical,
    Punctuation,
    Superscripts,
    Subscripts,
    SpecialUnicode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_search() {
        let strategy = ClipboardSearchStrategy;

        let entries = vec![
            ClipboardEntry {
                id: 1,
                content_type: ContentType::Text,
                content: "Hello world".to_string(),
                content_hash: "hash1".to_string(),
                created_at: 1000,
                last_used_at: None,
                is_pinned: false,
            },
            ClipboardEntry {
                id: 2,
                content_type: ContentType::Text,
                content: "Goodbye world".to_string(),
                content_hash: "hash2".to_string(),
                created_at: 1001,
                last_used_at: None,
                is_pinned: false,
            },
        ];

        let query = SearchQuery {
            query: "world".to_string(),
            whole_word: false,
            case_sensitive: false,
        };

        let results = strategy.search(&entries, &query);
        assert_eq!(results.matches.len(), 2);
    }

    #[test]
    fn test_symbol_search() {
        let strategy = SymbolSearchStrategy;

        let symbols = vec![
            Symbol {
                value: "α".to_string(),
                category: SymbolCategory::GreekAlphabet,
                description: "Greek alpha".to_string(),
            },
            Symbol {
                value: "∞".to_string(),
                category: SymbolCategory::Mathematical,
                description: "Infinity".to_string(),
            },
        ];

        let query = SearchQuery {
            query: "alpha".to_string(),
            whole_word: false,
            case_sensitive: false,
        };

        let results = strategy.search(&symbols, &query);
        assert_eq!(results.matches.len(), 1);
        assert_eq!(results.matches[0].value, "α");
    }

    #[test]
    fn test_symbol_search_mathematical() {
        let strategy = SymbolSearchStrategy;

        let symbols = vec![
            Symbol {
                value: "α".to_string(),
                category: SymbolCategory::GreekAlphabet,
                description: "Greek alpha".to_string(),
            },
            Symbol {
                value: "∞".to_string(),
                category: SymbolCategory::Mathematical,
                description: "Infinity".to_string(),
            },
        ];

        let query = SearchQuery {
            query: "mathematical".to_string(),
            whole_word: false,
            case_sensitive: false,
        };

        let results = strategy.search(&symbols, &query);
        assert_eq!(results.matches.len(), 1);
        assert_eq!(results.matches[0].value, "∞");
    }
}