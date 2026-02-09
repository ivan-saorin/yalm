//! ManualFileCache: dictionary cache backed by hand-written .md files.
//!
//! Reuses the existing DAFHNE dictionary format:
//! ```text
//! **word** — definition text.
//! - "example one"
//! - "example two"
//! ```
//!
//! Accepts a single .md file or a directory of .md files.

use std::collections::HashMap;
use std::path::Path;

use dafhne_parser::parse_dictionary;

use crate::cache_trait::{CacheEntry, DictionaryCache};

pub struct ManualFileCache {
    entries: HashMap<String, CacheEntry>,
}

impl ManualFileCache {
    /// Load from a single .md file or a directory of .md files.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let mut entries = HashMap::new();

        let files = if path.is_dir() {
            let mut paths: Vec<_> = std::fs::read_dir(path)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map_or(false, |ext| ext == "md"))
                .collect();
            paths.sort();
            paths
        } else {
            vec![path.to_path_buf()]
        };

        for file_path in files {
            let content = std::fs::read_to_string(&file_path)?;
            let dict = parse_dictionary(&content);
            for entry in dict.entries {
                entries.insert(
                    entry.word.clone(),
                    CacheEntry {
                        word: entry.word,
                        definitions: vec![entry.definition],
                        examples: entry.examples,
                    },
                );
            }
        }

        Ok(Self { entries })
    }
}

impl DictionaryCache for ManualFileCache {
    fn lookup(&self, word: &str) -> Option<CacheEntry> {
        self.entries.get(&word.to_lowercase()).cloned()
    }

    fn contains(&self, word: &str) -> bool {
        self.entries.contains_key(&word.to_lowercase())
    }

    fn name(&self) -> &str {
        "ManualFileCache"
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn load_dict5() {
        let path = PathBuf::from("../../dictionaries/dict5.md");
        if !path.exists() {
            eprintln!("Skipping test: dict5.md not found at {:?}", path);
            return;
        }
        let cache = ManualFileCache::load(&path).unwrap();
        assert!(cache.len() > 40, "Expected 51 entries, got {}", cache.len());

        // Check a known entry
        let dog = cache.lookup("dog").expect("dog should be in cache");
        assert!(!dog.definitions.is_empty());
        assert!(dog.definitions[0].contains("animal"));
        assert!(!dog.examples.is_empty());

        // Check case insensitivity
        assert!(cache.contains("Dog"));
        assert!(cache.contains("DOG"));
    }
}
