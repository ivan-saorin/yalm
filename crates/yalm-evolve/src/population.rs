use yalm_core::{EngineParams, SimpleRng};

use crate::genome::*;

/// Generate the initial random population.
pub fn initialize_population(
    size: usize,
    ranges: &ParamRanges,
    rng: &mut SimpleRng,
    generation: usize,
    id_counter: &mut u64,
) -> Vec<Genome> {
    let mut population = Vec::with_capacity(size);
    for _ in 0..size {
        *id_counter += 1;
        population.push(random_genome(ranges, rng, generation, *id_counter));
    }
    population
}

fn random_genome(
    ranges: &ParamRanges,
    rng: &mut SimpleRng,
    generation: usize,
    id: u64,
) -> Genome {
    let params = EngineParams {
        dimensions: random_usize_range(rng, ranges.dimensions.0, ranges.dimensions.1),
        learning_passes: random_usize_range(
            rng,
            ranges.learning_passes.0,
            ranges.learning_passes.1,
        ),
        force_magnitude: random_f64_range(
            rng,
            ranges.force_magnitude.0,
            ranges.force_magnitude.1,
        ),
        force_decay: random_f64_range(rng, ranges.force_decay.0, ranges.force_decay.1),
        connector_min_frequency: random_usize_range(
            rng,
            ranges.connector_min_frequency.0,
            ranges.connector_min_frequency.1,
        ),
        connector_max_length: random_usize_range(
            rng,
            ranges.connector_max_length.0,
            ranges.connector_max_length.1,
        ),
        yes_threshold: random_f64_range(rng, ranges.yes_threshold.0, ranges.yes_threshold.1),
        no_threshold: random_f64_range(rng, ranges.no_threshold.0, ranges.no_threshold.1),
        negation_inversion: random_f64_range(
            rng,
            ranges.negation_inversion.0,
            ranges.negation_inversion.1,
        ),
        bidirectional_force: random_f64_range(
            rng,
            ranges.bidirectional_force.0,
            ranges.bidirectional_force.1,
        ),
        grammar_weight: random_f64_range(
            rng,
            ranges.grammar_weight.0,
            ranges.grammar_weight.1,
        ),
        max_follow_per_hop: random_usize_range(
            rng,
            ranges.max_follow_per_hop.0,
            ranges.max_follow_per_hop.1,
        ),
        max_chain_hops: random_usize_range(
            rng,
            ranges.max_chain_hops.0,
            ranges.max_chain_hops.1,
        ),
        weighted_distance_alpha: random_f64_range(
            rng,
            ranges.weighted_distance_alpha.0,
            ranges.weighted_distance_alpha.1,
        ),
        uniformity_num_buckets: random_usize_range(
            rng,
            ranges.uniformity_num_buckets.0,
            ranges.uniformity_num_buckets.1,
        ),
        uniformity_threshold: random_f64_range(
            rng,
            ranges.uniformity_threshold.0,
            ranges.uniformity_threshold.1,
        ),
        rng_seed: 0, // overridden by to_engine_params()
    };

    Genome {
        params,
        force_function: ForceFunction::random(rng),
        connector_detection: ConnectorDetection::random(rng),
        space_init: SpaceInitialization::random(rng),
        multi_connector: MultiConnectorHandling::random(rng),
        negation_model: NegationModel::random(rng),
        use_connector_axis: rng.next_f64() < 0.5,
        id,
        generation,
        parent_ids: vec![],
        fitness: None,
        primary_fitness: None,
        cross_fitness: None,
    }
}

pub fn random_usize_range(rng: &mut SimpleRng, min: usize, max: usize) -> usize {
    if max <= min {
        return min;
    }
    min + (rng.next_u64() as usize) % (max - min + 1)
}

pub fn random_f64_range(rng: &mut SimpleRng, min: f64, max: f64) -> f64 {
    min + rng.next_f64() * (max - min)
}
