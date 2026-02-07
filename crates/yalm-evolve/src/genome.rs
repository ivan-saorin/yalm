use serde::{Deserialize, Serialize};
use yalm_core::EngineParams;

// Import strategy types from yalm-engine (the canonical source)
pub use yalm_engine::strategy::{
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
