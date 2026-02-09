use std::collections::HashMap;
use dafhne_core::*;

use crate::strategy::{
    ForceFunction, MultiConnectorHandling, NegationModel, SpaceInitialization, StrategyConfig,
};

/// Build the geometric space by initializing word positions and applying forces.
pub fn build_space(
    dictionary: &Dictionary,
    connectors: &[Connector],
    relations: &[SentenceRelation],
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> GeometricSpace {
    let mut words = initialize_positions(dictionary, connectors, params, strategy);

    // Build lookup from connector pattern to connector index
    let connector_lookup: HashMap<Vec<String>, usize> = connectors
        .iter()
        .enumerate()
        .map(|(i, c)| (c.pattern.clone(), i))
        .collect();

    // For Weighted strategy: find max frequency for normalization
    let max_frequency = connectors.iter().map(|c| c.frequency).max().unwrap_or(1) as f64;

    // Multiple learning passes with decaying force magnitude
    let mut current_magnitude = params.force_magnitude;

    for _pass in 0..params.learning_passes {
        match strategy.multi_connector {
            MultiConnectorHandling::Sequential => {
                // Original: process each relation independently
                for relation in relations {
                    let connector_idx = match connector_lookup.get(&relation.connector_pattern) {
                        Some(idx) => *idx,
                        None => continue,
                    };
                    let connector = &connectors[connector_idx];

                    if !words.contains_key(&relation.left_word)
                        || !words.contains_key(&relation.right_word)
                    {
                        continue;
                    }

                    apply_force(
                        &mut words,
                        &relation.left_word,
                        &relation.right_word,
                        &connector.force_direction,
                        current_magnitude * relation.weight,
                        relation.negated,
                        params,
                        strategy,
                    );
                }
            }
            MultiConnectorHandling::FirstOnly => {
                // Only apply force for the highest-frequency connector per word pair per pass
                let mut applied_pairs: HashMap<(String, String), usize> = HashMap::new();

                for relation in relations {
                    let connector_idx = match connector_lookup.get(&relation.connector_pattern) {
                        Some(idx) => *idx,
                        None => continue,
                    };
                    let connector = &connectors[connector_idx];

                    if !words.contains_key(&relation.left_word)
                        || !words.contains_key(&relation.right_word)
                    {
                        continue;
                    }

                    let pair = (relation.left_word.clone(), relation.right_word.clone());
                    let prev_freq = applied_pairs.get(&pair).copied().unwrap_or(0);
                    if connector.frequency < prev_freq {
                        continue; // Skip lower-frequency connector for this pair
                    }
                    applied_pairs.insert(pair, connector.frequency);

                    apply_force(
                        &mut words,
                        &relation.left_word,
                        &relation.right_word,
                        &connector.force_direction,
                        current_magnitude * relation.weight,
                        relation.negated,
                        params,
                        strategy,
                    );
                }
            }
            MultiConnectorHandling::Weighted => {
                // Scale force by connector.frequency / max_frequency
                for relation in relations {
                    let connector_idx = match connector_lookup.get(&relation.connector_pattern) {
                        Some(idx) => *idx,
                        None => continue,
                    };
                    let connector = &connectors[connector_idx];

                    if !words.contains_key(&relation.left_word)
                        || !words.contains_key(&relation.right_word)
                    {
                        continue;
                    }

                    let freq_weight = connector.frequency as f64 / max_frequency;
                    apply_force(
                        &mut words,
                        &relation.left_word,
                        &relation.right_word,
                        &connector.force_direction,
                        current_magnitude * freq_weight * relation.weight,
                        relation.negated,
                        params,
                        strategy,
                    );
                }
            }
            MultiConnectorHandling::Compositional => {
                // Average all connector directions for the same word pair, apply once
                // Tuple: (weighted direction sum, total weight, negated)
                let mut pair_directions: HashMap<(String, String), (Vec<f64>, f64, bool)> =
                    HashMap::new();

                for relation in relations {
                    let _connector_idx = match connector_lookup.get(&relation.connector_pattern) {
                        Some(idx) => *idx,
                        None => continue,
                    };
                    let connector = &connectors[_connector_idx];

                    if !words.contains_key(&relation.left_word)
                        || !words.contains_key(&relation.right_word)
                    {
                        continue;
                    }

                    let pair = (relation.left_word.clone(), relation.right_word.clone());
                    let entry = pair_directions
                        .entry(pair)
                        .or_insert_with(|| (vec![0.0; params.dimensions], 0.0, false));
                    for (d, c) in entry.0.iter_mut().zip(connector.force_direction.iter()) {
                        *d += c * relation.weight;
                    }
                    entry.1 += relation.weight;
                    if relation.negated {
                        entry.2 = true; // any negation in the group triggers negation
                    }
                }

                // Average and apply
                for ((left, right), (mut direction, total_weight, negated)) in pair_directions {
                    if total_weight > 1e-10 {
                        for d in &mut direction {
                            *d /= total_weight;
                        }
                    }
                    // Normalize the averaged direction
                    let norm: f64 = direction.iter().map(|d| d * d).sum::<f64>().sqrt();
                    if norm > 1e-10 {
                        for d in &mut direction {
                            *d /= norm;
                        }
                    }

                    apply_force(
                        &mut words,
                        &left,
                        &right,
                        &direction,
                        current_magnitude,
                        negated,
                        params,
                        strategy,
                    );
                }
            }
        }

        current_magnitude *= params.force_decay;
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

/// Initialize all entry words at positions in N-dimensional space.
fn initialize_positions(
    dictionary: &Dictionary,
    connectors: &[Connector],
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> HashMap<String, WordPoint> {
    let mut rng = SimpleRng::new(params.rng_seed.wrapping_add(1000));
    let mut words = HashMap::new();

    match strategy.space_init {
        SpaceInitialization::Random => {
            // Original: uniform random [-1, 1] per dimension
            for entry_word in &dictionary.entry_words {
                let position: Vec<f64> =
                    (0..params.dimensions).map(|_| rng.next_f64_signed()).collect();
                words.insert(
                    entry_word.clone(),
                    WordPoint {
                        word: entry_word.clone(),
                        position,
                    },
                );
            }
        }
        SpaceInitialization::Spherical => {
            // Random position normalized to unit sphere
            for entry_word in &dictionary.entry_words {
                let position: Vec<f64> =
                    (0..params.dimensions).map(|_| rng.next_f64_signed()).collect();
                let norm: f64 = position.iter().map(|x| x * x).sum::<f64>().sqrt();
                let position = if norm > 1e-10 {
                    position.iter().map(|x| x / norm).collect()
                } else {
                    position
                };
                words.insert(
                    entry_word.clone(),
                    WordPoint {
                        word: entry_word.clone(),
                        position,
                    },
                );
            }
        }
        SpaceInitialization::FromConnectors => {
            // Start near origin, offset based on which connectors mention this word
            // Words sharing connectors start closer together
            let mut word_connector_membership: HashMap<String, Vec<usize>> = HashMap::new();
            // We use the relations indirectly through connectors: for each connector,
            // check if a word appears in its pattern (structural words)
            for (i, connector) in connectors.iter().enumerate() {
                for pat_word in &connector.pattern {
                    word_connector_membership
                        .entry(pat_word.clone())
                        .or_default()
                        .push(i);
                }
            }

            for entry_word in &dictionary.entry_words {
                let mut position: Vec<f64> =
                    (0..params.dimensions).map(|_| rng.next_f64_signed() * 0.1).collect();

                // Add small offset in the direction of connectors this word is associated with
                if let Some(conn_indices) = word_connector_membership.get(entry_word) {
                    for &ci in conn_indices {
                        if ci < connectors.len() {
                            for (p, d) in
                                position.iter_mut().zip(connectors[ci].force_direction.iter())
                            {
                                *p += d * 0.3;
                            }
                        }
                    }
                }

                words.insert(
                    entry_word.clone(),
                    WordPoint {
                        word: entry_word.clone(),
                        position,
                    },
                );
            }
        }
    }

    words
}

/// Apply a force between two words along a connector's axis.
///
/// The force ATTRACTS the left word toward the right word, projected onto the
/// connector's axis. This means words connected by the same connector pattern
/// are pulled together along that specific dimension.
/// If negated, the behavior depends on the NegationModel strategy.
pub fn apply_force(
    words: &mut HashMap<String, WordPoint>,
    left_word: &str,
    right_word: &str,
    connector_direction: &[f64],
    magnitude: f64,
    negated: bool,
    params: &EngineParams,
    strategy: &StrategyConfig,
) {
    // Get positions (need to clone to avoid borrow issues)
    let left_pos = words.get(left_word).unwrap().position.clone();
    let right_pos = words.get(right_word).unwrap().position.clone();

    // Compute displacement from left to right
    let displacement: Vec<f64> = left_pos
        .iter()
        .zip(right_pos.iter())
        .map(|(l, r)| r - l)
        .collect();

    let euclidean_dist: f64 = displacement.iter().map(|d| d * d).sum::<f64>().sqrt();

    // Handle negation model
    match strategy.negation_model {
        NegationModel::Inversion => {
            // Original behavior: sign = negation_inversion for negated, 1.0 for non-negated
            let sign = if negated {
                params.negation_inversion
            } else {
                1.0
            };
            apply_force_with_sign(
                words,
                left_word,
                right_word,
                connector_direction,
                &displacement,
                euclidean_dist,
                magnitude,
                sign,
                params,
                &strategy.force_function,
            );
        }
        NegationModel::Repulsion => {
            // sign = -1.0, doubled magnitude for negated
            let (sign, mag) = if negated {
                (-1.0, magnitude * 2.0)
            } else {
                (1.0, magnitude)
            };
            apply_force_with_sign(
                words,
                left_word,
                right_word,
                connector_direction,
                &displacement,
                euclidean_dist,
                mag,
                sign,
                params,
                &strategy.force_function,
            );
        }
        NegationModel::AxisShift => {
            if negated {
                // Rotate connector direction 90 degrees in the plane of the first two nonzero components
                let mut rotated = connector_direction.to_vec();
                // Find the two largest components
                let mut indices: Vec<(usize, f64)> = connector_direction
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| (i, v.abs()))
                    .collect();
                indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                if indices.len() >= 2 {
                    let i0 = indices[0].0;
                    let i1 = indices[1].0;
                    // 90-degree rotation in the plane of (i0, i1)
                    let a = connector_direction[i0];
                    let b = connector_direction[i1];
                    rotated[i0] = -b;
                    rotated[i1] = a;
                }

                // Normalize the rotated direction
                let norm: f64 = rotated.iter().map(|x| x * x).sum::<f64>().sqrt();
                if norm > 1e-10 {
                    for d in &mut rotated {
                        *d /= norm;
                    }
                }

                apply_force_with_sign(
                    words,
                    left_word,
                    right_word,
                    &rotated,
                    &displacement,
                    euclidean_dist,
                    magnitude,
                    1.0, // Attract along orthogonal axis
                    params,
                    &strategy.force_function,
                );
            } else {
                apply_force_with_sign(
                    words,
                    left_word,
                    right_word,
                    connector_direction,
                    &displacement,
                    euclidean_dist,
                    magnitude,
                    1.0,
                    params,
                    &strategy.force_function,
                );
            }
        }
        NegationModel::SeparateDimension => {
            if negated {
                // Project only onto dimension 0 (the negation dimension) and push apart
                let mut neg_direction = vec![0.0; connector_direction.len()];
                if !neg_direction.is_empty() {
                    neg_direction[0] = 1.0;
                }
                apply_force_with_sign(
                    words,
                    left_word,
                    right_word,
                    &neg_direction,
                    &displacement,
                    euclidean_dist,
                    magnitude,
                    -1.0, // Push apart on negation dimension
                    params,
                    &strategy.force_function,
                );
            } else {
                // Zero out dimension 0 from connector direction for non-negated
                let mut adjusted_dir = connector_direction.to_vec();
                if !adjusted_dir.is_empty() {
                    adjusted_dir[0] = 0.0;
                }
                // Re-normalize
                let norm: f64 = adjusted_dir.iter().map(|x| x * x).sum::<f64>().sqrt();
                if norm > 1e-10 {
                    for d in &mut adjusted_dir {
                        *d /= norm;
                    }
                }

                apply_force_with_sign(
                    words,
                    left_word,
                    right_word,
                    &adjusted_dir,
                    &displacement,
                    euclidean_dist,
                    magnitude,
                    1.0,
                    params,
                    &strategy.force_function,
                );
            }
        }
    }
}

/// Core force application with a given sign and force function strategy.
fn apply_force_with_sign(
    words: &mut HashMap<String, WordPoint>,
    left_word: &str,
    right_word: &str,
    direction: &[f64],
    displacement: &[f64],
    euclidean_dist: f64,
    magnitude: f64,
    sign: f64,
    params: &EngineParams,
    force_function: &ForceFunction,
) {
    // Project displacement onto the connector axis
    let projection_scalar: f64 = displacement
        .iter()
        .zip(direction.iter())
        .map(|(d, c)| d * c)
        .sum();

    // Compute force based on force function
    let force: Vec<f64> = match force_function {
        ForceFunction::Linear => {
            // Original: force = direction * projection * magnitude * sign
            direction
                .iter()
                .map(|c| c * projection_scalar * magnitude * sign)
                .collect()
        }
        ForceFunction::InverseDistance => {
            // Scale by 1/(1 + |projection|) — stronger when close, prevents runaway
            let scale = 1.0 / (1.0 + projection_scalar.abs());
            direction
                .iter()
                .map(|c| c * projection_scalar * magnitude * sign * scale)
                .collect()
        }
        ForceFunction::Gravitational => {
            // Inverse-square: force direction * magnitude * sign / max(distance^2, 0.01)
            let dist_sq = euclidean_dist * euclidean_dist;
            let scale = 1.0 / dist_sq.max(0.01);
            direction
                .iter()
                .map(|c| c * magnitude * sign * scale)
                .collect()
        }
        ForceFunction::Spring => {
            // Equilibrium at projection = 0.5: force ~ (projection - 0.5)
            let spring_force = projection_scalar - 0.5;
            direction
                .iter()
                .map(|c| c * spring_force * magnitude * sign)
                .collect()
        }
    };

    // Move left word toward right along connector axis
    if let Some(left_wp) = words.get_mut(left_word) {
        for (p, f) in left_wp.position.iter_mut().zip(force.iter()) {
            *p += f;
        }
    }

    // Move right word toward left (weaker reverse)
    if let Some(right_wp) = words.get_mut(right_word) {
        for (p, f) in right_wp.position.iter_mut().zip(force.iter()) {
            *p -= f * params.bidirectional_force;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector_discovery::discover_connectors;
    use crate::strategy::StrategyConfig;
    use dafhne_parser::parse_dictionary;

    fn load_and_build() -> GeometricSpace {
        let content = std::fs::read_to_string("../../dictionaries/dict5.md").unwrap();
        let dict = parse_dictionary(&content);
        let params = EngineParams::default();
        let strategy = StrategyConfig::default();
        let (connectors, relations) = discover_connectors(&dict, &params, &strategy);
        build_space(&dict, &connectors, &relations, &params, &strategy)
    }

    #[test]
    fn test_space_has_all_words() {
        let space = load_and_build();
        assert_eq!(space.words.len(), 51); // dict5 has 51 entries
    }

    #[test]
    fn test_positions_are_finite() {
        let space = load_and_build();
        for (word, wp) in &space.words {
            for (i, val) in wp.position.iter().enumerate() {
                assert!(
                    val.is_finite(),
                    "Word '{}' has non-finite value at dim {}: {}",
                    word,
                    i,
                    val
                );
            }
        }
    }

    #[test]
    fn test_related_words_proximity() {
        let space = load_and_build();

        let dog_animal = euclidean_distance(
            &space.words["dog"].position,
            &space.words["animal"].position,
        );
        let dog_sun = euclidean_distance(
            &space.words["dog"].position,
            &space.words["sun"].position,
        );

        eprintln!("dog-animal: {:.4}", dog_animal);
        eprintln!("dog-sun:    {:.4}", dog_sun);
        eprintln!(
            "cat-animal: {:.4}",
            euclidean_distance(
                &space.words["cat"].position,
                &space.words["animal"].position,
            )
        );
        eprintln!(
            "hot-cold:   {:.4}",
            euclidean_distance(
                &space.words["hot"].position,
                &space.words["cold"].position,
            )
        );

        // dog should be closer to animal than to sun (not a hard assert for v0.1)
        eprintln!(
            "dog closer to animal than sun: {}",
            dog_animal < dog_sun
        );
    }
}
