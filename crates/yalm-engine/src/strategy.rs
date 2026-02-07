use serde::{Deserialize, Serialize};
use yalm_core::SimpleRng;

// ─── Force Function ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForceFunction {
    Linear,
    InverseDistance,
    Gravitational,
    Spring,
}

impl ForceFunction {
    pub const ALL: &'static [Self] = &[
        Self::Linear,
        Self::InverseDistance,
        Self::Gravitational,
        Self::Spring,
    ];

    pub fn random(rng: &mut SimpleRng) -> Self {
        Self::ALL[rng.next_u64() as usize % Self::ALL.len()]
    }

    pub fn mutate(self, rng: &mut SimpleRng, rate: f64) -> Self {
        if rng.next_f64() < rate {
            Self::random(rng)
        } else {
            self
        }
    }
}

// ─── Connector Detection ───────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorDetection {
    FrequencyOnly,
    PositionalBias,
    MutualInformation,
}

impl ConnectorDetection {
    pub const ALL: &'static [Self] = &[
        Self::FrequencyOnly,
        Self::PositionalBias,
        Self::MutualInformation,
    ];

    pub fn random(rng: &mut SimpleRng) -> Self {
        Self::ALL[rng.next_u64() as usize % Self::ALL.len()]
    }

    pub fn mutate(self, rng: &mut SimpleRng, rate: f64) -> Self {
        if rng.next_f64() < rate {
            Self::random(rng)
        } else {
            self
        }
    }
}

// ─── Space Initialization ──────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceInitialization {
    Random,
    Spherical,
    FromConnectors,
}

impl SpaceInitialization {
    pub const ALL: &'static [Self] = &[Self::Random, Self::Spherical, Self::FromConnectors];

    pub fn random(rng: &mut SimpleRng) -> Self {
        Self::ALL[rng.next_u64() as usize % Self::ALL.len()]
    }

    pub fn mutate(self, rng: &mut SimpleRng, rate: f64) -> Self {
        if rng.next_f64() < rate {
            Self::random(rng)
        } else {
            self
        }
    }
}

// ─── Multi-Connector Handling ──────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiConnectorHandling {
    FirstOnly,
    Sequential,
    Weighted,
    Compositional,
}

impl MultiConnectorHandling {
    pub const ALL: &'static [Self] = &[
        Self::FirstOnly,
        Self::Sequential,
        Self::Weighted,
        Self::Compositional,
    ];

    pub fn random(rng: &mut SimpleRng) -> Self {
        Self::ALL[rng.next_u64() as usize % Self::ALL.len()]
    }

    pub fn mutate(self, rng: &mut SimpleRng, rate: f64) -> Self {
        if rng.next_f64() < rate {
            Self::random(rng)
        } else {
            self
        }
    }
}

// ─── Negation Model ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NegationModel {
    Inversion,
    Repulsion,
    AxisShift,
    SeparateDimension,
}

impl NegationModel {
    pub const ALL: &'static [Self] = &[
        Self::Inversion,
        Self::Repulsion,
        Self::AxisShift,
        Self::SeparateDimension,
    ];

    pub fn random(rng: &mut SimpleRng) -> Self {
        Self::ALL[rng.next_u64() as usize % Self::ALL.len()]
    }

    pub fn mutate(self, rng: &mut SimpleRng, rate: f64) -> Self {
        if rng.next_f64() < rate {
            Self::random(rng)
        } else {
            self
        }
    }
}

// ─── Strategy Config ───────────────────────────────────────────

/// Configuration selecting which algorithmic strategy to use for each
/// component of the geometric comprehension engine.
///
/// `Default::default()` matches the original hardcoded engine behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub force_function: ForceFunction,
    pub connector_detection: ConnectorDetection,
    pub space_init: SpaceInitialization,
    pub multi_connector: MultiConnectorHandling,
    pub negation_model: NegationModel,
    #[serde(default)]
    pub use_connector_axis: bool,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            force_function: ForceFunction::Linear,
            connector_detection: ConnectorDetection::FrequencyOnly,
            space_init: SpaceInitialization::Random,
            multi_connector: MultiConnectorHandling::Sequential,
            negation_model: NegationModel::Inversion,
            use_connector_axis: false,
        }
    }
}
