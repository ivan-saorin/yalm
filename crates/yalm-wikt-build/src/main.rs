mod parser;

use std::io::BufReader;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "yalm-wikt-build",
    about = "Parse Simple English Wiktionary XML dump into JSON cache"
)]
struct Cli {
    /// Path to Wiktionary XML dump file (e.g., simplewiktionary-*-pages-articles.xml)
    #[arg(long)]
    input: PathBuf,

    /// Output path for JSON cache file
    #[arg(long, default_value = "wikt_cache.json")]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    println!("=== YALM Wiktionary Cache Builder ===\n");
    println!("Input:  {:?}", cli.input);
    println!("Output: {:?}", cli.output);
    println!();

    // Open the XML file
    let file = std::fs::File::open(&cli.input).expect("Failed to open input XML file");
    let reader = BufReader::new(file);

    println!("Parsing XML dump...");
    let entries = parser::parse_wiktionary_dump(reader);

    println!("\nExtracted {} dictionary entries", entries.len());

    // Show some stats
    let total_defs: usize = entries.values().map(|e| e.definitions.len()).sum();
    let total_examples: usize = entries.values().map(|e| e.examples.len()).sum();
    let with_examples = entries.values().filter(|e| !e.examples.is_empty()).count();

    println!("  Total definitions:   {}", total_defs);
    println!("  Total examples:      {}", total_examples);
    println!(
        "  Entries w/ examples: {} ({:.1}%)",
        with_examples,
        100.0 * with_examples as f64 / entries.len().max(1) as f64
    );

    // Sample entries
    println!("\n--- Sample entries ---");
    for word in &["dog", "cat", "water", "sun", "animal", "person", "run", "big"] {
        if let Some(entry) = entries.get(*word) {
            println!(
                "  {} — {} ({}d, {}e)",
                entry.word,
                entry.definitions.first().map(|d| &d[..d.len().min(60)]).unwrap_or("?"),
                entry.definitions.len(),
                entry.examples.len(),
            );
        }
    }

    // Write JSON
    println!("\nWriting JSON to {:?}...", cli.output);
    let json = serde_json::to_string(&entries).expect("JSON serialization failed");
    std::fs::write(&cli.output, &json).expect("Failed to write output file");

    let size_mb = json.len() as f64 / (1024.0 * 1024.0);
    println!("Done. Output size: {:.1} MB", size_mb);
}
