//! Multi-Space Architecture (Phase 16)
//!
//! Multiple independent geometric spaces ("thought domains") that compose
//! results at query time via bridge terms and a TASK dispatcher.

use std::collections::{HashMap, HashSet};

use dafhne_core::*;
use dafhne_parser::{load_dictionary, stem_to_entry, tokenize};

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
    /// Union of all per-space structural word sets. Used instead of
    /// the old hardcoded is_structural() function for routing and filtering.
    pub structural_words_cache: HashSet<String>,
    /// Words unique to SELF space vocabulary — used as trigger words for
    /// SELF-space activation instead of the old hardcoded ["dafhne"] list.
    pub self_trigger_words: HashSet<String>,
}

// ─── Structural Words ────────────────────────────────────────

/// Hardcoded structural word list — kept as fallback for standalone functions
/// (like `yes_no_to_declarative`) that don't have access to MultiSpace.
/// All MultiSpace methods use `self.is_structural_cached()` instead, which
/// draws from the discovered structural word sets (classify_word_roles).
#[allow(dead_code)]
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

/// Map a number word to its ordinal value.
fn number_word_to_value(word: &str) -> Option<u32> {
    match word {
        "zero" => Some(0),
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
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
            let dictionary = load_dictionary(&config.dict_path)
                .unwrap_or_else(|_| panic!("Failed to read dictionary: {}", config.dict_path));

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
            structural_words_cache: HashSet::new(),
            self_trigger_words: HashSet::new(),
        };
        ms.finish_construction();

        ms
    }

    /// Construct a MultiSpace with per-space parameters.
    /// Each space looks up its own (EngineParams, StrategyConfig) from `space_params`,
    /// falling back to `default_params`/`default_strategy` for any space not in the map.
    pub fn new_per_space(
        configs: Vec<SpaceConfig>,
        space_params: &HashMap<String, (EngineParams, StrategyConfig)>,
        default_params: &EngineParams,
        default_strategy: &StrategyConfig,
        build_mode: BuildMode,
    ) -> Self {
        let mut spaces = HashMap::new();
        let mut space_order = Vec::new();

        for config in configs {
            let dictionary = load_dictionary(&config.dict_path)
                .unwrap_or_else(|_| panic!("Failed to read dictionary: {}", config.dict_path));

            let (params, strategy) = space_params
                .get(&config.name)
                .cloned()
                .unwrap_or_else(|| (default_params.clone(), default_strategy.clone()));

            println!(
                "[Space {}] Loading {} ({} entries, dims={})",
                config.name,
                config.dict_path,
                dictionary.entries.len(),
                params.dimensions
            );

            let mut engine = Engine::with_strategy(params.clone(), strategy.clone());
            engine.set_quiet(true);
            engine.set_mode(build_mode);
            engine.train(&dictionary);

            let space = Space {
                name: config.name.clone(),
                engine,
                dictionary,
                params,
                strategy,
            };

            space_order.push(config.name.clone());
            spaces.insert(config.name, space);
        }

        let mut ms = MultiSpace {
            spaces,
            bridges: HashMap::new(),
            space_order,
            structural_words_cache: HashSet::new(),
            self_trigger_words: HashSet::new(),
        };
        ms.finish_construction();

        ms
    }

    /// Shared post-construction setup: bridges, structural words, self trigger words.
    fn finish_construction(&mut self) {
        self.identify_bridges();

        let mut structural_cache = HashSet::new();
        for (_, space) in &self.spaces {
            structural_cache.extend(space.engine.structural().iter().cloned());
        }
        for qw in &["what", "who", "where", "when", "why", "how", "which",
                     "yes", "no", "you", "your", "are", "be", "do", "does"] {
            structural_cache.insert(qw.to_string());
        }
        self.structural_words_cache = structural_cache.clone();

        if let Some(self_space) = self.spaces.get("self") {
            let self_vocab: &HashSet<String> = &self_space.dictionary.entry_set;
            let mut other_vocab: HashSet<String> = HashSet::new();
            for (name, space) in &self.spaces {
                if name != "self" && name != "task" {
                    other_vocab.extend(space.dictionary.entry_set.iter().cloned());
                }
            }
            self.self_trigger_words = self_vocab
                .iter()
                .filter(|w| !other_vocab.contains(w.as_str()) && !structural_cache.contains(w.as_str()))
                .cloned()
                .collect();
        }
    }

    /// Check whether a word is structural (high doc-frequency across spaces).
    /// Uses the discovered structural word cache built from per-space classify_word_roles().
    fn is_structural_cached(&self, word: &str) -> bool {
        self.structural_words_cache.contains(word)
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
            if self.is_structural_cached(token) {
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

        // SELF-space activation: identity and capability queries
        // Check raw tokens (before structural filtering) because "you", "are", "can"
        // are all structural words that get filtered in the content word loop above.
        //
        // Trigger words are derived from SELF vocabulary (words unique to SELF space,
        // not found in other domain spaces or in the structural set).
        // Pronoun patterns below are structural-word patterns not discoverable from
        // vocabulary alone — they detect "are you", "can you", "do you" bigrams.
        let self_patterns: [(&str, &str); 3] = [
            ("are", "you"),  // "What are you?", "Are you a person?"
            ("can", "you"),  // "Can you count?", "Can you see?"
            ("do", "you"),   // "Do you know?", "Do you learn?"
        ];

        let has_self_trigger = tokens.iter().any(|t| self.self_trigger_words.contains(t.as_str()));
        let has_self_pattern = self_patterns.iter().any(|(a, b)| {
            tokens.contains(&a.to_string()) && tokens.contains(&b.to_string())
        });

        if has_self_trigger || has_self_pattern {
            if self.spaces.contains_key("self") {
                exclusive.insert("self".to_string());
            }
        }

        // If no exclusive activations, use TASK space to disambiguate
        if exclusive.is_empty() {
            if let Some(task_space) = self.spaces.get("task") {
                let mut math_score = 0.0f64;
                let mut grammar_score = 0.0f64;
                let mut content_score = 0.0f64;
                let mut self_score = 0.0f64;

                for token in &tokens {
                    if self.is_structural_cached(token) {
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
                        if let Some(content_pos) =
                            task_space.engine.space().words.get("content")
                        {
                            let d = euclidean_distance(
                                &token_pos.position,
                                &content_pos.position,
                            );
                            content_score += 1.0 / (1.0 + d);
                        }
                        if let Some(self_pos) =
                            task_space.engine.space().words.get("self")
                        {
                            let d = euclidean_distance(
                                &token_pos.position,
                                &self_pos.position,
                            );
                            self_score += 1.0 / (1.0 + d);
                        }
                    }
                }

                // 4-way disambiguation: find best and second-best domain
                let mut scores = [
                    ("math", math_score),
                    ("grammar", grammar_score),
                    ("content", content_score),
                    ("self", self_score),
                ];
                scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                let (best_name, best_score) = scores[0];
                let (_second_name, second_score) = scores[1];

                if best_score > 0.0 {
                    if best_score > second_score * 1.2 {
                        // Clear winner
                        exclusive.insert(best_name.to_string());
                    } else {
                        // Top two are close — activate both
                        exclusive.insert(scores[0].0.to_string());
                        exclusive.insert(scores[1].0.to_string());
                    }
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

        // ─── SELF Space Patterns ─────────────────────────────
        // Only activate when SELF space is loaded.
        if self.spaces.contains_key("self") {
            let tokens = tokenize(query);

            // Pattern A: "What are you?" — SELF identity
            if lower.contains("what") && lower.contains("are") && lower.contains("you") {
                return self.resolve_self_identity();
            }

            // Pattern B: "Can you X?" — SELF capability check
            if lower.contains("can") && lower.contains("you") {
                return self.resolve_self_capability(query, &tokens);
            }

            // Pattern C: "Do you have X?" — SELF possession check
            if lower.contains("you") && lower.contains("have") {
                return self.resolve_self_possession(&tokens);
            }

            // Pattern D: "Are you a X?" — SELF identity classification
            if lower.contains("are") && lower.contains("you")
                && !lower.contains("what")
            {
                return self.resolve_self_identity_check(&tokens);
            }

            // Pattern E: "Do you know X?" — SELF meta-knowledge check
            if lower.contains("do") && lower.contains("you") && lower.contains("know") {
                return self.resolve_self_meta_check(&tokens);
            }
        }

        // Pattern: "Is X more/less than Y?" — ordinal comparison
        if let Some(result) = self.resolve_ordinal_comparison(&lower) {
            return Some(result);
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
            if !self.is_structural_cached(word) {
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

    /// Resolve ordinal comparison: "Is three more than one?" → Yes.
    /// Compares number-word values directly.
    fn resolve_ordinal_comparison(
        &self,
        lower: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        // Detect comparison pattern
        let (is_more, pivot) = if lower.contains("more than") {
            (true, "more than")
        } else if lower.contains("bigger than") {
            (true, "bigger than")
        } else if lower.contains("greater than") {
            (true, "greater than")
        } else if lower.contains("less than") {
            (false, "less than")
        } else if lower.contains("smaller than") {
            (false, "smaller than")
        } else {
            return None;
        };

        // Must be a yes/no question starting with "is"
        if !lower.starts_with("is ") {
            return None;
        }

        // Extract operands: "is X more than Y"
        let after_is = lower.strip_prefix("is ")?.trim();
        let parts: Vec<&str> = after_is.split(pivot).collect();
        if parts.len() != 2 {
            return None;
        }

        let left_word = parts[0].trim();
        let right_word = parts[1].trim().trim_end_matches(|c: char| !c.is_alphanumeric());

        let left_val = number_word_to_value(left_word)?;
        let right_val = number_word_to_value(right_word)?;

        let answer = if is_more {
            left_val > right_val
        } else {
            left_val < right_val
        };
        Some((
            if answer { Answer::Yes } else { Answer::No },
            Some(0.0),
            Some("ordinal-comparison".to_string()),
        ))
    }

    // ─── Multi-Instruction Detection ─────────────────────────

    /// Detect multi-instruction queries separated by periods.
    /// Handles: arithmetic+format, question+format, compound questions,
    /// arithmetic pipeline with result substitution.
    fn detect_multi_instruction(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        // Split on periods or question marks (treating ? as segment separator
        // when followed by more text). We split on both '.' and '?' to handle
        // compound queries like "Is the sun big? Is five big?"
        let segments: Vec<&str> = query
            .split(|c: char| c == '.' || c == '?')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if segments.len() < 2 {
            return None;
        }

        let question_starters = [
            "is", "can", "what", "who", "where", "when", "why", "does", "has", "are",
        ];

        // Check if any segment has a formatting instruction
        let has_formatting_instruction = segments.iter().any(|s| {
            let lower = s.to_lowercase();
            lower.contains("write") || lower.contains("sentence")
        });

        // ─── Path A: Has formatting instruction ─────────────
        if has_formatting_instruction {
            let first = segments[0];

            // A1: Try arithmetic on first segment + formatting
            if let Some(arith) = self.detect_arithmetic(first) {
                if let Some(Answer::Word(result)) = self.resolve_arithmetic(&arith) {
                    let second_lower = segments[1].to_lowercase();
                    if second_lower.contains("sentence") || second_lower.contains("write") {
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

            // A2: Try Yes/No question on first segment + sentence formatting
            let first_lower = first.to_lowercase();
            let first_word_of_first = first_lower.split_whitespace().next().unwrap_or("");
            if question_starters.contains(&first_word_of_first) {
                let (answer, dist, _conn) = self.resolve(first);
                let wants_sentence = segments[1..].iter().any(|s| {
                    let sl = s.to_lowercase();
                    sl.contains("write") || sl.contains("sentence")
                });
                if wants_sentence {
                    if answer == Answer::Yes {
                        let sentence = yes_no_to_declarative(first);
                        return Some((
                            Answer::Word(sentence),
                            dist,
                            Some("multi-instruction:question+format".to_string()),
                        ));
                    }
                }
            }

            // A3: Fallback — resolve last question segment
            let last = segments.last()?;
            let last_lower = last.to_lowercase();
            let first_word = last_lower.split_whitespace().next()?;
            if question_starters.contains(&first_word) {
                let (answer, dist, conn) = self.resolve(last);
                return Some((answer, dist, conn));
            }

            return None;
        }

        // ─── Path B: No formatting — check for compound questions ───
        let all_questions = segments.iter().all(|s| {
            let fw = s.to_lowercase();
            let first_w = fw.split_whitespace().next().unwrap_or("");
            question_starters.contains(&first_w)
        });

        if all_questions {
            let mut answers: Vec<String> = Vec::new();
            let mut all_yes_no = true;

            for seg in &segments {
                let (answer, _, _) = self.resolve(seg);
                match &answer {
                    Answer::Yes => answers.push("Yes".to_string()),
                    Answer::No => answers.push("No".to_string()),
                    Answer::Word(w) => {
                        answers.push(w.clone());
                        all_yes_no = false;
                    }
                    Answer::IDontKnow => {
                        answers.push("I don't know".to_string());
                        all_yes_no = false;
                    }
                }
            }

            let combined = if all_yes_no {
                answers.join(" and ")
            } else {
                let joined = answers.join(". ");
                format!("{}.", joined)
            };
            return Some((
                Answer::Word(combined),
                Some(0.0),
                Some("multi-instruction:compound-question".to_string()),
            ));
        }

        // ─── Path C: Arithmetic pipeline with result substitution ───
        // "One plus one. Is the result equal to two?"
        let first = segments[0];
        if let Some(arith) = self.detect_arithmetic(first) {
            if let Some(Answer::Word(result)) = self.resolve_arithmetic(&arith) {
                // Find the last segment that's a question
                if let Some(q) = segments[1..].iter().rev().find(|s| {
                    let fw = s.to_lowercase();
                    let first_w = fw.split_whitespace().next().unwrap_or("");
                    question_starters.contains(&first_w)
                }) {
                    // Substitute "the result" / "the answer" / " it " with computed value
                    let substituted = q
                        .to_lowercase()
                        .replace("the result", &result)
                        .replace("the answer", &result)
                        .replace(" it ", &format!(" {} ", result));

                    // For property questions about arithmetic results: if the result
                    // word isn't in the CONTENT space AND the question asks about a
                    // physical property (big/small/hot/cold), return No. Numbers don't
                    // have physical properties — math-space geometry would misleadingly
                    // say Yes based on numeric examples like "five is big".
                    // BUT: equality questions ("is X equal to Y") should NOT be blocked.
                    if substituted.starts_with("is ") {
                        let result_in_content = self
                            .spaces
                            .get("content")
                            .map_or(false, |s| {
                                s.dictionary.entry_set.contains(result.as_str())
                            });
                        if !result_in_content {
                            // Only block for content-property questions, not math questions
                            let sub_tokens = tokenize(&substituted);
                            let has_content_property = self
                                .spaces
                                .get("content")
                                .map_or(false, |content_space| {
                                    sub_tokens.iter().any(|t| {
                                        t.as_str() != result.as_str()
                                            && content_space
                                                .dictionary
                                                .entry_set
                                                .contains(t.as_str())
                                            && !self.is_structural_cached(t)
                                    })
                                });
                            if has_content_property {
                                return Some((
                                    Answer::No,
                                    Some(0.0),
                                    Some(
                                        "multi-instruction:arithmetic-pipeline-no-content"
                                            .to_string(),
                                    ),
                                ));
                            }
                        }
                    }

                    let (answer, dist, _conn) = self.resolve(&substituted);
                    return Some((
                        answer,
                        dist,
                        Some("multi-instruction:arithmetic-pipeline".to_string()),
                    ));
                }
            }
        }

        // ─── Path D: Last segment is a question (original fallback) ───
        let last = segments.last()?;
        let last_lower = last.to_lowercase();
        let first_word = last_lower.split_whitespace().next()?;
        if question_starters.contains(&first_word) {
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
            .filter(|t| !self.is_structural_cached(t))
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
                        tgt.params.max_chain_hops,
                        &mut visited,
                        tgt.engine.space(),
                        tgt.params.max_follow_per_hop,
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
                        tgt.params.max_chain_hops,
                        &mut visited2,
                        tgt.engine.space(),
                        tgt.params.max_follow_per_hop,
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
                        src.params.max_chain_hops,
                        &mut visited,
                        src.engine.space(),
                        src.params.max_follow_per_hop,
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
                src.params.max_chain_hops,
                &mut visited1,
                src.engine.space(),
                src.params.max_follow_per_hop,
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
                tgt.params.max_chain_hops,
                &mut visited2,
                tgt.engine.space(),
                tgt.params.max_follow_per_hop,
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
                tgt.params.max_chain_hops,
                &mut visited3,
                tgt.engine.space(),
                tgt.params.max_follow_per_hop,
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
            .filter(|t| !self.is_structural_cached(t))
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
    /// Analyzes the quoted content to determine if it's a number, word, or content task.
    fn resolve_task_classification(
        &self,
        query: &str,
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let quoted = extract_quoted(query)?;
        let lower = query.to_lowercase();

        // Determine what task type is being asked about
        let asking_number = lower.contains("number task");
        let asking_word = lower.contains("word task");
        let asking_content = lower.contains("content task");

        if !asking_number && !asking_word && !asking_content {
            return None;
        }

        // Analyze the quoted content for domain indicators.
        // Grammar and content indicators are derived from space vocabularies
        // (not hardcoded lists). Math indicators remain hardcoded because
        // math operator words ("plus", "minus") are the same across dict sizes.
        let quoted_tokens = tokenize(&quoted);

        let math_indicators = ["plus", "minus", "count", "number", "equal", "more", "less"];

        // Grammar/content indicator sets: use words that are in exactly one
        // domain space (exclusive vocabulary) and are not structural.
        let grammar_vocab: HashSet<&str> = self.spaces.get("grammar")
            .map(|s| s.dictionary.entry_set.iter().map(|w| w.as_str()).collect())
            .unwrap_or_default();
        let content_vocab: HashSet<&str> = self.spaces.get("content")
            .map(|s| s.dictionary.entry_set.iter().map(|w| w.as_str()).collect())
            .unwrap_or_default();

        let math_count = quoted_tokens
            .iter()
            .filter(|t| math_indicators.contains(&t.as_str()))
            .count();
        let grammar_count = quoted_tokens
            .iter()
            .filter(|t| grammar_vocab.contains(t.as_str()) && !self.is_structural_cached(t))
            .count();
        let content_count = quoted_tokens
            .iter()
            .filter(|t| content_vocab.contains(t.as_str()) && !self.is_structural_cached(t))
            .count();

        // 3-way classification: each domain wins only if it beats BOTH others
        let is_math = math_count > grammar_count && math_count > content_count;
        let is_grammar = grammar_count > math_count && grammar_count > content_count;
        let is_content = content_count > math_count && content_count > grammar_count;

        if asking_number {
            if is_math {
                return Some((Answer::Yes, Some(0.0), Some("task-classify:number".to_string())));
            } else if is_grammar || is_content {
                return Some((Answer::No, Some(0.0), Some("task-classify:not-number".to_string())));
            }
        }

        if asking_word {
            if is_grammar {
                return Some((Answer::Yes, Some(0.0), Some("task-classify:word".to_string())));
            } else if is_math || is_content {
                return Some((Answer::No, Some(0.0), Some("task-classify:not-word".to_string())));
            }
        }

        if asking_content {
            if is_content {
                return Some((Answer::Yes, Some(0.0), Some("task-classify:content".to_string())));
            } else if is_math || is_grammar {
                return Some((Answer::No, Some(0.0), Some("task-classify:not-content".to_string())));
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
            .filter(|t| !self.is_structural_cached(t))
            .last()?
            .clone();

        // Object: first content word after "as"
        let object = tokens[as_idx + 1..]
            .iter()
            .filter(|t| !self.is_structural_cached(t))
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
                    if !self.is_structural_cached(t) {
                        let stemmed = stem_to_entry(t, &space.dictionary.entry_set)
                            .unwrap_or_else(|| t.clone());
                        subject_categories.push(stemmed.clone());

                        // Follow one more hop
                        if let Some(cat_entry) = space.dictionary.entries.iter().find(|e| e.word == stemmed) {
                            let cat_tokens = tokenize(&cat_entry.definition);
                            for ct in &cat_tokens {
                                if !self.is_structural_cached(ct) {
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
                    if let Some(last_content) = def_tokens.iter().rev().find(|t| !self.is_structural_cached(t)) {
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

        // Check for quoted phrases — meta-classification of quoted content.
        // "What kind of task is 'how many animals'?" → classify the quoted content.
        if let Some(quoted) = extract_quoted(&lower) {
            if let Some(_task_space) = self.spaces.get("task") {
                // First: check for known domain indicator PHRASES in the quoted text.
                // Multi-word phrases like "how many" are more specific than single tokens,
                // so they take priority. This prevents "animal" (content) from overriding
                // "how many" (math) when both appear in the quoted text.
                let math_indicators = [
                    "how many", "plus", "minus", "count", "number", "equal",
                ];
                let grammar_indicators = ["noun", "verb", "sentence", "write"];
                let content_indicators = ["animal", "dog", "cat", "sun", "hot", "cold"];

                if math_indicators.iter().any(|ind| quoted.contains(ind)) {
                    return Some((
                        Answer::Word("a number task".to_string()),
                        Some(0.0),
                        Some("kind-lookup:quoted-indicator".to_string()),
                    ));
                }
                if grammar_indicators.iter().any(|ind| quoted.contains(ind)) {
                    return Some((
                        Answer::Word("a word task".to_string()),
                        Some(0.0),
                        Some("kind-lookup:quoted-indicator".to_string()),
                    ));
                }
                if content_indicators.iter().any(|ind| quoted.contains(ind)) {
                    return Some((
                        Answer::Word("a content task".to_string()),
                        Some(0.0),
                        Some("kind-lookup:quoted-indicator".to_string()),
                    ));
                }
            }
        }

        // Find the subject word (last content word, excluding "task", "kind")
        let content: Vec<&String> = tokens
            .iter()
            .filter(|t| {
                !self.is_structural_cached(t) && t.as_str() != "kind" && t.as_str() != "task"
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
                if text_lower.contains("content task") {
                    return Some((
                        Answer::Word("a content task".to_string()),
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
                if text_lower.contains("content task") {
                    return Some((
                        Answer::Word("a content task".to_string()),
                        Some(0.0),
                        Some("kind-lookup:task-space".to_string()),
                    ));
                }
            }
        }

        None
    }

    // ─── SELF Space Pattern Handlers ──────────────────────────

    /// Pattern A: "What are you?" — return dafhne's identity from SELF space.
    fn resolve_self_identity(&self) -> Option<(Answer, Option<f64>, Option<String>)> {
        let self_space = self.spaces.get("self")?;
        let dafhne_entry = self_space
            .dictionary
            .entries
            .iter()
            .find(|e| e.word == "dafhne")?;
        let first_sentence = dafhne_entry
            .definition
            .split('.')
            .next()
            .unwrap_or(&dafhne_entry.definition)
            .trim();
        Some((
            Answer::Word(first_sentence.to_lowercase()),
            Some(0.0),
            Some("self:identity".to_string()),
        ))
    }

    /// Pattern B: "Can you X?" — check dafhne's capabilities in SELF space.
    /// Scans dafhne's definition and all SELF entries for "dafhne can [not] X" patterns.
    fn resolve_self_capability(
        &self,
        query: &str,
        tokens: &[String],
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let self_space = self.spaces.get("self")?;

        // Build the verb phrase after "can you": e.g., "count", "see", "make mistakes"
        let lower = query.to_lowercase();
        let can_you_pos = lower.find("can you")?;
        let after = lower[can_you_pos + 7..].trim().trim_end_matches('?').trim();
        if after.is_empty() {
            return None;
        }

        // Strategy 1: Check dafhne's definition directly
        if let Some(dafhne_entry) = self_space
            .dictionary
            .entries
            .iter()
            .find(|e| e.word == "dafhne")
        {
            let def_lower = dafhne_entry.definition.to_lowercase();

            // Check for negation first: "can not X"
            let cannot_pattern = format!("can not {}", after);
            if def_lower.contains(&cannot_pattern) {
                return Some((
                    Answer::No,
                    Some(0.0),
                    Some("self:capability:no".to_string()),
                ));
            }

            // Check for positive: "can X"
            let can_pattern = format!("can {}", after);
            if def_lower.contains(&can_pattern) {
                return Some((
                    Answer::Yes,
                    Some(0.0),
                    Some("self:capability:yes".to_string()),
                ));
            }

            // Also check single-word action (last content word)
            let action = tokens
                .iter()
                .filter(|t| !self.is_structural_cached(t))
                .last();
            if let Some(action_word) = action {
                let stemmed = stem_to_entry(action_word, &self_space.dictionary.entry_set)
                    .unwrap_or_else(|| action_word.clone());
                let cannot_single = format!("can not {}", stemmed);
                let can_single = format!("can {}", stemmed);
                if def_lower.contains(&cannot_single) {
                    return Some((
                        Answer::No,
                        Some(0.0),
                        Some("self:capability:no".to_string()),
                    ));
                }
                if def_lower.contains(&can_single) {
                    return Some((
                        Answer::Yes,
                        Some(0.0),
                        Some("self:capability:yes".to_string()),
                    ));
                }
            }
        }

        // Strategy 2: Scan all SELF entries for "dafhne can [not] {action}" patterns
        let action_word = tokens
            .iter()
            .filter(|t| !self.is_structural_cached(t))
            .last()?;
        let stemmed = stem_to_entry(action_word, &self_space.dictionary.entry_set)
            .unwrap_or_else(|| action_word.clone());

        for entry in &self_space.dictionary.entries {
            let all_text = format!(
                "{}. {}",
                entry.definition,
                entry.examples.join(". ")
            );
            let text_lower = all_text.to_lowercase();

            // Check for "dafhne can not {action}"
            let cannot_pat = format!("dafhne can not {}", stemmed);
            if text_lower.contains(&cannot_pat) {
                return Some((
                    Answer::No,
                    Some(0.0),
                    Some("self:capability:no".to_string()),
                ));
            }

            // Check for "dafhne can make {action}" (for "Can you make mistakes?")
            let can_make_pat = format!("dafhne can make {}", stemmed);
            if text_lower.contains(&can_make_pat) {
                return Some((
                    Answer::Yes,
                    Some(0.0),
                    Some("self:capability:yes".to_string()),
                ));
            }

            // Check for "dafhne can {action}"
            let can_pat = format!("dafhne can {}", stemmed);
            if text_lower.contains(&can_pat) {
                return Some((
                    Answer::Yes,
                    Some(0.0),
                    Some("self:capability:yes".to_string()),
                ));
            }
        }

        None
    }

    /// Pattern C: "Do you have X?" — check dafhne's possessions in SELF space.
    fn resolve_self_possession(
        &self,
        tokens: &[String],
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let self_space = self.spaces.get("self")?;
        let dafhne_entry = self_space
            .dictionary
            .entries
            .iter()
            .find(|e| e.word == "dafhne")?;
        let def_lower = dafhne_entry.definition.to_lowercase();

        // Extract the object: last content word
        let object = tokens
            .iter()
            .filter(|t| !self.is_structural_cached(t) && t.as_str() != "have")
            .last()?;
        let stemmed = stem_to_entry(object, &self_space.dictionary.entry_set)
            .unwrap_or_else(|| object.clone());

        // Check for "has no X" (negation first)
        let no_pattern = format!("has no {}", stemmed);
        if def_lower.contains(&no_pattern) {
            return Some((
                Answer::No,
                Some(0.0),
                Some("self:possession:no".to_string()),
            ));
        }

        // Check for "has X"
        let has_pattern = format!("has {}", stemmed);
        if def_lower.contains(&has_pattern) {
            return Some((
                Answer::Yes,
                Some(0.0),
                Some("self:possession:yes".to_string()),
            ));
        }

        None
    }

    /// Pattern D: "Are you a X?" — check dafhne's identity classification in SELF space.
    fn resolve_self_identity_check(
        &self,
        tokens: &[String],
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        let self_space = self.spaces.get("self")?;
        let dafhne_entry = self_space
            .dictionary
            .entries
            .iter()
            .find(|e| e.word == "dafhne")?;
        let def_lower = dafhne_entry.definition.to_lowercase();

        // Extract the category: last content word
        let category = tokens
            .iter()
            .filter(|t| !self.is_structural_cached(t))
            .last()?;
        let stemmed = stem_to_entry(category, &self_space.dictionary.entry_set)
            .unwrap_or_else(|| category.clone());

        // Check for "is not a X" / "is not an X" (negation first)
        let not_a = format!("not a {}", stemmed);
        let not_an = format!("not an {}", stemmed);
        if def_lower.contains(&not_a) || def_lower.contains(&not_an) {
            return Some((
                Answer::No,
                Some(0.0),
                Some("self:identity:no".to_string()),
            ));
        }

        // Check for "is a X" / "is an X"
        let is_a = format!("is a {}", stemmed);
        let is_an = format!("is an {}", stemmed);
        if def_lower.contains(&is_a) || def_lower.contains(&is_an) {
            return Some((
                Answer::Yes,
                Some(0.0),
                Some("self:identity:yes".to_string()),
            ));
        }

        None
    }

    /// Pattern E: "Do you know X?" — meta-knowledge check across all domain spaces.
    fn resolve_self_meta_check(
        &self,
        tokens: &[String],
    ) -> Option<(Answer, Option<f64>, Option<String>)> {
        // Extract the concept: last content word (excluding "know")
        let concept = tokens
            .iter()
            .filter(|t| !self.is_structural_cached(t) && t.as_str() != "know")
            .last()?;

        // Check if the concept exists in any domain space vocabulary
        for (name, space) in &self.spaces {
            if name == "task" || name == "self" {
                continue; // Only check domain knowledge spaces
            }
            if space.dictionary.entry_set.contains(concept.as_str())
                || stem_to_entry(concept, &space.dictionary.entry_set).is_some()
            {
                return Some((
                    Answer::Yes,
                    Some(0.0),
                    Some(format!("self:meta-know:{}", name)),
                ));
            }
        }

        // Concept not found in any domain space
        Some((
            Answer::No,
            Some(0.0),
            Some("self:meta-unknown".to_string()),
        ))
    }
}

// ─── Helper Functions ────────────────────────────────────────

/// Convert a Yes/No question to a declarative sentence.
/// "Can an animal eat" → "an animal can eat"
/// "Is the sun hot" → "the sun is hot"
/// "Can a person make a sound" → "a person can make a sound"
fn yes_no_to_declarative(question: &str) -> String {
    let q = question.trim().trim_end_matches('?').to_lowercase();
    let words: Vec<&str> = q.split_whitespace().collect();
    if words.len() < 3 {
        return q;
    }

    let verb = words[0]; // "can", "is", "does"
    let rest = &words[1..]; // ["an", "animal", "eat"] or ["the", "sun", "hot"]

    // Find the subject: articles + first content word
    let articles = ["a", "an", "the"];
    let mut subject_end = 0;
    for (i, w) in rest.iter().enumerate() {
        subject_end = i + 1;
        if !articles.contains(w) {
            break; // found the noun, include it
        }
    }

    let subject = &rest[..subject_end];
    let predicate = &rest[subject_end..];

    format!("{} {} {}", subject.join(" "), verb, predicate.join(" "))
        .trim()
        .to_string()
}

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
