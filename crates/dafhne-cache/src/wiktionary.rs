//! WiktionaryCache: dictionary cache backed by a pre-parsed JSON file.
//!
//! The JSON file is produced by `dafhne-wikt-build` from a Simple English
//! Wiktionary XML dump. Format: `HashMap<String, CacheEntry>` serialized
//! as JSON.

use std::collections::HashMap;
use std::path::Path;

use crate::cache_trait::{CacheEntry, DictionaryCache};

pub struct WiktionaryCache {
    entries: HashMap<String, CacheEntry>,
}

impl WiktionaryCache {
    /// Load from a pre-processed JSON cache file.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let entries: HashMap<String, CacheEntry> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self { entries })
    }

    /// Create an empty cache (useful for testing).
    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

impl DictionaryCache for WiktionaryCache {
    fn lookup(&self, word: &str) -> Option<CacheEntry> {
        self.entries.get(&word.to_lowercase()).cloned()
    }

    fn contains(&self, word: &str) -> bool {
        self.entries.contains_key(&word.to_lowercase())
    }

    fn name(&self) -> &str {
        "WiktionaryCache"
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}
