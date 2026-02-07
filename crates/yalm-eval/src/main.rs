use std::path::PathBuf;

use clap::Parser;
use yalm_core::*;
use yalm_engine::strategy::StrategyConfig;
use yalm_engine::Engine;
use yalm_eval::{evaluate, print_space_statistics};
use yalm_parser::{parse_dictionary, parse_grammar_text, parse_test_questions};

#[derive(Parser)]
#[command(name = "yalm-eval", about = "Evaluate YALM engine on test questions")]
struct Cli {
    /// Path to dictionary file
    #[arg(long, default_value = "dictionaries/dict5.md")]
    dict: PathBuf,
    /// Path to test questions file
    #[arg(long, default_value = "dictionaries/dict5_test.md")]
    test: PathBuf,
    /// Path to grammar text file (optional, enables grammar reinforcement)
    #[arg(long)]
    grammar: Option<PathBuf>,
    /// Path to genome JSON file (optional, loads evolved parameters + strategy)
    #[arg(long)]
    genome: Option<PathBuf>,
    /// Build mode: "forcefield" (default) or "equilibrium"
    #[arg(long, default_value = "forcefield")]
    mode: String,
}

/// Genome structure matching what yalm-evolve produces.
#[derive(serde::Deserialize)]
struct GenomeFile {
    params: EngineParams,
    force_function: String,
    connector_detection: String,
    space_init: String,
    multi_connector: String,
    negation_model: String,
    #[serde(default)]
    use_connector_axis: bool,
}

fn parse_strategy(genome: &GenomeFile) -> StrategyConfig {
    use yalm_engine::strategy::*;
    StrategyConfig {
        force_function: match genome.force_function.as_str() {
            "Linear" => ForceFunction::Linear,
            "InverseDistance" => ForceFunction::InverseDistance,
            "Gravitational" => ForceFunction::Gravitational,
            "Spring" => ForceFunction::Spring,
            other => { eprintln!("Unknown force_function: {}, using Linear", other); ForceFunction::Linear }
        },
        connector_detection: match genome.connector_detection.as_str() {
            "FrequencyOnly" => ConnectorDetection::FrequencyOnly,
            "PositionalBias" => ConnectorDetection::PositionalBias,
            "MutualInformation" => ConnectorDetection::MutualInformation,
            other => { eprintln!("Unknown connector_detection: {}, using FrequencyOnly", other); ConnectorDetection::FrequencyOnly }
        },
        space_init: match genome.space_init.as_str() {
            "Random" => SpaceInitialization::Random,
            "Spherical" => SpaceInitialization::Spherical,
            "FromConnectors" => SpaceInitialization::FromConnectors,
            other => { eprintln!("Unknown space_init: {}, using Random", other); SpaceInitialization::Random }
        },
        multi_connector: match genome.multi_connector.as_str() {
            "FirstOnly" => MultiConnectorHandling::FirstOnly,
            "Sequential" => MultiConnectorHandling::Sequential,
            "Weighted" => MultiConnectorHandling::Weighted,
            "Compositional" => MultiConnectorHandling::Compositional,
            other => { eprintln!("Unknown multi_connector: {}, using Sequential", other); MultiConnectorHandling::Sequential }
        },
        negation_model: match genome.negation_model.as_str() {
            "Inversion" => NegationModel::Inversion,
            "Repulsion" => NegationModel::Repulsion,
            "AxisShift" => NegationModel::AxisShift,
            "SeparateDimension" => NegationModel::SeparateDimension,
            other => { eprintln!("Unknown negation_model: {}, using Inversion", other); NegationModel::Inversion }
        },
        use_connector_axis: genome.use_connector_axis,
    }
}

fn main() {
    let cli = Cli::parse();

    println!("=== YALM v0.1 \u{2014} Geometric Comprehension Engine ===\n");

    // ── Step 1: Parse dictionary ──────────────────────────────────
    let dict_content =
        std::fs::read_to_string(&cli.dict).expect("Failed to read dictionary file");
    let dictionary = parse_dictionary(&dict_content);
    println!("Parsed {} dictionary entries", dictionary.entries.len());

    // ── Step 2: Parse test questions ──────────────────────────────
    let test_content =
        std::fs::read_to_string(&cli.test).expect("Failed to read test file");
    let test_suite = parse_test_questions(&test_content);
    println!("Parsed {} test questions", test_suite.questions.len());

    // ── Step 3: Parse grammar text (optional) ─────────────────────
    let grammar = cli.grammar.as_ref().map(|path| {
        let content = std::fs::read_to_string(path).expect("Failed to read grammar file");
        let g = parse_grammar_text(&content);
        println!("Parsed {} grammar sections", g.entries.len());
        g
    });

    if grammar.is_some() {
        println!("[Grammar reinforcement: ON]");
    }
    println!();

    // ── Step 4: Load parameters from genome or use defaults ───────
    let (params, strategy): (EngineParams, StrategyConfig) = if let Some(genome_path) = &cli.genome {
        let content = std::fs::read_to_string(genome_path).expect("Failed to read genome file");
        let genome: GenomeFile = serde_json::from_str(&content).expect("Failed to parse genome JSON");
        let strategy = parse_strategy(&genome);
        let mut params = genome.params;
        params.rng_seed = 123; // consistent seed for manual testing
        println!("[Loaded genome from {:?}]", genome_path);
        (params, strategy)
    } else {
        (EngineParams::default(), StrategyConfig::default())
    };

    println!("Engine parameters:");
    println!("  dimensions:              {}", params.dimensions);
    println!("  learning_passes:         {}", params.learning_passes);
    println!("  force_magnitude:         {:.4}", params.force_magnitude);
    println!("  force_decay:             {:.4}", params.force_decay);
    println!("  connector_min_frequency: {}", params.connector_min_frequency);
    println!("  yes_threshold:           {:.4}", params.yes_threshold);
    println!("  no_threshold:            {:.4}", params.no_threshold);
    println!("  negation_inversion:      {:.4}", params.negation_inversion);
    println!("  bidirectional_force:     {:.4}", params.bidirectional_force);
    println!("  grammar_weight:          {:.4}", params.grammar_weight);
    println!("Strategy: {:?}", strategy);
    println!();

    // ── Step 5: Train ─────────────────────────────────────────────
    let mut engine = Engine::with_strategy(params.clone(), strategy.clone());

    let build_mode = match cli.mode.as_str() {
        "equilibrium" | "eq" => yalm_engine::BuildMode::Equilibrium,
        _ => yalm_engine::BuildMode::ForceField,
    };
    engine.set_mode(build_mode);
    println!("Build mode: {:?}", build_mode);

    if let Some(ref grammar) = grammar {
        engine.train_with_grammar(&dictionary, grammar);
    } else {
        engine.train(&dictionary);
    }
    println!();

    // ── Step 6: Dump space statistics ─────────────────────────────
    print_space_statistics(engine.space(), &dictionary);

    // ── Step 7: Run test questions ────────────────────────────────
    println!("\n=== Test Results ===\n");
    let report = evaluate(&engine, &test_suite, &dictionary, &params, &strategy);

    for result in &report.results {
        let status = if result.correct { "PASS" } else { "FAIL" };
        println!(
            "[{}] {} \u{2014} {} | expected: {} | actual: {} | dist: {:.4} | connector: {}",
            status,
            result.question_id,
            result.question_text,
            result.expected,
            result.actual,
            result.projection_distance.unwrap_or(f64::NAN),
            result.connector_used.as_deref().unwrap_or("none"),
        );
    }

    // ── Step 8: Print fitness ─────────────────────────────────────
    println!("\n=== Fitness Report ===");
    println!(
        "  Accuracy:  {:.2} ({}/{} answerable correct)",
        report.accuracy,
        report
            .results
            .iter()
            .filter(|r| r.expected != ExpectedAnswer::IDontKnow && r.correct)
            .count(),
        report
            .results
            .iter()
            .filter(|r| r.expected != ExpectedAnswer::IDontKnow)
            .count(),
    );
    println!(
        "  Honesty:   {:.2} ({}/{} unknowable correct)",
        report.honesty,
        report
            .results
            .iter()
            .filter(|r| r.expected == ExpectedAnswer::IDontKnow && r.correct)
            .count(),
        report
            .results
            .iter()
            .filter(|r| r.expected == ExpectedAnswer::IDontKnow)
            .count(),
    );
    println!("  FITNESS:   {:.4}", report.fitness);
    println!(
        "  Total:     {}/{} correct",
        report.total_correct, report.total_questions
    );
}
