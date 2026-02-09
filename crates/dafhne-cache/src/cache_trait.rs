//! Pluggable dictionary cache trait.
//!
//! Any word-definition backend implements `DictionaryCache`.
//! Current implementations: ManualFileCache, WiktionaryCache.
//! Future: LLM-backed cache, WordNet, Oxford API, etc.

use serde::{Deserialize, Serialize};

/// A single cached definition entry. May contain multiple senses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The canonical word form (lowercase).
    pub word: String,
    /// Definition senses, ordered by primacy.
    pub definitions: Vec<String>,
    /// Example sentences (may be empty for some sources).
    pub examples: Vec<String>,
}

/// Pluggable dictionary backend trait.
///
/// Implementations provide word lookups from different sources.
/// The trait is object-safe and uses `&self` (sync).
/// Future backends needing mutation (e.g., caching API responses)
/// can use interior mutability (`RefCell` or `Mutex`).
///
/// # Implementing a new backend
///
/// ```ignore
/// struct MyCache { /* ... */ }
///
/// impl DictionaryCache for MyCache {
///     fn lookup(&self, word: &str) -> Option<CacheEntry> { /* ... */ }
///     fn contains(&self, word: &str) -> bool { /* ... */ }
///     fn name(&self) -> &str { "MyCache" }
///     fn len(&self) -> usize { /* ... */ }
/// }
/// ```
pub trait DictionaryCache {
    /// Look up a word. Returns `None` if the word is not in the cache.
    fn lookup(&self, word: &str) -> Option<CacheEntry>;

    /// Check if a word exists without loading the full entry.
    fn contains(&self, word: &str) -> bool;

    /// Human-readable name of this cache backend (for logging/reports).
    fn name(&self) -> &str;

    /// Total number of entries in the cache (0 if unknown).
    fn len(&self) -> usize;

    /// Whether the cache is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
