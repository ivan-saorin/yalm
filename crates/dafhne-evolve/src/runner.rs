use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use dafhne_core::SimpleRng;
use dafhne_parser::{parse_dictionary, parse_grammar_text, parse_test_questions};

use crate::fitness::{build_trained_space, evaluate_genome, evaluate_multi_genome, EvalResult, MultiSpaceEvalResult};
use crate::genome::{Genome, MultiSpaceGenome, ParamRanges};
use crate::lineage::LineageTracker;
use crate::operators::{crossover, crossover_multi, mutate, mutate_multi, tournament_select, tournament_select_multi};
use crate::population::{initialize_population, initialize_multi_population};
use crate::reporting::{save_generation, save_space_dump, write_status_md, write_suggestions_md,
                        save_multi_generation, write_multi_status_md};

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

// ═══════════════════════════════════════════════════════════════════
// Multi-Space Evolution
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSpaceEvolutionConfig {
    pub population_size: usize,
    pub generations: usize,
    pub elitism_count: usize,
    pub tournament_size: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub strategy_mutation_rate: f64,
    pub base_seed: u64,
    pub results_dir: PathBuf,
    /// Space definitions: Vec<(name, dict_path)>
    pub space_configs: Vec<(String, PathBuf)>,
    /// Path to unified test file
    pub test_path: PathBuf,
    /// Optional: path to seed genome JSON (bootstrap from existing v11 best)
    #[serde(default)]
    pub seed_genome_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiGenerationStats {
    pub generation: usize,
    pub best_fitness: f64,
    pub average_fitness: f64,
    pub worst_fitness: f64,
    pub best_genome_id: u64,
    pub population_size: usize,
    pub eval_results: Vec<MultiSpaceEvalResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSpaceCheckpoint {
    pub generation: usize,
    pub population: Vec<MultiSpaceGenome>,
    pub id_counter: u64,
    pub config: MultiSpaceEvolutionConfig,
    pub all_generation_stats: Vec<MultiGenerationStats>,
}

/// Run multi-space evolution from scratch.
pub fn evolve_multi(config: &MultiSpaceEvolutionConfig) -> MultiSpaceGenome {
    let ranges = ParamRanges::default();
    let mut rng = SimpleRng::new(config.base_seed);
    let mut id_counter: u64 = 0;

    let space_names: Vec<String> = config.space_configs.iter().map(|(n, _)| n.clone()).collect();

    // Load seed genome if provided — accept either single-space Genome or MultiSpaceGenome
    let (seed_single, seed_multi) = match config.seed_genome_path.as_ref() {
        Some(path) => {
            let content = std::fs::read_to_string(path).expect("Failed to read seed genome");
            // Try MultiSpaceGenome first (more specific), fall back to single-space Genome
            if let Ok(msg) = serde_json::from_str::<MultiSpaceGenome>(&content) {
                eprintln!("Loaded multi-space seed genome (ID {})", msg.id);
                (None, Some(msg))
            } else if let Ok(g) = serde_json::from_str::<Genome>(&content) {
                eprintln!("Loaded single-space seed genome (ID {}), broadcasting to all spaces", g.id);
                (Some(g), None)
            } else {
                panic!("Failed to parse seed genome as either Genome or MultiSpaceGenome");
            }
        }
        None => (None, None),
    };

    let population = initialize_multi_population(
        config.population_size,
        &space_names,
        &ranges,
        &mut rng,
        0,
        &mut id_counter,
        seed_single.as_ref(),
        seed_multi.as_ref(),
    );

    let all_stats: Vec<MultiGenerationStats> = Vec::new();
    evolve_multi_inner(config, population, id_counter, rng, all_stats, 0)
}

/// Resume multi-space evolution from a checkpoint.
pub fn resume_multi(results_dir: &PathBuf) -> MultiSpaceGenome {
    let checkpoint_path = results_dir.join("checkpoint.json");
    let content =
        std::fs::read_to_string(&checkpoint_path).expect("Failed to read checkpoint.json");
    let checkpoint: MultiSpaceCheckpoint =
        serde_json::from_str(&content).expect("Failed to parse checkpoint.json");

    eprintln!(
        "Resuming multi-space from generation {} with {} genomes",
        checkpoint.generation,
        checkpoint.population.len()
    );

    let rng = SimpleRng::new(
        checkpoint
            .config
            .base_seed
            .wrapping_add(checkpoint.generation as u64 * 1000),
    );

    evolve_multi_inner(
        &checkpoint.config,
        checkpoint.population,
        checkpoint.id_counter,
        rng,
        checkpoint.all_generation_stats,
        checkpoint.generation,
    )
}

fn evolve_multi_inner(
    config: &MultiSpaceEvolutionConfig,
    mut population: Vec<MultiSpaceGenome>,
    mut id_counter: u64,
    mut rng: SimpleRng,
    mut all_stats: Vec<MultiGenerationStats>,
    start_gen: usize,
) -> MultiSpaceGenome {
    // Graceful shutdown
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();
    let _ = ctrlc::set_handler(move || {
        eprintln!("\nCtrl+C received, finishing current generation...");
        shutdown_clone.store(true, Ordering::SeqCst);
    });

    // Load test suite once
    let test_content =
        std::fs::read_to_string(&config.test_path).expect("Failed to read test file");
    let test_suite = parse_test_questions(&test_content);

    std::fs::create_dir_all(&config.results_dir).expect("Failed to create results dir");

    let ranges = ParamRanges::default();
    let mut best_ever: Option<MultiSpaceGenome> = None;

    // Adaptive mutation
    let mut effective_mutation_rate = config.mutation_rate;
    let mut stall_counter: usize = 0;
    let mut prev_best_fitness: f64 = 0.0;

    let space_configs = config.space_configs.clone();

    for gen in start_gen..config.generations {
        if shutdown.load(Ordering::SeqCst) {
            save_multi_checkpoint(gen, &population, id_counter, config, &all_stats);
            break;
        }

        eprintln!("=== Multi-Space Generation {} ===", gen);

        // ── EVALUATE (parallel) ──
        let base_seed = config.base_seed;
        let eval_results: Vec<MultiSpaceEvalResult> = population
            .par_iter()
            .map(|genome| {
                evaluate_multi_genome(genome, &space_configs, &test_suite, base_seed)
            })
            .collect();

        // Write fitness back
        for (genome, result) in population.iter_mut().zip(eval_results.iter()) {
            genome.fitness = Some(result.final_fitness);
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
        let stats = MultiGenerationStats {
            generation: gen,
            best_fitness: population.first().and_then(|g| g.fitness).unwrap_or(0.0),
            average_fitness: population.iter().map(|g| g.fitness.unwrap_or(0.0)).sum::<f64>()
                / population.len() as f64,
            worst_fitness: population.last().and_then(|g| g.fitness).unwrap_or(0.0),
            best_genome_id: population.first().map(|g| g.id).unwrap_or(0),
            population_size: population.len(),
            eval_results: eval_results.clone(),
        };

        // Save generation results
        save_multi_generation(gen, &population, &stats, config);
        all_stats.push(stats);

        // Update STATUS.md
        write_multi_status_md(&all_stats, &population, config);

        // Print summary
        let best = population.first().and_then(|g| g.fitness).unwrap_or(0.0);
        let avg = population.iter().map(|g| g.fitness.unwrap_or(0.0)).sum::<f64>()
            / population.len() as f64;
        let best_id = population.first().map(|g| g.id).unwrap_or(0);
        eprintln!("  Best: {:.4}  Avg: {:.4}  Best ID: {}", best, avg, best_id);

        // Print best genome's per-question results
        if let Some(best_eval) = eval_results.first() {
            let correct = best_eval.report.total_correct;
            let total = best_eval.report.total_questions;
            eprintln!("  Score: {}/{} ({:.1}%)", correct, total, 100.0 * correct as f64 / total as f64);
        }

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
            let mut next_population: Vec<MultiSpaceGenome> =
                Vec::with_capacity(config.population_size);

            // Elitism
            for elite in population.iter().take(config.elitism_count) {
                let mut preserved = elite.clone();
                preserved.generation = gen + 1;
                preserved.fitness = None;
                next_population.push(preserved);
            }

            // Crossover + mutation
            while next_population.len() < config.population_size {
                let parent_a =
                    tournament_select_multi(&population, config.tournament_size, &mut rng);
                let parent_b =
                    tournament_select_multi(&population, config.tournament_size, &mut rng);

                id_counter += 1;
                let child = if rng.next_f64() < config.crossover_rate {
                    let crossed =
                        crossover_multi(parent_a, parent_b, &mut rng, id_counter, gen + 1);
                    mutate_multi(
                        &crossed,
                        &ranges,
                        effective_mutation_rate,
                        config.strategy_mutation_rate,
                        &mut rng,
                        id_counter,
                        gen + 1,
                    )
                } else {
                    mutate_multi(
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
            save_multi_checkpoint(gen + 1, &population, id_counter, config, &all_stats);
            break;
        }
    }

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

fn save_multi_checkpoint(
    gen: usize,
    population: &[MultiSpaceGenome],
    id_counter: u64,
    config: &MultiSpaceEvolutionConfig,
    all_stats: &[MultiGenerationStats],
) {
    let checkpoint = MultiSpaceCheckpoint {
        generation: gen,
        population: population.to_vec(),
        id_counter,
        config: config.clone(),
        all_generation_stats: all_stats.to_vec(),
    };
    let path = config.results_dir.join("checkpoint.json");
    let json = serde_json::to_string_pretty(&checkpoint).unwrap();
    std::fs::write(path, json).expect("Failed to write checkpoint");
    eprintln!("Multi-space checkpoint saved at generation {}.", gen);
}
