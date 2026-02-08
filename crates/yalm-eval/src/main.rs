use std::path::PathBuf;

use clap::Parser;
use yalm_core::*;
use yalm_cache::{AssemblerConfig, DictionaryAssembler, DictionaryCache, ManualFileCache, OllamaCache, WiktionaryCache};
use yalm_engine::strategy::StrategyConfig;
use yalm_engine::Engine;
use yalm_eval::{evaluate, print_space_statistics};
use yalm_parser::{parse_dictionary, parse_grammar_text, parse_test_questions};

#[derive(Parser)]
#[command(name = "yalm-eval", about = "Evaluate YALM engine on test questions")]
struct Cli {
    /// Path to dictionary file (closed mode)
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

    // ── Open mode (dictionary cache) ──────────────────────────────
    /// Path to free text file (triggers open mode when provided)
    #[arg(long)]
    text: Option<PathBuf>,
    /// Cache backend: "manual", "wiktionary", or "ollama"
    #[arg(long, default_value = "manual")]
    cache_type: String,
    /// Path to cache file or directory
    #[arg(long)]
    cache: Option<PathBuf>,
    /// Maximum BFS depth for closure chase (open mode)
    #[arg(long, default_value = "3")]
    max_depth: usize,
    /// Maximum words in assembled dictionary (open mode)
    #[arg(long, default_value = "5000")]
    max_words: usize,

    // ── Entity definitions ─────────────────────────────────────────
    /// Path to entity definitions file (merged into dictionary, overrides cache)
    #[arg(long)]
    entities: Option<PathBuf>,

    // ── Ollama options ─────────────────────────────────────────────
    /// Ollama API base URL (only used with --cache-type ollama)
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,
    /// Ollama model name (only used with --cache-type ollama)
    #[arg(long, default_value = "qwen3:8b")]
    ollama_model: String,
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

    // ── Load parameters from genome or use defaults ──────────────
    let (params, strategy): (EngineParams, StrategyConfig) = if let Some(genome_path) = &cli.genome {
        let content = std::fs::read_to_string(genome_path).expect("Failed to read genome file");
        let genome: GenomeFile = serde_json::from_str(&content).expect("Failed to parse genome JSON");
        let strategy = parse_strategy(&genome);
        let mut params = genome.params;
        params.rng_seed = 123;
        println!("[Loaded genome from {:?}]", genome_path);
        (params, strategy)
    } else {
        (EngineParams::default(), StrategyConfig::default())
    };

    // ── Build or assemble dictionary ─────────────────────────────
    let mut dictionary = if let Some(text_path) = &cli.text {
        // OPEN MODE: assemble dictionary from free text + cache
        let text = std::fs::read_to_string(text_path).expect("Failed to read text file");

        let cache_path = cli.cache.as_ref().unwrap_or_else(|| {
            eprintln!("Error: --cache is required when using --text (open mode)");
            std::process::exit(1);
        });

        let cache: Box<dyn DictionaryCache> = match cli.cache_type.as_str() {
            "manual" => {
                Box::new(ManualFileCache::load(cache_path).expect("Failed to load manual cache"))
            }
            "wiktionary" | "wikt" => {
                Box::new(WiktionaryCache::load(cache_path).expect("Failed to load wiktionary cache"))
            }
            "ollama" => {
                let ollama = OllamaCache::new(&cli.ollama_url, &cli.ollama_model, cache_path)
                    .expect("Failed to create OllamaCache");
                // Health check: fail fast if Ollama isn't running
                if let Err(e) = ollama.check_health() {
                    eprintln!("Error: Ollama health check failed: {}", e);
                    eprintln!("  Is Ollama running? Try: ollama serve");
                    eprintln!("  Is model pulled? Try: ollama pull {}", cli.ollama_model);
                    std::process::exit(1);
                }
                // Pre-load disk cache for fast second runs
                ollama.preload_disk_cache();
                Box::new(ollama)
            }
            other => {
                eprintln!("Unknown cache type: '{}'. Use 'manual', 'wiktionary', or 'ollama'.", other);
                std::process::exit(1);
            }
        };

        println!("[Open mode: assembling dictionary from {:?}]", text_path);
        println!("Cache: {} ({} entries)", cache.name(), cache.len());

        let config = AssemblerConfig {
            max_depth: cli.max_depth,
            max_words: cli.max_words,
            ..Default::default()
        };
        let assembler = DictionaryAssembler::new(cache.as_ref(), config);
        let (dictionary, report) = assembler.assemble(&text);

        // Print assembly report
        println!("\n=== Assembly Report ===");
        println!("  Seed words:      {}", report.seed_words);
        for (depth, count) in report.words_per_depth.iter().enumerate() {
            println!("  Depth {} found:    {}", depth, count);
        }
        println!("  Total assembled: {}", report.total_entries);
        println!("  Not found:       {}", report.words_not_found.len());
        if !report.words_not_found.is_empty() {
            let display: Vec<&str> = report.words_not_found.iter().take(20).map(|s| s.as_str()).collect();
            println!("    {:?}{}", display, if report.words_not_found.len() > 20 { " ..." } else { "" });
        }
        println!("  Closure ratio:   {:.1}%", report.closure_ratio * 100.0);

        dictionary
    } else if cli.entities.is_some() && cli.dict == PathBuf::from("dictionaries/dict5.md") {
        // ENTITIES-ONLY MODE: no --text and no explicit --dict override
        // The entities file will be loaded as the dictionary below
        let entities_path = cli.entities.as_ref().unwrap();
        let content = std::fs::read_to_string(entities_path)
            .expect("Failed to read entities file");
        let dict = parse_dictionary(&content);
        println!("[Entities-only mode: {} entries from {:?}]", dict.entries.len(), entities_path);
        dict
    } else {
        // CLOSED MODE: parse dictionary from .md file
        let dict_content =
            std::fs::read_to_string(&cli.dict).expect("Failed to read dictionary file");
        parse_dictionary(&dict_content)
    };

    // ── Merge entity definitions ──────────────────────────────────
    if let Some(entities_path) = &cli.entities {
        // Only merge if we didn't already use entities as the full dictionary
        let is_entities_only = cli.text.is_none() && cli.dict == PathBuf::from("dictionaries/dict5.md");
        if !is_entities_only {
            let entities_content = std::fs::read_to_string(entities_path)
                .expect("Failed to read entities file");
            let entities_dict = parse_dictionary(&entities_content);

            println!("[Entities: {} entries from {:?}]", entities_dict.entries.len(), entities_path);

            // Merge: entity entries override assembled entries for same word
            let mut entry_map: std::collections::HashMap<String, DictionaryEntry> =
                dictionary.entries.into_iter().map(|e| (e.word.clone(), e)).collect();

            for entity_entry in entities_dict.entries {
                entry_map.insert(entity_entry.word.clone(), entity_entry);
            }

            let mut entries: Vec<DictionaryEntry> = entry_map.into_values().collect();
            entries.sort_by(|a, b| a.word.cmp(&b.word));
            let entry_words: Vec<String> = entries.iter().map(|e| e.word.clone()).collect();
            let entry_set: std::collections::HashSet<String> = entry_words.iter().cloned().collect();

            dictionary = Dictionary { entries, entry_words, entry_set };
            println!("Dictionary after entity merge: {} entries", dictionary.entries.len());
        }
    }

    println!("Dictionary: {} entries", dictionary.entries.len());

    // ── Parse test questions ──────────────────────────────────────
    let test_content =
        std::fs::read_to_string(&cli.test).expect("Failed to read test file");
    let test_suite = parse_test_questions(&test_content);
    println!("Test questions: {}", test_suite.questions.len());

    // ── Parse grammar text (optional) ─────────────────────────────
    let grammar = cli.grammar.as_ref().map(|path| {
        let content = std::fs::read_to_string(path).expect("Failed to read grammar file");
        let g = parse_grammar_text(&content);
        println!("Grammar sections: {}", g.entries.len());
        g
    });

    if grammar.is_some() {
        println!("[Grammar reinforcement: ON]");
    }

    // ── Print engine config ───────────────────────────────────────
    println!();
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

    // ── Train ─────────────────────────────────────────────────────
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

    // ── Space statistics ──────────────────────────────────────────
    print_space_statistics(engine.space(), &dictionary);

    // ── Test results ──────────────────────────────────────────────
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

    // ── Fitness report ────────────────────────────────────────────
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
