use std::collections::HashMap;

/// Structure representing a Unicode symbol
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// The actual symbol character
    pub value: String,
    /// Category the symbol belongs to
    pub category: SymbolCategory,
    /// Unicode code point description
    pub description: String,
    /// Whether the symbol is a favorite
    pub is_favorite: bool,
}

/// Symbol category enum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl Symbol {
    /// Create a new symbol
    pub fn new(value: String, category: SymbolCategory, description: String) -> Self {
        Self {
            value,
            category,
            description,
            is_favorite: false,
        }
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
    }

    /// Check if symbol is favorite
    pub fn is_favorite(&self) -> bool {
        self.is_favorite
    }
}

/// Symbol library containing a collection of symbols
#[derive(Debug, Clone)]
pub struct SymbolLibrary {
    /// All symbols indexed by value
    symbols: HashMap<String, Symbol>,
    /// Symbols indexed by category
    by_category: HashMap<SymbolCategory, Vec<Symbol>>,
}

impl SymbolLibrary {
    /// Create a new empty library
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            by_category: HashMap::new(),
        }
    }

    /// Add a symbol to the library
    pub fn add(&mut self, symbol: Symbol) {
        let value_key = symbol.value.clone();
        self.symbols.insert(value_key.clone(), symbol.clone());

        self.by_category
            .entry(symbol.category.clone())
            .or_default()
            .push(symbol);
    }

    /// Get a symbol by value
    pub fn get(&self, value: &str) -> Option<&Symbol> {
        self.symbols.get(value)
    }

    /// Get symbols by category
    pub fn get_by_category(&self, category: &SymbolCategory) -> Option<&Vec<Symbol>> {
        self.by_category.get(category)
    }

    /// Search symbols by query
    pub fn search(&self, query: &str) -> Vec<&Symbol> {
        self.symbols
            .values()
            .filter(|s| {
                s.value.to_lowercase().contains(&query.to_lowercase())
                    || s.description.to_lowercase().contains(&query.to_lowercase())
                    || format!("{:?}", s.category).to_lowercase().contains(&query.to_lowercase())
            })
            .collect()
    }

    /// Get all categories
    pub fn categories(&self) -> Vec<&SymbolCategory> {
        self.by_category.keys().collect()
    }

    /// Get all symbols
    pub fn all(&self) -> Vec<&Symbol> {
        self.symbols.values().collect()
    }

    /// Toggle favorite for a symbol
    pub fn toggle_favorite(&mut self, value: &str) -> Option<()> {
        self.symbols.get_mut(value).map(|s| s.toggle_favorite())
    }

    /// Check if symbol exists
    pub fn contains(&self, value: &str) -> bool {
        self.symbols.contains_key(value)
    }
}

/// Create a common symbol library with frequently used symbols
pub fn create_common_symbol_library() -> SymbolLibrary {
    let mut library = SymbolLibrary::new();

    // Mathematical symbols
    library.add(Symbol::new(
        "∞".to_string(),
        SymbolCategory::Mathematical,
        "Infinity".to_string(),
    ));

    library.add(Symbol::new(
        "≠".to_string(),
        SymbolCategory::Mathematical,
        "Not equal".to_string(),
    ));

    library.add(Symbol::new(
        "≥".to_string(),
        SymbolCategory::Mathematical,
        "Greater than or equal".to_string(),
    ));

    library.add(Symbol::new(
        "≤".to_string(),
        SymbolCategory::Mathematical,
        "Less than or equal".to_string(),
    ));

    library.add(Symbol::new(
        "∑".to_string(),
        SymbolCategory::Mathematical,
        "Sum".to_string(),
    ));

    library.add(Symbol::new(
        "√".to_string(),
        SymbolCategory::Mathematical,
        "Square root".to_string(),
    ));

    // Currency symbols
    library.add(Symbol::new(
        "€".to_string(),
        SymbolCategory::Currency,
        "Euro".to_string(),
    ));

    library.add(Symbol::new(
        "£".to_string(),
        SymbolCategory::Currency,
        "Pound sterling".to_string(),
    ));

    library.add(Symbol::new(
        "¥".to_string(),
        SymbolCategory::Currency,
        "Japanese yen".to_string(),
    ));

    // Arrows
    library.add(Symbol::new(
        "→".to_string(),
        SymbolCategory::Arrows,
        "Right arrow".to_string(),
    ));

    library.add(Symbol::new(
        "←".to_string(),
        SymbolCategory::Arrows,
        "Left arrow".to_string(),
    ));

    library.add(Symbol::new(
        "↑".to_string(),
        SymbolCategory::Arrows,
        "Up arrow".to_string(),
    ));

    library.add(Symbol::new(
        "↓".to_string(),
        SymbolCategory::Arrows,
        "Down arrow".to_string(),
    ));

    // Greek alphabet
    library.add(Symbol::new(
        "α".to_string(),
        SymbolCategory::GreekAlphabet,
        "Alpha".to_string(),
    ));

    library.add(Symbol::new(
        "β".to_string(),
        SymbolCategory::GreekAlphabet,
        "Beta".to_string(),
    ));

    library.add(Symbol::new(
        "γ".to_string(),
        SymbolCategory::GreekAlphabet,
        "Gamma".to_string(),
    ));

    // Technical symbols
    library.add(Symbol::new(
        "©".to_string(),
        SymbolCategory::Technical,
        "Copyright".to_string(),
    ));

    library.add(Symbol::new(
        "®".to_string(),
        SymbolCategory::Technical,
        "Registered".to_string(),
    ));

    library.add(Symbol::new(
        "™".to_string(),
        SymbolCategory::Technical,
        "Trademark".to_string(),
    ));

    // Punctuation
    library.add(Symbol::new(
        "…".to_string(),
        SymbolCategory::Punctuation,
        "Ellipsis".to_string(),
    ));

    // Superscripts
    library.add(Symbol::new(
        "²".to_string(),
        SymbolCategory::Superscripts,
        "Superscript 2".to_string(),
    ));

    library.add(Symbol::new(
        "³".to_string(),
        SymbolCategory::Superscripts,
        "Superscript 3".to_string(),
    ));

    // Subscripts
    library.add(Symbol::new(
        "₁".to_string(),
        SymbolCategory::Subscripts,
        "Subscript 1".to_string(),
    ));

    library.add(Symbol::new(
        "₂".to_string(),
        SymbolCategory::Subscripts,
        "Subscript 2".to_string(),
    ));

    library
}

/// Common symbol library
/// Get the common symbol library
pub fn get_common_symbol_library() -> SymbolLibrary {
    create_common_symbol_library()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new("α".to_string(), SymbolCategory::GreekAlphabet, "Alpha".to_string());
        assert_eq!(symbol.value, "α");
        assert_eq!(symbol.category, SymbolCategory::GreekAlphabet);
        assert_eq!(symbol.description, "Alpha");
        assert!(!symbol.is_favorite());
    }

    #[test]
    fn test_symbol_toggle_favorite() {
        let mut symbol = Symbol::new("α".to_string(), SymbolCategory::GreekAlphabet, "Alpha".to_string());
        assert!(!symbol.is_favorite());
        symbol.toggle_favorite();
        assert!(symbol.is_favorite());
        symbol.toggle_favorite();
        assert!(!symbol.is_favorite());
    }

    #[test]
    fn test_library_search() {
        let library = create_common_symbol_library();
        let results = library.search("arrow");
        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.value == "→"));
    }

    #[test]
    fn test_library_get_by_category() {
        let library = create_common_symbol_library();
        let categories = library.categories();
        assert!(!categories.is_empty());
        assert!(categories.contains(&&SymbolCategory::Mathematical));
    }

    #[test]
    fn test_symbol_contains() {
        let library = create_common_symbol_library();
        assert!(library.contains("∞"));
        assert!(!library.contains("nonexistent"));
    }
}