use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use dafhne_core::*;
use dafhne_cache::{AssemblerConfig, DictionaryAssembler, DictionaryCache, ManualFileCache, OllamaCache, WiktionaryCache};
use dafhne_engine::strategy::StrategyConfig;
use dafhne_engine::Engine;
use dafhne_parser::parse_dictionary;

// ─── CLI ────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "dafhne-demo", about = "DAPHNE — Geometric Comprehension Demo")]
struct Cli {
    /// Knowledge files (dictionaries or free text)
    #[arg(short = 'k', long = "knowledge", required = true, num_args = 1..)]
    knowledge: Vec<PathBuf>,

    /// Questions file (one question per line)
    #[arg(short = 'q', long = "questions", required = true)]
    questions: PathBuf,

    /// Build mode: "equilibrium" (default) or "forcefield"
    #[arg(long, default_value = "equilibrium")]
    mode: String,

    // ── Open mode (dictionary cache) ──────────────────────────────
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
    /// Path to entity definitions file (merged into dictionary)
    #[arg(long)]
    entities: Option<PathBuf>,

    // ── Ollama options ─────────────────────────────────────────────
    /// Ollama API base URL
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,
    /// Ollama model name
    #[arg(long, default_value = "qwen3:8b")]
    ollama_model: String,
}

// ─── Helpers ────────────────────────────────────────────────────

/// Detect if file content is dictionary format (has `**word** —` entries).
fn is_dictionary_format(content: &str) -> bool {
    content.lines().any(|line| {
        let t = line.trim();
        t.starts_with("**")
            && t.contains("**")
            && (t.contains('\u{2014}') || t.contains("---"))
    })
}

/// Parse a simple questions file: one question per line.
/// Lines starting with `#`, `---`, or blank are skipped.
fn parse_simple_questions(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#') && *l != "---")
        .map(|l| l.to_string())
        .collect()
}

/// Get just the filename from a path.
fn filename(path: &PathBuf) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

// ─── Pretty-print helpers ───────────────────────────────────────

const BOX_W: usize = 52;

fn print_header() {
    println!();
    println!("  \u{2554}{}\u{2557}", "\u{2550}".repeat(BOX_W));
    println!("  \u{2551}{:^width$}\u{2551}", "DAPHNE \u{2014} Geometric Comprehension", width = BOX_W);
    println!("  \u{255A}{}\u{255D}", "\u{2550}".repeat(BOX_W));
    println!();
}

fn print_box_top(title: &str) {
    let dash_len = BOX_W - 2 - title.len();
    println!("  \u{250C}\u{2500} {} {}\u{2510}", title, "\u{2500}".repeat(dash_len));
}

fn print_box_line(text: &str) {
    // Pad text to box width, accounting for Unicode display width
    let visible_len = text.chars().count();
    let pad = if visible_len < BOX_W { BOX_W - visible_len } else { 0 };
    println!("  \u{2502} {}{}\u{2502}", text, " ".repeat(pad));
}

fn print_box_empty() {
    println!("  \u{2502}{}\u{2502}", " ".repeat(BOX_W + 1));
}

fn print_box_bottom() {
    println!("  \u{2514}{}\u{2518}", "\u{2500}".repeat(BOX_W + 1));
}

// ─── Main ───────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let total_start = Instant::now();

    // ── Print header ─────────────────────────────────────────────
    print_header();

    // ── Load questions ───────────────────────────────────────────
    let q_content = std::fs::read_to_string(&cli.questions)
        .expect("Failed to read questions file");
    let questions = parse_simple_questions(&q_content);

    // ── Detect mode and load knowledge ───────────────────────────
    let knowledge_names: Vec<String> = cli.knowledge.iter().map(|p| filename(p)).collect();

    // Read all knowledge files
    let mut dict_contents: Vec<String> = Vec::new();
    let mut text_contents: Vec<String> = Vec::new();

    for path in &cli.knowledge {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| {
                eprintln!("Error reading {:?}: {}", path, e);
                std::process::exit(1);
            });
        if is_dictionary_format(&content) {
            dict_contents.push(content);
        } else {
            text_contents.push(content);
        }
    }

    let has_text = !text_contents.is_empty();

    // Build dictionary from dictionary files
    let mut dictionary = if !dict_contents.is_empty() {
        let combined = dict_contents.join("\n---\n");
        parse_dictionary(&combined)
    } else {
        Dictionary {
            entries: Vec::new(),
            entry_words: Vec::new(),
            entry_set: std::collections::HashSet::new(),
        }
    };

    // Assemble from free text if any
    if has_text {
        let cache_path = cli.cache.as_ref().unwrap_or_else(|| {
            eprintln!("Error: --cache is required when knowledge files contain free text");
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
                if let Err(e) = ollama.check_health() {
                    eprintln!("Error: Ollama health check failed: {}", e);
                    std::process::exit(1);
                }
                ollama.preload_disk_cache();
                Box::new(ollama)
            }
            other => {
                eprintln!("Unknown cache type: '{}'. Use 'manual', 'wiktionary', or 'ollama'.", other);
                std::process::exit(1);
            }
        };

        let config = AssemblerConfig {
            max_depth: cli.max_depth,
            max_words: cli.max_words,
            ..Default::default()
        };
        let assembler = DictionaryAssembler::new(cache.as_ref(), config);

        for text in &text_contents {
            let (text_dict, _report) = assembler.assemble(text);

            // Merge: text entries added, dict entries take precedence
            let mut entry_map: std::collections::HashMap<String, DictionaryEntry> =
                text_dict.entries.into_iter().map(|e| (e.word.clone(), e)).collect();

            // Existing dictionary entries override
            for entry in dictionary.entries.drain(..) {
                entry_map.insert(entry.word.clone(), entry);
            }

            let mut entries: Vec<DictionaryEntry> = entry_map.into_values().collect();
            entries.sort_by(|a, b| a.word.cmp(&b.word));
            let entry_words: Vec<String> = entries.iter().map(|e| e.word.clone()).collect();
            let entry_set: std::collections::HashSet<String> = entry_words.iter().cloned().collect();
            dictionary = Dictionary { entries, entry_words, entry_set };
        }
    }

    // Merge entity definitions
    if let Some(ref entities_path) = cli.entities {
        let entities_content = std::fs::read_to_string(entities_path)
            .expect("Failed to read entities file");
        let entities_dict = parse_dictionary(&entities_content);

        let mut entry_map: std::collections::HashMap<String, DictionaryEntry> =
            dictionary.entries.into_iter().map(|e| (e.word.clone(), e)).collect();

        for mut entity_entry in entities_dict.entries {
            entity_entry.is_entity = true;
            entry_map.insert(entity_entry.word.clone(), entity_entry);
        }

        let mut entries: Vec<DictionaryEntry> = entry_map.into_values().collect();
        entries.sort_by(|a, b| a.word.cmp(&b.word));
        let entry_words: Vec<String> = entries.iter().map(|e| e.word.clone()).collect();
        let entry_set: std::collections::HashSet<String> = entry_words.iter().cloned().collect();
        dictionary = Dictionary { entries, entry_words, entry_set };
    }

    // ── Print config summary ─────────────────────────────────────
    let build_mode = match cli.mode.as_str() {
        "equilibrium" | "eq" => dafhne_engine::BuildMode::Equilibrium,
        _ => dafhne_engine::BuildMode::ForceField,
    };
    let mode_str = match build_mode {
        dafhne_engine::BuildMode::Equilibrium => "Equilibrium",
        dafhne_engine::BuildMode::ForceField => "ForceField",
    };
    let params = EngineParams::default();

    println!("  Knowledge : {} ({})", knowledge_names.join(", "),
        if has_text { format!("{} entries (assembled)", dictionary.entries.len()) }
        else { format!("{} entries", dictionary.entries.len()) });
    if let Some(ref ep) = cli.entities {
        println!("  Entities  : {}", filename(ep));
    }
    println!("  Questions : {} ({} questions)", filename(&cli.questions), questions.len());
    println!("  Build mode: {}", mode_str);
    println!("  Dimensions: {}", params.dimensions);
    println!();

    // ── Train ────────────────────────────────────────────────────
    let train_start = Instant::now();

    let strategy = StrategyConfig::default();
    let mut engine = Engine::with_strategy(params.clone(), strategy.clone());
    engine.set_mode(build_mode);
    engine.set_quiet(true);
    engine.train(&dictionary);

    let train_elapsed = train_start.elapsed();

    // ── Print build stats ────────────────────────────────────────
    let space = engine.space();
    print_box_top("Building space");
    print_box_line(&format!("Dictionary:  {} entries", dictionary.entries.len()));
    print_box_line(&format!("Connectors:  {} discovered", space.connectors.len()));
    print_box_line(&format!("Space:       {} words in {}-D", space.words.len(), space.dimensions));
    print_box_line(&format!("Build time:  {:.3}s", train_elapsed.as_secs_f64()));
    print_box_bottom();
    println!();

    // ── Answer questions ─────────────────────────────────────────
    let qa_start = Instant::now();

    let mut results: Vec<(String, Answer)> = Vec::new();
    for q in &questions {
        let (answer, _distance, _connector) = dafhne_engine::resolver::resolve_question(
            q,
            engine.space(),
            &dictionary,
            engine.structural(),
            engine.content(),
            &params,
            &strategy,
        );
        results.push((q.clone(), answer));
    }

    let qa_elapsed = qa_start.elapsed();

    // ── Print Q&A ────────────────────────────────────────────────
    print_box_top("Q&A");
    for (question, answer) in &results {
        print_box_empty();
        print_box_line(&format!("Q: {}", question));
        print_box_line(&format!("A: {}", answer));
    }
    print_box_empty();
    print_box_bottom();
    println!();

    // ── Summary ──────────────────────────────────────────────────
    let total_elapsed = total_start.elapsed();
    println!(
        "  Answered {} questions in {:.3}s (total: {:.3}s)",
        results.len(),
        qa_elapsed.as_secs_f64(),
        total_elapsed.as_secs_f64(),
    );
    println!();
}
