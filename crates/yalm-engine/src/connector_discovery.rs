use std::collections::{HashMap, HashSet};
use yalm_core::*;
use yalm_parser::{stem_to_entry, tokenize};

use crate::strategy::{ConnectorDetection, StrategyConfig};

/// Classify each entry word as structural (high doc-frequency) or content (low doc-frequency).
/// Structural words are the "glue" — they appear across many definitions (is, a, the, can, not...).
/// Content words are the "substance" — they appear in fewer definitions (dog, cat, hot, animal...).
pub fn classify_word_roles(dictionary: &Dictionary) -> (HashSet<String>, HashSet<String>) {
    let mut doc_freq: HashMap<String, usize> = HashMap::new();

    for entry in &dictionary.entries {
        let mut words_in_entry: HashSet<String> = HashSet::new();
        // Only use definition text for classification (not examples, which
        // are full sentences that reference many entity words)
        let tokens = tokenize(&entry.definition);
        for token in &tokens {
            if let Some(e) = stem_to_entry(token, &dictionary.entry_set) {
                if e != entry.word {
                    words_in_entry.insert(e);
                }
            }
        }
        for w in words_in_entry {
            *doc_freq.entry(w).or_insert(0) += 1;
        }
    }

    // Threshold: structural if appears in > 20% of entry definitions
    let threshold = dictionary.entries.len() * 20 / 100;
    let mut structural = HashSet::new();
    let mut content = HashSet::new();

    for entry_word in &dictionary.entry_words {
        let df = doc_freq.get(entry_word).copied().unwrap_or(0);
        if df > threshold {
            structural.insert(entry_word.clone());
        } else {
            content.insert(entry_word.clone());
        }
    }

    (structural, content)
}

/// Extract all individual sentences from all definitions and examples.
pub fn extract_all_sentences(dictionary: &Dictionary) -> Vec<String> {
    let mut sentences = Vec::new();
    for entry in &dictionary.entries {
        for sentence in entry.definition.split('.') {
            let s = sentence.trim();
            if !s.is_empty() {
                sentences.push(s.to_string());
            }
        }
        for example in &entry.examples {
            for sentence in example.split('.') {
                let s = sentence.trim();
                if !s.is_empty() {
                    sentences.push(s.to_string());
                }
            }
        }
    }
    sentences
}

/// Extract relations from sentences: pairs of "topic" words with all words between them.
///
/// Topic words are content words that are NOT common function words. The tokens
/// between two topic words form a connector candidate. Frequency filtering later
/// determines which are real connectors.
pub fn extract_relations(
    sentences: &[String],
    dictionary: &Dictionary,
    _structural: &HashSet<String>,
    _content: &HashSet<String>,
    params: &EngineParams,
) -> Vec<SentenceRelation> {
    // "Topic" words are content words that represent entities/properties/actions,
    // not common function words. We define topic words as content words that are
    // NOT in a small set of known function-like content words.
    // However, since we can't hardcode, we use a heuristic: topic words are content
    // words with document frequency < 40% of entries (rare enough to be "about" something).
    let mut def_freq: HashMap<String, usize> = HashMap::new();
    for entry in &dictionary.entries {
        let mut seen: HashSet<String> = HashSet::new();
        let tokens = tokenize(&entry.definition);
        for token in &tokens {
            if let Some(e) = stem_to_entry(token, &dictionary.entry_set) {
                if e != entry.word {
                    seen.insert(e);
                }
            }
        }
        // Also count from examples
        for ex in &entry.examples {
            let tokens = tokenize(ex);
            for token in &tokens {
                if let Some(e) = stem_to_entry(token, &dictionary.entry_set) {
                    if e != entry.word {
                        seen.insert(e);
                    }
                }
            }
        }
        for w in seen {
            *def_freq.entry(w).or_insert(0) += 1;
        }
    }

    // Topic words: appear in < threshold of entries (specific enough to be subjects/objects)
    // This makes "can", "has", "in" etc. non-topic so they appear as connectors between
    // entity/property words.
    //
    // Scale topic threshold sub-linearly with dictionary size.
    // For small dicts (~50 entries), threshold ≈ 25% (unchanged).
    // For large dicts (783+), effective % drops as 25% / ln(n/50),
    // allowing more content words to be topic endpoints → more relations → more connectors.
    let n = dictionary.entries.len() as f64;
    let base = 50.0_f64;
    let log_scale = (n / base).ln().max(1.0);
    let topic_threshold = (n * 0.25 / log_scale) as usize;
    let topic_words: HashSet<String> = dictionary
        .entry_words
        .iter()
        .filter(|w| def_freq.get(w.as_str()).copied().unwrap_or(0) < topic_threshold)
        .cloned()
        .collect();

    let mut relations = Vec::new();

    for sentence in sentences {
        let tokens = tokenize(sentence);
        let mapped: Vec<Option<String>> = tokens
            .iter()
            .map(|t| stem_to_entry(t, &dictionary.entry_set))
            .collect();

        // Find topic word positions
        let topic_positions: Vec<(usize, String)> = mapped
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| {
                opt.as_ref()
                    .filter(|w| topic_words.contains(w.as_str()))
                    .map(|w| (i, w.clone()))
            })
            .collect();

        // For each consecutive pair of topic words
        for window in topic_positions.windows(2) {
            let (left_pos, ref left_word) = window[0];
            let (right_pos, ref right_word) = window[1];

            if right_pos <= left_pos + 1 {
                continue; // Adjacent topic words
            }

            // Extract ALL entry words between the two topic words as the connector
            let between: Vec<String> = (left_pos + 1..right_pos)
                .filter_map(|i| mapped[i].clone())
                .collect();

            if between.is_empty() {
                continue;
            }

            // Check for "not" prefix → negation
            let negated = between.first().map_or(false, |w| w == "not");
            let connector_pattern = if negated && between.len() > 1 {
                between[1..].to_vec()
            } else if negated {
                vec!["not".to_string()]
            } else {
                between
            };

            if connector_pattern.len() > params.connector_max_length {
                continue;
            }

            relations.push(SentenceRelation {
                left_word: left_word.clone(),
                right_word: right_word.clone(),
                connector_pattern,
                negated,
                source: sentence.clone(),
                weight: 1.0,
            });
        }
    }

    relations
}

/// Compute uniformity score for a connector pattern across alphabetical buckets.
/// Returns how uniformly the pattern is distributed (0.0 = clustered, 1.0 = perfectly uniform).
fn compute_uniformity(
    pattern: &[String],
    buckets: &[Vec<&DictionaryEntry>],
) -> f64 {
    let epsilon = 1e-10;

    let ratios: Vec<f64> = buckets
        .iter()
        .map(|bucket| {
            if bucket.is_empty() {
                return 0.0;
            }
            let hits = bucket
                .iter()
                .filter(|entry| {
                    // Check if the pattern appears in this entry's definition + examples
                    let mut all_text = entry.definition.clone();
                    for ex in &entry.examples {
                        all_text.push(' ');
                        all_text.push_str(ex);
                    }
                    let tokens = tokenize(&all_text);
                    let pat_len = pattern.len();
                    if pat_len == 0 {
                        return false;
                    }
                    tokens.windows(pat_len).any(|window| {
                        window
                            .iter()
                            .zip(pattern.iter())
                            .all(|(tok, pat)| tok.to_lowercase() == *pat)
                    })
                })
                .count();
            hits as f64 / bucket.len() as f64
        })
        .collect();

    let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
    let variance = ratios.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / ratios.len() as f64;
    1.0 - (variance / (mean * mean + epsilon))
}

/// Build alphabetical buckets from dictionary entries for uniformity testing.
fn build_alphabetical_buckets(dictionary: &Dictionary, num_buckets: usize) -> Vec<Vec<&DictionaryEntry>> {
    let n = dictionary.entries.len();
    let num_buckets = num_buckets.min(n).max(1);
    let bucket_size = n / num_buckets;

    let mut sorted_indices: Vec<usize> = (0..n).collect();
    sorted_indices.sort_by(|&a, &b| dictionary.entries[a].word.cmp(&dictionary.entries[b].word));

    (0..num_buckets)
        .map(|i| {
            let start = i * bucket_size;
            let end = if i == num_buckets - 1 { n } else { (i + 1) * bucket_size };
            sorted_indices[start..end]
                .iter()
                .map(|&idx| &dictionary.entries[idx])
                .collect()
        })
        .collect()
}

/// Internal pipeline: from relations → connectors.
/// Frequency filter (strategy-dependent) → uniformity filter → RNG force directions.
/// Shared by `discover_connectors()` and `discover_connectors_from_sentences()`.
fn connector_pipeline(
    relations: &[SentenceRelation],
    dictionary: &Dictionary,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> Vec<Connector> {
    // Count connector pattern frequencies
    let mut freq: HashMap<Vec<String>, usize> = HashMap::new();
    for rel in relations {
        *freq.entry(rel.connector_pattern.clone()).or_insert(0) += 1;
    }

    // === PASS 1: Frequency filter — collect candidates ===
    let mut candidates: Vec<(Vec<String>, usize)> = Vec::new();

    match strategy.connector_detection {
        ConnectorDetection::FrequencyOnly => {
            for (pattern, count) in &freq {
                if *count >= params.connector_min_frequency && !pattern.is_empty() {
                    candidates.push((pattern.clone(), *count));
                }
            }
        }
        ConnectorDetection::PositionalBias => {
            let mut early_count: HashMap<Vec<String>, usize> = HashMap::new();
            for entry in &dictionary.entries {
                let tokens = tokenize(&entry.definition);
                let mapped: Vec<Option<String>> = tokens
                    .iter()
                    .map(|t| stem_to_entry(t, &dictionary.entry_set))
                    .collect();

                for (pattern, _) in &freq {
                    let pat_len = pattern.len();
                    if pat_len == 0 {
                        continue;
                    }
                    for start in 0..5usize.min(mapped.len()) {
                        if start + pat_len > mapped.len() {
                            break;
                        }
                        let matches = (0..pat_len).all(|k| {
                            mapped[start + k]
                                .as_ref()
                                .map_or(false, |w| *w == pattern[k])
                        });
                        if matches {
                            *early_count.entry(pattern.clone()).or_insert(0) += 1;
                            break;
                        }
                    }
                }
            }

            for (pattern, count) in &freq {
                if pattern.is_empty() {
                    continue;
                }
                let early = early_count.get(pattern).copied().unwrap_or(0);
                let adjusted = *count + early / 2;
                if adjusted >= params.connector_min_frequency {
                    candidates.push((pattern.clone(), *count));
                }
            }
        }
        ConnectorDetection::MutualInformation => {
            let total_relations = relations.len().max(1) as f64;

            let mut left_counts: HashMap<String, usize> = HashMap::new();
            let mut right_counts: HashMap<String, usize> = HashMap::new();
            for rel in relations {
                *left_counts.entry(rel.left_word.clone()).or_insert(0) += 1;
                *right_counts.entry(rel.right_word.clone()).or_insert(0) += 1;
            }

            let mut pattern_scores: HashMap<Vec<String>, f64> = HashMap::new();
            for rel in relations {
                let left_c = *left_counts.get(&rel.left_word).unwrap_or(&1) as f64;
                let right_c = *right_counts.get(&rel.right_word).unwrap_or(&1) as f64;
                let pair_count = freq.get(&rel.connector_pattern).copied().unwrap_or(1) as f64;

                let pmi = (pair_count * total_relations / (left_c * right_c)).ln();
                *pattern_scores
                    .entry(rel.connector_pattern.clone())
                    .or_insert(0.0) += pmi;
            }

            for (pattern, score) in &mut pattern_scores {
                let count = freq.get(pattern).copied().unwrap_or(1) as f64;
                *score /= count;
            }

            for (pattern, count) in &freq {
                if pattern.is_empty() || *count < 1 {
                    continue;
                }
                let score = pattern_scores.get(pattern).copied().unwrap_or(0.0);
                if score > 0.0 {
                    candidates.push((pattern.clone(), *count));
                }
            }
        }
    }

    // === PASS 2: Uniformity filter ===
    let num_buckets = params.uniformity_num_buckets;
    let uniformity_threshold = params.uniformity_threshold;
    let skip_uniformity = dictionary.entries.len() < 100;

    let buckets = if !skip_uniformity {
        build_alphabetical_buckets(dictionary, num_buckets)
    } else {
        Vec::new()
    };

    let mut rng = SimpleRng::new(params.rng_seed);
    let mut connectors = Vec::new();
    let mut rejected_count = 0usize;

    // Sort candidates by frequency desc for deterministic RNG consumption
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    for (pattern, count) in &candidates {
        let uniformity = if skip_uniformity {
            1.0 // Small dict: assume all candidates are uniform
        } else {
            compute_uniformity(pattern, &buckets)
        };

        if uniformity > uniformity_threshold || skip_uniformity {
            let direction = random_unit_vector(params.dimensions, &mut rng);
            connectors.push(Connector {
                pattern: pattern.clone(),
                force_direction: direction,
                magnitude: params.force_magnitude,
                frequency: *count,
                uniformity,
            });
        } else {
            rejected_count += 1;
        }
    }

    // Print uniformity diagnostics for medium+ dictionaries
    if !skip_uniformity {
        eprintln!(
            "=== Uniformity Filter: {} entries | {} candidates | {} buckets ===",
            dictionary.entries.len(),
            candidates.len(),
            num_buckets
        );
        eprintln!(
            "{:<25} {:>6} {:>10} {:>10}",
            "Pattern", "Freq", "Uniform", "Status"
        );
        // Show all candidates sorted by uniformity descending
        let mut diag: Vec<(&Vec<String>, usize, f64)> = candidates
            .iter()
            .map(|(p, c)| (p, *c, compute_uniformity(p, &buckets)))
            .collect();
        diag.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        for (pattern, freq, uni) in &diag {
            let pat_str: String = pattern
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(" ");
            let status = if *uni > uniformity_threshold {
                "CONNECTOR"
            } else {
                "content"
            };
            eprintln!(
                "{:<25} {:>6} {:>10.4} {:>10}",
                pat_str, freq, uni, status
            );
        }
        eprintln!(
            "Accepted: {} connectors | Rejected: {} content words",
            connectors.len(),
            rejected_count
        );
    }

    // Sort by frequency descending for deterministic ordering
    connectors.sort_by(|a, b| b.frequency.cmp(&a.frequency).then(a.pattern.cmp(&b.pattern)));

    connectors
}

/// Discover connectors from the dictionary text and return both the connectors
/// and all extracted sentence relations.
///
/// Two-pass pipeline:
/// 1. Frequency filter: select candidate patterns above min_frequency
/// 2. Uniformity filter: keep only candidates distributed uniformly across the dictionary
///    (skipped for small dictionaries < 100 entries where frequency is sufficient)
pub fn discover_connectors(
    dictionary: &Dictionary,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> (Vec<Connector>, Vec<SentenceRelation>) {
    let sentences = extract_all_sentences(dictionary);
    let (structural, content) = classify_word_roles(dictionary);
    let relations = extract_relations(&sentences, dictionary, &structural, &content, params);
    let connectors = connector_pipeline(&relations, dictionary, params, strategy);
    (connectors, relations)
}

/// Discover connectors from an arbitrary set of sentences (not just dictionary text).
/// Used by the bootstrap loop to combine dictionary + generated sentences.
///
/// Word-role classification still uses the dictionary (structural/content is a property
/// of how words appear in definitions). The provided sentences are scanned for
/// topic-word pairs, yielding relation instances and potential new connector patterns.
pub fn discover_connectors_from_sentences(
    sentences: &[String],
    dictionary: &Dictionary,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> (Vec<Connector>, Vec<SentenceRelation>) {
    let (structural, content) = classify_word_roles(dictionary);
    let relations = extract_relations(sentences, dictionary, &structural, &content, params);
    let connectors = connector_pipeline(&relations, dictionary, params, strategy);
    (connectors, relations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yalm_parser::parse_dictionary;

    fn load_dict() -> Dictionary {
        let content = std::fs::read_to_string("../../dictionaries/dict5.md").unwrap();
        parse_dictionary(&content)
    }

    #[test]
    fn test_classify_word_roles() {
        let dict = load_dict();
        let (structural, content) = classify_word_roles(&dict);

        eprintln!("Structural ({}):", structural.len());
        let mut s: Vec<&String> = structural.iter().collect();
        s.sort();
        eprintln!("  {:?}", s);

        eprintln!("Content ({}):", content.len());
        let mut c: Vec<&String> = content.iter().collect();
        c.sort();
        eprintln!("  {:?}", c);

        // "is", "a", "the" should be structural
        assert!(structural.contains("is"), "Expected 'is' to be structural");
        assert!(structural.contains("a"), "Expected 'a' to be structural");
        assert!(structural.contains("the"), "Expected 'the' to be structural");

        // "dog", "cat" should be content
        assert!(content.contains("dog"), "Expected 'dog' to be content");
        assert!(content.contains("cat"), "Expected 'cat' to be content");
    }

    #[test]
    fn test_discover_at_least_5_connectors() {
        let dict = load_dict();
        let params = EngineParams::default();
        let strategy = StrategyConfig::default();
        let (connectors, _) = discover_connectors(&dict, &params, &strategy);

        eprintln!("Discovered {} connectors:", connectors.len());
        for c in &connectors {
            eprintln!("  {:?} (freq: {})", c.pattern, c.frequency);
        }

        assert!(
            connectors.len() >= 5,
            "Expected at least 5 connectors, found {}: {:?}",
            connectors.len(),
            connectors.iter().map(|c| &c.pattern).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_is_connector_discovered() {
        let dict = load_dict();
        let params = EngineParams::default();
        let strategy = StrategyConfig::default();
        let (connectors, _) = discover_connectors(&dict, &params, &strategy);

        let has_is = connectors.iter().any(|c| c.pattern == vec!["is".to_string()]);
        let has_is_a = connectors
            .iter()
            .any(|c| c.pattern == vec!["is".to_string(), "a".to_string()]);

        assert!(
            has_is || has_is_a,
            "Should discover 'is' or 'is a' connector. Found: {:?}",
            connectors.iter().map(|c| &c.pattern).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_can_connector_discovered() {
        let dict = load_dict();
        let params = EngineParams::default();
        let strategy = StrategyConfig::default();
        let (connectors, _) = discover_connectors(&dict, &params, &strategy);

        let has_can = connectors
            .iter()
            .any(|c| c.pattern.contains(&"can".to_string()));
        assert!(
            has_can,
            "Should discover 'can' connector. Found: {:?}",
            connectors.iter().map(|c| &c.pattern).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_negation_detected() {
        let dict = load_dict();
        let params = EngineParams::default();
        let strategy = StrategyConfig::default();
        let (_, relations) = discover_connectors(&dict, &params, &strategy);

        let negated: Vec<&SentenceRelation> = relations.iter().filter(|r| r.negated).collect();
        eprintln!("Negated relations ({}):", negated.len());
        for r in &negated {
            eprintln!(
                "  {} [{:?}] {} (from: {})",
                r.left_word,
                r.connector_pattern,
                r.right_word,
                r.source
            );
        }

        assert!(
            negated.len() >= 2,
            "Should find at least 2 negated relations, found {}",
            negated.len()
        );
    }

    #[test]
    fn test_discover_from_sentences_matches_original() {
        let dict = load_dict();
        let params = EngineParams::default();
        let strategy = StrategyConfig::default();

        let (orig_connectors, _) = discover_connectors(&dict, &params, &strategy);

        let sentences = extract_all_sentences(&dict);
        let (from_sentences, _) =
            discover_connectors_from_sentences(&sentences, &dict, &params, &strategy);

        // Same sentences should produce identical connectors
        assert_eq!(
            orig_connectors.len(),
            from_sentences.len(),
            "Connector count mismatch: original={}, from_sentences={}",
            orig_connectors.len(),
            from_sentences.len()
        );
        for (a, b) in orig_connectors.iter().zip(from_sentences.iter()) {
            assert_eq!(a.pattern, b.pattern, "Pattern mismatch");
            assert_eq!(a.frequency, b.frequency, "Frequency mismatch for {:?}", a.pattern);
        }
    }

    #[test]
    fn test_topic_threshold_scaling() {
        let cases: Vec<(usize, usize, usize)> = vec![
            (51, 10, 15),       // dict5: ~12, same as old 25%
            (100, 15, 30),      // small: slightly lower %
            (783, 50, 100),     // Ollama: ~71 (was 195)
            (1005, 60, 110),    // dict12: ~84 (was 251)
            (2008, 100, 180),   // dict18: ~136 (was 502)
        ];
        for (n, min_t, max_t) in cases {
            let nf = n as f64;
            let log_scale = (nf / 50.0_f64).ln().max(1.0);
            let threshold = (nf * 0.25 / log_scale) as usize;
            assert!(
                threshold >= min_t && threshold <= max_t,
                "n={}: threshold={} not in [{}, {}]",
                n, threshold, min_t, max_t
            );
        }
    }
}
