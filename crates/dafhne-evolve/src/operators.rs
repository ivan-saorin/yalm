use std::collections::HashMap;
use dafhne_core::{EngineParams, SimpleRng};

use crate::genome::*;

// ─── Shared Helpers ─────────────────────────────────────────────

/// Mutate EngineParams in-place. Shared by single-space and multi-space operators.
fn mutate_params(
    params: &mut EngineParams,
    ranges: &ParamRanges,
    mutation_rate: f64,
    rng: &mut SimpleRng,
) {
    // f64 parameters: Gaussian perturbation
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.force_magnitude.1 - ranges.force_magnitude.0);
        params.force_magnitude =
            clamp_f64(params.force_magnitude + gaussian(rng) * sigma, ranges.force_magnitude.0, ranges.force_magnitude.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.force_decay.1 - ranges.force_decay.0);
        params.force_decay =
            clamp_f64(params.force_decay + gaussian(rng) * sigma, ranges.force_decay.0, ranges.force_decay.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.yes_threshold.1 - ranges.yes_threshold.0);
        params.yes_threshold =
            clamp_f64(params.yes_threshold + gaussian(rng) * sigma, ranges.yes_threshold.0, ranges.yes_threshold.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.no_threshold.1 - ranges.no_threshold.0);
        params.no_threshold =
            clamp_f64(params.no_threshold + gaussian(rng) * sigma, ranges.no_threshold.0, ranges.no_threshold.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.negation_inversion.1 - ranges.negation_inversion.0);
        params.negation_inversion =
            clamp_f64(params.negation_inversion + gaussian(rng) * sigma, ranges.negation_inversion.0, ranges.negation_inversion.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.bidirectional_force.1 - ranges.bidirectional_force.0);
        params.bidirectional_force =
            clamp_f64(params.bidirectional_force + gaussian(rng) * sigma, ranges.bidirectional_force.0, ranges.bidirectional_force.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.grammar_weight.1 - ranges.grammar_weight.0);
        params.grammar_weight =
            clamp_f64(params.grammar_weight + gaussian(rng) * sigma, ranges.grammar_weight.0, ranges.grammar_weight.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.weighted_distance_alpha.1 - ranges.weighted_distance_alpha.0);
        params.weighted_distance_alpha =
            clamp_f64(params.weighted_distance_alpha + gaussian(rng) * sigma, ranges.weighted_distance_alpha.0, ranges.weighted_distance_alpha.1);
    }
    if rng.next_f64() < mutation_rate {
        let sigma = 0.1 * (ranges.uniformity_threshold.1 - ranges.uniformity_threshold.0);
        params.uniformity_threshold =
            clamp_f64(params.uniformity_threshold + gaussian(rng) * sigma, ranges.uniformity_threshold.0, ranges.uniformity_threshold.1);
    }

    // usize parameters: ±1-3 perturbation
    if rng.next_f64() < mutation_rate {
        params.dimensions = mutate_usize(params.dimensions, ranges.dimensions, rng);
    }
    if rng.next_f64() < mutation_rate {
        params.learning_passes = mutate_usize(params.learning_passes, ranges.learning_passes, rng);
    }
    if rng.next_f64() < mutation_rate {
        params.connector_min_frequency = mutate_usize(params.connector_min_frequency, ranges.connector_min_frequency, rng);
    }
    if rng.next_f64() < mutation_rate {
        params.connector_max_length = mutate_usize(params.connector_max_length, ranges.connector_max_length, rng);
    }
    if rng.next_f64() < mutation_rate {
        params.max_follow_per_hop = mutate_usize(params.max_follow_per_hop, ranges.max_follow_per_hop, rng);
    }
    if rng.next_f64() < mutation_rate {
        params.max_chain_hops = mutate_usize(params.max_chain_hops, ranges.max_chain_hops, rng);
    }
    if rng.next_f64() < mutation_rate {
        params.uniformity_num_buckets = mutate_usize(params.uniformity_num_buckets, ranges.uniformity_num_buckets, rng);
    }
}

/// Mutate strategy enums on a SpaceGenome in-place.
fn mutate_strategies(
    sg: &mut SpaceGenome,
    strategy_mutation_rate: f64,
    rng: &mut SimpleRng,
) {
    sg.force_function = sg.force_function.mutate(rng, strategy_mutation_rate);
    sg.connector_detection = sg.connector_detection.mutate(rng, strategy_mutation_rate);
    sg.space_init = sg.space_init.mutate(rng, strategy_mutation_rate);
    sg.multi_connector = sg.multi_connector.mutate(rng, strategy_mutation_rate);
    sg.negation_model = sg.negation_model.mutate(rng, strategy_mutation_rate);
    if rng.next_f64() < strategy_mutation_rate {
        sg.use_connector_axis = !sg.use_connector_axis;
    }
}

/// Uniform crossover for EngineParams: each field from `a` or `b` with 50% probability.
/// Result is written into `child` (which starts as a clone of `a`).
fn crossover_params(
    child: &mut EngineParams,
    b: &EngineParams,
    rng: &mut SimpleRng,
) {
    if rng.next_f64() < 0.5 { child.dimensions = b.dimensions; }
    if rng.next_f64() < 0.5 { child.learning_passes = b.learning_passes; }
    if rng.next_f64() < 0.5 { child.force_magnitude = b.force_magnitude; }
    if rng.next_f64() < 0.5 { child.force_decay = b.force_decay; }
    if rng.next_f64() < 0.5 { child.connector_min_frequency = b.connector_min_frequency; }
    if rng.next_f64() < 0.5 { child.connector_max_length = b.connector_max_length; }
    if rng.next_f64() < 0.5 { child.yes_threshold = b.yes_threshold; }
    if rng.next_f64() < 0.5 { child.no_threshold = b.no_threshold; }
    if rng.next_f64() < 0.5 { child.negation_inversion = b.negation_inversion; }
    if rng.next_f64() < 0.5 { child.bidirectional_force = b.bidirectional_force; }
    if rng.next_f64() < 0.5 { child.grammar_weight = b.grammar_weight; }
    if rng.next_f64() < 0.5 { child.max_follow_per_hop = b.max_follow_per_hop; }
    if rng.next_f64() < 0.5 { child.max_chain_hops = b.max_chain_hops; }
    if rng.next_f64() < 0.5 { child.weighted_distance_alpha = b.weighted_distance_alpha; }
    if rng.next_f64() < 0.5 { child.uniformity_num_buckets = b.uniformity_num_buckets; }
    if rng.next_f64() < 0.5 { child.uniformity_threshold = b.uniformity_threshold; }
}

/// Uniform crossover for strategy enums on a SpaceGenome.
fn crossover_strategies(
    child: &mut SpaceGenome,
    b: &SpaceGenome,
    rng: &mut SimpleRng,
) {
    if rng.next_f64() < 0.5 { child.force_function = b.force_function; }
    if rng.next_f64() < 0.5 { child.connector_detection = b.connector_detection; }
    if rng.next_f64() < 0.5 { child.space_init = b.space_init; }
    if rng.next_f64() < 0.5 { child.multi_connector = b.multi_connector; }
    if rng.next_f64() < 0.5 { child.negation_model = b.negation_model; }
    if rng.next_f64() < 0.5 { child.use_connector_axis = b.use_connector_axis; }
}

// ─── Single-Space Mutation ──────────────────────────────────────

/// Mutate a single-space genome. Returns a new child genome.
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

    mutate_params(&mut child.params, ranges, mutation_rate, rng);

    // Tier 2: Strategy mutation (inline since Genome stores strategies directly)
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

// ─── Single-Space Crossover ─────────────────────────────────────

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

    crossover_params(&mut child.params, &parent_b.params, rng);

    // Tier 2: uniform crossover per strategy
    if rng.next_f64() < 0.5 { child.force_function = parent_b.force_function; }
    if rng.next_f64() < 0.5 { child.connector_detection = parent_b.connector_detection; }
    if rng.next_f64() < 0.5 { child.space_init = parent_b.space_init; }
    if rng.next_f64() < 0.5 { child.multi_connector = parent_b.multi_connector; }
    if rng.next_f64() < 0.5 { child.negation_model = parent_b.negation_model; }
    if rng.next_f64() < 0.5 { child.use_connector_axis = parent_b.use_connector_axis; }

    child
}

// ─── Single-Space Selection ─────────────────────────────────────

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

// ─── Multi-Space Mutation ───────────────────────────────────────

/// Mutate a multi-space genome: independently mutate each space's params and strategies.
pub fn mutate_multi(
    genome: &MultiSpaceGenome,
    ranges: &ParamRanges,
    mutation_rate: f64,
    strategy_mutation_rate: f64,
    rng: &mut SimpleRng,
    new_id: u64,
    generation: usize,
) -> MultiSpaceGenome {
    let mut spaces = HashMap::new();

    for name in &genome.space_order {
        let mut sg = genome.spaces[name].clone();
        mutate_params(&mut sg.params, ranges, mutation_rate, rng);
        mutate_strategies(&mut sg, strategy_mutation_rate, rng);
        spaces.insert(name.clone(), sg);
    }

    MultiSpaceGenome {
        spaces,
        space_order: genome.space_order.clone(),
        id: new_id,
        generation,
        parent_ids: vec![genome.id],
        fitness: None,
    }
}

// ─── Multi-Space Crossover ──────────────────────────────────────

/// Per-space uniform crossover: for each space, each parameter from parent_a or parent_b.
pub fn crossover_multi(
    parent_a: &MultiSpaceGenome,
    parent_b: &MultiSpaceGenome,
    rng: &mut SimpleRng,
    new_id: u64,
    generation: usize,
) -> MultiSpaceGenome {
    let mut spaces = HashMap::new();

    for name in &parent_a.space_order {
        let sg_a = &parent_a.spaces[name];
        let sg_b = &parent_b.spaces[name];

        let mut child_sg = sg_a.clone();
        crossover_params(&mut child_sg.params, &sg_b.params, rng);
        crossover_strategies(&mut child_sg, sg_b, rng);

        spaces.insert(name.clone(), child_sg);
    }

    MultiSpaceGenome {
        spaces,
        space_order: parent_a.space_order.clone(),
        id: new_id,
        generation,
        parent_ids: vec![parent_a.id, parent_b.id],
        fitness: None,
    }
}

// ─── Multi-Space Selection ──────────────────────────────────────

/// Tournament selection for multi-space genomes.
pub fn tournament_select_multi<'a>(
    population: &'a [MultiSpaceGenome],
    tournament_size: usize,
    rng: &mut SimpleRng,
) -> &'a MultiSpaceGenome {
    let mut best: Option<&MultiSpaceGenome> = None;
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

// ─── Low-Level Helpers ──────────────────────────────────────────

fn mutate_usize(value: usize, range: (usize, usize), rng: &mut SimpleRng) -> usize {
    let delta = (rng.next_u64() % 3) as isize + 1; // 1, 2, or 3
    let sign: isize = if rng.next_f64() < 0.5 { -1 } else { 1 };
    let new_val = (value as isize + sign * delta)
        .max(range.0 as isize)
        .min(range.1 as isize);
    new_val as usize
}

/// Box-Muller transform: convert two uniform [0,1] samples into a standard normal sample.
fn gaussian(rng: &mut SimpleRng) -> f64 {
    let u1 = rng.next_f64().max(1e-10);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

fn clamp_f64(val: f64, min: f64, max: f64) -> f64 {
    val.max(min).min(max)
}
