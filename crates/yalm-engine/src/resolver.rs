use std::collections::HashSet;
use yalm_core::*;
use yalm_parser::{stem_to_entry, tokenize};

use crate::strategy::{NegationModel, StrategyConfig};

// ─── Definition-Chain Negation (Fix 1) ────────────────────────

/// Check if `subject` is definitionally linked to `object` by traversing dictionary definitions.
///
/// Returns:
/// - `Some(true)` — subject's definition chain reaches object (definitionally linked)
/// - `Some(false)` — subject's definition chain explicitly negates object (antonym found)
/// - `None` — can't determine from definitions (chain doesn't reach object)
///
/// The chain traversal follows content words in definitions up to `max_hops` deep.
/// At each hop, it checks for direct mention and "not {object}" antonym patterns.
fn definition_chain_check(
    subject: &str,
    object: &str,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    max_hops: usize,
    visited: &mut HashSet<String>,
    space: &GeometricSpace,
) -> Option<bool> {
    // Avoid infinite loops
    if visited.contains(subject) {
        return None;
    }
    visited.insert(subject.to_string());

    // Find subject's definition in the dictionary
    let subject_entry = dictionary.entries.iter().find(|e| e.word == subject)?;
    // Filter out example sentences (those containing quote marks) from the definition.
    // In dict5 format, examples are quoted: '"dog" is the name of an animal.'
    // We keep only definitional sentences to avoid false positive chain matches.
    let def_text: String = subject_entry.definition
        .split('.')
        .filter(|s| !s.contains('"') && !s.contains('\u{201C}') && !s.contains('\u{201D}'))
        .collect::<Vec<_>>()
        .join(".");
    let def_words = tokenize(&def_text);

    // Direct check: does object appear in subject's definition?
    if def_words.iter().any(|w| {
        stem_to_entry(w, &dictionary.entry_set)
            .map_or(false, |stemmed| stemmed == object)
    }) {
        // Check for negation: "not {object}" pattern
        if preceded_by_not(&def_words, object, &dictionary.entry_set) {
            return Some(false); // definitionally negated (e.g., "not cold")
        }

        return Some(true); // definitionally linked (e.g., "an animal")
    }

    // One-hop check: follow content words from the FIRST SENTENCE of the definition only.
    // The first sentence contains the core definition: "an animal", "a big hot thing that is up".
    // Subsequent sentences are elaborations ("it can make sound. it can live with a person.")
    // which introduce diverse vocabulary that causes false positive chain matches.
    //
    // IMPORTANT: Only follow the first 3 content words per hop to avoid search explosion.
    // In large dictionaries (1000+ entries), following ALL content words from long
    // definitions (20+ words) causes nearly everything to connect to everything.
    // The first 3 content words capture the core semantic category:
    //   "a small domestic mammal with soft fur..." → follows: small, domestic, mammal
    //   "an animal. it can make sound." → follows: animal (only 1 content word)
    const MAX_FOLLOW_PER_HOP: usize = 3;

    if max_hops > 0 {
        let first_sentence = def_text.split('.').next().unwrap_or(&def_text);
        let first_words = tokenize(first_sentence);
        let mut followed = 0;
        for word in &first_words {
            if followed >= MAX_FOLLOW_PER_HOP {
                break;
            }
            let stemmed = match stem_to_entry(word, &dictionary.entry_set) {
                Some(s) => s,
                None => continue,
            };
            if structural.contains(&stemmed) || stemmed == subject {
                continue;
            }
            if !dictionary.entry_set.contains(&stemmed) {
                continue;
            }
            followed += 1;

            if let Some(result) = definition_chain_check(
                &stemmed, object, dictionary, structural,
                max_hops - 1, visited, space,
            ) {
                return Some(result);
            }
        }
    }

    None // can't determine from definitions
}

/// Check if `target` is preceded by "not" in the word list.
/// Handles stemming: looks for stemmed forms of each word matching target.
/// Also handles "not a {target}" patterns where articles intervene.
fn preceded_by_not(words: &[String], target: &str, entry_set: &HashSet<String>) -> bool {
    let articles: HashSet<&str> = ["a", "an", "the"].iter().copied().collect();
    for (i, word) in words.iter().enumerate() {
        let stemmed = stem_to_entry(word, entry_set).unwrap_or_else(|| word.clone());
        if stemmed == target && i > 0 {
            // Check immediate predecessor
            let prev = stem_to_entry(&words[i - 1], entry_set)
                .unwrap_or_else(|| words[i - 1].clone());
            if prev == "not" {
                return true;
            }
            // Check two positions back (skipping articles: "not a {target}")
            if i > 1 && articles.contains(words[i - 1].as_str()) {
                let prev2 = stem_to_entry(&words[i - 2], entry_set)
                    .unwrap_or_else(|| words[i - 2].clone());
                if prev2 == "not" {
                    return true;
                }
            }
        }
    }
    false
}

// ─── Definition-Category Extraction (Fix 2 fallback) ─────────

/// Extract the category word from a subject's definition.
/// Returns the first content word in the definition that is itself a dictionary entry
/// and is NOT a function word (not appearing in any connector pattern).
///
/// For "What is a dog?" → dog's definition: "an animal. it can make sound..."
/// → first content word that's a dict entry = "animal" → return "animal"
fn definition_category(
    subject: &str,
    dictionary: &Dictionary,
    space: &GeometricSpace,
    structural: &HashSet<String>,
) -> Option<String> {
    let entry = dictionary.entries.iter().find(|e| e.word == subject)?;
    // Only look at the first sentence for category extraction
    let first_sentence = entry.definition.split('.').next().unwrap_or(&entry.definition);
    let words = tokenize(first_sentence);

    for word in &words {
        let stemmed = match stem_to_entry(word, &dictionary.entry_set) {
            Some(s) => s,
            None => continue,
        };
        // Skip the subject itself
        if stemmed == subject {
            continue;
        }

        // ENTITY FAST PATH: entity definitions are hand-crafted in
        // ELI5 format ("a person", "a dog", "a river"). The first
        // non-subject, non-article content word IS the category.
        // Skip all heuristic filters that were designed for messy
        // auto-generated definitions.
        if entry.is_entity {
            let articles: std::collections::HashSet<&str> = ["a", "an", "the"].iter().copied().collect();
            if articles.contains(stemmed.as_str()) {
                continue;
            }
            // First non-article word is the category
            if dictionary.entry_set.contains(&stemmed) {
                return Some(stemmed);
            }
            continue;
        }

        // STANDARD PATH: apply all filters for auto-generated definitions
        // Skip structural/function words
        if structural.contains(&stemmed) {
            continue;
        }
        // Skip words that appear in connector patterns (function words)
        if is_connector_word(&stemmed, space) {
            continue;
        }
        // Skip non-noun words: adjectives, verbs, property words
        if is_property_word(&stemmed, dictionary) {
            continue;
        }
        // Must be a dictionary entry AND have a noun-like definition
        // (starting with an article: "a thing", "an animal", "the act of")
        if dictionary.entry_set.contains(&stemmed) {
            let is_noun = dictionary.entries.iter()
                .find(|e| e.word == stemmed)
                .map_or(false, |e| {
                    let fw = tokenize(&e.definition).into_iter().next().unwrap_or_default();
                    matches!(fw.as_str(), "a" | "an" | "the" | "one" | "any" | "something")
                });
            if is_noun {
                return Some(stemmed);
            }
        }
    }
    None
}

/// Check if a word is a property/adjective/verb word rather than a category noun.
/// Category nouns have definitions starting with articles ("a thing", "an animal").
/// Non-nouns include:
/// - Adjectives: "not X" antonym patterns, or def starts with "having", "relating"
/// - Verbs: def starts with "to" ("to use the mind...")
/// - Other modifiers: def starts with verb-forms
fn is_property_word(word: &str, dictionary: &Dictionary) -> bool {
    dictionary.entries.iter()
        .find(|e| e.word == word)
        .map_or(false, |e| {
            let first_word = tokenize(&e.definition).into_iter().next()
                .unwrap_or_default();
            // Verb definitions: "to go", "to use"
            if first_word == "to" { return true; }
            // Adjective-like: "having positive qualities", "relating to the home"
            // These are present participles used as definition starters for adjectives
            if first_word.ends_with("ing") { return true; }
            // Property words: definitions containing "not X" antonym patterns (exactly 2 words).
            // "not cold", "not small" → true antonyms (property words).
            // "not a plant" (3 words) → category exclusion, not an antonym.
            if e.definition.split('.').any(|sentence| {
                let words = tokenize(sentence.trim());
                words.len() == 2 && words.first().map_or(false, |w| w == "not")
            }) {
                return true;
            }
            // Adverb/modifier: "in a way that..."
            if first_word == "in" || first_word == "very" || first_word == "more" {
                return true;
            }
            false
        })
}

/// Check if a word appears in any connector pattern (marking it as a function word).
fn is_connector_word(word: &str, space: &GeometricSpace) -> bool {
    for connector in &space.connectors {
        if connector.pattern.iter().any(|p| p == word) {
            return true;
        }
    }
    false
}

// ─── Weighted Distance (Fix 2) ───────────────────────────────

/// Compute axis-weighted euclidean distance between two word positions.
/// Dimensions aligned with the connector's force_direction get higher weight.
/// `alpha` controls minimum weight for non-connector dimensions (0.05..0.5).
fn weighted_distance(
    pos_a: &[f64],
    pos_b: &[f64],
    connector_direction: &[f64],
    alpha: f64,
) -> f64 {
    let mut sum = 0.0;
    for i in 0..pos_a.len().min(pos_b.len()).min(connector_direction.len()) {
        let weight = alpha + (1.0 - alpha) * connector_direction[i].abs();
        let diff = pos_a[i] - pos_b[i];
        sum += weight * diff * diff;
    }
    sum.sqrt()
}

// ──────────────────────────────────────────────────────────────

/// The type of question being asked.
enum QuestionType {
    YesNo {
        subject: String,
        object: String,
        connector: Vec<String>,
        negated: bool,
    },
    WhatIs {
        subject: String,
        connector: Vec<String>,
        /// Number of content words in the question besides the subject.
        /// 0 = pure "What is X?" (safe for definition fallback)
        /// >0 = property query like "What color is X?" (should NOT use definition fallback)
        extra_content_words: usize,
    },
}

/// Resolve a question against the geometric space.
/// Returns (answer, projection_distance, connector_pattern_used).
///
/// Answers combine geometric distance with definition-chain verification:
/// - Yes/No: geometry decides first, then definition chain confirms or overrides
/// - What-Is: weighted distance + definition fallback for category extraction
pub fn resolve_question(
    question: &str,
    space: &GeometricSpace,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    content: &HashSet<String>,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> (Answer, Option<f64>, Option<String>) {
    let tokens = tokenize(question);

    let question_type = detect_question_type(&tokens, dictionary, content, structural);

    match question_type {
        Some(QuestionType::YesNo {
            subject,
            object,
            connector,
            negated,
        }) => {
            let connector_str = connector.join(" ");
            let (answer, distance) =
                resolve_yes_no(&subject, &object, negated, &connector, space,
                               dictionary, structural, params, strategy);
            (answer, Some(distance), Some(connector_str))
        }
        Some(QuestionType::WhatIs {
            subject,
            connector,
            extra_content_words,
        }) => {
            let connector_str = connector.join(" ");
            let (answer, distance) =
                resolve_what_is(&subject, &connector, space, content,
                                dictionary, structural, params, strategy,
                                extra_content_words);
            (answer, Some(distance), Some(connector_str))
        }
        None => (Answer::IDontKnow, None, None),
    }
}

// ─── Connector Matching ────────────────────────────────────────

/// Find a connector in the space that best matches the given pattern.
/// Tries: exact match → question pattern is subset → connector pattern is subset → highest-frequency fallback.
fn find_matching_connector<'a>(
    space: &'a GeometricSpace,
    pattern: &[String],
) -> Option<&'a Connector> {
    if pattern.is_empty() || space.connectors.is_empty() {
        return space.connectors.first();
    }

    // 1. Exact match
    if let Some(c) = space.connectors.iter().find(|c| c.pattern == pattern) {
        return Some(c);
    }

    // 2. Question pattern is a subset of a connector pattern
    //    e.g., question has ["is"], space has ["is", "a"]
    if let Some(c) = space.connectors.iter().find(|c| {
        pattern.iter().all(|p| c.pattern.contains(p))
    }) {
        return Some(c);
    }

    // 3. Connector pattern is a subset of question's pattern
    //    e.g., question has ["is", "the"], space has ["is"]
    if let Some(c) = space.connectors.iter().find(|c| {
        c.pattern.iter().all(|p| pattern.contains(p))
    }) {
        return Some(c);
    }

    // 4. Highest-frequency fallback (connectors are sorted by freq desc)
    space.connectors.first()
}

// ─── Axis-Projected Distance ───────────────────────────────────

/// Compute the absolute projected distance between two positions along a direction.
fn projected_distance(pos_a: &[f64], pos_b: &[f64], direction: &[f64]) -> f64 {
    let proj: f64 = pos_a
        .iter()
        .zip(pos_b.iter())
        .zip(direction.iter())
        .map(|((a, b), d)| (b - a) * d)
        .sum();
    proj.abs()
}

/// Compute euclidean distance with a specific direction projected OUT.
/// This removes the influence of one axis (e.g., the negation axis) from
/// the distance computation while preserving all other dimensions.
fn euclidean_distance_excluding_axis(pos_a: &[f64], pos_b: &[f64], exclude_direction: &[f64]) -> f64 {
    // Compute the full displacement vector
    let displacement: Vec<f64> = pos_a.iter().zip(pos_b.iter())
        .map(|(a, b)| b - a)
        .collect();

    // Compute projection of displacement onto the excluded direction
    let proj_scalar: f64 = displacement.iter().zip(exclude_direction.iter())
        .map(|(d, e)| d * e)
        .sum();

    // Subtract the projection to get the component orthogonal to the excluded axis
    let orthogonal: Vec<f64> = displacement.iter().zip(exclude_direction.iter())
        .map(|(d, e)| d - proj_scalar * e)
        .collect();

    // Return euclidean norm of the orthogonal component
    orthogonal.iter().map(|x| x * x).sum::<f64>().sqrt()
}

/// Normalize distance by dividing by mean (ratio normalization).
/// Values < 1.0 = closer than average, > 1.0 = farther than average.
fn ratio_normalize(distance: f64, mean: f64) -> f64 {
    if mean > 1e-10 { distance / mean } else { distance }
}

/// Compute mean and std_dev of pairwise projected distances along an axis.
fn compute_axis_distance_stats(space: &GeometricSpace, direction: &[f64]) -> (f64, f64) {
    let positions: Vec<&Vec<f64>> = space.words.values().map(|wp| &wp.position).collect();
    let n = positions.len();
    if n < 2 {
        return (1.0, 1.0);
    }

    let mut total = 0.0;
    let mut total_sq = 0.0;
    let mut count = 0u64;
    for i in 0..n {
        for j in (i + 1)..n {
            let d: f64 = positions[i]
                .iter()
                .zip(positions[j].iter())
                .zip(direction.iter())
                .map(|((a, b), dir)| (b - a) * dir)
                .sum::<f64>()
                .abs();
            total += d;
            total_sq += d * d;
            count += 1;
        }
    }

    if count == 0 { return (1.0, 1.0); }
    let mean = total / count as f64;
    let variance = (total_sq / count as f64) - mean * mean;
    let std_dev = if variance > 0.0 { variance.sqrt() } else { 1.0 };
    (mean, std_dev)
}

/// Compute mean and std_dev of pairwise euclidean distances excluding one axis.
fn compute_distance_stats_excluding_axis(space: &GeometricSpace, exclude_direction: &[f64]) -> (f64, f64) {
    let positions: Vec<&Vec<f64>> = space.words.values().map(|wp| &wp.position).collect();
    let n = positions.len();
    if n < 2 {
        return (1.0, 1.0);
    }

    let mut total = 0.0;
    let mut total_sq = 0.0;
    let mut count = 0u64;
    for i in 0..n {
        for j in (i + 1)..n {
            let d = euclidean_distance_excluding_axis(positions[i], positions[j], exclude_direction);
            total += d;
            total_sq += d * d;
            count += 1;
        }
    }

    if count == 0 { return (1.0, 1.0); }
    let mean = total / count as f64;
    let variance = (total_sq / count as f64) - mean * mean;
    let std_dev = if variance > 0.0 { variance.sqrt() } else { 1.0 };
    (mean, std_dev)
}

/// Compute mean and std_dev of pairwise dim-0-only distances.
fn compute_dim0_distance_stats(space: &GeometricSpace) -> (f64, f64) {
    let words: Vec<&[f64]> = space.words.values().map(|wp| wp.position.as_slice()).collect();
    let n = words.len();
    if n < 2 {
        return (1.0, 1.0);
    }

    let mut total = 0.0;
    let mut total_sq = 0.0;
    let mut count = 0u64;
    for i in 0..n {
        for j in (i + 1)..n {
            if !words[i].is_empty() && !words[j].is_empty() {
                let d = (words[i][0] - words[j][0]).abs();
                total += d;
                total_sq += d * d;
                count += 1;
            }
        }
    }

    if count == 0 { return (1.0, 1.0); }
    let mean = total / count as f64;
    let variance = (total_sq / count as f64) - mean * mean;
    let std_dev = if variance > 0.0 { variance.sqrt() } else { 1.0 };
    (mean, std_dev)
}

/// Compute mean and std_dev of pairwise distances excluding dim 0.
fn compute_excl_dim0_distance_stats(space: &GeometricSpace) -> (f64, f64) {
    let words: Vec<&[f64]> = space.words.values().map(|wp| wp.position.as_slice()).collect();
    let n = words.len();
    if n < 2 {
        return (1.0, 1.0);
    }

    let mut total = 0.0;
    let mut total_sq = 0.0;
    let mut count = 0u64;
    for i in 0..n {
        for j in (i + 1)..n {
            let d: f64 = words[i].iter().zip(words[j].iter())
                .skip(1)
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<f64>()
                .sqrt();
            total += d;
            total_sq += d * d;
            count += 1;
        }
    }

    if count == 0 { return (1.0, 1.0); }
    let mean = total / count as f64;
    let variance = (total_sq / count as f64) - mean * mean;
    let std_dev = if variance > 0.0 { variance.sqrt() } else { 1.0 };
    (mean, std_dev)
}

/// Find the negation connector in the space (the ["not"] connector).
fn find_negation_connector(space: &GeometricSpace) -> Option<&Connector> {
    space.connectors.iter().find(|c| {
        c.pattern.len() == 1 && c.pattern[0] == "not"
    })
}

// ─── Question Type Detection ───────────────────────────────────

/// Detect whether the question is a Yes/No, What-Is, Who-Is, or Where-Is question.
fn detect_question_type(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    if tokens.is_empty() {
        return None;
    }

    match tokens[0].as_str() {
        "what" => detect_what_question(tokens, dictionary, content, structural),
        "who" => detect_who_question(tokens, dictionary, content, structural),
        "where" => detect_where_question(tokens, dictionary, content, structural),
        _ => detect_yes_no_question(tokens, dictionary, content, structural),
    }
}

/// Detect "Who is X?" questions.
/// Semantically identical to "What is X?" — the resolver doesn't
/// enforce person-category constraints. The question word is just routing.
fn detect_who_question(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    // Reuse what-question detection: "who" behaves like "what" for resolution.
    // The geometric space and definition-category extraction handle the rest.
    detect_what_question(tokens, dictionary, content, structural)
}

/// Detect "Where is X?" questions.
/// Currently uses same resolution as "What is X?" (definition-category extraction).
/// Future: dedicated location-relation extraction from definitions.
fn detect_where_question(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    detect_what_question(tokens, dictionary, content, structural)
}

/// Detect "What is X?" questions.
fn detect_what_question(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    // Find all content words in the question (after "what")
    let content_entries: Vec<(usize, String)> = tokens
        .iter()
        .enumerate()
        .skip(1) // skip "what"
        .filter_map(|(i, t)| {
            stem_to_entry(t, &dictionary.entry_set).and_then(|e| {
                if content.contains(&e) {
                    Some((i, e))
                } else {
                    None
                }
            })
        })
        .collect();

    // Also check for non-content entry words after "what" (e.g., "thing" which is structural)
    let all_entry_words: Vec<(usize, String)> = tokens
        .iter()
        .enumerate()
        .skip(1)
        .filter_map(|(i, t)| {
            stem_to_entry(t, &dictionary.entry_set).map(|e| (i, e))
        })
        .collect();

    if all_entry_words.is_empty() {
        return None;
    }

    // The subject is the last content word. If no content words, use last entry word.
    let subject = if !content_entries.is_empty() {
        content_entries[content_entries.len() - 1].1.clone()
    } else {
        all_entry_words[all_entry_words.len() - 1].1.clone()
    };

    let connector: Vec<String> = tokens[1..]
        .iter()
        .filter_map(|t| {
            stem_to_entry(t, &dictionary.entry_set)
                .filter(|e| structural.contains(e))
        })
        .collect();

    let connector = if connector.is_empty() {
        vec!["is".to_string()]
    } else {
        connector
    };

    // Count extra content words (content words besides the subject).
    // Filter out common question-syntax words that may be classified as content
    // in some dictionaries: "is", "a", "an", "the" should not count as
    // extra content in "What is a cat?" vs "What color is a cat?"
    let question_syntax: HashSet<&str> = ["is", "a", "an", "the", "of", "do", "does", "can", "has"].iter().copied().collect();
    let extra_content_words = content_entries.iter()
        .filter(|(_, w)| *w != subject && !question_syntax.contains(w.as_str()))
        .count();

    Some(QuestionType::WhatIs {
        subject,
        connector,
        extra_content_words,
    })
}

/// Detect Yes/No questions like "Is X a Y?", "Can X do Y?"
fn detect_yes_no_question(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    // Skip leading question verbs: "is", "can", "does", "do", "has"
    // These are question syntax, not content.
    let question_verbs: HashSet<&str> = ["is", "can", "does", "do", "has"].iter().copied().collect();
    let skip_start = if !tokens.is_empty() && question_verbs.contains(tokens[0].as_str()) {
        1
    } else {
        0
    };

    // Find all content words (skipping the leading question verb)
    let mut content_entries: Vec<(usize, String)> = tokens
        .iter()
        .enumerate()
        .skip(skip_start)
        .filter_map(|(i, t)| {
            stem_to_entry(t, &dictionary.entry_set).and_then(|e| {
                if content.contains(&e) {
                    Some((i, e))
                } else {
                    None
                }
            })
        })
        .collect();

    // Fallback: if we don't have enough content words, also include ALL entry words
    // (except common structural ones like "is", "a", "the", "it", "not")
    if content_entries.len() < 2 {
        let skip_words: HashSet<&str> = ["is", "a", "the", "it", "not"].iter().copied().collect();
        content_entries = tokens
            .iter()
            .enumerate()
            .skip(skip_start)
            .filter_map(|(i, t)| {
                stem_to_entry(t, &dictionary.entry_set).and_then(|e| {
                    if !skip_words.contains(e.as_str()) {
                        Some((i, e))
                    } else {
                        None
                    }
                })
            })
            .collect();
    }

    if content_entries.len() < 2 {
        return None;
    }

    let (left_pos, ref subject) = content_entries[0];
    let (right_pos, ref object) = content_entries[content_entries.len() - 1];

    // Extract structural words between subject and object
    let between: Vec<String> = if right_pos > left_pos + 1 {
        (left_pos + 1..right_pos)
            .filter_map(|i| {
                stem_to_entry(&tokens[i], &dictionary.entry_set)
                    .filter(|e| structural.contains(e))
            })
            .collect()
    } else {
        Vec::new()
    };

    // Also check for structural words before the first content word
    let prefix_structural: Vec<String> = (0..left_pos)
        .filter_map(|i| {
            stem_to_entry(&tokens[i], &dictionary.entry_set)
                .filter(|e| structural.contains(e))
        })
        .collect();

    // Combine: use between if non-empty, otherwise use prefix
    let all_connectors = if !between.is_empty() {
        between
    } else if !prefix_structural.is_empty() {
        prefix_structural
    } else {
        vec!["is".to_string()] // default
    };

    let negated = all_connectors.first().map_or(false, |w| w == "not");
    let connector = if negated && all_connectors.len() > 1 {
        all_connectors[1..].to_vec()
    } else {
        all_connectors
    };

    Some(QuestionType::YesNo {
        subject: subject.clone(),
        object: object.clone(),
        connector,
        negated,
    })
}

// ─── Yes/No Resolution ─────────────────────────────────────────

/// Resolve a Yes/No question using axis-aware distance + definition-chain verification.
///
/// Strategy:
/// 1. Compute geometric distance-based answer (existing logic)
/// 2. If geometry says Yes → verify with definition chain:
///    - Chain confirms → keep Yes
///    - Chain says No → override to No (negation detected)
///    - Chain inconclusive → trust geometry
/// 3. If geometry says No or IDK → trust it (no chain check needed)
fn resolve_yes_no(
    subject: &str,
    object: &str,
    negated: bool,
    connector_pattern: &[String],
    space: &GeometricSpace,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> (Answer, f64) {
    let subject_pos = match space.words.get(subject) {
        Some(wp) => &wp.position,
        None => return (Answer::IDontKnow, f64::MAX),
    };
    let object_pos = match space.words.get(object) {
        Some(wp) => &wp.position,
        None => return (Answer::IDontKnow, f64::MAX),
    };

    // Step 1: Compute geometric answer (all existing distance logic)
    let (geometric_answer, distance) = compute_geometric_yes_no(
        subject, object, subject_pos, object_pos, negated,
        connector_pattern, space, params, strategy,
    );

    // Step 2: Definition-chain gate
    // Fires on all non-negated questions. The chain traversal provides definitive
    // answers based on dictionary definitions, overriding geometric distance:
    // - Some(true) → Yes (chain found definitional link: dog→animal)
    // - Some(false) → No (chain found negation: sun→hot→"not cold"→cold)
    // - None + both in dict → No (category objects) or IDK (property objects)
    // - None + not both in dict → trust geometry
    // For negated questions, the geometric pipeline handles via threshold inversion.
    if !negated {
        let max_hops = 3; // traversal depth (was 2; increased for Level 1 ontological chains)

        // Forward check: subject → object
        let mut visited = HashSet::new();
        let forward = definition_chain_check(
            subject, object, dictionary, structural, max_hops, &mut visited, space,
        );
        match forward {
            Some(false) => return (Answer::No, distance), // chain says definitionally negated
            Some(true) => return (Answer::Yes, distance),  // chain confirms → Yes
            None => {
                // Forward inconclusive — try reverse: object → subject
                let mut visited_rev = HashSet::new();
                let reverse = definition_chain_check(
                    object, subject, dictionary, structural, max_hops, &mut visited_rev, space,
                );
                match reverse {
                    Some(false) => return (Answer::No, distance), // reverse chain says negated
                    Some(true) => return (Answer::Yes, distance),  // reverse confirms → Yes
                    None => {
                        // Both chains inconclusive → neither defines the other.
                        // When both words are in the dictionary and chains can't
                        // connect them in either direction, that's evidence of No.
                        if dictionary.entries.iter().any(|e| e.word == subject)
                            && dictionary.entries.iter().any(|e| e.word == object)
                        {
                            // Only return No when the object is clearly a category NOUN.
                            // Verbs, adjectives, and property words → IDK (we can't
                            // determine the relationship from definitions alone).
                            //
                            // Category nouns have definitions starting with "a/an {noun}"
                            // pattern: "animal" → "a living thing...", "mammal" → "a type of..."
                            //
                            // Non-nouns to detect:
                            // - Verbs: def starts with "to" ("think" → "to use the mind...")
                            // - Properties: def has "not X" antonym pattern ("hot" → "...not cold.")
                            // - Adjective-like: def starts with verb-ing ("good" → "having...")
                            let object_is_noun = dictionary.entries.iter()
                                .find(|e| e.word == object)
                                .map_or(false, |e| {
                                    let first_word = tokenize(&e.definition).into_iter().next()
                                        .unwrap_or_default();
                                    // Category noun defs start with articles: "a", "an", "the", "one"
                                    let starts_noun = matches!(first_word.as_str(),
                                        "a" | "an" | "the" | "one" | "any" | "something");
                                    // Not a verb (starts with "to")
                                    let starts_verb = first_word == "to";
                                    // Not a property word (has "not X" antonym)
                                    let is_property = is_property_word(object, dictionary);
                                    starts_noun && !starts_verb && !is_property
                                });
                            if object_is_noun {
                                return (Answer::No, distance);
                            } else {
                                return (Answer::IDontKnow, distance);
                            }
                        }
                        // If either word isn't a dictionary entry: trust geometry
                    }
                }
            }
        }
    }

    (geometric_answer, distance)
}

/// Pure geometric distance computation for Yes/No (extracted from resolve_yes_no).
/// This contains all the original distance logic without the definition-chain gate.
fn compute_geometric_yes_no(
    _subject: &str,
    _object: &str,
    subject_pos: &[f64],
    object_pos: &[f64],
    negated: bool,
    connector_pattern: &[String],
    space: &GeometricSpace,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> (Answer, f64) {
    // Connector-axis path takes priority: project onto the connector's trained
    // direction axis to measure relationship-specific distance. This overrides
    // negation-model-specific paths when enabled.
    if strategy.use_connector_axis {
        if let Some(conn) = find_matching_connector(space, connector_pattern) {
            let proj_dist = projected_distance(subject_pos, object_pos, &conn.force_direction);
            let (axis_mean, _) = compute_axis_distance_stats(space, &conn.force_direction);
            let normalized = ratio_normalize(proj_dist, axis_mean);
            return decide_yes_no(normalized, negated, params);
        }
    }

    // SeparateDimension uses its own specialized path
    if strategy.negation_model == NegationModel::SeparateDimension {
        return resolve_yes_no_separate_dimension(
            subject_pos, object_pos, negated, space, params,
        );
    }

    // Repulsion negation model: the ["not"] connector creates a dominant axis
    // that explains ~99% of variance. We need to handle this specially.
    if strategy.negation_model == NegationModel::Repulsion {
        if let Some(neg_conn) = find_negation_connector(space) {
            if negated {
                // For negated questions, use the negation axis projected distance.
                let proj_dist = projected_distance(subject_pos, object_pos, &neg_conn.force_direction);
                let (axis_mean, _) = compute_axis_distance_stats(space, &neg_conn.force_direction);
                let normalized = ratio_normalize(proj_dist, axis_mean);
                if normalized > params.no_threshold {
                    return (Answer::Yes, normalized);
                } else if normalized < params.yes_threshold {
                    return (Answer::No, normalized);
                } else {
                    return (Answer::IDontKnow, normalized);
                }
            } else {
                let dist = euclidean_distance_excluding_axis(
                    subject_pos, object_pos, &neg_conn.force_direction,
                );
                let (excl_mean, _) = compute_distance_stats_excluding_axis(space, &neg_conn.force_direction);
                let normalized = ratio_normalize(dist, excl_mean);
                return decide_yes_no(normalized, false, params);
            }
        }
    }

    // Default path: standard euclidean distance with ratio normalization.
    let euclidean = euclidean_distance(subject_pos, object_pos);
    let stats = space.get_distance_stats();
    let normalized = ratio_normalize(euclidean, stats.mean);
    decide_yes_no(normalized, negated, params)
}

/// SeparateDimension resolver: dim 0 is the negation dimension.
/// For negated questions: use only dim 0 distance.
/// For non-negated questions: use all dimensions EXCEPT dim 0.
fn resolve_yes_no_separate_dimension(
    subject_pos: &[f64],
    object_pos: &[f64],
    negated: bool,
    space: &GeometricSpace,
    params: &EngineParams,
) -> (Answer, f64) {
    if negated {
        // Use only dimension 0 (the negation dimension)
        let dim0_dist = if !subject_pos.is_empty() && !object_pos.is_empty() {
            (subject_pos[0] - object_pos[0]).abs()
        } else {
            0.0
        };

        let (dim0_mean, _) = compute_dim0_distance_stats(space);
        let normalized = ratio_normalize(dim0_dist, dim0_mean);

        // For negated with SeparateDimension: large dim0 distance means words are
        // pushed apart on negation axis -> they ARE different -> "not X" is true -> Yes
        // Small dim0 distance -> they are similar on negation axis -> "not X" is false -> No
        if normalized > params.no_threshold {
            (Answer::Yes, normalized)
        } else if normalized < params.yes_threshold {
            (Answer::No, normalized)
        } else {
            (Answer::IDontKnow, normalized)
        }
    } else {
        // Non-negated: use all dimensions except dim 0
        let dist: f64 = subject_pos
            .iter()
            .zip(object_pos.iter())
            .skip(1) // skip dimension 0
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt();

        let (excl_mean, _) = compute_excl_dim0_distance_stats(space);
        let normalized = ratio_normalize(dist, excl_mean);

        // Standard threshold logic for non-negated
        if normalized < params.yes_threshold {
            (Answer::Yes, normalized)
        } else if normalized > params.no_threshold {
            (Answer::No, normalized)
        } else {
            (Answer::IDontKnow, normalized)
        }
    }
}

/// Apply threshold to decide Yes/No/IDontKnow.
fn decide_yes_no(distance: f64, negated: bool, params: &EngineParams) -> (Answer, f64) {
    if negated {
        // Negation inverts: small distance = No (they ARE close, but question asks "not")
        if distance < params.yes_threshold {
            (Answer::No, distance)
        } else if distance > params.no_threshold {
            (Answer::Yes, distance)
        } else {
            (Answer::IDontKnow, distance)
        }
    } else {
        if distance < params.yes_threshold {
            (Answer::Yes, distance)
        } else if distance > params.no_threshold {
            (Answer::No, distance)
        } else {
            (Answer::IDontKnow, distance)
        }
    }
}

// ─── What-is Resolution ────────────────────────────────────────

/// Resolve a "What is X?" question using multi-axis weighted nearest neighbor
/// + definition-category fallback.
///
/// Strategy:
/// 1. Try axis-weighted nearest neighbor (uses connector direction as weight vector)
/// 2. Fall back to standard nearest neighbor (existing logic)
/// 3. Final fallback: extract category from subject's definition text
fn resolve_what_is(
    subject: &str,
    connector_pattern: &[String],
    space: &GeometricSpace,
    content: &HashSet<String>,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    params: &EngineParams,
    strategy: &StrategyConfig,
    extra_content_words: usize,
) -> (Answer, f64) {
    // For pure "What is X?" questions, try definition-category extraction FIRST.
    // Definitions are ground truth — geometric nearest neighbor sometimes picks
    // a geometrically close but semantically wrong word (e.g., "bad" for "person"
    // because they co-occur in examples). Definition extraction directly reads
    // "person — an animal that can..." → "animal".
    if extra_content_words == 0 {
        if let Some(category) = definition_category(subject, dictionary, space, structural) {
            let article = if category.starts_with(|c: char| "aeiou".contains(c)) { "an" } else { "a" };
            return (Answer::Word(format!("{} {}", article, category)), 0.0);
        }
    }

    // Property queries ("What color is X?", "What is the name of X?") ask for
    // specific property values that the geometric space doesn't encode.
    // The dictionary defines words, not their specific attribute values.
    // Return IDK rather than a misleading geometric nearest-neighbor answer.
    if extra_content_words > 0 {
        return (Answer::IDontKnow, f64::MAX);
    }

    let subject_pos = match space.words.get(subject) {
        Some(wp) => &wp.position,
        None => {
            return (Answer::IDontKnow, f64::MAX);
        }
    };

    // Try axis-weighted nearest neighbor using the "is"/"is a" connector direction
    let is_connector = find_matching_connector(space, connector_pattern)
        .or_else(|| space.connectors.iter().find(|c| {
            c.pattern == vec!["is".to_string()] || c.pattern == vec!["is".to_string(), "a".to_string()]
        }));

    let alpha = 0.2; // connector axis emphasis (hardcoded for now, will be evolvable)

    // Connector-axis mode takes priority (existing)
    let connector_axis = if strategy.use_connector_axis {
        find_matching_connector(space, connector_pattern)
    } else {
        None
    };

    // Fallback: axis-excluded distance for Repulsion model
    let use_axis_exclusion = connector_axis.is_none()
        && strategy.negation_model == NegationModel::Repulsion;
    let neg_connector = if use_axis_exclusion {
        find_negation_connector(space)
    } else {
        None
    };

    let mut best_word = String::new();
    let mut best_distance = f64::MAX;

    for (word, wp) in &space.words {
        if word == subject {
            continue;
        }
        // Skip function words and connector words
        if !content.contains(word.as_str()) || is_connector_word(word, space) {
            continue;
        }

        let dist = if let Some(conn) = connector_axis {
            projected_distance(subject_pos, &wp.position, &conn.force_direction)
        } else if let Some(neg_conn) = neg_connector {
            euclidean_distance_excluding_axis(subject_pos, &wp.position, &neg_conn.force_direction)
        } else if let Some(is_conn) = is_connector {
            // NEW: use axis-weighted distance for better category discrimination
            weighted_distance(subject_pos, &wp.position, &is_conn.force_direction, alpha)
        } else {
            euclidean_distance(subject_pos, &wp.position)
        };
        if dist < best_distance {
            best_distance = dist;
            best_word = word.clone();
        }
    }

    if best_word.is_empty() {
        return (Answer::IDontKnow, f64::MAX);
    }

    // Threshold comparison (same logic as before)
    let threshold_distance = if let Some(conn) = connector_axis {
        let (axis_mean, _) = compute_axis_distance_stats(space, &conn.force_direction);
        ratio_normalize(best_distance, axis_mean)
    } else if use_axis_exclusion && neg_connector.is_some() {
        let (excl_mean, _) = compute_distance_stats_excluding_axis(
            space,
            &neg_connector.unwrap().force_direction,
        );
        ratio_normalize(best_distance, excl_mean)
    } else {
        best_distance
    };

    if threshold_distance < params.no_threshold {
        // Determine article ("a" vs "an") based on the word
        let article = if best_word.starts_with(|c: char| "aeiou".contains(c)) { "an" } else { "a" };
        (Answer::Word(format!("{} {}", article, best_word)), best_distance)
    } else {
        // Geometric nearest neighbor is too far — definition fallback already tried above
        (Answer::IDontKnow, best_distance)
    }
}

// ─── Helpers ───────────────────────────────────────────────────
