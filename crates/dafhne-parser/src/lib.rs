pub mod dictionary;
pub mod questions;

pub use dictionary::parse_dictionary;
pub use dictionary::parse_grammar_text;
pub use questions::parse_test_questions;

use std::collections::HashSet;

/// Tokenize text: lowercase, split on whitespace/punctuation, strip non-alphanumeric.
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| c.is_whitespace() || matches!(c, '.' | ',' | '?' | '!' | '"' | ';' | ':' | '(' | ')' | '\u{201C}' | '\u{201D}'))
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Try to reduce an inflected token to its base entry word.
/// Returns Some(entry_word) if found, None otherwise.
pub fn stem_to_entry(token: &str, entry_set: &HashSet<String>) -> Option<String> {
    let lower = token.to_lowercase();

    // Direct match
    if entry_set.contains(&lower) {
        return Some(lower);
    }

    // Special case: "an" -> "a"
    if lower == "an" {
        return Some("a".to_string());
    }

    // Try removing common suffixes (ordered longest first to avoid partial strips)
    let suffixes = ["iest", "ier", "ing", "est", "er", "ly", "es", "ed", "s"];
    for suffix in &suffixes {
        if let Some(stem) = lower.strip_suffix(suffix) {
            if !stem.is_empty() && entry_set.contains(stem) {
                return Some(stem.to_string());
            }
            // Handle "e" restoration: "living" -> "liv" -> "live"
            let with_e = format!("{}e", stem);
            if entry_set.contains(&with_e) {
                return Some(with_e);
            }
            // Handle consonant doubling: "bigger" -> "bigg" -> "big"
            let bytes = stem.as_bytes();
            if bytes.len() >= 2 && bytes[bytes.len() - 1] == bytes[bytes.len() - 2] {
                let undoubled = &stem[..stem.len() - 1];
                if !undoubled.is_empty() && entry_set.contains(undoubled) {
                    return Some(undoubled.to_string());
                }
            }
            // Handle y -> i transformation: "happiest" -> "happi" -> "happy"
            if stem.ends_with('i') {
                let with_y = format!("{}y", &stem[..stem.len() - 1]);
                if entry_set.contains(&with_y) {
                    return Some(with_y);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("a dog is an animal.");
        assert_eq!(tokens, vec!["a", "dog", "is", "an", "animal"]);
    }

    #[test]
    fn test_tokenize_question() {
        let tokens = tokenize("Is a dog an animal?");
        assert_eq!(tokens, vec!["is", "a", "dog", "an", "animal"]);
    }

    #[test]
    fn test_tokenize_quoted() {
        let tokens = tokenize("\"the dog eats food\"");
        assert_eq!(tokens, vec!["the", "dog", "eats", "food"]);
    }

    #[test]
    fn test_stem_to_entry() {
        let mut set = HashSet::new();
        for w in &["eat", "dog", "a", "move", "live", "make", "see", "feel", "give"] {
            set.insert(w.to_string());
        }

        assert_eq!(stem_to_entry("eats", &set), Some("eat".to_string()));
        assert_eq!(stem_to_entry("eating", &set), Some("eat".to_string()));
        assert_eq!(stem_to_entry("moves", &set), Some("move".to_string()));
        assert_eq!(stem_to_entry("lives", &set), Some("live".to_string()));
        assert_eq!(stem_to_entry("living", &set), Some("live".to_string()));
        assert_eq!(stem_to_entry("makes", &set), Some("make".to_string()));
        assert_eq!(stem_to_entry("an", &set), Some("a".to_string()));
        assert_eq!(stem_to_entry("dog", &set), Some("dog".to_string()));
        assert_eq!(stem_to_entry("sees", &set), Some("see".to_string()));
        assert_eq!(stem_to_entry("xyz", &set), None);
    }

    #[test]
    fn test_stem_comparative_superlative() {
        let mut set = HashSet::new();
        for w in &["big", "large", "quick", "happy", "easy", "small", "fast", "hot", "sad"] {
            set.insert(w.to_string());
        }

        // -er comparative
        assert_eq!(stem_to_entry("bigger", &set), Some("big".to_string()));
        assert_eq!(stem_to_entry("larger", &set), Some("large".to_string()));
        assert_eq!(stem_to_entry("quicker", &set), Some("quick".to_string()));
        assert_eq!(stem_to_entry("faster", &set), Some("fast".to_string()));
        assert_eq!(stem_to_entry("smaller", &set), Some("small".to_string()));
        assert_eq!(stem_to_entry("hotter", &set), Some("hot".to_string()));

        // -est superlative
        assert_eq!(stem_to_entry("biggest", &set), Some("big".to_string()));
        assert_eq!(stem_to_entry("largest", &set), Some("large".to_string()));
        assert_eq!(stem_to_entry("quickest", &set), Some("quick".to_string()));
        assert_eq!(stem_to_entry("saddest", &set), Some("sad".to_string()));

        // -ly adverb
        assert_eq!(stem_to_entry("quickly", &set), Some("quick".to_string()));

        // -ier / -iest (y -> i transformation)
        assert_eq!(stem_to_entry("happier", &set), Some("happy".to_string()));
        assert_eq!(stem_to_entry("happiest", &set), Some("happy".to_string()));
        assert_eq!(stem_to_entry("easier", &set), Some("easy".to_string()));
        assert_eq!(stem_to_entry("easiest", &set), Some("easy".to_string()));
    }
}
