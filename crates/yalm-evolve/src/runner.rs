use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use yalm_core::SimpleRng;
use yalm_parser::{parse_dictionary, parse_grammar_text, parse_test_questions};

use crate::fitness::{build_trained_space, evaluate_genome, EvalResult};
use crate::genome::{Genome, ParamRanges};
use crate::lineage::LineageTracker;
use crate::operators::{crossover, mutate, tournament_select};
use crate::population::initialize_population;
use crate::reporting::{save_generation, save_space_dump, write_status_md, write_suggestions_md};

// ─── Configuration ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    pub population_size: usize,
    pub generations: usize,
    pub elitism_count: usize,
    pub tournament_size: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub strategy_mutation_rate: f64,
    pub cross_validation_threshold: f64,
    pub base_seed: u64,
    pub dict5_path: PathBuf,
    pub dict5_test_path: PathBuf,
    pub dict12_path: Option<PathBuf>,
    pub dict12_test_path: Option<PathBuf>,
    #[serde(default)]
    pub grammar5_path: Option<PathBuf>,
    #[serde(default)]
    pub dict18_path: Option<PathBuf>,
    #[serde(default)]
    pub dict18_test_path: Option<PathBuf>,
    #[serde(default)]
    pub grammar18_path: Option<PathBuf>,
    pub results_dir: PathBuf,
}

// ─── Per-generation Statistics ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    pub generation: usize,
    pub best_fitness: f64,
    pub average_fitness: f64,
    pub worst_fitness: f64,
    pub best_genome_id: u64,
    pub population_size: usize,
    pub eval_results: Vec<EvalResult>,
}

// ─── Checkpoint ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub generation: usize,
    pub population: Vec<Genome>,
    pub id_counter: u64,
    pub config: EvolutionConfig,
    pub lineage: LineageTracker,
    pub all_generation_stats: Vec<GenerationStats>,
}

// ─── Main Evolution ─────────────────────────────────────────────

/// Run evolution from scratch.
pub fn evolve(config: &EvolutionConfig) -> Genome {
    let ranges = ParamRanges::default();
    let mut rng = SimpleRng::new(config.base_seed);
    let mut id_counter: u64 = 0;
    let lineage = LineageTracker::new();
    let all_stats: Vec<GenerationStats> = Vec::new();
    let population = initialize_population(
        config.population_size,
        &ranges,
        &mut rng,
        0,
        &mut id_counter,
    );

    evolve_inner(config, population, id_counter, rng, lineage, all_stats, 0)
}

/// Resume evolution from a checkpoint.
pub fn resume(results_dir: &PathBuf) -> Genome {
    let checkpoint_path = results_dir.join("checkpoint.json");
    let content =
        std::fs::read_to_string(&checkpoint_path).expect("Failed to read checkpoint.json");
    let checkpoint: Checkpoint =
        serde_json::from_str(&content).expect("Failed to parse checkpoint.json");

    eprintln!(
        "Resuming from generation {} with {} genomes",
        checkpoint.generation,
        checkpoint.population.len()
    );

    let rng = SimpleRng::new(
        checkpoint
            .config
            .base_seed
            .wrapping_add(checkpoint.generation as u64 * 1000),
    );

    evolve_inner(
        &checkpoint.config,
        checkpoint.population,
        checkpoint.id_counter,
        rng,
        checkpoint.lineage,
        checkpoint.all_generation_stats,
        checkpoint.generation,
    )
}

fn evolve_inner(
    config: &EvolutionConfig,
    mut population: Vec<Genome>,
    mut id_counter: u64,
    mut rng: SimpleRng,
    mut lineage: LineageTracker,
    mut all_stats: Vec<GenerationStats>,
    start_gen: usize,
) -> Genome {
    // Setup graceful shutdown
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();
    let _ = ctrlc::set_handler(move || {
        eprintln!("\nCtrl+C received, finishing current generation...");
        shutdown_clone.store(true, Ordering::SeqCst);
    });

    // Load data files once
    let dict5_content =
        std::fs::read_to_string(&config.dict5_path).expect("Failed to read dict5");
    let dict5 = parse_dictionary(&dict5_content);

    let test5_content =
        std::fs::read_to_string(&config.dict5_test_path).expect("Failed to read dict5_test");
    let test5 = parse_test_questions(&test5_content);

    let dict12 = config.dict12_path.as_ref().map(|p| {
        let content = std::fs::read_to_string(p).expect("Failed to read dict12");
        parse_dictionary(&content)
    });
    let test12 = config.dict12_test_path.as_ref().map(|p| {
        let content = std::fs::read_to_string(p).expect("Failed to read dict12_test");
        parse_test_questions(&content)
    });

    let grammar5 = config.grammar5_path.as_ref().map(|p| {
        let content = std::fs::read_to_string(p).expect("Failed to read grammar5");
        let g = parse_grammar_text(&content);
        eprintln!("Loaded grammar5: {} sections", g.entries.len());
        g
    });

    let dict18 = config.dict18_path.as_ref().map(|p| {
        let content = std::fs::read_to_string(p).expect("Failed to read dict18");
        parse_dictionary(&content)
    });
    let test18 = config.dict18_test_path.as_ref().map(|p| {
        let content = std::fs::read_to_string(p).expect("Failed to read dict18_test");
        parse_test_questions(&content)
    });
    let grammar18 = config.grammar18_path.as_ref().map(|p| {
        let content = std::fs::read_to_string(p).expect("Failed to read grammar18");
        let g = parse_grammar_text(&content);
        eprintln!("Loaded grammar18: {} sections", g.entries.len());
        g
    });

    // Create results directory
    std::fs::create_dir_all(&config.results_dir).expect("Failed to create results dir");

    let ranges = ParamRanges::default();
    let mut best_ever: Option<Genome> = None;

    // Adaptive mutation: increase rate when fitness stalls
    let mut effective_mutation_rate = config.mutation_rate;
    let mut stall_counter: usize = 0;
    let mut prev_best_fitness: f64 = 0.0;

    for gen in start_gen..config.generations {
        if shutdown.load(Ordering::SeqCst) {
            save_checkpoint(gen, &population, id_counter, config, &lineage, &all_stats);
            break;
        }

        eprintln!("=== Generation {} ===", gen);

        // ── EVALUATE (parallel) ──
        let base_seed = config.base_seed;
        let threshold = config.cross_validation_threshold;
        let eval_results: Vec<EvalResult> = population
            .par_iter()
            .map(|genome| {
                evaluate_genome(
                    genome,
                    &dict5,
                    &test5,
                    dict12.as_ref(),
                    test12.as_ref(),
                    grammar5.as_ref(),
                    dict18.as_ref(),
                    test18.as_ref(),
                    grammar18.as_ref(),
                    base_seed,
                    threshold,
                )
            })
            .collect();

        // Write fitness back to genomes
        for (genome, result) in population.iter_mut().zip(eval_results.iter()) {
            genome.fitness = Some(result.final_fitness);
            genome.primary_fitness = Some(result.primary_report.fitness);
            genome.cross_fitness = result.cross_report.as_ref().map(|cr| cr.fitness);
        }

        // ── SORT by fitness descending ──
        population.sort_by(|a, b| {
            b.fitness
                .unwrap_or(0.0)
                .partial_cmp(&a.fitness.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Track best ever
        if let Some(current_best) = population.first() {
            let current_fitness = current_best.fitness.unwrap_or(0.0);
            let is_new_best = match &best_ever {
                None => true,
                Some(prev) => current_fitness > prev.fitness.unwrap_or(0.0),
            };
            if is_new_best {
                best_ever = Some(current_best.clone());
            }
        }

        // ── RECORD ──
        let stats = GenerationStats {
            generation: gen,
            best_fitness: population
                .first()
                .and_then(|g| g.fitness)
                .unwrap_or(0.0),
            average_fitness: population.iter().map(|g| g.fitness.unwrap_or(0.0)).sum::<f64>()
                / population.len() as f64,
            worst_fitness: population
                .last()
                .and_then(|g| g.fitness)
                .unwrap_or(0.0),
            best_genome_id: population.first().map(|g| g.id).unwrap_or(0),
            population_size: population.len(),
            eval_results: eval_results.clone(),
        };

        // Update lineage
        for genome in &population {
            lineage.record(genome);
        }
        if let Some(best) = population.first() {
            lineage.record_generation_best(gen, best.id, best.fitness.unwrap_or(0.0));
        }

        // Save generation results
        save_generation(gen, &population, &stats, config);

        // Save space dump for the best genome
        if let Some(best) = population.first() {
            let space = build_trained_space(best, &dict5, grammar5.as_ref(), config.base_seed);
            save_space_dump(gen, &space, config);
        }

        all_stats.push(stats);

        // Update STATUS.md and SUGGESTIONS.md
        write_status_md(&all_stats, &population, &lineage, config);
        write_suggestions_md(
            &all_stats,
            &population,
            &eval_results,
            config,
        );

        print_generation_summary(gen, &population);

        // ── ADAPTIVE MUTATION ──
        let current_best = population.first().and_then(|g| g.fitness).unwrap_or(0.0);
        if (current_best - prev_best_fitness).abs() < 0.005 {
            stall_counter += 1;
        } else {
            stall_counter = 0;
        }
        prev_best_fitness = current_best;
        effective_mutation_rate = match stall_counter {
            0..=1 => config.mutation_rate,
            2 => config.mutation_rate * 1.5,
            _ => (config.mutation_rate * 2.5).min(0.6),
        };
        if stall_counter >= 2 {
            eprintln!("  Mutation rate: {:.2} (stall: {})", effective_mutation_rate, stall_counter);
        }

        // ── BREED next generation ──
        if gen < config.generations - 1 {
            let mut next_population: Vec<Genome> =
                Vec::with_capacity(config.population_size);

            // Elitism: top N survive unchanged
            for elite in population.iter().take(config.elitism_count) {
                let mut preserved = elite.clone();
                preserved.generation = gen + 1;
                preserved.fitness = None;
                preserved.primary_fitness = None;
                preserved.cross_fitness = None;
                next_population.push(preserved);
            }

            // Fill remaining with crossover + mutation
            while next_population.len() < config.population_size {
                let parent_a =
                    tournament_select(&population, config.tournament_size, &mut rng);
                let parent_b =
                    tournament_select(&population, config.tournament_size, &mut rng);

                id_counter += 1;
                let child = if rng.next_f64() < config.crossover_rate {
                    let crossed =
                        crossover(parent_a, parent_b, &mut rng, id_counter, gen + 1);
                    mutate(
                        &crossed,
                        &ranges,
                        effective_mutation_rate,
                        config.strategy_mutation_rate,
                        &mut rng,
                        id_counter,
                        gen + 1,
                    )
                } else {
                    mutate(
                        parent_a,
                        &ranges,
                        effective_mutation_rate,
                        config.strategy_mutation_rate,
                        &mut rng,
                        id_counter,
                        gen + 1,
                    )
                };
                next_population.push(child);
            }

            population = next_population;
        }

        if shutdown.load(Ordering::SeqCst) {
            save_checkpoint(
                gen + 1,
                &population,
                id_counter,
                config,
                &lineage,
                &all_stats,
            );
            break;
        }
    }

    // Return best ever
    best_ever.unwrap_or_else(|| {
        population
            .into_iter()
            .max_by(|a, b| {
                a.fitness
                    .partial_cmp(&b.fitness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap()
    })
}

fn print_generation_summary(_gen: usize, population: &[Genome]) {
    let best = population
        .first()
        .and_then(|g| g.fitness)
        .unwrap_or(0.0);
    let avg =
        population.iter().map(|g| g.fitness.unwrap_or(0.0)).sum::<f64>() / population.len() as f64;
    let best_id = population.first().map(|g| g.id).unwrap_or(0);
    eprintln!(
        "  Best: {:.4}  Avg: {:.4}  Best ID: {}",
        best, avg, best_id
    );
}

fn save_checkpoint(
    gen: usize,
    population: &[Genome],
    id_counter: u64,
    config: &EvolutionConfig,
    lineage: &LineageTracker,
    all_stats: &[GenerationStats],
) {
    let checkpoint = Checkpoint {
        generation: gen,
        population: population.to_vec(),
        id_counter,
        config: config.clone(),
        lineage: lineage.clone(),
        all_generation_stats: all_stats.to_vec(),
    };
    let path = config.results_dir.join("checkpoint.json");
    let json = serde_json::to_string_pretty(&checkpoint).unwrap();
    std::fs::write(path, json).expect("Failed to write checkpoint");
    eprintln!("Checkpoint saved at generation {}.", gen);
}
