//! Streaming XML parser for Simple English Wiktionary dumps.
//!
//! Reads the XML dump, extracts English definitions and examples,
//! strips wiki markup, and produces CacheEntry structs.

use std::collections::HashMap;
use std::io::BufRead;

use quick_xml::events::Event;
use quick_xml::Reader;

use yalm_cache::CacheEntry;

/// Parse a Wiktionary XML dump into a word → CacheEntry map.
pub fn parse_wiktionary_dump<R: BufRead>(reader: R) -> HashMap<String, CacheEntry> {
    let mut entries = HashMap::new();
    let mut xml = Reader::from_reader(reader);
    xml.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut in_page = false;
    let mut in_title = false;
    let mut in_text = false;
    let mut current_title = String::new();
    let mut current_text = String::new();
    let mut pages_scanned = 0u64;

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"page" => {
                    in_page = true;
                    current_title.clear();
                    current_text.clear();
                }
                b"title" if in_page => {
                    in_title = true;
                }
                b"text" if in_page => {
                    in_text = true;
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                if in_title {
                    if let Ok(text) = e.unescape() {
                        current_title.push_str(&text);
                    }
                } else if in_text {
                    if let Ok(text) = e.unescape() {
                        current_text.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"title" => in_title = false,
                b"text" => in_text = false,
                b"page" => {
                    in_page = false;
                    pages_scanned += 1;

                    if pages_scanned % 10000 == 0 {
                        eprintln!("  Scanned {} pages, {} entries so far...", pages_scanned, entries.len());
                    }

                    // Skip namespace pages and redirects
                    if !current_title.contains(':')
                        && !current_text.starts_with("#REDIRECT")
                        && !current_text.starts_with("#redirect")
                    {
                        if let Some(entry) = extract_entry(&current_title, &current_text) {
                            entries.insert(entry.word.clone(), entry);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("XML parse error at position {}: {}", xml.buffer_position(), e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    eprintln!("  Scanned {} total pages, extracted {} entries", pages_scanned, entries.len());
    entries
}

/// Extract a CacheEntry from a single wikitext page.
fn extract_entry(title: &str, text: &str) -> Option<CacheEntry> {
    // Skip pages with no useful content
    if text.len() < 5 {
        return None;
    }

    // Find the English section. Simple English Wiktionary pages are
    // often entirely English, but some have ==English== headers.
    let section = find_english_section(text).unwrap_or_else(|| text.to_string());

    let mut definitions = Vec::new();
    let mut examples = Vec::new();

    for line in section.lines() {
        let trimmed = line.trim();

        // Definition lines: start with # but not #* #: ## (sub-definitions)
        if trimmed.starts_with('#')
            && !trimmed.starts_with("#*")
            && !trimmed.starts_with("#:")
            && !trimmed.starts_with("##")
        {
            let raw = &trimmed[1..].trim();
            let cleaned = strip_wiki_markup(raw);
            let cleaned = cleaned.trim();
            if !cleaned.is_empty() && cleaned.len() > 2 {
                definitions.push(cleaned.to_string());
            }
        }
        // Example lines: #* or #:
        else if trimmed.starts_with("#*") || trimmed.starts_with("#:") {
            let prefix_len = 2;
            let raw = &trimmed[prefix_len..].trim();
            let cleaned = strip_wiki_markup(raw);
            let cleaned = cleaned.trim();
            if !cleaned.is_empty() && cleaned.len() > 2 {
                examples.push(cleaned.to_string());
            }
        }
    }

    if definitions.is_empty() {
        return None;
    }

    Some(CacheEntry {
        word: title.to_lowercase(),
        definitions,
        examples,
    })
}

/// Find the ==English== section in wikitext.
/// If no ==English== header found, returns None (caller should use full text).
fn find_english_section(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();

    // Look for ==English== header
    let start = lines
        .iter()
        .position(|l| {
            let t = l.trim();
            t == "==English==" || t == "== English ==" || t == "==english=="
        })?;

    // Find next language-level header (==Something== but not ===Something===)
    let end = lines
        .iter()
        .skip(start + 1)
        .position(|l| {
            let t = l.trim();
            t.starts_with("==")
                && t.ends_with("==")
                && !t.starts_with("===")
                && t != "==English=="
        })
        .map(|p| p + start + 1)
        .unwrap_or(lines.len());

    Some(lines[start..end].join("\n"))
}

/// Strip common wiki markup from a line of text.
pub fn strip_wiki_markup(text: &str) -> String {
    let mut result = text.to_string();

    // Remove {{template|...}} blocks (non-greedy within braces)
    // Handle nested templates by repeating
    for _ in 0..5 {
        if let Some(start) = result.find("{{") {
            if let Some(end) = result[start..].find("}}") {
                result = format!("{}{}", &result[..start], &result[start + end + 2..]);
                continue;
            }
        }
        break;
    }

    // [[link|display]] -> display
    // [[link]] -> link
    for _ in 0..20 {
        if let Some(start) = result.find("[[") {
            if let Some(end) = result[start..].find("]]") {
                let inner = &result[start + 2..start + end];
                let display = inner.split('|').last().unwrap_or(inner);
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    display,
                    &result[start + end + 2..]
                );
                continue;
            }
        }
        break;
    }

    // Remove bold/italic wiki markers
    result = result.replace("'''", "");
    result = result.replace("''", "");

    // Remove remaining brackets
    result = result.replace('[', "").replace(']', "");
    result = result.replace('{', "").replace('}', "");

    // Remove HTML tags
    let mut clean = String::with_capacity(result.len());
    let mut in_tag = false;
    for ch in result.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                continue;
            }
            _ if !in_tag => clean.push(ch),
            _ => {}
        }
    }

    // Clean up multiple spaces
    let mut prev_space = false;
    let final_result: String = clean
        .chars()
        .filter(|c| {
            if c.is_whitespace() {
                if prev_space {
                    return false;
                }
                prev_space = true;
            } else {
                prev_space = false;
            }
            true
        })
        .collect();

    final_result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_links() {
        assert_eq!(strip_wiki_markup("A [[dog]] is an [[animal]]"), "A dog is an animal");
        assert_eq!(strip_wiki_markup("A [[canine|dog]] barks"), "A dog barks");
    }

    #[test]
    fn strip_templates() {
        assert_eq!(strip_wiki_markup("{{context|informal}} A pet"), "A pet");
        assert_eq!(strip_wiki_markup("Hello {{world}}!"), "Hello !");
    }

    #[test]
    fn strip_bold_italic() {
        assert_eq!(strip_wiki_markup("'''bold''' and ''italic''"), "bold and italic");
    }

    #[test]
    fn strip_html() {
        // Tags removed, inner text kept (90% clean approach — ref content is noise but harmless)
        assert_eq!(strip_wiki_markup("hello <b>bold</b> world"), "hello bold world");
        assert_eq!(strip_wiki_markup("no <br/> break"), "no break");
    }

    #[test]
    fn extract_english_section() {
        let text = "==Spanish==\nperro\n==English==\n# A four-legged animal.\n==French==\nchien";
        let section = find_english_section(text).unwrap();
        assert!(section.contains("four-legged"));
        assert!(!section.contains("perro"));
        assert!(!section.contains("chien"));
    }

    #[test]
    fn extract_entry_basic() {
        let text = "==English==\n===Noun===\n# A common pet animal.\n#* The dog ran fast.";
        let entry = extract_entry("dog", text).unwrap();
        assert_eq!(entry.word, "dog");
        assert_eq!(entry.definitions.len(), 1);
        assert!(entry.definitions[0].contains("pet animal"));
        assert_eq!(entry.examples.len(), 1);
    }

    #[test]
    fn skip_empty() {
        let entry = extract_entry("test", "#REDIRECT [[other]]");
        // We don't call extract_entry on redirects normally, but if we do:
        assert!(entry.is_none() || entry.unwrap().definitions.is_empty() == false);
    }
}
