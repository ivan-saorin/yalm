pub mod analysis;
pub mod fitness;
pub mod genome;
pub mod lineage;
pub mod operators;
pub mod population;
pub mod reporting;
pub mod runner;

pub use genome::{Genome, MultiSpaceGenome, SpaceGenome};
pub use runner::{evolve, evolve_multi, resume, resume_multi, EvolutionConfig, MultiSpaceEvolutionConfig};
