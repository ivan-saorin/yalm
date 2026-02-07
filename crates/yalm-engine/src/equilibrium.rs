//! Sequential Equilibrium: incremental space construction.
//!
//! Words are placed one at a time as definitions are read.
//! Connector discovery is batch (runs first on full corpus).
//! Space construction is sequential per-definition, with local
//! relaxation after each placement.

use std::collections::HashMap;
use yalm_core::*;
use yalm_parser::{stem_to_entry, tokenize};

use crate::force_field::apply_force;
use crate::strategy::StrategyConfig;

// ─── Parameters ─────────────────────────────────────────────────

/// Fixed parameters for sequential equilibrium (not evolved by GA).
#[derive(Debug, Clone)]
pub struct EquilibriumParams {
    /// Noise magnitude added to centroid initialization.
    pub perturbation_strength: f64,
    /// Per-step force decay during local relaxation.
    pub damping: f64,
    /// Maximum relaxation iterations per entry placement.
    pub max_relax_steps: usize,
    /// Stop relaxation when total energy drops below this.
    pub energy_threshold: f64,
    /// Base force magnitude (replaces EngineParams.force_magnitude).
    pub learning_rate: f64,
    /// Number of full re-read passes over all entries.
    pub passes: usize,
    /// Whether to shuffle entry order between passes.
    pub shuffle_between_passes: bool,
}

impl Default for EquilibriumParams {
    fn default() -> Self {
        Self {
            perturbation_strength: 0.1,
            damping: 0.95,
            max_relax_steps: 20,
            energy_threshold: 0.001,
            learning_rate: 0.05,
            passes: 3,
            shuffle_between_passes: true,
        }
    }
}

// ─── Builder ────────────────────────────────────────────────────

/// Build geometric space using sequential equilibrium.
///
/// Algorithm:
/// 1. Connectors and relations are pre-computed (batch).
/// 2. For each pass (with decaying learning rate):
///    a. For each dictionary entry (shuffled after first pass):
///       i.  Initialize word at centroid of placed definition words + noise.
///       ii. Apply forces from this entry's relations.
///       iii. Local relaxation: settle neighbors until energy < threshold.
///    b. Apply grammar relations as batch regularizer.
pub fn build_space_equilibrium(
    dictionary: &Dictionary,
    connectors: &[Connector],
    dict_relations: &[SentenceRelation],
    grammar_relations: &[SentenceRelation],
    params: &EngineParams,
    strategy: &StrategyConfig,
    eq_params: &EquilibriumParams,
    quiet: bool,
) -> GeometricSpace {
    let mut rng = SimpleRng::new(params.rng_seed.wrapping_add(2000));

    // Build connector lookup
    let connector_lookup: HashMap<Vec<String>, usize> = connectors
        .iter()
        .enumerate()
        .map(|(i, c)| (c.pattern.clone(), i))
        .collect();

    // Group dict_relations by entry word (as left_word)
    let mut entry_relations: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, rel) in dict_relations.iter().enumerate() {
        if dictionary.entry_set.contains(&rel.left_word) {
            entry_relations
                .entry(rel.left_word.clone())
                .or_default()
                .push(idx);
        }
    }

    // Group dict_relations by word involvement (left OR right) for relaxation
    let mut word_relations: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, rel) in dict_relations.iter().enumerate() {
        word_relations
            .entry(rel.left_word.clone())
            .or_default()
            .push(idx);
        if rel.right_word != rel.left_word {
            word_relations
                .entry(rel.right_word.clone())
                .or_default()
                .push(idx);
        }
    }

    let mut words: HashMap<String, WordPoint> = HashMap::new();

    // Entry processing order (indices into dictionary.entries)
    let mut entry_order: Vec<usize> = (0..dictionary.entries.len()).collect();

    for pass in 0..eq_params.passes {
        let lr = eq_params.learning_rate / (1.0 + pass as f64 * 0.5);

        // Shuffle after first pass
        if pass > 0 && eq_params.shuffle_between_passes {
            fisher_yates_shuffle(&mut entry_order, &mut rng);
        }

        for &entry_idx in &entry_order {
            let entry = &dictionary.entries[entry_idx];
            let word = &entry.word;

            // 1. INITIALIZE if not yet placed
            if !words.contains_key(word) {
                let position = initialize_word_position(
                    entry,
                    &words,
                    &dictionary.entry_set,
                    params.dimensions,
                    eq_params.perturbation_strength,
                    &mut rng,
                );
                words.insert(
                    word.clone(),
                    WordPoint {
                        word: word.clone(),
                        position,
                    },
                );
            }

            // 2. APPLY FORCES from this entry's relations
            if let Some(rel_indices) = entry_relations.get(word) {
                for &idx in rel_indices {
                    let rel = &dict_relations[idx];
                    let conn_idx = match connector_lookup.get(&rel.connector_pattern) {
                        Some(i) => *i,
                        None => continue,
                    };
                    let connector = &connectors[conn_idx];

                    if !words.contains_key(&rel.left_word)
                        || !words.contains_key(&rel.right_word)
                    {
                        continue;
                    }

                    apply_force(
                        &mut words,
                        &rel.left_word,
                        &rel.right_word,
                        &connector.force_direction,
                        lr * rel.weight,
                        rel.negated,
                        params,
                        strategy,
                    );
                }
            }

            // 3. LOCAL RELAXATION — settle neighbors
            if let Some(rel_indices) = word_relations.get(word) {
                for step in 0..eq_params.max_relax_steps {
                    let step_lr = lr * eq_params.damping.powi(step as i32 + 1);
                    let mut energy = 0.0;

                    for &idx in rel_indices {
                        let rel = &dict_relations[idx];
                        if !words.contains_key(&rel.left_word)
                            || !words.contains_key(&rel.right_word)
                        {
                            continue;
                        }

                        // Compute energy contribution (squared displacement)
                        let left_pos = &words[&rel.left_word].position;
                        let right_pos = &words[&rel.right_word].position;
                        let disp_sq: f64 = left_pos
                            .iter()
                            .zip(right_pos.iter())
                            .map(|(l, r)| (r - l) * (r - l))
                            .sum();
                        energy += disp_sq;

                        let conn_idx = match connector_lookup.get(&rel.connector_pattern) {
                            Some(i) => *i,
                            None => continue,
                        };
                        let connector = &connectors[conn_idx];

                        apply_force(
                            &mut words,
                            &rel.left_word,
                            &rel.right_word,
                            &connector.force_direction,
                            step_lr * rel.weight,
                            rel.negated,
                            params,
                            strategy,
                        );
                    }

                    if energy < eq_params.energy_threshold {
                        break;
                    }
                }
            }
        }

        // Grammar batch pass after all entries
        for rel in grammar_relations {
            let conn_idx = match connector_lookup.get(&rel.connector_pattern) {
                Some(i) => *i,
                None => continue,
            };
            let connector = &connectors[conn_idx];

            if !words.contains_key(&rel.left_word) || !words.contains_key(&rel.right_word) {
                continue;
            }

            apply_force(
                &mut words,
                &rel.left_word,
                &rel.right_word,
                &connector.force_direction,
                lr * rel.weight,
                rel.negated,
                params,
                strategy,
            );
        }

        if !quiet {
            // Compute pass energy for logging
            let total_energy: f64 = dict_relations
                .iter()
                .filter(|r| words.contains_key(&r.left_word) && words.contains_key(&r.right_word))
                .map(|r| {
                    let lp = &words[&r.left_word].position;
                    let rp = &words[&r.right_word].position;
                    lp.iter()
                        .zip(rp.iter())
                        .map(|(l, r)| (r - l) * (r - l))
                        .sum::<f64>()
                })
                .sum();
            eprintln!(
                "  Equilibrium pass {}: {} words placed, energy={:.4}, lr={:.4}",
                pass,
                words.len(),
                total_energy,
                lr
            );
        }
    }

    let mut space = GeometricSpace {
        dimensions: params.dimensions,
        words,
        connectors: connectors.to_vec(),
        distance_stats: None,
    };
    space.compute_distance_stats();
    space
}

// ─── Helpers ────────────────────────────────────────────────────

/// Initialize a word's position at the centroid of already-placed definition
/// words, plus small random noise. Falls back to random small values if no
/// definition words are placed yet.
fn initialize_word_position(
    entry: &DictionaryEntry,
    placed_words: &HashMap<String, WordPoint>,
    entry_set: &std::collections::HashSet<String>,
    dimensions: usize,
    noise_scale: f64,
    rng: &mut SimpleRng,
) -> Vec<f64> {
    // Collect all words from definition + examples that are already placed
    let mut placed_positions: Vec<&Vec<f64>> = Vec::new();

    // Tokenize definition
    let def_tokens = tokenize(&entry.definition);
    for token in &def_tokens {
        if let Some(base) = stem_to_entry(token, entry_set) {
            if base != entry.word {
                if let Some(wp) = placed_words.get(&base) {
                    placed_positions.push(&wp.position);
                }
            }
        }
    }

    // Tokenize examples
    for example in &entry.examples {
        let ex_tokens = tokenize(example);
        for token in &ex_tokens {
            if let Some(base) = stem_to_entry(token, entry_set) {
                if base != entry.word {
                    if let Some(wp) = placed_words.get(&base) {
                        placed_positions.push(&wp.position);
                    }
                }
            }
        }
    }

    if placed_positions.is_empty() {
        // No context — random small initialization
        (0..dimensions)
            .map(|_| rng.next_f64_signed() * noise_scale)
            .collect()
    } else {
        // Centroid of placed definition words + noise
        let n = placed_positions.len() as f64;
        (0..dimensions)
            .map(|d| {
                let sum: f64 = placed_positions.iter().map(|pos| pos[d]).sum();
                sum / n + rng.next_f64_signed() * noise_scale
            })
            .collect()
    }
}

/// Fisher-Yates shuffle using SimpleRng.
fn fisher_yates_shuffle(slice: &mut [usize], rng: &mut SimpleRng) {
    let n = slice.len();
    for i in (1..n).rev() {
        let j = (rng.next_u64() as usize) % (i + 1);
        slice.swap(i, j);
    }
}
