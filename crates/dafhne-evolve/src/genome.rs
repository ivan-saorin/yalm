use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use dafhne_core::EngineParams;

// Import strategy types from dafhne-engine (the canonical source)
pub use dafhne_engine::strategy::{
    ConnectorDetection, ForceFunction, MultiConnectorHandling, NegationModel,
    SpaceInitialization, StrategyConfig,
};

// ─── Parameter Ranges ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParamRanges {
    pub dimensions: (usize, usize),
    pub learning_passes: (usize, usize),
    pub force_magnitude: (f64, f64),
    pub force_decay: (f64, f64),
    pub connector_min_frequency: (usize, usize),
    pub connector_max_length: (usize, usize),
    pub yes_threshold: (f64, f64),
    pub no_threshold: (f64, f64),
    pub negation_inversion: (f64, f64),
    pub bidirectional_force: (f64, f64),
    pub grammar_weight: (f64, f64),
    pub max_follow_per_hop: (usize, usize),
    pub max_chain_hops: (usize, usize),
    pub weighted_distance_alpha: (f64, f64),
    pub uniformity_num_buckets: (usize, usize),
    pub uniformity_threshold: (f64, f64),
}

impl Default for ParamRanges {
    fn default() -> Self {
        Self {
            dimensions: (4, 32),
            learning_passes: (1, 50),
            force_magnitude: (0.01, 1.0),
            force_decay: (0.5, 0.99),
            connector_min_frequency: (1, 10),
            connector_max_length: (1, 4),
            yes_threshold: (0.05, 0.35),
            no_threshold: (0.15, 0.6),
            negation_inversion: (-1.0, 1.0),
            bidirectional_force: (0.0, 1.0),
            grammar_weight: (0.0, 1.0),
            max_follow_per_hop: (1, 6),
            max_chain_hops: (1, 6),
            weighted_distance_alpha: (0.05, 0.5),
            uniformity_num_buckets: (5, 20),
            uniformity_threshold: (0.5, 0.95),
        }
    }
}

// ─── Genome ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    // Tier 1: Parameters (directly mapped to EngineParams)
    pub params: EngineParams,

    // Tier 2: Strategy choices (wired to Engine via StrategyConfig)
    pub force_function: ForceFunction,
    pub connector_detection: ConnectorDetection,
    pub space_init: SpaceInitialization,
    pub multi_connector: MultiConnectorHandling,
    pub negation_model: NegationModel,
    #[serde(default)]
    pub use_connector_axis: bool,

    // Metadata
    pub id: u64,
    pub generation: usize,
    pub parent_ids: Vec<u64>,
    pub fitness: Option<f64>,
    pub primary_fitness: Option<f64>,
    pub cross_fitness: Option<f64>,
}

impl Genome {
    /// Build EngineParams from this genome with a unique per-genome RNG seed.
    pub fn to_engine_params(&self, base_seed: u64) -> EngineParams {
        let mut p = self.params.clone();
        p.rng_seed = base_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.id);
        p
    }

    /// Build a StrategyConfig from this genome's Tier 2 choices.
    pub fn to_strategy_config(&self) -> StrategyConfig {
        StrategyConfig {
            force_function: self.force_function,
            connector_detection: self.connector_detection,
            space_init: self.space_init,
            multi_connector: self.multi_connector,
            negation_model: self.negation_model,
            use_connector_axis: self.use_connector_axis,
        }
    }
}

// ─── Per-Space Genome (for multi-space evolution) ───────────────

/// Parameters and strategy choices for a single space within a MultiSpaceGenome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceGenome {
    pub params: EngineParams,
    pub force_function: ForceFunction,
    pub connector_detection: ConnectorDetection,
    pub space_init: SpaceInitialization,
    pub multi_connector: MultiConnectorHandling,
    pub negation_model: NegationModel,
    #[serde(default)]
    pub use_connector_axis: bool,
}

impl SpaceGenome {
    /// Build EngineParams with a unique RNG seed per genome AND per space.
    pub fn to_engine_params(&self, base_seed: u64, genome_id: u64, space_name: &str) -> EngineParams {
        let mut p = self.params.clone();
        let space_hash = space_name
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        p.rng_seed = base_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(genome_id)
            .wrapping_add(space_hash);
        p
    }

    /// Build a StrategyConfig from this space genome's choices.
    pub fn to_strategy_config(&self) -> StrategyConfig {
        StrategyConfig {
            force_function: self.force_function,
            connector_detection: self.connector_detection,
            space_init: self.space_init,
            multi_connector: self.multi_connector,
            negation_model: self.negation_model,
            use_connector_axis: self.use_connector_axis,
        }
    }

    /// Create a SpaceGenome from an existing single-space Genome's parameters.
    pub fn from_genome(genome: &Genome) -> Self {
        SpaceGenome {
            params: genome.params.clone(),
            force_function: genome.force_function,
            connector_detection: genome.connector_detection,
            space_init: genome.space_init,
            multi_connector: genome.multi_connector,
            negation_model: genome.negation_model,
            use_connector_axis: genome.use_connector_axis,
        }
    }
}

// ─── Multi-Space Genome ─────────────────────────────────────────

/// A genome for multi-space evolution: independent parameters per space,
/// evolved jointly via unified fitness evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSpaceGenome {
    /// Per-space parameter sets, keyed by space name.
    pub spaces: HashMap<String, SpaceGenome>,
    /// Ordered list of space names (for deterministic iteration).
    pub space_order: Vec<String>,

    // Metadata
    pub id: u64,
    pub generation: usize,
    pub parent_ids: Vec<u64>,
    pub fitness: Option<f64>,
}

impl MultiSpaceGenome {
    /// Bootstrap a MultiSpaceGenome from a single-space Genome.
    /// All spaces start with the same params (for warm-starting from v11 best).
    pub fn from_genome(genome: &Genome, space_names: &[String]) -> Self {
        let sg = SpaceGenome::from_genome(genome);
        let mut spaces = HashMap::new();
        for name in space_names {
            spaces.insert(name.clone(), sg.clone());
        }
        MultiSpaceGenome {
            spaces,
            space_order: space_names.to_vec(),
            id: genome.id,
            generation: genome.generation,
            parent_ids: genome.parent_ids.clone(),
            fitness: genome.fitness,
        }
    }
}
