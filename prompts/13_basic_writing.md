# PROMPT 13 — Basic Writing: Geometric Expression

## PREAMBLE

Phase 12 achieved boolean compound queries (9/10 dict5_bool, 5/5 Three Men bool) with zero regressions. YALM now comprehends: Yes/No, What/Who/Where, boolean AND/OR, negation, transitive chains up to 3 hops, and entity definitions.

Phase 13 flips the direction: from **comprehension** to **generation**. The system reads its own geometry and definitions to produce descriptive sentences about a word. This is YALM expressing what it knows.

## GOAL

Add a `--describe` CLI mode to yalm-eval that generates natural-language descriptions of words by traversing definitions and geometric neighborhoods.

```
cargo run -p yalm-eval -- --dict dictionaries/dict5.md --describe dog,cat,sun
```

Output:
```
=== Describe: dog ===
a dog is an animal.
a dog can make sound.
a dog can live with a person.
a dog is not a cat.
a dog is not a person.
```

The description is built from three sources:
1. **Definition sentences** — the entry's own definition text (ground truth)
2. **Definition-chain negation** — sibling words that the chain cannot connect (inferred)
3. **Self-consistency verification** — feed description back as questions, verify round-trip

## WHY NOT GEOMETRIC NEAREST NEIGHBORS?

The stub suggested using geometric proximity for generation. After 12 phases of evidence, this is wrong:

- Geometric proximity gives **similarity**, not **identity**. Dog is near cat (both animals), but "a dog is a cat" is false.
- Definition-chain traversal is the reliable path for both comprehension (Yes/No) and generation.
- The geometry's value is in the **distance thresholds** (Yes/No/IDK zones), not in producing prose.

Phase 13 generates from **definitions**, not from **distances**. The geometry is used only for one thing: validating the generated sentences via self-consistency queries.

## ARCHITECTURE

### Overview

```
Input: word + dictionary + space
   │
   ├─ Step 1: Category extraction (definition_category)
   │   → "a dog is an animal."
   │
   ├─ Step 2: Definition sentence rewriting
   │   → "a dog can make sound."
   │   → "a dog can live with a person."
   │
   ├─ Step 3: Negation inference (definition_chain_check)
   │   → "a dog is not a cat."
   │   → "a dog is not a person."
   │
   └─ Output: Vec<String> of sentences
```

### Where code lives

- **New function `describe()`** in `resolver.rs` — generation logic
- **New function `describe_entry()`** in `resolver.rs` — single-word description
- **New CLI flag `--describe`** in `yalm-eval/main.rs` — invocation
- **No changes** to engine, parser, core, equilibrium, connectors

## IMPLEMENTATION

### Step 1: Add `describe()` to resolver.rs

```rust
/// Generate a natural-language description of a word by reading its
/// definition and inferring negations from definition-chain failures.
///
/// Returns a Vec of simple sentences describing the word.
pub fn describe(
    subject: &str,
    space: &GeometricSpace,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    content: &HashSet<String>,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> Vec<String> {
    let entry = match dictionary.entries.iter().find(|e| e.word == subject) {
        Some(e) => e,
        None => return vec![format!("I don't know what {} is.", subject)],
    };

    let mut sentences: Vec<String> = Vec::new();

    // ── 1. Category sentence ──────────────────────────────────
    // Extract the category from the first sentence of the definition.
    // Reuse definition_category() logic but construct a full sentence.
    let category = definition_category(subject, dictionary, space, structural);
    if let Some(ref cat) = category {
        let article_subject = make_article(subject, dictionary);
        let article_cat = if cat.starts_with(|c: char| "aeiou".contains(c)) { "an" } else { "a" };
        sentences.push(format!("{} is {} {}.", article_subject, article_cat, cat));
    }

    // ── 2. Definition sentence rewriting ──────────────────────
    // Split definition into sentences. Skip the first sentence
    // (already captured as category). Rewrite remaining sentences
    // by replacing pronoun subjects ("it", "you") with the word.
    let def_sentences: Vec<&str> = entry.definition
        .split('.')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let article_subject = make_article(subject, dictionary);

    for (i, sentence) in def_sentences.iter().enumerate() {
        if i == 0 {
            continue; // category already extracted
        }

        let lower = sentence.to_lowercase();
        let tokens = tokenize(&lower);
        if tokens.is_empty() {
            continue;
        }

        // Skip sentences that start with "you" — these describe
        // the observer, not the subject ("you can see it").
        if tokens[0] == "you" {
            continue;
        }

        // Rewrite: replace leading "it" with the article+subject.
        // "it can make sound" → "a dog can make sound"
        let rewritten = if tokens[0] == "it" {
            format!("{} {}", article_subject, tokens[1..].join(" "))
        } else {
            // Sentence doesn't start with a pronoun — prepend subject.
            // "not a plant" → skip (fragment)
            // "an animal eats" → skip (doesn't describe subject)
            // Generally, non-pronoun sentences in ELI5 defs are rare.
            // Skip them rather than risk incorrect attribution.
            continue;
        };

        // Clean up: ensure sentence ends with period
        let cleaned = format!("{}.", rewritten.trim_end_matches('.'));
        sentences.push(cleaned);
    }

    // ── 3. Negation inference ─────────────────────────────────
    // Find sibling words: words with the same category as the subject.
    // For each sibling, check if the definition chain connects them.
    // If not → "X is not Y."
    if let Some(ref cat) = category {
        let siblings = find_siblings(subject, cat, dictionary, space, structural);
        for sibling in &siblings {
            let mut visited = HashSet::new();
            let chain = definition_chain_check(
                subject, sibling, dictionary, structural, 3, &mut visited, space,
            );
            match chain {
                Some(true) => {} // linked — don't negate
                _ => {
                    // Not linked or explicitly negated → "X is not Y"
                    let article_sib = if sibling.starts_with(|c: char| "aeiou".contains(c)) {
                        "an"
                    } else {
                        "a"
                    };
                    sentences.push(format!(
                        "{} is not {} {}.",
                        article_subject, article_sib, sibling
                    ));
                }
            }
        }
    }

    sentences
}

/// Find words that share the same definition category as the subject.
/// Returns content words whose definition_category matches `category`,
/// excluding the subject itself. Limited to 5 siblings to keep output concise.
fn find_siblings(
    subject: &str,
    category: &str,
    dictionary: &Dictionary,
    space: &GeometricSpace,
    structural: &HashSet<String>,
) -> Vec<String> {
    let mut siblings = Vec::new();
    for entry in &dictionary.entries {
        if entry.word == subject || entry.word == category {
            continue;
        }
        if let Some(cat) = definition_category(&entry.word, dictionary, space, structural) {
            if cat == category {
                siblings.push(entry.word.clone());
            }
        }
        if siblings.len() >= 5 {
            break;
        }
    }
    siblings
}

/// Generate the appropriate article + word for sentence construction.
/// Entities get bare names: "montmorency", "harris".
/// Regular nouns get articles: "a dog", "the sun".
///
/// Heuristic for "a" vs "the":
/// - Unique/singular nouns defined with "the" → use "the" (sun, thames)
/// - General nouns → use "a"/"an"
fn make_article(word: &str, dictionary: &Dictionary) -> String {
    let entry = dictionary.entries.iter().find(|e| e.word == word);

    // Entity entries get bare names
    if let Some(e) = entry {
        if e.is_entity {
            return word.to_string();
        }
    }

    // Check if definition starts with "the" → unique noun
    if let Some(e) = entry {
        let first_word = tokenize(&e.definition).into_iter().next().unwrap_or_default();
        if first_word == "the" {
            return format!("the {}", word);
        }
    }

    // Default: "a"/"an" + word
    let article = if word.starts_with(|c: char| "aeiou".contains(c)) { "an" } else { "a" };
    format!("{} {}", article, word)
}
```

### Step 2: Make `definition_category()` and `definition_chain_check()` public

Both functions are currently private in resolver.rs. `describe()` needs `definition_chain_check()` which is already in the same module, so no visibility change needed — just ensure `describe()` can call them (same module = fine).

However, `describe()` itself needs to be `pub` so yalm-eval can call it:

```rust
// Already in resolver.rs, just add pub:
pub fn describe(...) -> Vec<String> { ... }
```

### Step 3: Add `--describe` flag to yalm-eval CLI

In `yalm-eval/src/main.rs`, add to the `Cli` struct:

```rust
/// Describe words: generate natural-language descriptions (comma-separated)
#[arg(long)]
describe: Option<String>,

/// Run self-consistency test on describe output
#[arg(long)]
describe_verify: bool,
```

Add handling after the engine is trained (before test evaluation):

```rust
// ── Describe mode ─────────────────────────────────────────────
if let Some(ref words) = cli.describe {
    let word_list: Vec<&str> = words.split(',').map(|w| w.trim()).collect();
    println!("\n=== Describe Mode ===");

    for word in &word_list {
        let sentences = yalm_engine::resolver::describe(
            word,
            engine.space(),
            &dictionary,
            engine.structural(),
            engine.content(),
            &params,
            &strategy,
        );
        println!("\n--- {} ---", word);
        for s in &sentences {
            println!("  {}", s);
        }

        // Self-consistency verification
        if cli.describe_verify {
            println!("  [verify]");
            // For each generated sentence, construct a Yes/No question
            // and verify the system agrees with its own output.
            let mut pass = 0;
            let mut fail = 0;
            for s in &sentences {
                let question = sentence_to_question(s);
                if let Some(ref q) = question {
                    let (answer, dist, _) = yalm_engine::resolver::resolve_question(
                        q,
                        engine.space(),
                        &dictionary,
                        engine.structural(),
                        engine.content(),
                        &params,
                        &strategy,
                    );
                    // Category sentences ("X is a Y") should resolve to Yes.
                    // Negation sentences ("X is not Y") should resolve to No.
                    let is_negation = s.contains("is not");
                    let expected = if is_negation { Answer::No } else { Answer::Yes };
                    let ok = answer == expected;
                    let status = if ok { "✓" } else { "✗" };
                    println!(
                        "    {} {} → {} (dist: {:.4}) [expected: {}]",
                        status, q, answer,
                        dist.unwrap_or(f64::NAN),
                        expected
                    );
                    if ok { pass += 1; } else { fail += 1; }
                }
            }
            println!("  Consistency: {}/{} pass", pass, pass + fail);
        }
    }

    if cli.describe.is_some() && cli.test == PathBuf::from("dictionaries/dict5_test.md")
        && cli.text.is_none() && !word_list.is_empty() {
        // If describe-only run (no explicit --test override), skip evaluation
        return;
    }
}
```

### Step 4: `sentence_to_question()` helper in yalm-eval

Converts a generated sentence back into a Yes/No question for verification.

```rust
/// Convert a descriptive sentence into a Yes/No question for verification.
///
/// "a dog is an animal." → "Is a dog an animal?"
/// "a dog can make sound." → "Can a dog make sound?"
/// "a dog is not a cat." → "Is a dog not a cat?"
/// "a dog can live with a person." → "Can a dog live with a person?"
fn sentence_to_question(sentence: &str) -> Option<String> {
    let s = sentence.trim().trim_end_matches('.');
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.len() < 3 {
        return None;
    }

    // Find the verb: "is", "can", "has", "does"
    // Pattern: [subject...] [verb] [rest...]
    // "a dog is an animal" → verb at position 2
    // "the sun is big" → verb at position 2
    // "a dog can make sound" → verb at position 2
    // "montmorency is a dog" → verb at position 1
    let verbs = ["is", "can", "has", "does"];
    let verb_pos = words.iter().position(|w| verbs.contains(w))?;

    if verb_pos == 0 {
        return None; // sentence starts with verb — can't restructure
    }

    let subject_part = words[..verb_pos].join(" ");
    let verb = words[verb_pos];
    let rest = words[verb_pos + 1..].join(" ");

    // Construct question: "Is/Can [subject] [rest]?"
    let q_verb = match verb {
        "is" => "Is",
        "can" => "Can",
        "has" => "Has",
        "does" => "Does",
        _ => return None,
    };

    Some(format!("{} {} {}?", q_verb, subject_part, rest))
}
```

## TESTING

### Test 1: dict5 describe (closed mode)

```bash
cargo run -p yalm-eval -- \
    --dict dictionaries/dict5.md \
    --describe dog,cat,sun,person,animal \
    --describe-verify \
    --mode equilibrium
```

Expected output for `dog`:
```
--- dog ---
  a dog is an animal.
  a dog can make sound.
  a dog can live with a person.
  a dog is not a cat.
  a dog is not a person.
  [verify]
    ✓ Is a dog an animal? → Yes (dist: 0.XXXX) [expected: Yes]
    ✓ Can a dog make sound? → Yes (dist: 0.XXXX) [expected: Yes]
    ✓ Can a dog live with a person? → Yes (dist: 0.XXXX) [expected: Yes]
    ✓ Is a dog not a cat? → No (dist: 0.XXXX) [expected: No]
    ✓ Is a dog not a person? → No (dist: 0.XXXX) [expected: No]
  Consistency: 5/5 pass
```

**Note on negation verification**: "Is a dog not a cat?" is a negated question. The `resolve_yes_no()` function handles negation by inverting thresholds. The expected answer for a true negation ("X is NOT Y" where X really isn't Y) is **No** through the resolver's lens — wait, that's wrong.

Let me re-check: "Is a dog not a cat?" → negated=true, subject=dog, object=cat.
- Chain check: dog→cat = None (no chain link) → both in dict → cat is noun → returns No.
- But wait, the chain check fires before negation is applied. The negated flag only affects the geometric threshold inversion.
- Actually, re-reading `resolve_yes_no()`: the chain gate only fires when `!negated`. For negated questions, the geometric pipeline handles it via threshold inversion.

So: "Is a dog not a cat?" → negated=true → chain gate skipped → geometric answer.
Geometry: dog↔cat distance. In dict5 with equilibrium, dog and cat are close (both animals). The geometric negated answer would be: distance < yes_threshold → Answer::No (close = not-different = negation is false).

This means the verification for negation sentences is tricky. The system would need to answer "Yes" to "Is a dog not a cat?" (yes, a dog is indeed not a cat), but the geometric negation model returns No for close words.

**Resolution**: Skip negation sentences in self-consistency verification. The negation model for geometric queries has known limitations (Phase 10 findings). Only verify positive sentences (category + capability).

**Updated verify logic**:
```rust
if s.contains("is not") {
    println!("    - {} [negation — skipped]", s);
    continue;
}
```

### Test 2: Three Men describe (open mode + entities)

```bash
cargo run -p yalm-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --describe montmorency,harris,thames,kingston \
    --describe-verify \
    --mode equilibrium
```

Expected output for `montmorency`:
```
--- montmorency ---
  montmorency is a dog.
  montmorency is not a person.
  montmorency is not a river.
  [verify]
    ✓ Is montmorency a dog? → Yes (dist: 0.XXXX) [expected: Yes]
  Consistency: 1/1 pass
```

Entity definitions are short ("a dog. he is a man's best friend."), so fewer capability sentences. The negation inference adds value: montmorency is not a person, not a river.

Expected for `harris`:
```
--- harris ---
  harris is a person.
  harris is not a dog.
  harris is not a river.
  [verify]
    ✓ Is harris a person? → Yes (dist: 0.XXXX) [expected: Yes]
  Consistency: 1/1 pass
```

### Test 3: Regressions

Describe mode is additive — when `--describe` is not specified, behavior is identical to Phase 12. Run existing tests to confirm:

```bash
# dict5
cargo run -p yalm-eval -- --dict dictionaries/dict5.md --test dictionaries/dict5_test.md --mode equilibrium
# Expected: 20/20

# dict12
cargo run -p yalm-eval -- --dict dictionaries/dict12.md --test dictionaries/dict12_test.md --mode equilibrium
# Expected: 14/20

# passage1
cargo run -p yalm-eval -- --text texts/passage1.md --cache-type ollama --cache dictionaries/cache/ollama-qwen3 --test texts/passage1_test.md --mode equilibrium
# Expected: 5/5

# full_test
cargo run -p yalm-eval -- --text texts/three_men/combined.md --entities texts/three_men_supplementary/entities.md --cache-type ollama --cache dictionaries/cache/ollama-qwen3 --test texts/three_men/full_test.md --mode equilibrium
# Expected: 19/21

# 3w_test
cargo run -p yalm-eval -- --text texts/three_men/combined.md --entities texts/three_men_supplementary/entities.md --cache-type ollama --cache dictionaries/cache/ollama-qwen3 --test texts/three_men/3w_test.md --mode equilibrium
# Expected: 10/10

# dict5_bool_test
cargo run -p yalm-eval -- --dict dictionaries/dict5.md --test dictionaries/dict5_bool_test.md --mode equilibrium
# Expected: 9/10

# bool_test
cargo run -p yalm-eval -- --text texts/three_men/combined.md --entities texts/three_men_supplementary/entities.md --cache-type ollama --cache dictionaries/cache/ollama-qwen3 --test texts/three_men/bool_test.md --mode equilibrium
# Expected: 5/5
```

## EXPECTED OUTPUT

### dict5 descriptions (5 words)

| Word | Category | Capabilities | Negations |
|------|----------|-------------|-----------|
| dog | animal | make sound, live with person | not cat, not person |
| cat | animal | move with not-sound, live with person | not dog, not person |
| sun | thing ("the sun") | — ("you" sentences skipped) | depends on siblings |
| person | animal | make things, give names | not dog, not cat |
| animal | thing | move, eat, feel | — |

**Note on "sun"**: The sun's definition is "a big hot thing that is up. you can see it. you can feel it. it makes things hot." Sentences 2-3 start with "you" → skipped. Sentence 4 starts with "it" → "the sun makes things hot." Sentence 1 is "a big hot thing that is up" → category = "thing". The properties "big", "hot", "up" are in the first sentence but extracting them requires parsing beyond just category. This is a known limitation — the description will show category + sentence 4 + negations.

### Self-consistency expectations

Category sentences: **100% pass** — "Is a dog an animal?" already passes in dict5_test.

Capability sentences: **~80% pass** — "Can a dog make sound?" may depend on whether the chain dog→sound resolves. Some multi-word predicates ("live with a person") may not parse correctly as questions.

Negation sentences: **skipped** — known geometric limitation.

## WHAT NOT TO DO

- Do NOT use geometric nearest neighbors for sentence generation. Definitions are ground truth.
- Do NOT add new `Answer` variants. `describe()` returns `Vec<String>`, not `Answer`.
- Do NOT add a new `QuestionType::Describe`. This is a CLI mode, not a question type.
- Do NOT modify engine, equilibrium, force field, or connector discovery.
- Do NOT modify existing test scoring or evaluation logic.
- Do NOT try to generate sentences for words not in the dictionary.
- Do NOT attempt complex NLP for sentence rewriting — simple pronoun replacement is sufficient for ELI5 definitions.

## KNOWN LIMITATIONS

1. **"you" sentences lost**: Definitions like "you can see it" describe observer actions, not subject properties. These are skipped. The system cannot invert perspective ("it can be seen") without passive voice generation.

2. **First-sentence properties lost**: "a big hot thing that is up" has properties (big, hot, up) embedded in the category sentence. Only "thing" is extracted as category. Property extraction from the first sentence is deferred.

3. **Negation verification skipped**: The geometric negation model doesn't reliably answer "Is X not Y?" when X and Y are geometrically close (same category). Negation sentences are generated but not verified.

4. **Multi-word predicates in verification**: "Can a dog live with a person?" has a complex predicate. The resolver may not parse this correctly for verification. Verification focuses on simple sentences.

5. **Entity descriptions are short**: Entity definitions ("a dog. he is a man's best friend.") produce few capability sentences. Negation inference adds the most value for entities.

## SUCCESS CRITERIA

| Metric | Expected |
|--------|----------|
| dict5 describe: 5 words produce non-empty output | ✅ |
| dict5 describe: category correct for dog, cat, person, animal | ≥4/4 |
| dict5 describe: ≥2 capability sentences for dog | ✅ |
| dict5 describe: ≥1 negation sentence for dog | ✅ |
| Three Men describe: entity category correct for montmorency, harris, thames | 3/3 |
| Self-consistency: ≥80% of positive sentences verify | ✅ |
| dict5 regression | 20/20 |
| dict12 regression | 14/20 |
| passage1 regression | 5/5 |
| full_test regression | 19/21 |
| 3w_test regression | 10/10 |
| dict5_bool_test regression | 9/10 |
| bool_test regression | 5/5 |
| Code changes | resolver.rs (describe + helpers) + main.rs (CLI flag) only |

## OUTPUT CHECKLIST

1. ☐ `describe()` function in resolver.rs
2. ☐ `find_siblings()` helper in resolver.rs
3. ☐ `make_article()` helper in resolver.rs
4. ☐ `--describe` and `--describe-verify` CLI flags in yalm-eval
5. ☐ `sentence_to_question()` helper in yalm-eval
6. ☐ dict5 describe output for dog, cat, sun, person, animal
7. ☐ Three Men describe output for montmorency, harris, thames, kingston
8. ☐ Self-consistency verification results
9. ☐ All 7 regression tests pass
10. ☐ RECAP.md updated with Phase 13 results
