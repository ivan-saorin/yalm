# PROMPT 14 — 2W: When/Why (Definition-Chain Reasoning)

## PREAMBLE

Phase 13 proved that DAFHNE can express what it knows: `describe()` generates sentences from definitions with 100% self-consistency. The system reads its own definitions and produces natural language.

Phase 14 adds the two "reasoning" question words: **When** and **Why**. Both are answered by reading definitions — not by geometric distance.

- **Why**: present the definition chain as an explanation
- **When**: extract conditional/purpose clauses from definitions

This confirms the architectural finding from Phase 10: **geometry for association, definitions for reasoning**.

## GOAL

Extend the resolver to handle:
- **Why is X Y?** → trace X→Y definition chain, present hops as explanation
- **Why can X Y?** → trace X→Y capability chain, present as explanation
- **When does X Y?** → extract condition/purpose from Y's definition

```
"Why is a dog an animal?" → "because a dog is an animal"
"Why can a dog eat?" → "because a dog is an animal and an animal can eat"
"When does a person eat?" → "to feel good"
"When is it cold?" → I don't know
```

## ROOT CAUSE: WHY THIS DOESN'T WORK TODAY

"Why" and "when" tokens are not recognized by `detect_question_type()`. They fall through to `detect_yes_no_question()`, which treats them as regular content words. The resolver either produces a garbled Yes/No answer or IDK.

```
"Why is a dog an animal?"
  → detect_question_type: tokens[0] = "why" → no match → detect_yes_no_question
  → content_entries: [(1, "dog"), (3, "animal")]  // "why" not in entry_set
  → subject = "dog", object = "animal"
  → Treated as "Is a dog an animal?" → Yes
  → Correct answer by accident, but wrong format (should be an explanation)
```

## THE FIX

### Architecture

New `QuestionType` variants routed through `detect_question_type()`, each with a dedicated resolver function.

```
"Why is X Y?"   → QuestionType::WhyIs { subject, object, connector }
                 → resolve_why() → Answer::Word("because ...")

"When does X Y?" → QuestionType::WhenIs { subject, action }
                  → resolve_when() → Answer::Word("to ..." / condition)
```

### Step 1: Add QuestionType variants

In `resolver.rs`, extend the `QuestionType` enum:

```rust
enum QuestionType {
    YesNo { subject: String, object: String, connector: Vec<String>, negated: bool },
    WhatIs { subject: String, connector: Vec<String>, extra_content_words: usize },
    WhyIs {
        subject: String,
        object: String,
        connector: Vec<String>,
    },
    WhenIs {
        subject: String,
        action: String,
    },
}
```

### Step 2: Route "why" and "when" in detect_question_type()

```rust
fn detect_question_type(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    if tokens.is_empty() {
        return None;
    }

    match tokens[0].as_str() {
        "what" => detect_what_question(tokens, dictionary, content, structural),
        "who" => detect_who_question(tokens, dictionary, content, structural),
        "where" => detect_where_question(tokens, dictionary, content, structural),
        "why" => detect_why_question(tokens, dictionary, content, structural),
        "when" => detect_when_question(tokens, dictionary, content, structural),
        _ => detect_yes_no_question(tokens, dictionary, content, structural),
    }
}
```

### Step 3: detect_why_question()

Extracts subject and object from "Why is X Y?" / "Why can X Y?" patterns.

```rust
/// Detect "Why is X Y?" or "Why can X Y?" questions.
///
/// Pattern: why [verb] [articles...] [subject] [articles...] [object]
/// Examples:
///   "Why is a dog an animal?" → subject=dog, object=animal
///   "Why can a dog eat?" → subject=dog, object=eat
///   "Why is the sun hot?" → subject=sun, object=hot
fn detect_why_question(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    // "why" is at position 0. Skip it and find content words.
    // Reuse the same content-word extraction as Yes/No questions.
    let question_verbs: HashSet<&str> = ["is", "can", "does", "do", "has"]
        .iter().copied().collect();
    let skip_start = if tokens.len() > 1 && question_verbs.contains(tokens[1].as_str()) {
        2  // skip "why" + verb
    } else {
        1  // skip "why" only
    };

    let mut content_entries: Vec<(usize, String)> = tokens
        .iter()
        .enumerate()
        .skip(skip_start)
        .filter_map(|(i, t)| {
            stem_to_entry(t, &dictionary.entry_set).and_then(|e| {
                if content.contains(&e) {
                    Some((i, e))
                } else {
                    None
                }
            })
        })
        .collect();

    // Fallback: include non-structural entry words if < 2 content words
    if content_entries.len() < 2 {
        let skip_words: HashSet<&str> = ["is", "a", "the", "it", "not"].iter().copied().collect();
        content_entries = tokens
            .iter()
            .enumerate()
            .skip(skip_start)
            .filter_map(|(i, t)| {
                stem_to_entry(t, &dictionary.entry_set).and_then(|e| {
                    if !skip_words.contains(e.as_str()) {
                        Some((i, e))
                    } else {
                        None
                    }
                })
            })
            .collect();
    }

    if content_entries.len() < 2 {
        return None;
    }

    let subject = content_entries[0].1.clone();
    let object = content_entries[content_entries.len() - 1].1.clone();

    // Extract connector from structural words between subject and object
    let left_pos = content_entries[0].0;
    let right_pos = content_entries[content_entries.len() - 1].0;
    let connector: Vec<String> = if right_pos > left_pos + 1 {
        (left_pos + 1..right_pos)
            .filter_map(|i| {
                stem_to_entry(&tokens[i], &dictionary.entry_set)
                    .filter(|e| structural.contains(e))
            })
            .collect()
    } else {
        Vec::new()
    };

    let connector = if connector.is_empty() {
        vec!["is".to_string()]
    } else {
        connector
    };

    Some(QuestionType::WhyIs {
        subject,
        object,
        connector,
    })
}
```

### Step 4: detect_when_question()

Extracts subject and action from "When does X Y?" patterns.

```rust
/// Detect "When does X Y?" questions.
///
/// Pattern: when [verb] [articles...] [subject] [action...]
/// Examples:
///   "When does a person eat?" → subject=person, action=eat
///   "When does a dog move?" → subject=dog, action=move
///   "When is it cold?" → subject=it (problem), action=cold
fn detect_when_question(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    _structural: &HashSet<String>,
) -> Option<QuestionType> {
    let question_verbs: HashSet<&str> = ["is", "can", "does", "do", "has"]
        .iter().copied().collect();
    let skip_start = if tokens.len() > 1 && question_verbs.contains(tokens[1].as_str()) {
        2
    } else {
        1
    };

    let content_entries: Vec<(usize, String)> = tokens
        .iter()
        .enumerate()
        .skip(skip_start)
        .filter_map(|(i, t)| {
            stem_to_entry(t, &dictionary.entry_set).and_then(|e| {
                if content.contains(&e) {
                    Some((i, e))
                } else {
                    None
                }
            })
        })
        .collect();

    if content_entries.len() < 2 {
        return None;
    }

    let subject = content_entries[0].1.clone();
    let action = content_entries[content_entries.len() - 1].1.clone();

    Some(QuestionType::WhenIs {
        subject,
        action,
    })
}
```

### Step 5: resolve_why() — Definition Chain as Explanation

The core insight: the definition chain IS the explanation. "Why is a dog an animal?" → dog's definition says "an animal" → that IS why.

For multi-hop chains: "Why is a dog a thing?" → dog→animal→thing → "because a dog is an animal, and an animal is a thing."

```rust
/// Resolve "Why is X Y?" by tracing the definition chain from X to Y
/// and presenting each hop as a "because" explanation.
///
/// Returns:
/// - Word("because X is Z, and Z is Y") → chain found
/// - IDontKnow → no chain connects X to Y
fn resolve_why(
    subject: &str,
    object: &str,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    space: &GeometricSpace,
) -> (Answer, f64) {
    // Trace the definition chain from subject to object,
    // recording the path for explanation.
    let max_hops = 3;
    let mut path: Vec<String> = Vec::new();
    path.push(subject.to_string());

    let found = trace_chain_path(
        subject, object, dictionary, structural, max_hops,
        &mut HashSet::new(), &mut path, space,
    );

    if !found {
        return (Answer::IDontKnow, f64::MAX);
    }

    // Build explanation from path.
    // path = ["dog", "animal", "thing"]
    // → "because a dog is an animal, and an animal is a thing"
    let explanation = build_chain_explanation(&path, dictionary);
    (Answer::Word(explanation), 0.0)
}

/// Trace the definition chain from `current` to `target`, recording
/// the path of words visited. Returns true if target is reached.
///
/// Uses the same traversal logic as definition_chain_check() but
/// records the path instead of just returning bool.
fn trace_chain_path(
    current: &str,
    target: &str,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    max_hops: usize,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
    space: &GeometricSpace,
) -> bool {
    if visited.contains(current) {
        return false;
    }
    visited.insert(current.to_string());

    let entry = match dictionary.entries.iter().find(|e| e.word == current) {
        Some(e) => e,
        None => return false,
    };

    let def_text: String = entry.definition
        .split('.')
        .filter(|s| !s.contains('"') && !s.contains('\u{201C}') && !s.contains('\u{201D}'))
        .collect::<Vec<_>>()
        .join(".");
    let def_words = tokenize(&def_text);

    // Direct check: does target appear in current's definition?
    if def_words.iter().any(|w| {
        stem_to_entry(w, &dictionary.entry_set)
            .map_or(false, |stemmed| stemmed == target)
    }) {
        // Check it's not negated
        if !preceded_by_not(&def_words, target, &dictionary.entry_set) {
            path.push(target.to_string());
            return true;
        }
    }

    // Hop: follow first-sentence content words
    const MAX_FOLLOW_PER_HOP: usize = 3;
    if max_hops > 0 {
        let first_sentence = def_text.split('.').next().unwrap_or(&def_text);
        let first_words = tokenize(first_sentence);
        let mut followed = 0;
        for word in &first_words {
            if followed >= MAX_FOLLOW_PER_HOP {
                break;
            }
            let stemmed = match stem_to_entry(word, &dictionary.entry_set) {
                Some(s) => s,
                None => continue,
            };
            if structural.contains(&stemmed) || stemmed == current {
                continue;
            }
            if !dictionary.entry_set.contains(&stemmed) {
                continue;
            }
            followed += 1;

            path.push(stemmed.clone());
            if trace_chain_path(
                &stemmed, target, dictionary, structural,
                max_hops - 1, visited, path, space,
            ) {
                return true;
            }
            path.pop(); // backtrack
        }
    }

    false
}

/// Build a natural-language explanation from a chain path.
///
/// path = ["dog", "animal"] → "because a dog is an animal"
/// path = ["dog", "animal", "thing"] → "because a dog is an animal, and an animal is a thing"
fn build_chain_explanation(
    path: &[String],
    dictionary: &Dictionary,
) -> String {
    if path.len() < 2 {
        return "I don't know".to_string();
    }

    let mut parts: Vec<String> = Vec::new();
    for window in path.windows(2) {
        let from = &window[0];
        let to = &window[1];
        let from_art = make_article(from, dictionary);
        let to_art = if to.starts_with(|c: char| "aeiou".contains(c)) {
            format!("an {}", to)
        } else {
            format!("a {}", to)
        };

        // Determine the linking verb from the definition.
        // If `to` appears after "can" in `from`'s definition → "can {to}"
        // Otherwise → "is {to}"
        let entry = dictionary.entries.iter().find(|e| e.word == *from);
        let uses_can = entry.map_or(false, |e| {
            let words = tokenize(&e.definition);
            words.windows(2).any(|w| w[0] == "can" && {
                stem_to_entry(&w[1], &dictionary.entry_set)
                    .map_or(false, |s| s == *to)
            })
        });

        if uses_can {
            parts.push(format!("{} can {}", from_art, to));
        } else {
            parts.push(format!("{} is {}", from_art, to_art));
        }
    }

    format!("because {}", parts.join(", and "))
}
```

### Step 6: resolve_when() — Condition/Purpose Extraction

"When does X Y?" → find Y in X's definition chain → extract surrounding conditional context.

Extraction strategy (ordered by priority):
1. **"when" clause**: If Y's definition contains "when ...", extract the clause.
2. **"to" purpose clause**: If X's definition contains "Y to Z" or "to Z", extract "to Z".
3. **"if" clause**: If Y's definition contains "if ...", extract the clause.
4. **No temporal info** → IDK.

```rust
/// Resolve "When does X Y?" by extracting conditional/purpose
/// clauses from definitions of X and Y.
///
/// Returns:
/// - Word("to feel good") → purpose clause found
/// - Word("when it is hungry") → condition clause found
/// - IDontKnow → no temporal/conditional info in definitions
fn resolve_when(
    subject: &str,
    action: &str,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    space: &GeometricSpace,
) -> (Answer, f64) {
    // Strategy 1: Look in subject's definition for clauses about the action.
    // "eat" in dog's def: not present directly. 
    // Follow chain: dog→animal→"it can eat" → found.
    // Then look in the action's definition for condition/purpose.

    // Strategy 2: Look in action's definition for condition/purpose clauses.
    // "eat" def: "you eat food. the food moves in you. you eat to feel good."
    // → extract "to feel good" as purpose.
    if let Some(clause) = extract_condition_clause(action, dictionary) {
        return (Answer::Word(clause), 0.0);
    }

    // Strategy 3: Look in subject's definition for condition about the action.
    // Check if subject's definition contains a sentence with both
    // the action word and a temporal/conditional marker.
    if let Some(clause) = extract_condition_from_subject(subject, action, dictionary) {
        return (Answer::Word(clause), 0.0);
    }

    // Strategy 4: Follow chain from subject to action, check intermediate defs.
    // dog→animal. animal def: "a thing that lives. it can move. it can eat. it can feel."
    // Look for conditions around "eat" in animal's definition.
    let max_hops = 3;
    let mut visited = HashSet::new();
    if let Some(clause) = extract_condition_via_chain(
        subject, action, dictionary, structural, max_hops, &mut visited, space,
    ) {
        return (Answer::Word(clause), 0.0);
    }

    (Answer::IDontKnow, f64::MAX)
}

/// Extract a condition/purpose clause from a word's definition.
///
/// Looks for:
/// - "X to Y" patterns (purpose: "you eat to feel good" → "to feel good")
/// - "when Y" patterns (condition: "when a person is not calm" → "when not calm")
/// - "if Y" patterns (condition)
fn extract_condition_clause(
    word: &str,
    dictionary: &Dictionary,
) -> Option<String> {
    let entry = dictionary.entries.iter().find(|e| e.word == word)?;

    // Split into sentences
    for sentence in entry.definition.split('.') {
        let trimmed = sentence.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();

        // Pattern 1: "to [verb]" purpose clause at end of sentence
        // "you eat to feel good" → extract "to feel good"
        if let Some(to_pos) = lower.rfind(" to ") {
            let clause = &trimmed[to_pos + 1..];
            // Validate: must contain at least one more word after "to"
            let clause_words: Vec<&str> = clause.split_whitespace().collect();
            if clause_words.len() >= 2 {
                return Some(clause.to_lowercase());
            }
        }

        // Pattern 2: "when [condition]" clause
        if let Some(when_pos) = lower.find("when ") {
            let clause = &trimmed[when_pos..];
            return Some(clause.to_lowercase());
        }

        // Pattern 3: "if [condition]" clause
        if let Some(if_pos) = lower.find("if ") {
            let clause = &trimmed[if_pos..];
            return Some(clause.to_lowercase());
        }
    }

    None
}

/// Extract a condition from the subject's definition about a specific action.
///
/// Looks for sentences in subject's definition that mention the action
/// word AND contain a conditional/temporal marker.
fn extract_condition_from_subject(
    subject: &str,
    action: &str,
    dictionary: &Dictionary,
) -> Option<String> {
    let entry = dictionary.entries.iter().find(|e| e.word == subject)?;

    for sentence in entry.definition.split('.') {
        let trimmed = sentence.trim();
        let lower = trimmed.to_lowercase();

        // Does this sentence mention the action?
        if !lower.contains(action) {
            continue;
        }

        // Look for purpose/condition around the action
        if let Some(to_pos) = lower.rfind(" to ") {
            let clause = &trimmed[to_pos + 1..];
            let clause_words: Vec<&str> = clause.split_whitespace().collect();
            if clause_words.len() >= 2 {
                return Some(clause.to_lowercase());
            }
        }
        if let Some(when_pos) = lower.find("when ") {
            return Some(trimmed[when_pos..].to_lowercase());
        }
    }

    None
}

/// Follow definition chain from subject toward action,
/// checking each intermediate definition for condition clauses.
fn extract_condition_via_chain(
    current: &str,
    action: &str,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    max_hops: usize,
    visited: &mut HashSet<String>,
    space: &GeometricSpace,
) -> Option<String> {
    if visited.contains(current) || max_hops == 0 {
        return None;
    }
    visited.insert(current.to_string());

    // Check current word's definition for the action + condition
    if let Some(clause) = extract_condition_from_subject(current, action, dictionary) {
        return Some(clause);
    }

    // Follow first-sentence content words
    let entry = dictionary.entries.iter().find(|e| e.word == current)?;
    let first_sentence = entry.definition.split('.').next().unwrap_or(&entry.definition);
    let first_words = tokenize(first_sentence);
    let mut followed = 0;
    const MAX_FOLLOW: usize = 3;

    for word in &first_words {
        if followed >= MAX_FOLLOW {
            break;
        }
        let stemmed = match stem_to_entry(word, &dictionary.entry_set) {
            Some(s) => s,
            None => continue,
        };
        if structural.contains(&stemmed) || stemmed == current {
            continue;
        }
        followed += 1;

        if let Some(clause) = extract_condition_via_chain(
            &stemmed, action, dictionary, structural,
            max_hops - 1, visited, space,
        ) {
            return Some(clause);
        }
    }

    None
}
```

### Step 7: Wire into resolve_question()

Add the new match arms in `resolve_question()` after compound detection, inside the existing match on `question_type`:

```rust
Some(QuestionType::WhyIs { subject, object, connector }) => {
    let connector_str = connector.join(" ");
    let (answer, distance) = resolve_why(
        &subject, &object, dictionary, structural, space,
    );
    (answer, Some(distance), Some(connector_str))
}
Some(QuestionType::WhenIs { subject, action }) => {
    let (answer, distance) = resolve_when(
        &subject, &action, dictionary, structural, space,
    );
    (answer, Some(distance), Some(format!("when {} {}", subject, action)))
}
```

## TESTING

### Test File: dict5_2w_test.md

Create `dictionaries/dict5_2w_test.md` with 10 questions using dict5's vocabulary.

```markdown
# dict5_2w — When/Why Questions (10)

## WHY — definition chain as explanation

---

**Q01**: Why is a dog an animal?
**A**: because a dog is an animal
**Chain**: dog → animal (1-hop direct)

---

**Q02**: Why is a dog a thing?
**A**: because a dog is an animal, and an animal is a thing
**Chain**: dog → animal → thing (2-hop)

---

**Q03**: Why is a cat an animal?
**A**: because a cat is an animal
**Chain**: cat → "a small animal" → animal (1-hop)

---

**Q04**: Why is the sun hot?
**A**: because the sun is hot
**Chain**: sun → "a big hot thing" → hot (1-hop, property in first sentence)

---

**Q05**: Why is a person an animal?
**A**: because a person is an animal
**Chain**: person → "an animal that can make things" → animal (1-hop)

---

## WHEN — condition/purpose extraction

---

**Q06**: When does a person eat?
**A**: to feel good
**Chain**: person → animal → eat def → "you eat to feel good"

---

**Q07**: When does a dog eat?
**A**: to feel good
**Chain**: dog → animal → eat def → "you eat to feel good"

---

**Q08**: When is it cold?
**A**: I don't know
**Chain**: cold def → "you feel cold. not hot." → no temporal info

---

**Q09**: When does a dog move?
**A**: I don't know
**Chain**: dog def mentions "move" but no condition. move def → no temporal info.

---

**Q10**: When does a cat eat?
**A**: to feel good
**Chain**: cat → animal → eat def → "you eat to feel good"
```

**Scoring note**: `fuzzy_word_match` compares last words. Expected answers are structured so the last word is the key discriminator:
- Q01: expected "...animal", actual should end in "animal" → match
- Q02: expected "...thing", actual should end in "thing" → match
- Q04: expected "...hot", actual should end in "hot" → match
- Q06: expected "to feel good", actual should end in "good" → match
- Q08: IDK → IDK → match

### Test File: 2w_test.md (Three Men in a Boat)

Create `texts/three_men/2w_test.md` with 5 questions.

```markdown
# 2w_test — When/Why for Three Men in a Boat (5)

---

**Q01**: Why is Montmorency a dog?
**A**: because montmorency is a dog
**Chain**: montmorency → dog (entity definition, 1-hop)

---

**Q02**: Why is Harris a person?
**A**: because harris is a person
**Chain**: harris → person (entity definition, 1-hop)

---

**Q03**: Why is the Thames a river?
**A**: because the thames is a river
**Chain**: thames → river (entity definition, 1-hop)

---

**Q04**: Why is Montmorency an animal?
**A**: because montmorency is a dog, and a dog is an animal
**Chain**: montmorency → dog → animal (2-hop)

---

**Q05**: Why is Kingston a place?
**A**: because kingston is a place
**Chain**: kingston → place (entity definition, 1-hop)
```

### Run Order

```bash
# 1. dict5 When/Why test
cargo run -p dafhne-eval -- \
    --dict dictionaries/dict5.md \
    --test dictionaries/dict5_2w_test.md \
    --mode equilibrium
# Expected: ≥7/10 (When questions depend on condition extraction working)

# 2. Three Men When/Why test
cargo run -p dafhne-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/2w_test.md \
    --mode equilibrium
# Expected: 5/5

# 3-9. Regressions
cargo run -p dafhne-eval -- --dict dictionaries/dict5.md --test dictionaries/dict5_test.md --mode equilibrium
# Expected: 20/20

cargo run -p dafhne-eval -- --dict dictionaries/dict12.md --test dictionaries/dict12_test.md --mode equilibrium
# Expected: 14/20

cargo run -p dafhne-eval -- --text texts/passage1.md --cache-type ollama --cache dictionaries/cache/ollama-qwen3 --test texts/passage1_test.md --mode equilibrium
# Expected: 5/5

cargo run -p dafhne-eval -- --text texts/three_men/combined.md --entities texts/three_men_supplementary/entities.md --cache-type ollama --cache dictionaries/cache/ollama-qwen3 --test texts/three_men/full_test.md --mode equilibrium
# Expected: 19/21

cargo run -p dafhne-eval -- --text texts/three_men/combined.md --entities texts/three_men_supplementary/entities.md --cache-type ollama --cache dictionaries/cache/ollama-qwen3 --test texts/three_men/3w_test.md --mode equilibrium
# Expected: 10/10

cargo run -p dafhne-eval -- --dict dictionaries/dict5.md --test dictionaries/dict5_bool_test.md --mode equilibrium
# Expected: 9/10

cargo run -p dafhne-eval -- --text texts/three_men/combined.md --entities texts/three_men_supplementary/entities.md --cache-type ollama --cache dictionaries/cache/ollama-qwen3 --test texts/three_men/bool_test.md --mode equilibrium
# Expected: 5/5
```

## EXPECTED IMPACT

### dict5_2w_test predictions

| Q | Question | Expected | Why/When Logic | Confidence |
|---|----------|----------|----------------|------------|
| Q01 | Why dog animal? | because a dog is an animal | 1-hop direct | HIGH |
| Q02 | Why dog thing? | because dog→animal, animal→thing | 2-hop chain | MEDIUM |
| Q03 | Why cat animal? | because a cat is an animal | 1-hop | HIGH |
| Q04 | Why sun hot? | because the sun is hot | 1-hop property | MEDIUM |
| Q05 | Why person animal? | because a person is an animal | 1-hop | HIGH |
| Q06 | When person eat? | to feel good | eat def → purpose | MEDIUM |
| Q07 | When dog eat? | to feel good | chain to eat def | MEDIUM |
| Q08 | When cold? | IDK | no condition | HIGH |
| Q09 | When dog move? | IDK | no condition | HIGH |
| Q10 | When cat eat? | to feel good | chain to eat def | MEDIUM |

**Risk factors**:
- Q02: trace_chain_path must find dog→animal→thing path and format correctly
- Q04: "hot" is in the first sentence "a big hot thing" — trace_chain_path needs to find it
- Q06-Q07,Q10: extract_condition_clause must find "to feel good" in eat's definition, and the chain from subject→animal→eat must trigger the right extraction path

### Three Men 2w_test predictions

All 5 are simple 1-2 hop "why" chains through entity definitions. These should be reliable — entity definitions are clean and short.

### Regressions: zero expected

"why" and "when" tokens don't appear in any existing test questions. Adding new match arms in detect_question_type() doesn't affect existing paths.

## WHAT NOT TO DO

- Do NOT add temporal dimensions to the geometric space. Definitions are the source.
- Do NOT attempt full causal reasoning. Chain presentation is sufficient for Phase 14.
- Do NOT modify engine, equilibrium, or connector discovery.
- Do NOT change the Answer enum. Word(String) handles explanatory text.
- Do NOT change fuzzy_word_match. Structure expected answers so last-word matching works.
- Do NOT handle compound When/Why ("Why is X Y and Z?"). Out of scope.
- Do NOT try to answer "When" questions with dates or times. Condition/purpose extraction only.

## KNOWN LIMITATIONS

1. **"Why" is tautological for 1-hop**: "Why is a dog an animal?" → "because a dog is an animal." The definition IS the explanation. This is honest — DAFHNE knows what it was told, not deeper causal mechanisms.

2. **"When" rarely has answers in dict5**: Most dict5 definitions lack temporal/conditional clauses. The "to feel good" pattern in eat's definition is one of few extractable conditions. Most "when" questions correctly return IDK.

3. **No inverse causation**: "Why is ice cold?" requires knowing that ice IS frozen water. If "ice" isn't in dict5, this fails. The system can only explain what's in its definitions.

4. **"can" vs "is" detection**: build_chain_explanation() tries to detect whether the link uses "can" (capability) or "is" (category). This heuristic checks for "can {word}" in the definition. It may produce "is" for capability links if the pattern doesn't match exactly.

5. **Purpose clause ambiguity**: "you eat to feel good" → "to feel good" is a purpose, not a time. Strictly, "when" should return temporal info. But in ELI5 definitions, purpose IS the answer to "when" — "you eat WHEN you want to feel good" is implied.

## SUCCESS CRITERIA

| Metric | Expected |
|--------|----------|
| dict5_2w_test | ≥7/10 |
| 2w_test (Three Men) | 5/5 |
| dict5 regression | 20/20 |
| dict12 regression | 14/20 |
| passage1 regression | 5/5 |
| full_test regression | 19/21 |
| 3w_test regression | 10/10 |
| dict5_bool_test regression | 9/10 |
| bool_test regression | 5/5 |
| Code changes | resolver.rs only (new variants + detect + resolve functions) |

## OUTPUT CHECKLIST

1. ☐ `WhyIs` and `WhenIs` variants added to `QuestionType`
2. ☐ `detect_why_question()` implemented
3. ☐ `detect_when_question()` implemented
4. ☐ `resolve_why()` + `trace_chain_path()` + `build_chain_explanation()` implemented
5. ☐ `resolve_when()` + `extract_condition_clause()` + helpers implemented
6. ☐ `resolve_question()` match arms wired
7. ☐ `dict5_2w_test.md` created (10 questions)
8. ☐ `2w_test.md` created (5 questions, Three Men)
9. ☐ dict5_2w_test results (expect ≥7/10)
10. ☐ 2w_test results (expect 5/5)
11. ☐ All 7 regression tests pass
12. ☐ RECAP.md updated with Phase 14 results
