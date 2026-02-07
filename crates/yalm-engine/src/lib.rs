pub mod connector_discovery;
pub mod force_field;
pub mod resolver;
pub mod strategy;

use std::collections::HashSet;
use yalm_core::*;

use connector_discovery::{classify_word_roles, discover_connectors, extract_all_sentences, extract_relations};
use force_field::build_space;
use resolver::resolve_question;
use strategy::StrategyConfig;

pub struct Engine {
    params: EngineParams,
    strategy: StrategyConfig,
    space: GeometricSpace,
    structural: HashSet<String>,
    content: HashSet<String>,
    dictionary: Option<Dictionary>,
    quiet: bool,
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
        }
    }

    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
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
                println!("  {:?} (freq: {})", c.pattern, c.frequency);
            }
            println!(
                "=== Extracted {} sentence relations ({} negated) ===",
                relations.len(),
                relations.iter().filter(|r| r.negated).count()
            );
        }

        let space = build_space(dictionary, &connectors, &relations, &self.params, &self.strategy);
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
        let (connectors, mut relations) = discover_connectors(dictionary, &self.params, &self.strategy);

        if !self.quiet {
            println!("=== Discovered {} connectors ===", connectors.len());
            for c in &connectors {
                println!("  {:?} (freq: {})", c.pattern, c.frequency);
            }
            println!(
                "=== Dict relations: {} ({} negated) ===",
                relations.len(),
                relations.iter().filter(|r| r.negated).count()
            );
        }

        // PASS 2: Extract additional relations from grammar text
        // Grammar sentences are processed with dict's vocabulary (entry_set + topic words)
        let grammar_sentences = extract_all_sentences(grammar);
        let mut grammar_relations = extract_relations(
            &grammar_sentences, dictionary, &self.structural, &self.content, &self.params
        );

        // Scale grammar relations by grammar_weight to prevent overwhelming dict signal
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

        // Merge: grammar relations reinforce connector patterns
        relations.extend(grammar_relations);

        if !self.quiet {
            println!(
                "=== Total relations: {} ({} negated) ===",
                relations.len(),
                relations.iter().filter(|r| r.negated).count()
            );
        }

        // Build space with combined relations
        let space = build_space(dictionary, &connectors, &relations, &self.params, &self.strategy);
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
