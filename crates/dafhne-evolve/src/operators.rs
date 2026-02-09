use dafhne_core::SimpleRng;

use crate::genome::*;

// ─── Mutation ───────────────────────────────────────────────────

/// Mutate a genome. Returns a new child genome.
pub fn mutate(
    genome: &Genome,
    ranges: &ParamRanges,
    mutation_rate: f64,
    strategy_mutation_rate: f64,
    rng: &mut SimpleRng,
    new_id: u64,
    generation: usize,
) -> Genome {
    let mut child = genome.clone();
    child.id = new_id;
    child.generation = generation;
    child.parent_ids = vec![genome.id];
    child.fitness = None;
    child.primary_fitness = None;
    child.cross_fitness = None;

    // Tier 1: Gaussian perturbation for f64 parameters
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.force_magnitude.1 - ranges.force_magnitude.0);
        child.params.force_magnitude =
            clamp_f64(child.params.force_magnitude + gaussian(rng) * sigma, ranges.force_magnitude.0, ranges.force_magnitude.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.force_decay.1 - ranges.force_decay.0);
        child.params.force_decay =
            clamp_f64(child.params.force_decay + gaussian(rng) * sigma, ranges.force_decay.0, ranges.force_decay.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.yes_threshold.1 - ranges.yes_threshold.0);
        child.params.yes_threshold =
            clamp_f64(child.params.yes_threshold + gaussian(rng) * sigma, ranges.yes_threshold.0, ranges.yes_threshold.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.no_threshold.1 - ranges.no_threshold.0);
        child.params.no_threshold =
            clamp_f64(child.params.no_threshold + gaussian(rng) * sigma, ranges.no_threshold.0, ranges.no_threshold.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.negation_inversion.1 - ranges.negation_inversion.0);
        child.params.negation_inversion =
            clamp_f64(child.params.negation_inversion + gaussian(rng) * sigma, ranges.negation_inversion.0, ranges.negation_inversion.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.bidirectional_force.1 - ranges.bidirectional_force.0);
        child.params.bidirectional_force =
            clamp_f64(child.params.bidirectional_force + gaussian(rng) * sigma, ranges.bidirectional_force.0, ranges.bidirectional_force.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.grammar_weight.1 - ranges.grammar_weight.0);
        child.params.grammar_weight =
            clamp_f64(child.params.grammar_weight + gaussian(rng) * sigma, ranges.grammar_weight.0, ranges.grammar_weight.1);
    }

    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.weighted_distance_alpha.1 - ranges.weighted_distance_alpha.0);
        child.params.weighted_distance_alpha =
            clamp_f64(child.params.weighted_distance_alpha + gaussian(rng) * sigma, ranges.weighted_distance_alpha.0, ranges.weighted_distance_alpha.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.uniformity_threshold.1 - ranges.uniformity_threshold.0);
        child.params.uniformity_threshold =
            clamp_f64(child.params.uniformity_threshold + gaussian(rng) * sigma, ranges.uniformity_threshold.0, ranges.uniformity_threshold.1);
    }

    // Tier 1: Integer perturbation for usize parameters
    if rng.next_f64() < mutation_rate {
        child.params.dimensions =
            mutate_usize(child.params.dimensions, ranges.dimensions, rng);
    }
    if rng.next_f64() < mutation_rate {
        child.params.learning_passes =
            mutate_usize(child.params.learning_passes, ranges.learning_passes, rng);
    }
    if rng.next_f64() < mutation_rate {
        child.params.connector_min_frequency =
            mutate_usize(child.params.connector_min_frequency, ranges.connector_min_frequency, rng);
    }
    if rng.next_f64() < mutation_rate {
        child.params.connector_max_length =
            mutate_usize(child.params.connector_max_length, ranges.connector_max_length, rng);
    }
    if rng.next_f64() < mutation_rate {
        child.params.max_follow_per_hop =
            mutate_usize(child.params.max_follow_per_hop, ranges.max_follow_per_hop, rng);
    }
    if rng.next_f64() < mutation_rate {
        child.params.max_chain_hops =
            mutate_usize(child.params.max_chain_hops, ranges.max_chain_hops, rng);
    }
    if rng.next_f64() < mutation_rate {
        child.params.uniformity_num_buckets =
            mutate_usize(child.params.uniformity_num_buckets, ranges.uniformity_num_buckets, rng);
    }

    // Tier 2: Strategy mutation
    child.force_function = child.force_function.mutate(rng, strategy_mutation_rate);
    child.connector_detection = child.connector_detection.mutate(rng, strategy_mutation_rate);
    child.space_init = child.space_init.mutate(rng, strategy_mutation_rate);
    child.multi_connector = child.multi_connector.mutate(rng, strategy_mutation_rate);
    child.negation_model = child.negation_model.mutate(rng, strategy_mutation_rate);
    if rng.next_f64() < strategy_mutation_rate {
        child.use_connector_axis = !child.use_connector_axis;
    }

    child
}

fn mutate_usize(value: usize, range: (usize, usize), rng: &mut SimpleRng) -> usize {
    let delta = (rng.next_u64() % 3) as isize + 1; // 1, 2, or 3
    let sign: isize = if rng.next_f64() < 0.5 { -1 } else { 1 };
    let new_val = (value as isize + sign * delta)
        .max(range.0 as isize)
        .min(range.1 as isize);
    new_val as usize
}

// ─── Crossover ──────────────────────────────────────────────────

/// Uniform crossover: each parameter from parent_a or parent_b with 50% probability.
pub fn crossover(
    parent_a: &Genome,
    parent_b: &Genome,
    rng: &mut SimpleRng,
    new_id: u64,
    generation: usize,
) -> Genome {
    let mut child = parent_a.clone();
    child.id = new_id;
    child.generation = generation;
    child.parent_ids = vec![parent_a.id, parent_b.id];
    child.fitness = None;
    child.primary_fitness = None;
    child.cross_fitness = None;

    // Tier 1: uniform crossover per parameter
    if rng.next_f64() < 0.5 {
        child.params.dimensions = parent_b.params.dimensions;
    }
    if rng.next_f64() < 0.5 {
        child.params.learning_passes = parent_b.params.learning_passes;
    }
    if rng.next_f64() < 0.5 {
        child.params.force_magnitude = parent_b.params.force_magnitude;
    }
    if rng.next_f64() < 0.5 {
        child.params.force_decay = parent_b.params.force_decay;
    }
    if rng.next_f64() < 0.5 {
        child.params.connector_min_frequency = parent_b.params.connector_min_frequency;
    }
    if rng.next_f64() < 0.5 {
        child.params.connector_max_length = parent_b.params.connector_max_length;
    }
    if rng.next_f64() < 0.5 {
        child.params.yes_threshold = parent_b.params.yes_threshold;
    }
    if rng.next_f64() < 0.5 {
        child.params.no_threshold = parent_b.params.no_threshold;
    }
    if rng.next_f64() < 0.5 {
        child.params.negation_inversion = parent_b.params.negation_inversion;
    }
    if rng.next_f64() < 0.5 {
        child.params.bidirectional_force = parent_b.params.bidirectional_force;
    }
    if rng.next_f64() < 0.5 {
        child.params.grammar_weight = parent_b.params.grammar_weight;
    }
    if rng.next_f64() < 0.5 {
        child.params.max_follow_per_hop = parent_b.params.max_follow_per_hop;
    }
    if rng.next_f64() < 0.5 {
        child.params.max_chain_hops = parent_b.params.max_chain_hops;
    }
    if rng.next_f64() < 0.5 {
        child.params.weighted_distance_alpha = parent_b.params.weighted_distance_alpha;
    }
    if rng.next_f64() < 0.5 {
        child.params.uniformity_num_buckets = parent_b.params.uniformity_num_buckets;
    }
    if rng.next_f64() < 0.5 {
        child.params.uniformity_threshold = parent_b.params.uniformity_threshold;
    }

    // Tier 2: uniform crossover per strategy
    if rng.next_f64() < 0.5 {
        child.force_function = parent_b.force_function;
    }
    if rng.next_f64() < 0.5 {
        child.connector_detection = parent_b.connector_detection;
    }
    if rng.next_f64() < 0.5 {
        child.space_init = parent_b.space_init;
    }
    if rng.next_f64() < 0.5 {
        child.multi_connector = parent_b.multi_connector;
    }
    if rng.next_f64() < 0.5 {
        child.negation_model = parent_b.negation_model;
    }
    if rng.next_f64() < 0.5 {
        child.use_connector_axis = parent_b.use_connector_axis;
    }

    child
}

// ─── Selection ──────────────────────────────────────────────────

/// Tournament selection: pick tournament_size random individuals, return the best.
pub fn tournament_select<'a>(
    population: &'a [Genome],
    tournament_size: usize,
    rng: &mut SimpleRng,
) -> &'a Genome {
    let mut best: Option<&Genome> = None;
    for _ in 0..tournament_size {
        let idx = rng.next_u64() as usize % population.len();
        let candidate = &population[idx];
        let better = match best {
            None => true,
            Some(b) => candidate.fitness.unwrap_or(0.0) > b.fitness.unwrap_or(0.0),
        };
        if better {
            best = Some(candidate);
        }
    }
    best.unwrap()
}

// ─── Helpers ────────────────────────────────────────────────────

/// Box-Muller transform: convert two uniform [0,1] samples into a standard normal sample.
fn gaussian(rng: &mut SimpleRng) -> f64 {
    let u1 = rng.next_f64().max(1e-10);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

fn clamp_f64(val: f64, min: f64, max: f64) -> f64 {
    val.max(min).min(max)
}
