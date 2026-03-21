use crate::types::FileResult;

/// Trait for language-specific parsers (D-21 trait-based adapter)
pub trait LanguageAdapter: Send + Sync {
    /// Human-readable language name
    fn language_name(&self) -> &str;

    /// File extensions this adapter handles (e.g., ["ts", "tsx", "js", "jsx"])
    fn file_extensions(&self) -> &[&str];

    /// Parse source code and extract structured facts
    fn parse(&self, source: &str, file_path: &str) -> FileResult;
}

/// Registry of all available language adapters
pub struct AdapterRegistry {
    adapters: Vec<Box<dyn LanguageAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    pub fn register(&mut self, adapter: Box<dyn LanguageAdapter>) {
        self.adapters.push(adapter);
    }

    pub fn find_adapter(&self, file_path: &str) -> Option<&dyn LanguageAdapter> {
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        self.adapters.iter().find_map(|a| {
            if a.file_extensions().contains(&ext) {
                Some(a.as_ref())
            } else {
                None
            }
        })
    }

    pub fn adapters(&self) -> &[Box<dyn LanguageAdapter>] {
        &self.adapters
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
