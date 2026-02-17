use std::collections::HashSet;
use dafhne_core::{Dictionary, DictionaryEntry};

/// Parse a dictionary markdown file into a Dictionary struct.
pub fn parse_dictionary(content: &str) -> Dictionary {
    let mut entries: Vec<DictionaryEntry> = Vec::new();
    let mut current_section = String::new();
    let mut current_word: Option<String> = None;
    let mut current_definition = String::new();
    let mut current_examples: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines, blockquotes, and the title line
        if trimmed.is_empty() || trimmed.starts_with('>') || trimmed.starts_with("# dict") {
            continue;
        }

        // Section separator
        if trimmed == "---" {
            // Finalize current entry if any
            if let Some(word) = current_word.take() {
                entries.push(DictionaryEntry {
                    word,
                    definition: current_definition.clone(),
                    examples: current_examples.clone(),
                    section: current_section.clone(),
                    is_entity: false,
                });
                current_definition.clear();
                current_examples.clear();
            }
            continue;
        }

        // Section header: ## SECTION NAME
        if trimmed.starts_with("## ") {
            // Finalize current entry if any
            if let Some(word) = current_word.take() {
                entries.push(DictionaryEntry {
                    word,
                    definition: current_definition.clone(),
                    examples: current_examples.clone(),
                    section: current_section.clone(),
                    is_entity: false,
                });
                current_definition.clear();
                current_examples.clear();
            }
            current_section = trimmed[3..].trim().to_string();
            continue;
        }

        // Entry line: **word** — definition
        if let Some((word, definition)) = parse_entry_line(trimmed) {
            // Finalize previous entry if any
            if let Some(prev_word) = current_word.take() {
                entries.push(DictionaryEntry {
                    word: prev_word,
                    definition: current_definition.clone(),
                    examples: current_examples.clone(),
                    section: current_section.clone(),
                    is_entity: false,
                });
                current_definition.clear();
                current_examples.clear();
            }
            current_word = Some(word);
            current_definition = definition;
            continue;
        }

        // Example line: - "example text"
        if let Some(example) = parse_example_line(trimmed) {
            current_examples.push(example);
            continue;
        }
    }

    // Finalize last entry
    if let Some(word) = current_word.take() {
        entries.push(DictionaryEntry {
            word,
            definition: current_definition,
            examples: current_examples,
            section: current_section,
            is_entity: false,
        });
    }

    let entry_words: Vec<String> = entries.iter().map(|e| e.word.clone()).collect();
    let entry_set: HashSet<String> = entry_words.iter().cloned().collect();

    Dictionary {
        entries,
        entry_words,
        entry_set,
    }
}

/// Parse an entry line like: **word** — definition text.
/// Returns (word, definition) or None if not an entry line.
fn parse_entry_line(line: &str) -> Option<(String, String)> {
    if !line.starts_with("**") {
        return None;
    }

    // Find the closing **
    let after_open = &line[2..];
    let close_pos = after_open.find("**")?;
    let word = after_open[..close_pos].trim().to_lowercase();

    // Find the em-dash (Unicode U+2014 or triple hyphen ---)
    let rest = &after_open[close_pos + 2..];

    let definition = if let Some(pos) = rest.find('\u{2014}') {
        // em-dash is 3 bytes in UTF-8
        rest[pos + '\u{2014}'.len_utf8()..].trim().to_string()
    } else if let Some(pos) = rest.find("---") {
        rest[pos + 3..].trim().to_string()
    } else {
        // Not an entry line (e.g., **Total entries**: or **Status**:)
        return None;
    };

    if word.is_empty() || definition.is_empty() {
        return None;
    }

    Some((word, definition))
}

/// Parse an example line like: - "example text here"
fn parse_example_line(line: &str) -> Option<String> {
    if !line.starts_with("- \"") && !line.starts_with("- \u{201C}") {
        return None;
    }

    // Find first and last quote marks (handle both regular and smart quotes)
    let start = line
        .find('"')
        .or_else(|| line.find('\u{201C}'))
        .map(|p| p + 1)?;

    let end = line
        .rfind('"')
        .or_else(|| line.rfind('\u{201D}'))
        .filter(|&p| p > start)?;

    // Handle the case where rfind found the opening quote
    if end <= start {
        return None;
    }

    Some(line[start..end].to_string())
}

/// Parse a grammar text markdown file into a Dictionary struct.
/// Grammar text has `## ` section headers followed by prose sentences.
/// Each section becomes a DictionaryEntry with a placeholder word (_grammar_N)
/// so `extract_all_sentences()` can pull out the sentences for relation extraction.
pub fn parse_grammar_text(content: &str) -> Dictionary {
    let mut entries: Vec<DictionaryEntry> = Vec::new();
    let mut current_lines: Vec<String> = Vec::new();
    let mut section_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines, blockquotes, horizontal rules, and the title line
        if trimmed.is_empty()
            || trimmed.starts_with('>')
            || trimmed == "---"
            || trimmed.starts_with("# ")
        {
            continue;
        }

        if trimmed.starts_with("## ") {
            // Finalize previous section
            if !current_lines.is_empty() {
                section_count += 1;
                finalize_grammar_section(&current_lines, section_count, &mut entries);
                current_lines.clear();
            }
            continue;
        }

        // Accumulate prose lines
        current_lines.push(trimmed.to_string());
    }

    // Finalize last section
    if !current_lines.is_empty() {
        section_count += 1;
        finalize_grammar_section(&current_lines, section_count, &mut entries);
    }

    let entry_words: Vec<String> = entries.iter().map(|e| e.word.clone()).collect();
    let entry_set: HashSet<String> = entry_words.iter().cloned().collect();

    Dictionary {
        entries,
        entry_words,
        entry_set,
    }
}

/// Check if a sentence is meta-language (describes how the system works)
/// rather than a factual statement about the world.
/// Meta sentences create spurious relations between unrelated words.
fn is_meta_sentence(sentence: &str) -> bool {
    let lower = sentence.to_lowercase();
    // Filter patterns that are descriptions ABOUT how connectors work.
    // These create spurious relations (e.g., "see" -> "is" -> "tell" -> "thing").
    // Be precise: "tell" and "see" in combination are the strongest meta markers.
    // Keep sentences with "not", "but", "know" since they teach negation/IDK.
    let meta_patterns = [
        "tells you",   // "This tells you:", "it tells you", "The name tells you"
        "you see",     // "When you see", "you can see it this way"
        "you say",     // "you say 'I do not know'"
        "you can not say",  // "You can not say..."
        "you can not give", // "You can not give a name..."
        "you can not make", // "You can not make one"
    ];
    meta_patterns.iter().any(|pattern| lower.contains(pattern))
}

/// Helper: convert accumulated prose lines into a DictionaryEntry.
/// Filters out meta-language sentences that describe how connectors work,
/// keeping only factual statements that reinforce word relationships.
fn finalize_grammar_section(
    lines: &[String],
    section_num: usize,
    entries: &mut Vec<DictionaryEntry>,
) {
    let sentences: Vec<String> = lines
        .iter()
        .flat_map(|l| l.split('.'))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| !is_meta_sentence(s))
        .collect();

    if sentences.is_empty() {
        return;
    }

    let definition = sentences[0].clone();
    let examples = if sentences.len() > 1 {
        sentences[1..].to_vec()
    } else {
        Vec::new()
    };

    entries.push(DictionaryEntry {
        word: format!("_grammar_{}", section_num),
        definition,
        examples,
        section: "grammar".to_string(),
        is_entity: false,
    });
}

/// Parse a TOML dictionary package file into a Dictionary struct.
/// Expected format:
/// ```toml
/// [package]
/// name = "dict-name"
/// ...
///
/// [dictionary]
/// word = "definition text"
/// ```
pub fn parse_toml_dictionary(content: &str) -> Dictionary {
    let toml_value: toml::Value = toml::from_str(content)
        .expect("Failed to parse TOML dictionary");

    let dict_table = toml_value
        .get("dictionary")
        .and_then(|v| v.as_table())
        .expect("TOML dictionary must have [dictionary] section");

    let mut entries: Vec<DictionaryEntry> = dict_table
        .iter()
        .map(|(word, definition)| {
            DictionaryEntry {
                word: word.to_lowercase(),
                definition: definition.as_str().unwrap_or("").to_string(),
                examples: Vec::new(),
                section: "default".to_string(),
                is_entity: false,
            }
        })
        .collect();

    entries.sort_by(|a, b| a.word.cmp(&b.word));

    let entry_words: Vec<String> = entries.iter().map(|e| e.word.clone()).collect();
    let entry_set: HashSet<String> = entry_words.iter().cloned().collect();

    Dictionary {
        entries,
        entry_words,
        entry_set,
    }
}

/// Load a dictionary from a file path, auto-detecting format by extension.
/// - `.toml` files → parse_toml_dictionary()
/// - everything else → parse_dictionary() (markdown)
pub fn load_dictionary(path: impl AsRef<std::path::Path>) -> std::io::Result<Dictionary> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)?;

    let is_toml = path.extension().map_or(false, |ext| ext == "toml");
    let dictionary = if is_toml {
        parse_toml_dictionary(&content)
    } else {
        parse_dictionary(&content)
    };

    Ok(dictionary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_entry_line_emdash() {
        let line = "**dog** \u{2014} an animal. it can make sound. it can live with a person.";
        let (word, def) = parse_entry_line(line).unwrap();
        assert_eq!(word, "dog");
        assert!(def.starts_with("an animal"));
    }

    #[test]
    fn test_parse_entry_line_not_entry() {
        assert!(parse_entry_line("**Total entries**: 50").is_none());
        assert!(parse_entry_line("**Status**: CLOSED").is_none());
        assert!(parse_entry_line("Some random line").is_none());
    }

    #[test]
    fn test_parse_example_line() {
        assert_eq!(
            parse_example_line("- \"a dog is a thing\""),
            Some("a dog is a thing".to_string())
        );
        assert_eq!(
            parse_example_line("- \"the sun is up\""),
            Some("the sun is up".to_string())
        );
        assert!(parse_example_line("just a line").is_none());
    }

    #[test]
    fn test_parse_full_dict5() {
        let content = std::fs::read_to_string("../../dictionaries/dict5.md").unwrap();
        let dict = parse_dictionary(&content);

        for (i, e) in dict.entries.iter().enumerate() {
            eprintln!("{:2}: {:12} [{}] examples={}", i, e.word, e.section, e.examples.len());
        }

        // dict5.md header says 50 but actually has 51 entries (category section has 11, not 10)
        assert_eq!(dict.entries.len(), 51, "Expected 51 entries, got {}", dict.entries.len());
        assert!(dict.entry_set.contains("dog"));
        assert!(dict.entry_set.contains("thing"));
        assert!(dict.entry_set.contains("ball"));
        assert!(dict.entry_set.contains("is"));
        assert!(dict.entry_set.contains("not"));

        // Check a specific entry
        let dog = dict.entries.iter().find(|e| e.word == "dog").unwrap();
        assert!(dog.definition.contains("animal"));
        assert_eq!(dog.examples.len(), 3);
    }

    #[test]
    fn test_all_50_words_present() {
        let content = std::fs::read_to_string("../../dictionaries/dict5.md").unwrap();
        let dict = parse_dictionary(&content);

        let expected_words = vec![
            "thing", "is", "a", "not", "it",
            "and", "or", "the", "to", "of", "in", "on", "has", "can", "with", "this",
            "what", "yes", "no", "you",
            "big", "small", "good", "bad", "hot", "cold", "up", "down",
            "see", "feel", "move", "make", "eat", "give", "live", "do",
            "animal", "person", "food", "water", "color", "place", "sound", "part", "name", "one", "all",
            "dog", "cat", "sun", "ball",
        ];

        for word in &expected_words {
            assert!(dict.entry_set.contains(*word), "Missing entry word: {}", word);
        }
        assert_eq!(expected_words.len(), 51);
    }

    #[test]
    fn test_parse_toml_basic() {
        let toml_content = r#"
[package]
name = "test-dict"
version = "1.0.0"

[dictionary]
dog = "an animal"
cat = "a small animal"
thing = "all that is"
"#;

        let dict = parse_toml_dictionary(toml_content);

        assert_eq!(dict.entries.len(), 3);
        assert!(dict.entry_set.contains("dog"));
        assert!(dict.entry_set.contains("cat"));
        assert!(dict.entry_set.contains("thing"));

        let dog = dict.entries.iter().find(|e| e.word == "dog").unwrap();
        assert_eq!(dog.definition, "an animal");
        assert!(dog.examples.is_empty());
        assert_eq!(dog.section, "default");
    }

    #[test]
    fn test_parse_full_dict5_toml() {
        let content = std::fs::read_to_string("../../dictionaries/dict5.pkg.toml").unwrap();
        let dict = parse_toml_dictionary(&content);

        assert!(dict.entries.len() > 11000, "Expected >11000 entries, got {}", dict.entries.len());

        // Core dict5 words must be present
        assert!(dict.entry_set.contains("dog"));
        assert!(dict.entry_set.contains("cat"));
        assert!(dict.entry_set.contains("thing"));
        assert!(dict.entry_set.contains("sun"));
        assert!(dict.entry_set.contains("is"));

        eprintln!("TOML dict has {} entries", dict.entries.len());
    }

    #[test]
    fn test_load_dictionary_auto_detect() {
        let md_dict = load_dictionary("../../dictionaries/dict5.md").unwrap();
        assert_eq!(md_dict.entries.len(), 51);

        let toml_dict = load_dictionary("../../dictionaries/dict5.pkg.toml").unwrap();
        assert!(toml_dict.entries.len() > 11000);
    }
}
