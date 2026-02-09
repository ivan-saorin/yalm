//! Minimal hardcoded stop words for the BFS closure chase.
//!
//! These high-frequency function words are skipped during assembly
//! to prevent the BFS frontier from exploding. They add no semantic
//! value — DAFHNE discovers them as connectors during training anyway.
//!
//! We hardcode rather than derive from connector discovery because
//! the stop words are needed BEFORE the dictionary exists.

use std::collections::HashSet;

/// Returns ~55 English function words.
pub fn stop_words() -> HashSet<String> {
    [
        // articles & determiners
        "the", "a", "an", "this", "that", "these", "those",
        // be-verbs
        "is", "are", "was", "were", "be", "been", "being", "am",
        // auxiliaries
        "have", "has", "had", "do", "does", "did",
        // modals
        "will", "would", "shall", "should", "may", "might", "can", "could", "must",
        // prepositions
        "to", "of", "in", "for", "on", "with", "at", "by", "from", "into", "about",
        // conjunctions & negation
        "and", "or", "but", "not", "no", "if", "then", "than", "so", "as",
        // pronouns
        "i", "you", "he", "she", "it", "we", "they",
        "me", "him", "her", "us", "them",
        "my", "your", "his", "our", "their", "its",
        // interrogatives
        "who", "what", "which", "when", "where", "how", "why",
        // adverbs
        "very", "also", "just", "too", "more", "most",
        // single-letter noise (variable names in dictionary defs, letters)
        "x", "y", "e", "s",
        // common verbs that act as function words
        "get", "got", "make", "made", "go", "went", "gone",
        "come", "came", "take", "took", "taken", "give", "gave", "given",
        // other high-frequency structural words
        "one", "some", "any", "all", "each", "every", "other", "another",
        "such", "like", "only", "own", "same", "new", "old",
        "many", "much", "few", "several",
        "there", "here", "now",
        "up", "out", "off", "over", "under", "between", "through", "after", "before",
        "something", "someone", "anything", "anyone", "nothing",
        "said", "says", "say",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_words_contain_basics() {
        let sw = stop_words();
        assert!(sw.contains("the"));
        assert!(sw.contains("is"));
        assert!(sw.contains("a"));
        assert!(sw.contains("not"));
        assert!(!sw.contains("dog"));
        assert!(!sw.contains("animal"));
    }
}
