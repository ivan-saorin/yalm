//! Multi-Space Architecture (Phase 16)
//!
//! Multiple independent geometric spaces ("thought domains") that compose
//! results at query time via bridge terms and a TASK dispatcher.

use std::collections::{HashMap, HashSet};

use yalm_core::*;
use yalm_parser::{parse_dictionary, stem_to_entry, tokenize};

use crate::resolver::{definition_chain_check, resolve_question};
use crate::strategy::StrategyConfig;
use crate::{BuildMode, Engine};

// ─── Data Structures ─────────────────────────────────────────

/// Configuration for loading a single space.
pub struct SpaceConfig {
    pub name: String,
    pub dict_path: String,
}

/// A named geometric space wrapping an Engine instance.
pub struct Space {
    pub name: String,
    pub engine: Engine,
    pub dictionary: Dictionary,
    pub params: EngineParams,
    pub strategy: StrategyConfig,
}

/// Result from a single space's resolution.
#[derive(Debug, Clone)]
struct SpaceResult {
    space_name: String,
    answer: Answer,
    distance: Option<f64>,
    connector: Option<String>,
}

/// Detected arithmetic query.
struct ArithmeticQuery {
    left_operand: String,
    operator: String,
    right_operand: String,
}

/// The multi-space orchestrator.
pub struct MultiSpace {
    pub spaces: HashMap<String, Space>,
    pub bridges: HashMap<(String, String), HashSet<String>>,
    pub space_order: Vec<String>,
}

// ─── Structural Words ────────────────────────────────────────

/// Words too common to be informative for routing.
fn is_structural(word: &str) -> bool {
    matches!(
        word,
        "is" | "a" | "an" | "the" | "it" | "not" | "and" | "of"
            | "can" | "what" | "yes" | "no" | "to" | "with"
            | "has" | "in" | "this" | "do" | "does" | "or"
            | "you" | "all" | "make" | "that" | "are" | "be"
            | "on" | "for" | "at" | "by" | "if" | "as"
    )
}

// ─── Number Word Mapping ─────────────────────────────────────

/// Map a count integer to a number word.
fn count_to_word(n: usize) -> Option<&'static str> {
    match n {
        0 => Some("zero"),
        1 => Some("one"),
        2 => Some("two"),
        3 => Some("three"),
        4 => Some("four"),
        5 => Some("five"),
        6 => Some("six"),
        7 => Some("seven"),
        8 => Some("eight"),
        9 => Some("nine"),
        10 => Some("ten"),
        _ => None,
    }
}

// ─── MultiSpace Implementation ───────────────────────────────

impl MultiSpace {
    /// Construct a MultiSpace from a list of space configs.
    /// Loads each dictionary, trains each engine independently.
    pub fn new(
        configs: Vec<SpaceConfig>,
        params: &EngineParams,
        strategy: &StrategyConfig,
        build_mode: BuildMode,
    ) -> Self {
        let mut spaces = HashMap::new();
        let mut space_order = Vec::new();

        for config in configs {
            let content = std::fs::read_to_string(&config.dict_path)
                .unwrap_or_else(|_| panic!("Failed to read dictionary: {}", config.dict_path));
            let dictionary = parse_dictionary(&content);

            println!(
                "[Space {}] Loading {} ({} entries)",
                config.name,
                config.dict_path,
                dictionary.entries.len()
            );

            let mut engine = Engine::with_strategy(params.clone(), strategy.clone());
            engine.set_quiet(true);
            engine.set_mode(build_mode);
            engine.train(&dictionary);

            let space = Space {
                name: config.name.clone(),
                engine,
                dictionary,
                params: params.clone(),
                strategy: strategy.clone(),
            };

            space_order.push(config.name.clone());
            spaces.insert(config.name, space);
        }

        let mut ms = MultiSpace {
            spaces,
            bridges: HashMap::new(),
            space_order,
        };
        ms.identify_bridges();
        ms
    }

    /// Compute vocabulary intersections between all space pairs.
    /// Keys are stored in alphabetical order for consistent lookup.
    pub fn identify_bridges(&mut self) {
        let names: Vec<String> = self.space_order.clone();
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                let a = &names[i];
                let b = &names[j];
                let vocab_a = &self.spaces[a].dictionary.entry_set;
                let vocab_b = &self.spaces[b].dictionary.entry_set;
                let bridge: HashSet<String> =
                    vocab_a.intersection(vocab_b).cloned().collect();
                // Store with alphabetical key ordering to match lookup functions
                let key = if a < b {
                    (a.clone(), b.clone())
                } else {
                    (b.clone(), a.clone())
                };
                self.bridges.insert(key, bridge);
            }
        }
    }

    /// Print bridge term information.
    pub fn print_bridges(&self) {
        println!("\n=== Bridge Terms ===");
        for ((a, b), terms) in &self.bridges {
            let sample: Vec<&String> = terms.iter().take(15).collect();
            println!(
                "  {} <-> {}: {} terms {:?}{}",
                a,
                b,
                terms.len(),
                sample,
                if terms.len() > 15 { " ..." } else { "" }
            );
        }
    }

    // ─── Main Resolve Entry Point ────────────────────────────

    /// Resolve a query against the multi-space architecture.
    pub fn resolve(&self, query: &str) -> (Answer, Option<f64>, Option<String>) {
        // Priority 1: Multi-instruction detection (period-separated)
        if let Some(result) = self.detect_multi_instruction(query) {
            return result;
        }

        // Priority 2: Arithmetic detection (X plus/minus Y)
        if let Some(arith) = self.detect_arithmetic(query) {
            if let Some(answer) = self.resolve_arithmetic(&arith) {
                return (answer, Some(0.0), Some("arithmetic".to_string()));
            }
        }

        // Priority 3: Special pattern detection
        if let Some(result) = self.detect_special_patterns(query) {
            return result;
        }

        // Priority 4: Route to space(s) and resolve
        let activated = self.route_query(query);

        if activated.is_empty() {
            return (Answer::IDontKnow, None, None);
        }

        // Resolve in each activated space
        let mut results: Vec<SpaceResult> = Vec::new();
        for space_name in &activated {
            if let Some(result) = self.resolve_in_space(space_name, query) {
                results.push(result);
            }
        }

        // If all results are IDK, try example-based lookup
        // (for queries like "Is dog a noun?" where "dog" isn't an entry
        // but appears in the "noun" definition)
        if results.iter().all(|r| r.answer == Answer::IDontKnow) {
            if let Some(result) = self.try_example_based_lookup(query) {
                return result;
            }
        }

        // Try cross-space chain when results are inconclusive
        let non_word_results: Vec<&SpaceResult> = results
            .iter()
            .filter(|r| !matches!(r.answer, Answer::Word(_)))
            .collect();
        let all_non_word_idk = non_word_results
            .iter()
            .all(|r| r.answer == Answer::IDontKnow);
        let has_idk = results.iter().any(|r| r.answer == Answer::IDontKnow);

        if has_idk || all_non_word_idk || results.is_empty() {
            if let Some(result) = self.try_cross_space_yes_no(query, &activated) {
                return result;
            }
        }

        // Compose results
        self.compose_results(results, query)
    }

    // ─── Routing ─────────────────────────────────────────────

    /// Route a query to the appropriate space(s).
    fn route_query(&self, query: &str) -> Vec<String> {
        let tokens = tokenize(query);

        // Count how many content words each space recognizes
        let mut space_hits: HashMap<String, Vec<String>> = HashMap::new();
        let mut exclusive: HashSet<String> = HashSet::new();

        for token in &tokens {
            if is_structural(token) {
                continue;
            }

            let mut containing: Vec<String> = Vec::new();
            for (name, space) in &self.spaces {
                if space.dictionary.entry_set.contains(token.as_str())
                    || stem_to_entry(token, &space.dictionary.entry_set).is_some()
                {
                    containing.push(name.clone());
                    space_hits
                        .entry(name.clone())
                        .or_default()
                        .push(token.clone());
                }
            }

            // If word is in exactly one domain space (excluding "task"), activate it
            let domain_spaces: Vec<&String> = containing
                .iter()
                .filter(|s| s.as_str() != "task")
                .collect();
            if domain_spaces.len() == 1 {
                exclusive.insert(domain_spaces[0].clone());
            }
        }

        // If no exclusive activations, use TASK space to disambiguate
        if exclusive.is_empty() {
            if let Some(task_space) = self.spaces.get("task") {
                let mut math_score = 0.0f64;
                let mut grammar_score = 0.0f64;

                for token in &tokens {
                    if is_structural(token) {
                        continue;
                    }
                    if let Some(token_pos) =
                        task_space.engine.space().words.get(token.as_str())
                    {
                        if let Some(number_pos) =
                            task_space.engine.space().words.get("number")
                        {
                            let d = euclidean_distance(
                                &token_pos.position,
                                &number_pos.position,
                            );
                            math_score += 1.0 / (1.0 + d);
                        }
                        if let Some(word_pos) =
                            task_space.engine.space().words.get("word")
                        {
                            let d = euclidean_distance(
                                &token_pos.position,
                                &word_pos.position,
                            );
                            grammar_score += 1.0 / (1.0 + d);
                        }
                    }
                }

                if math_score > grammar_score * 1.2 {
                    exclusive.insert("math".to_string());
                } else if grammar_score > math_score * 1.2 {
                    exclusive.insert("grammar".to_string());
                } else if math_score > 0.0 || grammar_score > 0.0 {
                    // Both are close — activate both
                    exclusive.insert("math".to_string());
                    exclusive.insert("grammar".to_string());
                }
            }
        }

        // Fallback: pick the space with most query words
        if exclusive.is_empty() {
            if let Some((best, _)) = space_hits
                .iter()
                .filter(|(name, _)| name.as_str() != "task")
                .max_by_key(|(_, words)| words.len())
            {
                exclusive.insert(best.clone());
            }
        }

        // Last resort: try all domain spaces
        if exclusive.is_empty() {
            for name in &self.space_order {
                if name != "task" {
                    exclusive.insert(name.clone());
                }
            }
        }

        exclusive.into_iter().collect()
    }

    // ─── Per-Space Resolution ────────────────────────────────

    /// Resolve a query within a single named space.
    fn resolve_in_space(&self, space_name: &str, query: &str) -> Option<SpaceResult> {
        let space = self.spaces.get(space_name)?;
        let (answer, distance, connector) = resolve_question(
            query,
            space.engine.space(),
            &space.dictionary,
            space.engine.structural(),
            space.engine.content(),
            &space.params,
            &space.strategy,
        );
        Some(SpaceResult {
            space_name: space_name.to_string(),
            answer,
            distance,
            connector,
        })
    }

    // ─── Result Composition ──────────────────────────────────

    /// Compose results from multiple spaces.
    fn compose_results(
        &self,
        results: Vec<SpaceResult>,
        _query: &str,
    ) -> (Answer, Option<f64>, Option<String>) {
        if results.is_empty() {
            return (Answer::IDontKnow, None, None);
        }

        if results.len() == 1 {
            let r = &results[0];
            return (r.answer.clone(), r.distance, r.connector.clone());
        }

        // Multiple results — compose

        // Case 1: All agree on Yes
        if results.iter().all(|r| r.answer == Answer::Yes) {
            let avg_dist = avg_distance(&results);
            return (
                Answer::Yes,
                avg_dist,
                Some(format!(
                    "multi-space:agree({})",
                    results
                        .iter()
                        .map(|r| r.space_name.as_str())
                        .collect::<Vec<_>>()
                        .join("+")
                )),
            );
        }

        // Case 2: All agree on No
        if results.iter().all(|r| r.answer == Answer::No) {
            let avg_dist = avg_distance(&results);
            return (
                Answer::No,
                avg_dist,
                Some(format!(
                    "multi-space:agree({})",
                    results
                        .iter()
                        .map(|r| r.space_name.as_str())
                        .collect::<Vec<_>>()
                        .join("+")
                )),
            );
        }

        // Case 3: One produces a Word answer, others don't
        let word_results: Vec<&SpaceResult> = results
            .iter()
            .filter(|r| matches!(r.answer, Answer::Word(_)))
            .collect();
        if word_results.len() == 1 {
            let r = word_results[0];
            return (r.answer.clone(), r.distance, r.connector.clone());
        }

        // Case 4: Mixed Yes/No/IDK — prefer non-IDK answers
        let non_idk: Vec<&SpaceResult> = results
            .iter()
            .filter(|r| r.answer != Answer::IDontKnow)
            .collect();

        if non_idk.len() == 1 {
            let r = non_idk[0];
            return (r.answer.clone(), r.distance, r.connector.clone());
        }

        // Case 5: Yes vs IDK — prefer Yes (the space that knows, knows)
        let yes_results: Vec<&SpaceResult> = results
            .iter()
            .filter(|r| r.answer == Answer::Yes)
            .collect();
        let no_results: Vec<&SpaceResult> = results
            .iter()
            .filter(|r| r.answer == Answer::No)
            .collect();

        if !yes_results.is_empty() && no_results.is_empty() {
            let r = yes_results[0];
            return (Answer::Yes, r.distance, r.connector.clone());
        }
        if !no_results.is_empty() && yes_results.is_empty() {
            let r = no_results[0];
            return (Answer::No, r.distance, r.connector.clone());
        }

        // Case 6: True disagreement (Yes vs No) — use distance confidence
        // Smaller distance = more confident for Yes, larger = more confident for No
        if !yes_results.is_empty() && !no_results.is_empty() {
            // The space that said Yes with smallest distance is most confident
            let best_yes = yes_results
                .iter()
                .min_by(|a, b| {
                    a.distance
                        .unwrap_or(f64::MAX)
                        .partial_cmp(&b.distance.unwrap_or(f64::MAX))
                        .unwrap()
                })
                .unwrap();
            let best_no = no_results
                .iter()
                .max_by(|a, b| {
                    a.distance
                        .unwrap_or(0.0)
                        .partial_cmp(&b.distance.unwrap_or(0.0))
                        .unwrap()
                })
                .unwrap();

            // Heuristic: prefer the answer from the space that has more
            // vocabulary coverage of the query
            // Heuristic: prefer Yes over No (the knowing space knows)
            // TODO: could improve with proper confidence scoring using best_no
            let _ = best_no;
            let r = best_yes; // default to Yes in ties
            return (r.answer.clone(), r.distance, r.connector.clone());
        }

        // Fallback: return first non-IDK result
        for r in &results {
            if r.answer != Answer::IDontKnow {
                return (r.answer.clone(), r.distance, r.connector.clone());
            }
        }

        // All IDK
        (Answer::IDontKnow, None, None)
    }

    // ─── Arithmetic Detection & Resolution ───────────────────

    /// Detect arithmetic patterns like "X plus Y" or "X minus Y".
    /// Only triggers for pure arithmetic queries (What is X plus Y?) or bare
    /// expressions (X plus Y), NOT for Yes/No questions or quoted strings.
    fn detect_arithmetic(&self, query: &str) -> Option<ArithmeticQuery> {
        let lower = query.to_lowercase();

        // Don't trigger arithmetic inside quoted strings
        if lower.contains('"') || lower.contains('\u{201C}') {
            return None;
        }

        // Don't trigger for Yes/No questions (starts with is/can/does/has/are)
        let first_word = lower.split_whitespace().next().unwrap_or("");
        if matches!(first_word, "is" | "can" | "does" | "has" | "are") {
            return None;
        }

        let tokens = tokenize(query);
        let operators = ["plus", "minus"];

        for (i, token) in tokens.iter().enumerate() {
            if operators.contains(&token.as_str()) && i > 0 && i + 1 < tokens.len() {
                let math = self.spaces.get("math")?;
                let left = &tokens[i - 1];
                let right = &tokens[i + 1];

                // Both operands must be in MATH vocabulary
                if math.dictionary.entry_set.contains(left.as_str())
                    && math.dictionary.entry_set.contains(right.as_str())
                {
                    return Some(ArithmeticQuery {
                        left_operand: left.clone(),
                        operator: token.clone(),
                        right_operand: right.clone(),
                    });
                }
            }
        }
        None
    }

    /// Resolve arithmetic by scanning dictionary definitions and examples
    /// for the pattern "{left} {op} {right} is {result}".
    fn resolve_arithmetic(&self, arith: &ArithmeticQuery) -> Option<Answer> {
        let math = self.spaces.get("math")?;

        let pattern = format!(
            "{} {} {} is ",
            arith.left_operand, arith.operator, arith.right_operand
        );

        // Scan ALL entries in the math dictionary for this pattern
        for entry in &math.dictionary.entries {
            let all_text = format!(
                "{}. {}",
                entry.definition,
                entry.examples.join(". ")
            );
            let lower = all_text.to_lowercase();

            if let Some(pos) = lower.find(&pattern) {
                let after = &lower[pos + pattern.len()..];
                let result_word = after
                    .split(|c: char| !c.is_alphanumeric())
                    .next()
                    .filter(|w| !w.is_empty())?;

                // Verify the result word is in the math dictionary
                if math.dictionary.entry_set.contains(result_word) {
                    return Some(Answer::Word(result_word.to_string()));
                }
            }
        }

        // Also try the reverse for minus: if "five minus two is three" isn't found,
        // try scanning number definitions for "X and Y" patterns
        None
    }

    // ─── Special Pattern Detection ───────────────────────────

    /// Detect and handle special query patterns that require compositional reasoning.
    fn detect_special_patterns(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let lower = query.to_lowercase();

        // Pattern: "How many words are in {quoted}?"
        if lower.contains("how many words") {
            return self.resolve_word_count(query);
        }

        // Pattern: "What is the subject in {quoted}?"
        if lower.contains("subject in") && lower.contains('"') {
            return self.resolve_subject_extraction(query);
        }

        // Pattern: "What comes after X?"
        if lower.contains("comes after") || lower.contains("come after") {
            return self.resolve_after_query(query);
        }

        // Pattern: "What kind of task is X?"
        if lower.contains("kind") {
            return self.resolve_kind_query(query);
        }

        // Pattern: "Is X a Y or a Z?" (choice question)
        if lower.contains(" or ") && lower.starts_with("is ") {
            if let Some(result) = self.resolve_or_choice(query) {
                return Some(result);
            }
        }

        // Pattern: "Is X the same as Y?"
        if lower.contains("the same as") {
            return self.resolve_same_as_query(query);
        }

        // Pattern: 'Is "{quoted}" a {type} task?'
        if lower.contains("task") && lower.contains('"') {
            return self.resolve_task_classification(query);
        }

        None
    }

    /// Resolve "How many words are in 'X Y Z'?" by counting tokens.
    fn resolve_word_count(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        // Extract quoted string
        let quoted = extract_quoted(query)?;
        let words = tokenize(&quoted);
        let count = words.len();
        let word = count_to_word(count)?;
        Some((
            Answer::Word(word.to_string()),
            Some(0.0),
            Some("word-count".to_string()),
        ))
    }

    /// Resolve "What is the subject in 'the dog eats'?" by finding the first noun.
    fn resolve_subject_extraction(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let quoted = extract_quoted(query)?;
        let words = tokenize(&quoted);

        // Subject = the first content word that isn't "the", "a", etc.
        // In ELI5 grammar: subject comes first, it's the thing that does the action.
        for word in &words {
            if !is_structural(word) {
                return Some((
                    Answer::Word(word.clone()),
                    Some(0.0),
                    Some("subject-extraction".to_string()),
                ));
            }
        }
        None
    }

    /// Resolve "What comes after X?" by scanning definitions for "the number after X".
    fn resolve_after_query(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let tokens = tokenize(query);

        // Find the word after "after"
        let after_idx = tokens.iter().position(|t| t == "after")?;
        let target = tokens.get(after_idx + 1)?;

        // Scan MATH dictionary for "the number after {target}"
        if let Some(math) = self.spaces.get("math") {
            let pattern = format!("after {}", target);
            for entry in &math.dictionary.entries {
                let lower = entry.definition.to_lowercase();
                if lower.contains(&pattern) {
                    return Some((
                        Answer::Word(entry.word.clone()),
                        Some(0.0),
                        Some("after-pattern".to_string()),
                    ));
                }
            }
        }
        None
    }

    // ─── Multi-Instruction Detection ─────────────────────────

    /// Detect multi-instruction queries separated by periods.
    /// E.g., "Two plus three. Write the answer as a sentence."
    fn detect_multi_instruction(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        // Split on periods, filter out empty segments
        let segments: Vec<&str> = query
            .split('.')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if segments.len() < 2 {
            return None;
        }

        // Check if this looks like multi-instruction (not just a regular sentence with periods)
        // Heuristic: if any segment contains "write" or "sentence", it's multi-instruction
        let has_formatting_instruction = segments.iter().any(|s| {
            let lower = s.to_lowercase();
            lower.contains("write") || lower.contains("sentence")
        });

        if !has_formatting_instruction {
            return None;
        }

        // Process first segment as a computation
        let first = segments[0];

        // Try arithmetic on the first segment
        if let Some(arith) = self.detect_arithmetic(first) {
            if let Some(Answer::Word(result)) = self.resolve_arithmetic(&arith) {
                // Check if second segment asks for sentence formatting
                let second_lower = segments[1].to_lowercase();
                if second_lower.contains("sentence") || second_lower.contains("write") {
                    // Format as: "{expression} is {result}"
                    let expression = first.to_lowercase();
                    let formatted = format!("{} is {}", expression, result);
                    return Some((
                        Answer::Word(formatted),
                        Some(0.0),
                        Some("multi-instruction:arithmetic+format".to_string()),
                    ));
                }
                return Some((
                    Answer::Word(result),
                    Some(0.0),
                    Some("multi-instruction:arithmetic".to_string()),
                ));
            }
        }

        // If first segment isn't arithmetic, try compound question detection:
        // "Count to five. Is count a verb?" — the second part is the actual question
        let last = segments.last()?;
        let last_lower = last.to_lowercase();

        // Check if the last segment is a question (starts with is/can/what/does/...)
        let question_starters = [
            "is", "can", "what", "who", "where", "when", "why", "does", "has",
        ];
        let first_word = last_lower.split_whitespace().next()?;
        if question_starters.contains(&first_word) {
            // Resolve just the question part
            let (answer, dist, conn) = self.resolve(last);
            return Some((answer, dist, conn));
        }

        None
    }

    // ─── Cross-Space Chain Resolution ────────────────────────

    /// Attempt cross-space Yes/No resolution using bridge terms.
    fn try_cross_space_yes_no(
        &self,
        query: &str,
        _activated: &[String],
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let tokens = tokenize(query);

        // Parse as Yes/No: find subject and object
        let content_words: Vec<&String> = tokens
            .iter()
            .filter(|t| !is_structural(t))
            .collect();

        if content_words.len() < 2 {
            return None;
        }

        let subject = content_words[0];
        let object = content_words.last()?;

        // Try cross-space chain across ALL domain spaces (not just activated)
        let domain_spaces: Vec<&String> = self
            .space_order
            .iter()
            .filter(|s| s.as_str() != "task")
            .collect();

        // Find which space contains subject and which contains object
        let subject_spaces: Vec<&&String> = domain_spaces
            .iter()
            .filter(|name| {
                self.spaces[name.as_str()]
                    .dictionary
                    .entry_set
                    .contains(subject.as_str())
            })
            .collect();
        let object_spaces: Vec<&&String> = domain_spaces
            .iter()
            .filter(|name| {
                self.spaces[name.as_str()]
                    .dictionary
                    .entry_set
                    .contains(object.as_str())
            })
            .collect();

        // Strategy 1: Definition-based cross-space chain
        // Subject's definition gives category → category is bridge → bridge in object's definition
        // E.g., five="the number after four", number="a thing"
        //        noun="a word that is a name for a thing" → "thing" is bridge → five is a noun
        for src_name in &subject_spaces {
            for tgt_name in &object_spaces {
                if src_name == tgt_name {
                    continue;
                }
                if let Some(true) = self.definition_bridge_chain(
                    subject, object, src_name, tgt_name,
                ) {
                    return Some((
                        Answer::Yes,
                        Some(0.0),
                        Some(format!("cross-space:{}→{}", src_name, tgt_name)),
                    ));
                }
            }
        }

        // Strategy 2: Formal cross-space chain (definition_chain_check based)
        for src in &subject_spaces {
            for tgt in &object_spaces {
                if src == tgt {
                    continue;
                }
                if let Some(true) = self.cross_space_chain(
                    subject,
                    object,
                    src,
                    tgt,
                ) {
                    return Some((
                        Answer::Yes,
                        Some(0.0),
                        Some(format!(
                            "cross-space-chain:{}→{}",
                            src, tgt
                        )),
                    ));
                }
            }
        }

        None
    }

    /// Definition-based cross-space chain.
    /// Traverses definition text to find bridge terms connecting subject to object.
    ///
    /// Algorithm:
    /// 1. Get subject's definition in source space
    /// 2. For each content word in that definition that's a bridge term:
    ///    a. Check if that bridge word appears in object's definition (target space)
    ///    b. Check if bridge word's definition in target contains object
    ///    c. If so, the chain holds: subject→category→bridge→object
    fn definition_bridge_chain(
        &self,
        subject: &str,
        object: &str,
        source_space: &str,
        target_space: &str,
    ) -> Option<bool> {
        let key = if source_space < target_space {
            (source_space.to_string(), target_space.to_string())
        } else {
            (target_space.to_string(), source_space.to_string())
        };
        let bridges = self.bridges.get(&key)?;

        let src = self.spaces.get(source_space)?;
        let tgt = self.spaces.get(target_space)?;

        // Get all words reachable from subject's definition chain (up to 2 hops)
        let mut reachable: HashSet<String> = HashSet::new();
        if let Some(subj_entry) = src.dictionary.entries.iter().find(|e| e.word == subject) {
            let def_words = tokenize(&subj_entry.definition);
            for w in &def_words {
                let stemmed = stem_to_entry(w, &src.dictionary.entry_set)
                    .unwrap_or_else(|| w.clone());
                reachable.insert(stemmed.clone());

                // One more hop: follow this word's definition too
                if let Some(next_entry) = src.dictionary.entries.iter().find(|e| e.word == stemmed) {
                    let next_words = tokenize(&next_entry.definition);
                    for nw in &next_words {
                        let ns = stem_to_entry(nw, &src.dictionary.entry_set)
                            .unwrap_or_else(|| nw.clone());
                        reachable.insert(ns);
                    }
                }
            }
        }

        // Check if any reachable word is a bridge term
        for bridge in bridges {
            if !reachable.contains(bridge.as_str()) {
                continue;
            }

            // Check if object's definition in target space mentions this bridge
            if let Some(obj_entry) = tgt.dictionary.entries.iter().find(|e| e.word == object) {
                let obj_words = tokenize(&obj_entry.definition);
                let obj_set: HashSet<String> = obj_words
                    .iter()
                    .map(|w| {
                        stem_to_entry(w, &tgt.dictionary.entry_set)
                            .unwrap_or_else(|| w.clone())
                    })
                    .collect();

                if obj_set.contains(bridge.as_str()) {
                    return Some(true);
                }

                // Also check one hop from object's definition words
                for ow in &obj_words {
                    let stemmed = stem_to_entry(ow, &tgt.dictionary.entry_set)
                        .unwrap_or_else(|| ow.clone());
                    if let Some(next_entry) = tgt.dictionary.entries.iter().find(|e| e.word == stemmed) {
                        let next_words = tokenize(&next_entry.definition);
                        for nw in &next_words {
                            let ns = stem_to_entry(nw, &tgt.dictionary.entry_set)
                                .unwrap_or_else(|| nw.clone());
                            if ns == *bridge {
                                return Some(true);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Cross-space chain: subject→bridge in source, bridge→object in target.
    /// Checks both forward (bridge→object) and reverse (object→bridge) in target.
    fn cross_space_chain(
        &self,
        subject: &str,
        object: &str,
        source_space: &str,
        target_space: &str,
    ) -> Option<bool> {
        // Find bridge terms between the two spaces
        let key = if source_space < target_space {
            (source_space.to_string(), target_space.to_string())
        } else {
            (target_space.to_string(), source_space.to_string())
        };
        let bridges = self.bridges.get(&key)?;

        let src = self.spaces.get(source_space)?;
        let tgt = self.spaces.get(target_space)?;

        for bridge in bridges {
            if bridge == subject || bridge == object {
                // Direct bridge: subject or object IS a bridge term
                if bridge == subject {
                    // subject is in both spaces — check target space directly
                    let mut visited = HashSet::new();
                    if definition_chain_check(
                        subject,
                        object,
                        &tgt.dictionary,
                        tgt.engine.structural(),
                        3,
                        &mut visited,
                        tgt.engine.space(),
                    ) == Some(true)
                    {
                        return Some(true);
                    }
                    // Also check reverse: does object's definition mention subject?
                    let mut visited2 = HashSet::new();
                    if definition_chain_check(
                        object,
                        subject,
                        &tgt.dictionary,
                        tgt.engine.structural(),
                        3,
                        &mut visited2,
                        tgt.engine.space(),
                    ) == Some(true)
                    {
                        return Some(true);
                    }
                }
                if bridge == object {
                    // object is in both spaces — check source space
                    let mut visited = HashSet::new();
                    if definition_chain_check(
                        subject,
                        object,
                        &src.dictionary,
                        src.engine.structural(),
                        3,
                        &mut visited,
                        src.engine.space(),
                    ) == Some(true)
                    {
                        return Some(true);
                    }
                }
                continue;
            }

            // Standard bridge traversal:
            // 1. subject → bridge in source space
            let mut visited1 = HashSet::new();
            let fwd = definition_chain_check(
                subject,
                bridge,
                &src.dictionary,
                src.engine.structural(),
                3,
                &mut visited1,
                src.engine.space(),
            );

            if fwd != Some(true) {
                continue;
            }

            // 2. Try both directions in target space:
            //    bridge → object OR object → bridge (reverse lookup)
            let mut visited2 = HashSet::new();
            let rev = definition_chain_check(
                bridge,
                object,
                &tgt.dictionary,
                tgt.engine.structural(),
                3,
                &mut visited2,
                tgt.engine.space(),
            );

            if rev == Some(true) {
                return Some(true);
            }

            // Reverse: does object's definition chain contain bridge?
            // E.g., "noun" def mentions "thing", bridge="thing"
            let mut visited3 = HashSet::new();
            let rev2 = definition_chain_check(
                object,
                bridge,
                &tgt.dictionary,
                tgt.engine.structural(),
                3,
                &mut visited3,
                tgt.engine.space(),
            );

            if rev2 == Some(true) {
                return Some(true);
            }
        }

        None
    }

    // ─── Example-Based Lookup ─────────────────────────────────

    /// For Yes/No questions where one term isn't a dictionary entry,
    /// check if it appears in definitions/examples of the other term.
    /// E.g., "Is dog a noun?" — "dog" isn't an entry, but "noun" definition
    /// says "dog is a noun."
    fn try_example_based_lookup(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let tokens = tokenize(query);

        // Only handle Yes/No questions (starts with is/can/does/has/are)
        let first = tokens.first()?;
        if !matches!(first.as_str(), "is" | "can" | "does" | "has" | "are") {
            return None;
        }

        // Find content words
        let content_words: Vec<&String> = tokens
            .iter()
            .filter(|t| !is_structural(t))
            .collect();

        if content_words.len() < 2 {
            return None;
        }

        let subject = content_words[0].as_str();
        let object = content_words.last()?.as_str();

        // Check if subject is mentioned in object's definition/examples
        // across all spaces
        for (_name, space) in &self.spaces {
            if let Some(obj_entry) = space
                .dictionary
                .entries
                .iter()
                .find(|e| e.word == object)
            {
                let all_text = format!(
                    "{}. {}",
                    obj_entry.definition,
                    obj_entry.examples.join(". ")
                );
                let lower = all_text.to_lowercase();

                // Check for "{subject} is a {object}" or "{subject} is {object}" pattern
                let pattern1 = format!("{} is a {}", subject, object);
                let pattern2 = format!("{} is an {}", subject, object);
                let pattern3 = format!("{} is {}", subject, object);

                if lower.contains(&pattern1)
                    || lower.contains(&pattern2)
                    || lower.contains(&pattern3)
                {
                    // Check for negation in query
                    if tokens.contains(&"not".to_string()) {
                        return Some((
                            Answer::No,
                            Some(0.0),
                            Some("example-lookup:negated".to_string()),
                        ));
                    }
                    return Some((
                        Answer::Yes,
                        Some(0.0),
                        Some("example-lookup".to_string()),
                    ));
                }

                // Also check reverse: "{subject} is a {object}" in subject's entry
                // (if subject has an entry)
            }
        }

        // Check if object is mentioned in subject's definition/examples
        for (_name, space) in &self.spaces {
            if let Some(subj_entry) = space
                .dictionary
                .entries
                .iter()
                .find(|e| e.word == subject)
            {
                let all_text = format!(
                    "{}. {}",
                    subj_entry.definition,
                    subj_entry.examples.join(". ")
                );
                let lower = all_text.to_lowercase();

                let pattern1 = format!("{} is a {}", subject, object);
                let pattern2 = format!("{} is an {}", subject, object);

                if lower.contains(&pattern1) || lower.contains(&pattern2) {
                    if tokens.contains(&"not".to_string()) {
                        return Some((
                            Answer::No,
                            Some(0.0),
                            Some("example-lookup:negated".to_string()),
                        ));
                    }
                    return Some((
                        Answer::Yes,
                        Some(0.0),
                        Some("example-lookup".to_string()),
                    ));
                }
            }
        }

        None
    }

    // ─── Task Classification ─────────────────────────────────

    /// Handle 'Is "{quoted}" a {type} task?' queries.
    /// Analyzes the quoted content to determine if it's a number or word task.
    fn resolve_task_classification(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let quoted = extract_quoted(query)?;
        let lower = query.to_lowercase();

        // Determine what task type is being asked about
        let asking_number = lower.contains("number task");
        let asking_word = lower.contains("word task");

        if !asking_number && !asking_word {
            return None;
        }

        // Analyze the quoted content for domain indicators
        let quoted_tokens = tokenize(&quoted);

        let math_indicators = ["plus", "minus", "count", "number", "equal", "more", "less"];
        let grammar_indicators = ["noun", "verb", "sentence", "word", "subject", "property"];

        let math_count = quoted_tokens
            .iter()
            .filter(|t| math_indicators.contains(&t.as_str()))
            .count();
        let grammar_count = quoted_tokens
            .iter()
            .filter(|t| grammar_indicators.contains(&t.as_str()))
            .count();

        let is_math_content = math_count > grammar_count;
        let is_grammar_content = grammar_count > math_count;

        if asking_number {
            if is_math_content {
                return Some((Answer::Yes, Some(0.0), Some("task-classify:number".to_string())));
            } else if is_grammar_content {
                return Some((Answer::No, Some(0.0), Some("task-classify:not-number".to_string())));
            }
        }

        if asking_word {
            if is_grammar_content {
                return Some((Answer::Yes, Some(0.0), Some("task-classify:word".to_string())));
            } else if is_math_content {
                return Some((Answer::No, Some(0.0), Some("task-classify:not-word".to_string())));
            }
        }

        None
    }

    // ─── "Same As" Query Handling ──────────────────────────

    /// Handle "Is X the same as Y?" queries by checking definitions.
    fn resolve_same_as_query(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let tokens = tokenize(query);

        // Find "same" position, extract subject (before) and object (after "as")
        let same_idx = tokens.iter().position(|t| t == "same")?;
        let as_idx = tokens.iter().position(|t| t == "as")?;

        // Subject: last content word before "same"
        let subject = tokens[..same_idx]
            .iter()
            .filter(|t| !is_structural(t))
            .last()?
            .clone();

        // Object: first content word after "as"
        let object = tokens[as_idx + 1..]
            .iter()
            .filter(|t| !is_structural(t))
            .next()?
            .clone();

        // Same word = Yes
        if subject == object {
            return Some((Answer::Yes, Some(0.0), Some("same-as:identical".to_string())));
        }

        // Check if definitions describe them differently
        for (_name, space) in &self.spaces {
            let subj_entry = space.dictionary.entries.iter().find(|e| e.word == subject);
            let obj_entry = space.dictionary.entries.iter().find(|e| e.word == object);

            if let (Some(se), Some(oe)) = (subj_entry, obj_entry) {
                // If definitions are very different → No
                // Check if one's definition mentions "not" + the other
                let s_def = se.definition.to_lowercase();
                let o_def = oe.definition.to_lowercase();

                if s_def.contains(&format!("not {}", object))
                    || o_def.contains(&format!("not {}", subject))
                {
                    return Some((
                        Answer::No,
                        Some(0.0),
                        Some("same-as:negated-in-definition".to_string()),
                    ));
                }

                // If they have same definition category but different actions → No
                // For now: different entry words with different first sentences → No
                let s_first = s_def.split('.').next().unwrap_or("");
                let o_first = o_def.split('.').next().unwrap_or("");
                if s_first != o_first {
                    return Some((
                        Answer::No,
                        Some(0.0),
                        Some("same-as:different-definitions".to_string()),
                    ));
                }
            }
        }

        None
    }

    // ─── OR Choice Query Handling ─────────────────────────

    /// Handle "Is X a Y or a Z?" by testing which option is correct.
    /// E.g., "Is three a noun or a verb?" → test "Is three a noun?" and "Is three a verb?"
    fn resolve_or_choice(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let lower = query.to_lowercase();

        // Parse: "is {subject} a {optionA} or a {optionB}"
        let or_idx = lower.find(" or ")?;
        let before_or = &lower[..or_idx];
        let after_or = lower[or_idx + 4..].trim().trim_end_matches('?');

        // Extract optionB (after "or"): remove leading "a "/"an "
        let option_b = after_or
            .strip_prefix("a ")
            .or_else(|| after_or.strip_prefix("an "))
            .unwrap_or(after_or)
            .trim();

        // Extract subject and optionA from before "or"
        // Format: "is {subject} a {optionA}"
        let parts: Vec<&str> = before_or.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        // Subject is word(s) between "is" and "a"
        let a_idx = parts.iter().rposition(|p| *p == "a" || *p == "an")?;
        if a_idx < 2 {
            return None;
        }
        let subject = parts[1..a_idx].join(" ");
        let option_a = parts[a_idx + 1..].join(" ");

        // Test each option by constructing "Is {subject} a {option}?"
        let query_a = format!("Is {} a {}?", subject, option_a);
        let query_b = format!("Is {} a {}?", subject, option_b);

        let (answer_a, _, _) = self.resolve(&query_a);
        let (answer_b, _, _) = self.resolve(&query_b);

        match (&answer_a, &answer_b) {
            (Answer::Yes, Answer::No) | (Answer::Yes, Answer::IDontKnow) => Some((
                Answer::Word(format!("a {}", option_a)),
                Some(0.0),
                Some("or-choice:a".to_string()),
            )),
            (Answer::No, Answer::Yes) | (Answer::IDontKnow, Answer::Yes) => Some((
                Answer::Word(format!("a {}", option_b)),
                Some(0.0),
                Some("or-choice:b".to_string()),
            )),
            (Answer::Yes, Answer::Yes) => {
                // Both passed — use definition proximity to tiebreak.
                // Check which option's definition more directly describes the subject's category.
                // "noun = a name for a thing" vs "verb = tells what a thing does"
                // Numbers are things (noun category), not actions (verb category).
                // Tiebreaker: check subject's category chain in its home space, then see
                // which option's definition directly names that category.
                if let Some(winner) = self.or_choice_tiebreak(&subject, &option_a, &option_b) {
                    Some((
                        Answer::Word(format!("a {}", winner)),
                        Some(0.0),
                        Some("or-choice:tiebreak".to_string()),
                    ))
                } else {
                    // Default to first option
                    Some((
                        Answer::Word(format!("a {}", option_a)),
                        Some(0.0),
                        Some("or-choice:default-a".to_string()),
                    ))
                }
            }
            _ => None, // ambiguous
        }
    }

    /// Tiebreak for OR choice when both options are Yes.
    /// Determines which option is the "primary" classification by checking
    /// which option's definition most directly describes the subject's category.
    fn or_choice_tiebreak(
        &self,
        subject: &str,
        option_a: &str,
        option_b: &str,
    ) -> Option<String> {
        // Find which category the subject belongs to in its home space.
        // E.g., "three" → "number" → "thing" in MATH
        let mut subject_categories: Vec<String> = Vec::new();

        for (_name, space) in &self.spaces {
            if let Some(entry) = space.dictionary.entries.iter().find(|e| e.word == subject) {
                let def_tokens = tokenize(&entry.definition);
                for t in &def_tokens {
                    if !is_structural(t) {
                        let stemmed = stem_to_entry(t, &space.dictionary.entry_set)
                            .unwrap_or_else(|| t.clone());
                        subject_categories.push(stemmed.clone());

                        // Follow one more hop
                        if let Some(cat_entry) = space.dictionary.entries.iter().find(|e| e.word == stemmed) {
                            let cat_tokens = tokenize(&cat_entry.definition);
                            for ct in &cat_tokens {
                                if !is_structural(ct) {
                                    let cs = stem_to_entry(ct, &space.dictionary.entry_set)
                                        .unwrap_or_else(|| ct.clone());
                                    subject_categories.push(cs);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Now check which option's definition uses those categories as its
        // primary description. "noun" = "a name for a thing" → category "thing"
        // is the thing being named. "verb" = "tells what a thing does" → "thing"
        // is incidental, the focus is "action/does".
        let score_a = self.or_option_category_score(option_a, &subject_categories);
        let score_b = self.or_option_category_score(option_b, &subject_categories);

        if score_a > score_b {
            Some(option_a.to_string())
        } else if score_b > score_a {
            Some(option_b.to_string())
        } else {
            None
        }
    }

    /// Score how well an option's definition matches subject categories.
    /// Higher = more direct match. Penalize definitions where the category
    /// word appears after "does" or as an agent rather than as the object.
    fn or_option_category_score(
        &self,
        option: &str,
        categories: &[String],
    ) -> i32 {
        let mut score = 0;

        for (_name, space) in &self.spaces {
            if let Some(entry) = space.dictionary.entries.iter().find(|e| e.word == option) {
                let def_lower = entry.definition.to_lowercase();
                let def_tokens = tokenize(&entry.definition);

                // Check if "name for a {category}" pattern exists (very direct)
                for cat in categories {
                    let pattern = format!("name for a {}", cat);
                    if def_lower.contains(&pattern) {
                        score += 10;
                    }

                    // Check if definition ends with category (direct object)
                    if let Some(last_content) = def_tokens.iter().rev().find(|t| !is_structural(t)) {
                        let stemmed = stem_to_entry(last_content, &space.dictionary.entry_set)
                            .unwrap_or_else(|| last_content.clone());
                        if stemmed == *cat {
                            score += 5;
                        }
                    }

                    // Penalize if category appears before "does" (agent, not object)
                    if def_lower.contains(&format!("{} does", cat)) {
                        score -= 3;
                    }
                }
            }
        }

        score
    }

    // ─── "What kind" Query Handling ──────────────────────────

    /// Handle "What kind of task is X?" queries by scanning TASK definitions.
    fn resolve_kind_query(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let lower = query.to_lowercase();

        if !lower.contains("kind") && !lower.contains("task") {
            return None;
        }

        let tokens = tokenize(query);

        // Find the subject word (last content word, excluding "task", "kind")
        let content: Vec<&String> = tokens
            .iter()
            .filter(|t| {
                !is_structural(t) && t.as_str() != "kind" && t.as_str() != "task"
            })
            .collect();

        let subject = content.last()?;

        // Search all spaces for "{subject} is a number task" or "{subject} is a word task"
        for (_name, space) in &self.spaces {
            for entry in &space.dictionary.entries {
                if entry.word != subject.as_str() {
                    continue;
                }
                let all_text = format!(
                    "{}. {}",
                    entry.definition,
                    entry.examples.join(". ")
                );
                let text_lower = all_text.to_lowercase();

                if text_lower.contains("number task") {
                    return Some((
                        Answer::Word("a number task".to_string()),
                        Some(0.0),
                        Some("kind-lookup".to_string()),
                    ));
                }
                if text_lower.contains("word task") {
                    return Some((
                        Answer::Word("a word task".to_string()),
                        Some(0.0),
                        Some("kind-lookup".to_string()),
                    ));
                }
            }
        }

        // Also check if the subject is mentioned in any "task" patterns
        let task_space = self.spaces.get("task")?;
        for entry in &task_space.dictionary.entries {
            let all_text = format!(
                "{}. {}",
                entry.definition,
                entry.examples.join(". ")
            );
            let text_lower = all_text.to_lowercase();

            // Check if this entry mentions the subject as part of a task type
            if text_lower.contains(subject.as_str()) {
                if text_lower.contains("number task") {
                    return Some((
                        Answer::Word("a number task".to_string()),
                        Some(0.0),
                        Some("kind-lookup:task-space".to_string()),
                    ));
                }
                if text_lower.contains("word task") {
                    return Some((
                        Answer::Word("a word task".to_string()),
                        Some(0.0),
                        Some("kind-lookup:task-space".to_string()),
                    ));
                }
            }
        }

        None
    }
}

// ─── Helper Functions ────────────────────────────────────────

/// Extract quoted substring from a query.
fn extract_quoted(query: &str) -> Option<String> {
    // Try double quotes first
    if let Some(start) = query.find('"') {
        if let Some(end) = query[start + 1..].find('"') {
            return Some(query[start + 1..start + 1 + end].to_string());
        }
    }
    // Try smart quotes
    if let Some(start) = query.find('\u{201C}') {
        let after = start + '\u{201C}'.len_utf8();
        if let Some(end) = query[after..].find('\u{201D}') {
            return Some(query[after..after + end].to_string());
        }
    }
    None
}

/// Compute average distance from results that have distances.
fn avg_distance(results: &[SpaceResult]) -> Option<f64> {
    let dists: Vec<f64> = results
        .iter()
        .filter_map(|r| r.distance)
        .collect();
    if dists.is_empty() {
        None
    } else {
        Some(dists.iter().sum::<f64>() / dists.len() as f64)
    }
}
