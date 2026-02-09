use std::path::PathBuf;

use clap::{Parser, Subcommand};
use dafhne_core::Comprehend;
use dafhne_engine::Engine;
use dafhne_evolve::analysis::run_full_analysis;
use dafhne_evolve::fitness::build_trained_space;
use dafhne_evolve::genome::Genome;
use dafhne_evolve::runner::{evolve, resume, EvolutionConfig};
use dafhne_parser::{parse_dictionary, parse_test_questions};

#[derive(Parser)]
#[command(name = "dafhne-evolve", about = "Evolutionary self-improvement for DAPHNE")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the evolutionary algorithm
    Run {
        #[arg(long)]
        dict5: PathBuf,
        #[arg(long)]
        test5: PathBuf,
        #[arg(long)]
        dict12: Option<PathBuf>,
        #[arg(long)]
        test12: Option<PathBuf>,
        /// Optional grammar text file for grammar reinforcement
        #[arg(long)]
        grammar5: Option<PathBuf>,
        #[arg(long)]
        dict18: Option<PathBuf>,
        #[arg(long)]
        test18: Option<PathBuf>,
        #[arg(long)]
        grammar18: Option<PathBuf>,
        #[arg(long, default_value = "30")]
        population: usize,
        #[arg(long, default_value = "20")]
        generations: usize,
        #[arg(long, default_value = "results")]
        results: PathBuf,
        #[arg(long, default_value = "42")]
        seed: u64,
        #[arg(long, default_value = "4")]
        elitism: usize,
        #[arg(long, default_value = "5")]
        tournament: usize,
    },
    /// Analyze a specific generation's results
    Analyze {
        /// Path to generation directory (e.g., results/gen_015/)
        path: PathBuf,
        /// Path to dict5.md for rebuilding the space
        #[arg(long)]
        dict5: PathBuf,
    },
    /// Run the best genome from results on a specific dictionary
    RunBest {
        /// Path to results directory
        results: PathBuf,
        #[arg(long)]
        dict: PathBuf,
        #[arg(long)]
        test: PathBuf,
    },
    /// Resume interrupted evolution from checkpoint
    Resume {
        /// Path to results directory containing checkpoint.json
        results: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            dict5,
            test5,
            dict12,
            test12,
            grammar5,
            dict18,
            test18,
            grammar18,
            population,
            generations,
            results,
            seed,
            elitism,
            tournament,
        } => {
            let config = EvolutionConfig {
                population_size: population,
                generations,
                elitism_count: elitism,
                tournament_size: tournament,
                mutation_rate: 0.2,
                crossover_rate: 0.7,
                strategy_mutation_rate: 0.1,
                cross_validation_threshold: 0.4,
                base_seed: seed,
                dict5_path: dict5,
                dict5_test_path: test5,
                dict12_path: dict12,
                dict12_test_path: test12,
                grammar5_path: grammar5,
                dict18_path: dict18,
                dict18_test_path: test18,
                grammar18_path: grammar18,
                results_dir: results,
            };
            let best = evolve(&config);
            println!(
                "\nEvolution complete. Best genome ID: {} fitness: {:.4}",
                best.id,
                best.fitness.unwrap_or(0.0)
            );
            println!("Parameters: {:?}", best.params);
        }
        Commands::Analyze { path, dict5 } => {
            cmd_analyze(&path, &dict5);
        }
        Commands::RunBest {
            results,
            dict,
            test,
        } => {
            cmd_run_best(&results, &dict, &test);
        }
        Commands::Resume { results } => {
            let best = resume(&results);
            println!(
                "Resumed evolution complete. Best: ID {} fitness {:.4}",
                best.id,
                best.fitness.unwrap_or(0.0)
            );
        }
    }
}

fn cmd_analyze(gen_path: &PathBuf, dict5_path: &PathBuf) {
    let best_json =
        std::fs::read_to_string(gen_path.join("best_genome.json")).expect("Failed to read best_genome.json");
    let best: Genome = serde_json::from_str(&best_json).expect("Failed to parse best_genome.json");

    println!("Analyzing best genome ID: {}", best.id);
    println!("Fitness: {:.4}", best.fitness.unwrap_or(0.0));
    println!("Parameters: {:?}", best.params);
    println!("Strategies:");
    println!("  force_function: {:?}", best.force_function);
    println!("  connector_detection: {:?}", best.connector_detection);
    println!("  space_init: {:?}", best.space_init);
    println!("  multi_connector: {:?}", best.multi_connector);
    println!("  negation_model: {:?}", best.negation_model);
    println!();

    // Rebuild space
    let dict_content = std::fs::read_to_string(dict5_path).expect("Failed to read dictionary");
    let dictionary = parse_dictionary(&dict_content);
    let space = build_trained_space(&best, &dictionary, None, 42);

    run_full_analysis(&space);
}

fn cmd_run_best(results_dir: &PathBuf, dict_path: &PathBuf, test_path: &PathBuf) {
    // Find the highest-numbered gen_NNN directory
    let mut best_gen_dir: Option<PathBuf> = None;
    let mut best_gen_num: i64 = -1;

    if let Ok(entries) = std::fs::read_dir(results_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("gen_") {
                if let Ok(num) = name[4..].parse::<i64>() {
                    if num > best_gen_num {
                        best_gen_num = num;
                        best_gen_dir = Some(entry.path());
                    }
                }
            }
        }
    }

    let gen_dir = best_gen_dir.expect("No generation directories found in results");
    let best_json =
        std::fs::read_to_string(gen_dir.join("best_genome.json")).expect("Failed to read best_genome.json");
    let best: Genome = serde_json::from_str(&best_json).expect("Failed to parse best_genome.json");

    println!("Using best genome from generation {}", best_gen_num);
    println!("Genome ID: {}, fitness: {:.4}", best.id, best.fitness.unwrap_or(0.0));
    println!("Parameters: {:?}", best.params);
    println!();

    // Load dictionary and test
    let dict_content = std::fs::read_to_string(dict_path).expect("Failed to read dictionary");
    let dictionary = parse_dictionary(&dict_content);
    let test_content = std::fs::read_to_string(test_path).expect("Failed to read test file");
    let test_suite = parse_test_questions(&test_content);

    // Build engine and evaluate
    let engine_params = best.to_engine_params(42);
    let strategy = best.to_strategy_config();
    let mut engine = Engine::with_strategy(engine_params.clone(), strategy.clone());
    engine.train(&dictionary);

    dafhne_eval::print_space_statistics(engine.space(), &dictionary);

    println!("\n=== Test Results ===\n");
    let report = dafhne_eval::evaluate(&engine, &test_suite, &dictionary, &engine_params, &strategy);

    for result in &report.results {
        let status = if result.correct { "PASS" } else { "FAIL" };
        println!(
            "[{}] {} \u{2014} {} | expected: {} | actual: {} | dist: {:.4}",
            status,
            result.question_id,
            result.question_text,
            result.expected,
            result.actual,
            result.projection_distance.unwrap_or(f64::NAN),
        );
    }

    println!("\n=== Fitness Report ===");
    println!("  Accuracy: {:.4}", report.accuracy);
    println!("  Honesty:  {:.4}", report.honesty);
    println!("  FITNESS:  {:.4}", report.fitness);
}
