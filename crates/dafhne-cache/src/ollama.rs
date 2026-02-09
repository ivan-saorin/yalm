//! OllamaCache: LLM-backed dictionary cache using a local Ollama instance.
//!
//! Calls Qwen3:8b (or any Ollama model) with a style prompt to generate
//! dict5-compatible definitions. Three-tier lookup: memory → disk → LLM.
//!
//! Disk memoization ensures each word is only generated once (~2-10s per call).
//! Second runs are instant.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::cache_trait::{CacheEntry, DictionaryCache};

// ─── Style Prompt ─────────────────────────────────────────────────

/// The style prompt that turns an LLM into a dict5-style definition factory.
/// Contains `{word}` placeholder for the word being defined.
const STYLE_PROMPT: &str = r#"You are a simple dictionary writer. Define the given word using ONLY basic English.

Rules:
- First sentence states the category: "a [category]." or "to [verb]." or "not [opposite]."
- Use patterns: "is a", "can", "not", "has", "part of", "makes"
- Maximum 3 short sentences
- Use the simplest words you know
- No examples, no etymology, no "such as", no parentheses
- Output ONLY the definition, nothing else

Examples:
dog: an animal. it can make sound and move fast.
sun: a big hot thing in the sky. it makes light.
cold: not hot. when something has little heat.
run: to move fast using legs.
water: a thing that is not hard. it can move and it is clear.
happy: a good feeling. when a person is not sad.
king: a person. a man that has power over a big place."#;

// ─── Ollama API Types ─────────────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    think: bool,
    options: ChatOptions,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatOptions {
    temperature: f64,
    num_predict: u32,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: Option<ChatResponseMessage>,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

// ─── OllamaCache ──────────────────────────────────────────────────

/// LLM-backed dictionary cache using a local Ollama instance.
///
/// Implements `DictionaryCache` using interior mutability (`RefCell`)
/// so that `lookup(&self, ...)` can update the in-memory and disk caches.
pub struct OllamaCache {
    /// Ollama API base URL (e.g., "http://localhost:11434").
    base_url: String,
    /// Model name (e.g., "qwen3:8b").
    model: String,
    /// In-memory cache (populated from disk + new lookups).
    memory: RefCell<HashMap<String, CacheEntry>>,
    /// Disk cache directory for persistence.
    disk_cache_dir: PathBuf,
    /// Stats: number of LLM calls made this session.
    llm_calls: RefCell<usize>,
    /// Stats: number of disk cache hits this session.
    disk_hits: RefCell<usize>,
    /// Stats: number of memory cache hits this session.
    memory_hits: RefCell<usize>,
    /// Stats: number of failures this session.
    failures: RefCell<usize>,
}

impl OllamaCache {
    /// Create a new OllamaCache.
    ///
    /// - `base_url`: Ollama API base URL (e.g., "http://localhost:11434")
    /// - `model`: Model name (e.g., "qwen3:8b")
    /// - `disk_cache_dir`: Directory for per-letter JSON files
    ///
    /// # Errors
    ///
    /// Returns an error if the disk cache directory cannot be created.
    pub fn new(base_url: &str, model: &str, disk_cache_dir: &Path) -> Result<Self, String> {
        // Create disk cache directory if it doesn't exist
        fs::create_dir_all(disk_cache_dir)
            .map_err(|e| format!("Cannot create disk cache dir {:?}: {}", disk_cache_dir, e))?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            memory: RefCell::new(HashMap::new()),
            disk_cache_dir: disk_cache_dir.to_path_buf(),
            llm_calls: RefCell::new(0),
            disk_hits: RefCell::new(0),
            memory_hits: RefCell::new(0),
            failures: RefCell::new(0),
        })
    }

    /// Check that Ollama is reachable and the model is available.
    /// Call this before starting a long BFS run to fail fast.
    pub fn check_health(&self) -> Result<(), String> {
        let url = format!("{}/api/tags", self.base_url);
        let mut resp = ureq::get(&url)
            .call()
            .map_err(|e| format!("Cannot reach Ollama at {}: {}", self.base_url, e))?;

        let body: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("Bad response from Ollama: {}", e))?;

        // Check if our model is in the list
        if let Some(models) = body.get("models").and_then(|m: &serde_json::Value| m.as_array()) {
            let model_names: Vec<&str> = models
                .iter()
                .filter_map(|m: &serde_json::Value| m.get("name").and_then(|n: &serde_json::Value| n.as_str()))
                .collect();

            // The model might be listed as "qwen3:8b" or "qwen3:8b-latest"
            let found = model_names.iter().any(|name: &&str| {
                name.starts_with(&self.model)
                    || self.model.starts_with(name.split(':').next().unwrap_or(""))
            });

            if !found {
                return Err(format!(
                    "Model '{}' not found. Available: {:?}",
                    self.model, model_names
                ));
            }
        }

        Ok(())
    }

    /// Print session statistics.
    pub fn print_stats(&self) {
        let total = *self.memory_hits.borrow()
            + *self.disk_hits.borrow()
            + *self.llm_calls.borrow()
            + *self.failures.borrow();
        eprintln!(
            "[OllamaCache] Session stats: {} total lookups — {} memory, {} disk, {} LLM calls, {} failed",
            total,
            self.memory_hits.borrow(),
            self.disk_hits.borrow(),
            self.llm_calls.borrow(),
            self.failures.borrow(),
        );
    }

    // ── Disk memoization ──────────────────────────────────────────

    /// Path to the per-letter JSON file for a given word.
    fn disk_path(&self, word: &str) -> PathBuf {
        let letter = word
            .chars()
            .next()
            .unwrap_or('_')
            .to_lowercase()
            .next()
            .unwrap_or('_');
        let filename = if letter.is_ascii_alphabetic() {
            format!("{}.json", letter)
        } else {
            "other.json".to_string()
        };
        self.disk_cache_dir.join(filename)
    }

    /// Load a single word from the disk cache.
    fn load_from_disk(&self, word: &str) -> Option<CacheEntry> {
        let path = self.disk_path(word);
        if !path.exists() {
            return None;
        }
        let file = File::open(&path).ok()?;
        let reader = BufReader::new(file);
        let data: HashMap<String, CacheEntry> = serde_json::from_reader(reader).ok()?;
        data.get(word).cloned()
    }

    /// Save a single word to the disk cache (read-modify-write the letter file).
    fn save_to_disk(&self, word: &str, entry: &CacheEntry) {
        let path = self.disk_path(word);
        let mut data: HashMap<String, CacheEntry> = if path.exists() {
            File::open(&path)
                .ok()
                .and_then(|f| serde_json::from_reader(BufReader::new(f)).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };
        data.insert(word.to_string(), entry.clone());
        if let Ok(file) = File::create(&path) {
            let _ = serde_json::to_writer_pretty(file, &data);
        }
    }

    // ── LLM call ──────────────────────────────────────────────────

    /// Call Ollama chat API to generate a definition.
    fn call_ollama(&self, word: &str) -> Result<CacheEntry, String> {
        let url = format!("{}/api/chat", self.base_url);

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: STYLE_PROMPT.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: word.to_string(),
                },
            ],
            stream: false,
            think: false,
            options: ChatOptions {
                temperature: 0.3,
                num_predict: 100,
            },
        };

        let mut resp = ureq::post(&url)
            .send_json(&request)
            .map_err(|e| format!("Ollama API error: {}", e))?;

        let chat_resp: ChatResponse = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let raw_content = chat_resp
            .message
            .map(|m| m.content)
            .unwrap_or_default();

        if raw_content.trim().is_empty() {
            return Err(format!("Empty response for '{}'", word));
        }

        Ok(self.parse_response(word, &raw_content))
    }

    // ── Response parsing ──────────────────────────────────────────

    /// Parse and clean the LLM response into a CacheEntry.
    fn parse_response(&self, word: &str, raw: &str) -> CacheEntry {
        let text = raw.trim();

        // Strip the word prefix if the model echoes it: "dog: an animal..." → "an animal..."
        let definition = if let Some(rest) = text.strip_prefix(&format!("{}:", word)) {
            rest.trim().to_string()
        } else if let Some(rest) = text.strip_prefix(&format!("{}:", &capitalize(word))) {
            rest.trim().to_string()
        } else {
            text.to_string()
        };

        // Strip any markdown, quotes, or thinking artifacts
        let definition = definition
            .replace("```", "")
            .replace('"', "")
            .replace('*', "")
            .trim()
            .to_string();

        // Truncate to max 3 sentences
        let definition = truncate_sentences(&definition, 3);

        CacheEntry {
            word: word.to_string(),
            definitions: vec![definition],
            examples: vec![],
        }
    }

    /// Pre-load all entries from disk cache into memory.
    /// Useful for second runs where disk is fully populated.
    pub fn preload_disk_cache(&self) {
        let mut loaded = 0usize;
        if let Ok(entries) = fs::read_dir(&self.disk_cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(file) = File::open(&path) {
                        if let Ok(data) =
                            serde_json::from_reader::<_, HashMap<String, CacheEntry>>(
                                BufReader::new(file),
                            )
                        {
                            let count = data.len();
                            self.memory.borrow_mut().extend(data);
                            loaded += count;
                        }
                    }
                }
            }
        }
        if loaded > 0 {
            eprintln!("[OllamaCache] Preloaded {} entries from disk cache", loaded);
        }
    }
}

impl DictionaryCache for OllamaCache {
    fn lookup(&self, word: &str) -> Option<CacheEntry> {
        let normalized = word.to_lowercase().trim().to_string();

        // 1. Check in-memory cache
        if let Some(entry) = self.memory.borrow().get(&normalized).cloned() {
            *self.memory_hits.borrow_mut() += 1;
            return Some(entry);
        }

        // 2. Check disk cache
        if let Some(entry) = self.load_from_disk(&normalized) {
            *self.disk_hits.borrow_mut() += 1;
            self.memory
                .borrow_mut()
                .insert(normalized.clone(), entry.clone());
            return Some(entry);
        }

        // 3. Call Ollama
        let start = Instant::now();
        match self.call_ollama(&normalized) {
            Ok(entry) => {
                let elapsed = start.elapsed();
                *self.llm_calls.borrow_mut() += 1;
                let call_count = *self.llm_calls.borrow();
                eprintln!(
                    "[OllamaCache] Generated: {} ({}) ... {:.1}s — \"{}\"",
                    normalized,
                    call_count,
                    elapsed.as_secs_f64(),
                    entry.definitions.first().map(|d| truncate_display(d, 60)).unwrap_or_default(),
                );
                self.save_to_disk(&normalized, &entry);
                self.memory
                    .borrow_mut()
                    .insert(normalized, entry.clone());
                Some(entry)
            }
            Err(e) => {
                *self.failures.borrow_mut() += 1;
                eprintln!("[OllamaCache] FAILED for '{}': {}", word, e);
                None
            }
        }
    }

    fn contains(&self, _word: &str) -> bool {
        // For LLM cache, assume any word CAN be defined.
        // Return true always — let lookup() handle failures.
        true
    }

    fn name(&self) -> &str {
        "ollama-qwen3"
    }

    fn len(&self) -> usize {
        self.memory.borrow().len()
    }
}

// ─── Helpers ──────────────────────────────────────────────────────

/// Capitalize first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Truncate text to at most `n` sentences (split on ". " or terminal ".").
fn truncate_sentences(text: &str, max: usize) -> String {
    let mut result = String::new();
    let mut count = 0;

    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        result.push(ch);
        if ch == '.' {
            count += 1;
            if count >= max {
                break;
            }
        }
    }

    result.trim().to_string()
}

/// Truncate a string for display purposes.
fn truncate_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_response_strips_word_prefix() {
        let cache = OllamaCache {
            base_url: "http://localhost:11434".to_string(),
            model: "qwen3:8b".to_string(),
            memory: RefCell::new(HashMap::new()),
            disk_cache_dir: PathBuf::from("/tmp/test-ollama-cache"),
            llm_calls: RefCell::new(0),
            disk_hits: RefCell::new(0),
            memory_hits: RefCell::new(0),
            failures: RefCell::new(0),
        };

        // Should strip "dog: " prefix
        let entry = cache.parse_response("dog", "dog: an animal. it can make sound.");
        assert_eq!(entry.definitions[0], "an animal. it can make sound.");

        // Should handle no prefix
        let entry = cache.parse_response("cat", "a small animal. it can move fast.");
        assert_eq!(entry.definitions[0], "a small animal. it can move fast.");

        // Should strip capitalized prefix
        let entry = cache.parse_response("happy", "Happy: a good feeling. not sad.");
        assert_eq!(entry.definitions[0], "a good feeling. not sad.");
    }

    #[test]
    fn truncate_sentences_works() {
        assert_eq!(
            truncate_sentences("one. two. three. four.", 3),
            "one. two. three."
        );
        assert_eq!(truncate_sentences("single sentence.", 3), "single sentence.");
        assert_eq!(
            truncate_sentences("a. b. c. d. e.", 2),
            "a. b."
        );
    }

    #[test]
    fn parse_response_strips_markdown() {
        let cache = OllamaCache {
            base_url: "http://localhost:11434".to_string(),
            model: "test".to_string(),
            memory: RefCell::new(HashMap::new()),
            disk_cache_dir: PathBuf::from("/tmp/test-ollama-cache"),
            llm_calls: RefCell::new(0),
            disk_hits: RefCell::new(0),
            memory_hits: RefCell::new(0),
            failures: RefCell::new(0),
        };

        let entry = cache.parse_response("test", "```a thing.```");
        assert_eq!(entry.definitions[0], "a thing.");

        let entry = cache.parse_response("test", "\"a thing.\"");
        assert_eq!(entry.definitions[0], "a thing.");
    }

    #[test]
    fn disk_path_construction() {
        let cache = OllamaCache {
            base_url: "http://localhost:11434".to_string(),
            model: "test".to_string(),
            memory: RefCell::new(HashMap::new()),
            disk_cache_dir: PathBuf::from("/tmp/cache"),
            llm_calls: RefCell::new(0),
            disk_hits: RefCell::new(0),
            memory_hits: RefCell::new(0),
            failures: RefCell::new(0),
        };

        assert_eq!(cache.disk_path("dog"), PathBuf::from("/tmp/cache/d.json"));
        assert_eq!(cache.disk_path("animal"), PathBuf::from("/tmp/cache/a.json"));
        assert_eq!(cache.disk_path("123"), PathBuf::from("/tmp/cache/other.json"));
    }
}
