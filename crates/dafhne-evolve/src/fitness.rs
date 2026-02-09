use serde::{Deserialize, Serialize};
use dafhne_core::*;
use dafhne_engine::Engine;

use crate::genome::Genome;

/// Result of evaluating a single genome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub genome_id: u64,
    pub primary_report: FitnessReport,
    pub cross_report: Option<FitnessReport>,
    /// Dict18 cross-validation report (single-space, same thresholds).
    #[serde(default)]
    pub cross18_report: Option<FitnessReport>,
    /// Dual-space ensemble report on dict5 (when grammar provided).
    /// This is used only for bonus scoring, not as the primary evaluation.
    pub dual_report: Option<FitnessReport>,
    pub final_fitness: f64,
}

/// Evaluate a single genome against dict5 (and optionally dict12 and dict18).
///
/// Fitness structure (when grammar is provided):
///   1. Single-space dict5 → primary_report (thresholds must work for single-space)
///   2. Single-space dict12 → cross_report (same thresholds, same mode — generalization test)
///   3. Single-space dict18 → cross18_report (same thresholds — scaling test)
///   4. Dual-space dict5 → dual_report (ensemble of dict-only + dict+grammar engines)
///   5. final_fitness = base_fitness + dual_bonus
///      where base_fitness depends on available cross-validation levels:
///        - dict5+dict12+dict18: 0.5*primary + 0.3*cross12 + 0.2*cross18
///        - dict5+dict12:        0.6*primary + 0.4*cross12
///        - dict5 only:          primary
///      Overfitting penalty applied when gap > 0.15
///      and dual_bonus = max(0, dual_fitness - primary_fitness) * 0.15
///
/// This structure forces thresholds to generalize across all dictionary sizes.
pub fn evaluate_genome(
    genome: &Genome,
    dict5: &Dictionary,
    test5: &TestSuite,
    dict12: Option<&Dictionary>,
    test12: Option<&TestSuite>,
    grammar5: Option<&Dictionary>,
    dict18: Option<&Dictionary>,
    test18: Option<&TestSuite>,
    grammar18: Option<&Dictionary>,
    base_seed: u64,
    cross_validation_threshold: f64,
) -> EvalResult {
    let engine_params = genome.to_engine_params(base_seed);
    let strategy = genome.to_strategy_config();

    // ALWAYS evaluate dict5 with single-space (this is the primary evaluation)
    let mut engine_dict5 = Engine::with_strategy(engine_params.clone(), strategy.clone());
    engine_dict5.set_quiet(true);
    engine_dict5.train(dict5);
    let primary_report = dafhne_eval::evaluate(&engine_dict5, test5, dict5, &engine_params, &strategy);

    // Dual-space ensemble evaluation on dict5 (when grammar provided)
    // This uses the already-trained dict5 engine + a new grammar-enhanced engine
    let dual_report = if let Some(grammar) = grammar5 {
        let mut engine_gram = Engine::with_strategy(engine_params.clone(), strategy.clone());
        engine_gram.set_quiet(true);
        engine_gram.train_with_grammar(dict5, grammar);

        Some(dafhne_eval::evaluate_dual(
            &engine_dict5, &engine_gram,
            test5, dict5,
            &engine_params, &engine_params,
            &strategy, &strategy,
        ))
    } else {
        None
    };

    // Cross-validate on dict12 (single-space, same thresholds — generalization test)
    let cross_report = if primary_report.fitness >= cross_validation_threshold {
        if let (Some(d12), Some(t12)) = (dict12, test12) {
            let mut engine12 = Engine::with_strategy(engine_params.clone(), strategy.clone());
            engine12.set_quiet(true);
            engine12.train(d12);
            Some(dafhne_eval::evaluate(&engine12, t12, d12, &engine_params, &strategy))
        } else {
            None
        }
    } else {
        None
    };

    // Cross-validate on dict18 (single-space, same thresholds — scaling test)
    // Uses grammar18 if provided (dual-space for dict18, same as dict5 gets grammar5)
    let cross18_report = if primary_report.fitness >= cross_validation_threshold {
        if let (Some(d18), Some(t18)) = (dict18, test18) {
            let mut engine18 = Engine::with_strategy(engine_params.clone(), strategy.clone());
            engine18.set_quiet(true);
            if let Some(g18) = grammar18 {
                engine18.train_with_grammar(d18, g18);
            } else {
                engine18.train(d18);
            }
            Some(dafhne_eval::evaluate(&engine18, t18, d18, &engine_params, &strategy))
        } else {
            None
        }
    } else {
        None
    };

    // Compute final fitness:
    //   Three-level: 0.5*primary + 0.3*cross12 + 0.2*cross18 (with overfitting penalty)
    //   Two-level:   0.6*primary + 0.4*cross12 (with overfitting penalty)
    //   One-level:   primary
    //   dual_bonus = max(0, dual_fitness - primary_fitness) * 0.15
    //   final = base + dual_bonus
    let base_fitness = match (&cross_report, &cross18_report) {
        (Some(cr12), Some(cr18)) => {
            let primary = primary_report.fitness;
            let cross12 = cr12.fitness;
            let cross18 = cr18.fitness;
            let base = 0.5 * primary + 0.3 * cross12 + 0.2 * cross18;
            // Overfitting penalty: penalize if primary >> worst cross score
            let worst_cross = cross12.min(cross18);
            let gap = primary - worst_cross;
            if gap > 0.15 {
                (base - 0.5 * (gap - 0.15)).max(0.0)
            } else {
                base
            }
        }
        (Some(cr), None) => {
            let primary = primary_report.fitness;
            let cross = cr.fitness;
            let base = 0.6 * primary + 0.4 * cross;
            let gap = primary - cross;
            if gap > 0.15 {
                (base - 0.5 * (gap - 0.15)).max(0.0)
            } else {
                base
            }
        }
        _ => primary_report.fitness,
    };

    // Dual-space bonus: reward ensemble uplift over single-space
    let dual_bonus = match &dual_report {
        Some(dr) => {
            let uplift = dr.fitness - primary_report.fitness;
            if uplift > 0.0 { uplift * 0.15 } else { 0.0 }
        }
        None => 0.0,
    };

    let final_fitness = base_fitness + dual_bonus;

    EvalResult {
        genome_id: genome.id,
        primary_report,
        cross_report,
        cross18_report,
        dual_report,
        final_fitness,
    }
}

/// Build an engine from a genome and return the trained space (for analysis).
pub fn build_trained_space(
    genome: &Genome,
    dictionary: &Dictionary,
    grammar: Option<&Dictionary>,
    base_seed: u64,
) -> GeometricSpace {
    let params = genome.to_engine_params(base_seed);
    let strategy = genome.to_strategy_config();
    let mut engine = Engine::with_strategy(params, strategy);
    engine.set_quiet(true);
    if let Some(grammar) = grammar {
        engine.train_with_grammar(dictionary, grammar);
    } else {
        engine.train(dictionary);
    }
    engine.space().clone()
}
