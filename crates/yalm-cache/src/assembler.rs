//! Dictionary Assembler: BFS closure chase from free text.
//!
//! Given free text and a `DictionaryCache` backend, the assembler:
//! 1. Extracts unique words from the text
//! 2. Looks each up in the cache
//! 3. Chases definition words (BFS) until closure or limits
//! 4. Produces a standard `Dictionary` (identical to dict5/12/18)
//!
//! The assembled dictionary feeds directly into the existing
//! equilibrium/force-field pipeline. The engine is untouched.

use std::collections::{HashMap, HashSet, VecDeque};

use yalm_core::{Dictionary, DictionaryEntry};
use yalm_parser::{stem_to_entry, tokenize};

use crate::cache_trait::{CacheEntry, DictionaryCache};
use crate::stop_words::stop_words;

// ─── Configuration ─────────────────────────────────────────────

/// Configuration for the assembly process.
#[derive(Debug, Clone)]
pub struct AssemblerConfig {
    /// Maximum BFS depth for closure chase (default: 3).
    pub max_depth: usize,
    /// Maximum number of words in the assembled dictionary (default: 5000).
    pub max_words: usize,
    /// Additional stop words beyond the default set.
    pub extra_stop_words: HashSet<String>,
}

impl Default for AssemblerConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_words: 5000,
            extra_stop_words: HashSet::new(),
        }
    }
}

// ─── Report ────────────────────────────────────────────────────

/// Assembly statistics report.
#[derive(Debug, Clone)]
pub struct AssemblyReport {
    /// Total unique non-stop words extracted from input text.
    pub seed_words: usize,
    /// Total words found in the cache.
    pub words_found: usize,
    /// Words not found in the cache.
    pub words_not_found: Vec<String>,
    /// Total entries in the assembled dictionary.
    pub total_entries: usize,
    /// Maximum BFS depth actually reached.
    pub depth_reached: usize,
    /// Fraction of entries whose definition words are all in the dictionary.
    pub closure_ratio: f64,
    /// Number of new words added at each BFS depth.
    pub words_per_depth: Vec<usize>,
}

// ─── Assembler ─────────────────────────────────────────────────

/// Assemble a `Dictionary` from free text using a cache backend.
pub struct DictionaryAssembler<'a> {
    cache: &'a dyn DictionaryCache,
    config: AssemblerConfig,
    stop_words: HashSet<String>,
}

impl<'a> DictionaryAssembler<'a> {
    pub fn new(cache: &'a dyn DictionaryCache, config: AssemblerConfig) -> Self {
        let mut sw = stop_words();
        sw.extend(config.extra_stop_words.iter().cloned());
        Self {
            cache,
            config,
            stop_words: sw,
        }
    }

    /// Assemble a dictionary from free text.
    pub fn assemble(&self, text: &str) -> (Dictionary, AssemblyReport) {
        // Phase 1: Extract seed words
        let tokens = tokenize(text);
        let mut seen_seeds = HashSet::new();
        let mut seed_words = Vec::new();
        for token in &tokens {
            let lower = token.to_lowercase();
            if !self.stop_words.contains(&lower) && seen_seeds.insert(lower.clone()) {
                seed_words.push(lower);
            }
        }
        let num_seeds = seed_words.len();

        // Phase 2: BFS closure chase
        let mut included: HashMap<String, CacheEntry> = HashMap::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut not_found: Vec<String> = Vec::new();
        let mut words_per_depth: Vec<usize> = Vec::new();
        let mut max_depth_reached: usize = 0;

        // Enqueue seeds at depth 0
        for word in &seed_words {
            visited.insert(word.clone());
            queue.push_back((word.clone(), 0));
        }

        while let Some((word, depth)) = queue.pop_front() {
            if included.len() >= self.config.max_words {
                break;
            }

            match self.cache.lookup(&word) {
                Some(entry) => {
                    // Track per-depth stats
                    while words_per_depth.len() <= depth {
                        words_per_depth.push(0);
                    }
                    words_per_depth[depth] += 1;
                    if depth > max_depth_reached {
                        max_depth_reached = depth;
                    }

                    // Chase definition words into the BFS frontier
                    if depth < self.config.max_depth {
                        self.enqueue_words_from(&entry, depth, &mut queue, &mut visited);
                    }

                    included.insert(word, entry);
                }
                None => {
                    not_found.push(word);
                }
            }
        }

        let words_found = included.len();

        // Phase 3: Build Dictionary
        let entries = self.build_entries(&included);
        let entry_words: Vec<String> = entries.iter().map(|e| e.word.clone()).collect();
        let entry_set: HashSet<String> = entry_words.iter().cloned().collect();

        // Phase 4: Compute closure ratio
        let closure_ratio = self.compute_closure_ratio(&entries, &entry_set);

        let dictionary = Dictionary {
            entries,
            entry_words,
            entry_set,
        };

        let report = AssemblyReport {
            seed_words: num_seeds,
            words_found,
            words_not_found: not_found,
            total_entries: dictionary.entries.len(),
            depth_reached: max_depth_reached,
            closure_ratio,
            words_per_depth,
        };

        (dictionary, report)
    }

    /// Enqueue definition and example words from a cache entry.
    fn enqueue_words_from(
        &self,
        entry: &CacheEntry,
        current_depth: usize,
        queue: &mut VecDeque<(String, usize)>,
        visited: &mut HashSet<String>,
    ) {
        let next_depth = current_depth + 1;

        // Chase definition words
        for def in &entry.definitions {
            for token in tokenize(def) {
                let lower = token.to_lowercase();
                if !self.stop_words.contains(&lower) && visited.insert(lower.clone()) {
                    queue.push_back((lower, next_depth));
                }
            }
        }

        // Chase example words (helps closure)
        for ex in &entry.examples {
            for token in tokenize(ex) {
                let lower = token.to_lowercase();
                if !self.stop_words.contains(&lower) && visited.insert(lower.clone()) {
                    queue.push_back((lower, next_depth));
                }
            }
        }
    }

    /// Convert collected CacheEntries into DictionaryEntries.
    fn build_entries(&self, included: &HashMap<String, CacheEntry>) -> Vec<DictionaryEntry> {
        let mut entries: Vec<DictionaryEntry> = included
            .iter()
            .map(|(word, cache_entry)| {
                // Pick first definition sense
                let definition = cache_entry
                    .definitions
                    .first()
                    .cloned()
                    .unwrap_or_else(|| format!("a {}", word));

                // Use available examples, pad to 3 with placeholders
                let examples = if cache_entry.examples.len() >= 3 {
                    cache_entry.examples[..3].to_vec()
                } else {
                    let mut exs = cache_entry.examples.clone();
                    while exs.len() < 3 {
                        exs.push(format!("{} is {}.", word, definition));
                    }
                    exs
                };

                DictionaryEntry {
                    word: word.clone(),
                    definition,
                    examples,
                    section: "assembled".to_string(),
                    is_entity: false,
                }
            })
            .collect();

        // Sort alphabetically for deterministic output
        entries.sort_by(|a, b| a.word.cmp(&b.word));
        entries
    }

    /// Compute what fraction of entries have fully-closed definitions.
    fn compute_closure_ratio(
        &self,
        entries: &[DictionaryEntry],
        entry_set: &HashSet<String>,
    ) -> f64 {
        if entries.is_empty() {
            return 0.0;
        }

        let mut closed_count = 0;
        for entry in entries {
            let tokens = tokenize(&entry.definition);
            let all_closed = tokens.iter().all(|t| {
                let lower = t.to_lowercase();
                self.stop_words.contains(&lower)
                    || entry_set.contains(&lower)
                    || stem_to_entry(&lower, entry_set).is_some()
            });
            if all_closed {
                closed_count += 1;
            }
        }

        closed_count as f64 / entries.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ManualFileCache;
    use std::path::PathBuf;

    fn load_dict5_cache() -> Option<ManualFileCache> {
        let path = PathBuf::from("../../dictionaries/dict5.md");
        if !path.exists() {
            return None;
        }
        Some(ManualFileCache::load(&path).unwrap())
    }

    #[test]
    fn assemble_simple_sentence() {
        let cache = match load_dict5_cache() {
            Some(c) => c,
            None => { eprintln!("Skipping: dict5.md not found"); return; }
        };

        let config = AssemblerConfig {
            max_depth: 2,
            max_words: 100,
            ..Default::default()
        };
        let assembler = DictionaryAssembler::new(&cache, config);
        let (dict, report) = assembler.assemble("a dog is an animal");

        println!("Seeds: {}, Found: {}, Not found: {:?}", report.seed_words, report.words_found, report.words_not_found);
        println!("Entries: {}, Closure: {:.1}%", report.total_entries, report.closure_ratio * 100.0);
        println!("Words: {:?}", dict.entry_words);

        // "dog" and "animal" should definitely be in the dict
        assert!(dict.entry_set.contains("dog"), "dog should be assembled");
        assert!(dict.entry_set.contains("animal"), "animal should be assembled");
        assert!(report.total_entries >= 2, "Should have at least dog + animal");
        assert!(report.closure_ratio > 0.0, "Closure ratio should be > 0");
    }

    #[test]
    fn max_words_cap() {
        let cache = match load_dict5_cache() {
            Some(c) => c,
            None => { eprintln!("Skipping: dict5.md not found"); return; }
        };

        let config = AssemblerConfig {
            max_depth: 10,
            max_words: 5,
            ..Default::default()
        };
        let assembler = DictionaryAssembler::new(&cache, config);
        let (dict, _report) = assembler.assemble("dog cat person animal ball sun water fire");

        assert!(dict.entries.len() <= 5, "Should cap at max_words=5, got {}", dict.entries.len());
    }

    #[test]
    fn depth_limit() {
        let cache = match load_dict5_cache() {
            Some(c) => c,
            None => { eprintln!("Skipping: dict5.md not found"); return; }
        };

        // Depth 0 = only text words, no chasing
        let config = AssemblerConfig {
            max_depth: 0,
            max_words: 5000,
            ..Default::default()
        };
        let assembler = DictionaryAssembler::new(&cache, config);
        let (dict_d0, _) = assembler.assemble("dog");

        // Depth 2 = chase definitions
        let config2 = AssemblerConfig {
            max_depth: 2,
            max_words: 5000,
            ..Default::default()
        };
        let assembler2 = DictionaryAssembler::new(&cache, config2);
        let (dict_d2, _) = assembler2.assemble("dog");

        // Deeper chase should find more words
        assert!(dict_d2.entries.len() > dict_d0.entries.len(),
            "Depth 2 ({}) should find more words than depth 0 ({})",
            dict_d2.entries.len(), dict_d0.entries.len());
    }
}
