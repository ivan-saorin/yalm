pub mod connector_discovery;
pub mod equilibrium;
pub mod force_field;
pub mod multispace;
pub mod resolver;
pub mod strategy;

use std::collections::HashSet;
use yalm_core::*;

use connector_discovery::{classify_word_roles, discover_connectors, extract_all_sentences, extract_relations};
use equilibrium::{build_space_equilibrium, EquilibriumParams};
use force_field::build_space;
use resolver::resolve_question;
use strategy::StrategyConfig;

// ─── Build Mode ─────────────────────────────────────────────────

/// Controls how the geometric space is constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    /// Batch force field: all positions initialized at once, forces applied
    /// iteratively over learning_passes with decaying magnitude. (Default)
    ForceField,
    /// Sequential equilibrium: words placed one at a time from definitions,
    /// with local relaxation after each placement. Fixed parameters, no GA.
    Equilibrium,
}

impl Default for BuildMode {
    fn default() -> Self {
        BuildMode::ForceField
    }
}

// ─── Engine ─────────────────────────────────────────────────────

pub struct Engine {
    params: EngineParams,
    strategy: StrategyConfig,
    space: GeometricSpace,
    structural: HashSet<String>,
    content: HashSet<String>,
    dictionary: Option<Dictionary>,
    quiet: bool,
    mode: BuildMode,
}

impl Engine {
    pub fn new(params: EngineParams) -> Self {
        Self::with_strategy(params, StrategyConfig::default())
    }

    pub fn with_strategy(params: EngineParams, strategy: StrategyConfig) -> Self {
        Self {
            space: GeometricSpace {
                dimensions: params.dimensions,
                words: std::collections::HashMap::new(),
                connectors: Vec::new(),
                distance_stats: None,
            },
            params,
            strategy,
            structural: HashSet::new(),
            content: HashSet::new(),
            dictionary: None,
            quiet: false,
            mode: BuildMode::ForceField,
        }
    }

    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    pub fn set_mode(&mut self, mode: BuildMode) {
        self.mode = mode;
    }

    pub fn strategy(&self) -> &StrategyConfig {
        &self.strategy
    }

    pub fn structural(&self) -> &HashSet<String> {
        &self.structural
    }

    pub fn content(&self) -> &HashSet<String> {
        &self.content
    }
}

impl Comprehend for Engine {
    fn train(&mut self, dictionary: &Dictionary) {
        self.dictionary = Some(dictionary.clone());

        let (structural, content) = classify_word_roles(dictionary);
        self.structural = structural;
        self.content = content;

        let (connectors, relations) = discover_connectors(dictionary, &self.params, &self.strategy);

        if !self.quiet {
            println!("=== Discovered {} connectors ===", connectors.len());
            for c in &connectors {
                println!("  {:?} (freq: {}, uni: {:.3})", c.pattern, c.frequency, c.uniformity);
            }
            println!(
                "=== Extracted {} sentence relations ({} negated) ===",
                relations.len(),
                relations.iter().filter(|r| r.negated).count()
            );
        }

        let space = match self.mode {
            BuildMode::ForceField => {
                build_space(dictionary, &connectors, &relations, &self.params, &self.strategy)
            }
            BuildMode::Equilibrium => {
                let eq_params = EquilibriumParams::default();
                build_space_equilibrium(
                    dictionary,
                    &connectors,
                    &relations,
                    &[],
                    &self.params,
                    &self.strategy,
                    &eq_params,
                    self.quiet,
                )
            }
        };

        if !self.quiet {
            let stats = space.get_distance_stats();
            println!("  Distance stats: mean={:.4}, std_dev={:.4} ({} words)", stats.mean, stats.std_dev, space.words.len());
        }
        self.space = space;
    }

    fn train_with_grammar(&mut self, dictionary: &Dictionary, grammar: &Dictionary) {
        self.dictionary = Some(dictionary.clone());

        let (structural, content) = classify_word_roles(dictionary);
        self.structural = structural;
        self.content = content;

        // PASS 1: Discover connectors and relations from dictionary
        let (connectors, relations) = discover_connectors(dictionary, &self.params, &self.strategy);

        if !self.quiet {
            println!("=== Discovered {} connectors ===", connectors.len());
            for c in &connectors {
                println!("  {:?} (freq: {}, uni: {:.3})", c.pattern, c.frequency, c.uniformity);
            }
            println!(
                "=== Dict relations: {} ({} negated) ===",
                relations.len(),
                relations.iter().filter(|r| r.negated).count()
            );
        }

        // PASS 2: Extract additional relations from grammar text
        let grammar_sentences = extract_all_sentences(grammar);
        let mut grammar_relations = extract_relations(
            &grammar_sentences, dictionary, &self.structural, &self.content, &self.params
        );

        // Scale grammar relations by grammar_weight
        let gw = self.params.grammar_weight;
        for r in &mut grammar_relations {
            r.weight = gw;
        }

        if !self.quiet {
            println!(
                "=== Grammar relations: {} ({} negated, weight={:.2}) ===",
                grammar_relations.len(),
                grammar_relations.iter().filter(|r| r.negated).count(),
                gw
            );
        }

        let space = match self.mode {
            BuildMode::ForceField => {
                // Merge relations and build space (existing behavior)
                let mut all_relations = relations;
                all_relations.extend(grammar_relations);

                if !self.quiet {
                    println!(
                        "=== Total relations: {} ({} negated) ===",
                        all_relations.len(),
                        all_relations.iter().filter(|r| r.negated).count()
                    );
                }

                build_space(dictionary, &connectors, &all_relations, &self.params, &self.strategy)
            }
            BuildMode::Equilibrium => {
                if !self.quiet {
                    println!(
                        "=== Total relations: {} dict + {} grammar ===",
                        relations.len(),
                        grammar_relations.len()
                    );
                }

                let eq_params = EquilibriumParams::default();
                build_space_equilibrium(
                    dictionary,
                    &connectors,
                    &relations,
                    &grammar_relations,
                    &self.params,
                    &self.strategy,
                    &eq_params,
                    self.quiet,
                )
            }
        };

        if !self.quiet {
            let stats = space.get_distance_stats();
            println!("  Distance stats: mean={:.4}, std_dev={:.4} ({} words)", stats.mean, stats.std_dev, space.words.len());
        }
        self.space = space;
    }

    fn query(&self, question: &str) -> Answer {
        let dict = self.dictionary.as_ref().expect("Must train before query");
        let (answer, _, _) = resolve_question(
            question,
            &self.space,
            dict,
            &self.structural,
            &self.content,
            &self.params,
            &self.strategy,
        );
        answer
    }

    fn distance(&self, word_a: &str, word_b: &str, connector: &Connector) -> f64 {
        let pos_a = match self.space.words.get(word_a) {
            Some(wp) => &wp.position,
            None => return f64::MAX,
        };
        let pos_b = match self.space.words.get(word_b) {
            Some(wp) => &wp.position,
            None => return f64::MAX,
        };
        let displacement: Vec<f64> = pos_a
            .iter()
            .zip(pos_b.iter())
            .map(|(a, b)| b - a)
            .collect();
        dot_product(&displacement, &connector.force_direction).abs()
    }

    fn space(&self) -> &GeometricSpace {
        &self.space
    }
}
