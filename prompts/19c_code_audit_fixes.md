# PROMPT 19c — Code Audit Fixes

> **STATUS: Implementation phase. Fix findings from `reports/19b_code_audit.md`.**

## CONTEXT

Phase 19b produced a 24-finding code audit. This prompt fixes most of them, organized into batches by theme. The goal is to bring YALM closer to its founding principles without breaking anything.

Project location: `D:\workspace\projects\yalm`

Use the filesystem MCP with path `D:\workspace\projects\yalm` to access files.

## PREREQUISITE READING

Read these before writing any code:
1. `reports/19b_code_audit.md` — the full audit (24 findings, severity ratings, recommendations)
2. `crates/yalm-engine/src/connector_discovery.rs` — understand `classify_word_roles()` and what it already discovers
3. `crates/yalm-engine/src/resolver.rs` — the largest file, most fixes land here
4. `crates/yalm-engine/src/multispace.rs` — structural word list and TASK routing
5. `crates/yalm-core/src/lib.rs` — `EngineParams` struct (constants to add)
6. `crates/yalm-engine/src/strategy.rs` — `StrategyConfig` struct
7. `crates/yalm-engine/src/force_field.rs` — negation model implementations

## WHAT TO FIX (and what NOT to fix)

### ❌ SKIP — Math-related (will be reworked separately)
- **A04**: Hardcoded Number-to-Word Mapping — leave as-is, math space redesign later
- **A14** (math indicators only): `math_indicators` list in TASK routing — leave math routing alone

### ❌ SKIP — Accepted architectural decisions
- **A06**: Definition Chain Check — accepted hybrid (geometry + symbols). The finding IS the research result.
- **A07**: Definition Category Extraction — accepted pragmatic. ELI5 definitions guarantee the format.
- **A08**: Why/When Resolution — irreducibly symbolic. Geometry can't explain WHY.
- **A22**: Entity Fast Path — clean solution for a real problem. Keep as-is.

### ❌ SKIP — Separate phase
- **A21**: Phase 15 gap — separate implementation (property extraction), not a fix

---

### ✅ BATCH 1: Replace Hardcoded Word Lists with Discovered Data
**Findings**: A02, A03, A20, A24
**Theme**: YALM's connector discovery already identifies structural words. Use that data instead of hardcoded English lists.

#### The Key Insight
`connector_discovery.rs` already has `classify_word_roles()` which computes document frequency for every word and classifies high-frequency words as structural. The result is available at space-build time. The hardcoded lists in resolver.rs and multispace.rs are shortcuts to information the system already discovers.

#### Changes

**Step 1: Expose structural words from Engine**

The `Engine` (in `lib.rs`) already holds a `GeometricSpace` which has `connectors`. But the structural word classification from `classify_word_roles()` is not stored — it's computed and discarded during connector discovery.

Fix: Store the structural word set on `GeometricSpace`:
```rust
// In yalm-core/src/lib.rs, add to GeometricSpace:
pub struct GeometricSpace {
    // ... existing fields ...
    /// Words classified as structural by connector discovery (high document frequency).
    /// These are function words like articles, prepositions, auxiliaries.
    /// Discovered from text, not hardcoded.
    #[serde(default)]
    pub structural_words: HashSet<String>,
}
```

In `connector_discovery.rs`, `connector_pipeline()` (or `discover_connectors()` / `discover_connectors_from_sentences()`): after `classify_word_roles()`, store the structural words in the returned data. The simplest approach: return a `(Vec<Connector>, HashSet<String>)` tuple, or add a method that exposes the structural set.

Then in `lib.rs` where `Engine::train()` builds the space, populate `structural_words` from the connector discovery output.

**Step 2: MultiSpace `is_structural()` uses discovered words (A03)**

Replace the hardcoded ~30-word list in `multispace.rs`:
```rust
// BEFORE (hardcoded):
fn is_structural(word: &str) -> bool {
    static STRUCTURAL: &[&str] = &["is", "a", "an", "the", ...];
    STRUCTURAL.contains(&word)
}

// AFTER (from spaces):
impl MultiSpace {
    fn is_structural(&self, word: &str) -> bool {
        self.structural_words_cache.contains(word)
    }
}
```

At `MultiSpace` construction time, compute the union of structural words from all constituent spaces:
```rust
let mut structural_words_cache = HashSet::new();
for space in &spaces {
    structural_words_cache.extend(space.engine.space().structural_words.iter().cloned());
}
```

**Step 3: Resolver uses discovered structural words (A02, A24)**

The resolver currently has hardcoded article lists and skip_words sets scattered across functions. Fix:

Pass the structural word set into the resolver. The `Engine` already holds the space, so the resolver can access `self.space.structural_words`. Replace every instance of:
```rust
let articles: HashSet<&str> = ["a", "an", "the"].iter().copied().collect();
```
with:
```rust
let structural = &self.space.structural_words;
```

Specific locations to fix:
- `definition_category()` — skip structural words instead of hardcoded articles
- `resolve_what_is()` — same
- `describe()` / `make_article()` — partially (article selection still needs "a"/"an"/"the" for generation, but detection uses discovered set)
- `detect_what_question()` — skip structural words
- `detect_why_question()` — skip structural words
- `detect_when_question()` — skip structural words
- Various `skip_words` sets in question parsing

**Important nuance**: For *output generation* (make_article, describe sentence templates), the system needs to produce "a dog" or "the sun". This IS English-specific output formatting. The fix is for *input parsing* — don't assume which words are structural, discover them.

**Step 4: `preceded_by_not()` uses discovered connectors (A20)**

Currently checks for literal "not". Replace with:
```rust
fn preceded_by_negation(&self, tokens: &[&str], idx: usize) -> bool {
    if idx == 0 { return false; }
    let prev = tokens[idx - 1];
    // Only check for negation if we discovered a negation connector
    self.space.connectors.iter().any(|c| c.pattern == vec![prev.to_string()] && prev == "not")
    // More general: any single-word connector that appears as negation
    // For now, "not" is the only negation connector YALM discovers
}
```

The simplest correct fix: check if "not" is a discovered connector pattern, and only use the negation check if it is. This makes the code self-documenting: "I only check for negation if the text taught me that negation exists."

---

### ✅ BATCH 2: Move Constants to EngineParams
**Findings**: A10, A11, A12
**Theme**: Hand-tuned constants should be evolvable parameters.

#### Changes

Add to `EngineParams` in `yalm-core/src/lib.rs`:

```rust
/// Maximum content words to follow per hop in definition chain traversal.
/// Default: 3. Higher values find longer chains but risk false positives.
#[serde(default = "default_max_follow_per_hop")]
pub max_follow_per_hop: usize,

/// Maximum hops in definition chain traversal.
/// Default: 3. Higher values enable deeper transitive reasoning.
#[serde(default = "default_max_hops")]
pub max_hops: usize,

/// Connector axis emphasis in nearest-neighbor search (0.0-1.0).
/// Default: 0.2. Higher values weight connector-axis alignment more.
#[serde(default = "default_connector_axis_alpha")]
pub connector_axis_alpha: f64,

/// Number of alphabetical buckets for uniformity filter.
/// Default: 10.
#[serde(default = "default_uniformity_buckets")]
pub uniformity_buckets: usize,

/// Uniformity threshold (coefficient of variation) for connector filtering.
/// Default: 0.75. Lower values are stricter.
#[serde(default = "default_uniformity_threshold")]
pub uniformity_threshold: f64,
```

With default functions preserving current behavior:
```rust
fn default_max_follow_per_hop() -> usize { 3 }
fn default_max_hops() -> usize { 3 }
fn default_connector_axis_alpha() -> f64 { 0.2 }
fn default_uniformity_buckets() -> usize { 10 }
fn default_uniformity_threshold() -> f64 { 0.75 }
```

Then update `resolver.rs` and `connector_discovery.rs` to read from `self.params` instead of using hardcoded values. **All existing genomes remain compatible** because `#[serde(default)]` fills in the new fields on deserialization.

---

### ✅ BATCH 3: Clean Up Heuristics
**Findings**: A09, A05

#### A09: `is_property_word()` and `is_connector_word()`

`is_connector_word()` already partially checks the discovered connector set. Make it fully derived:
```rust
fn is_connector_word(&self, word: &str) -> bool {
    self.space.connectors.iter().any(|c| c.pattern.contains(&word.to_string()))
}
```

`is_property_word()` currently checks definition-shape heuristics like "starts with 'a way to'" or "starts with 'having'". These are English patterns. The cleaner approach: a word is a property word if its definition starts with a structural word followed by another structural word (pattern: function-word + function-word + ...), suggesting it's defining a modifier or abstract concept rather than a concrete noun. But this requires care — don't break existing behavior.

**Conservative fix for A09**: Replace the hardcoded string patterns in `is_property_word()` with checks against the structural word set where possible. Keep the function but make it derive patterns from discovered data rather than hardcoded strings. If some patterns can't be derived (e.g., "having" detection), document them as accepted pragmatic English assumptions.

#### A05: SELF-Space Triggers

Currently hardcoded: `self_triggers = ["yalm"]` and `self_patterns = [("are", "you"), ...]`.

Fix: At `MultiSpace` construction, extract self-triggers from the SELF dictionary:
```rust
// Words in SELF that don't appear in any other space = unique self-triggers
let self_unique: HashSet<String> = self_dict_words
    .difference(&all_other_space_words)
    .cloned()
    .collect();
```

For pronoun patterns ("are you", "can you", "do you"): check if "you" is defined in the SELF dictionary. If yes, queries containing "you" get routed to SELF. This replaces the hardcoded pattern list with a dictionary-derived rule.

---

### ✅ BATCH 4: Architecture Cleanup
**Findings**: A01, A13, A14 (non-math), A15, A23

#### A01 + A13: Question-Type Detection

The ideal fix (move parser to interface layer) is too large for this phase. Instead, do a **partial fix**:

Make question verbs derived from discovered connectors + structural words:
```rust
fn detect_question_type(&self, question: &str) -> QuestionType {
    let tokens: Vec<&str> = question.split_whitespace().collect();
    if tokens.is_empty() { return QuestionType::Unknown; }
    
    let first = tokens[0].to_lowercase();
    
    // 5W detection: these ARE hardcoded English (accepted — interface layer)
    // But document clearly that this is the language-specific layer
    match first.as_str() {
        "what" | "who" | "where" => /* ... */,
        "when" => /* ... */,
        "why" => /* ... */,
        // For Yes/No: check if first word is a discovered connector or structural word
        _ if self.is_question_verb(&first) => /* Yes/No detection */,
        _ => QuestionType::Unknown,
    }
}

fn is_question_verb(&self, word: &str) -> bool {
    // A word is a question verb if it's structural (high frequency)
    // AND appears as the first token in connector patterns
    self.space.structural_words.contains(word)
}
```

This replaces the hardcoded `["is", "can", "does", "do", "has"]` with a check against discovered structural words. Not perfect (not all structural words are question verbs), but better than a hardcoded list.

Add a clear comment block at the top of question detection:
```rust
// ═════════════════════════════════════════════════════════════
// LANGUAGE-SPECIFIC LAYER
// The 5W question words (what, who, where, when, why) are
// hardcoded English. This is accepted as interface-layer code.
// A multilingual YALM would replace this function.
// The yes/no detection uses discovered structural words.
// ═════════════════════════════════════════════════════════════
```

#### A14: Task Routing Indicators (non-math)

Remove the `grammar_indicators` and `content_indicators` fallback lists. Keep ONLY geometric routing for grammar and content spaces. The TASK space should route by distance only.

For `math_indicators`: **leave as-is** per instructions (math rework later).

```rust
// BEFORE:
if math_indicators.iter().any(|i| query_lower.contains(i)) { return "math"; }
if grammar_indicators.iter().any(|i| query_lower.contains(i)) { return "grammar"; }
if content_indicators.iter().any(|i| query_lower.contains(i)) { return "content"; }
// geometric fallback...

// AFTER:
if math_indicators.iter().any(|i| query_lower.contains(i)) { return "math"; }
// For grammar and content: geometric routing ONLY
// (math indicators kept until math space rework)
let task_distances = self.compute_task_distances(query);
// route to closest space by geometry
```

#### A15: Negation Models — Document the Negative Result

Don't remove the code (it documents research). Add comments:
```rust
/// Four negation models were implemented and tested across 100+ evolution
/// generations. None successfully handles negation through geometry alone.
/// The winning strategy is definition-chain negation (resolve_yes_no gate).
///
/// These variants are preserved as research documentation.
/// Evolution consistently selects Inversion with negligible impact,
/// as the chain gate handles all negation before geometric negation fires.
///
/// See: reports/19b_code_audit.md, finding A15
pub enum NegationModel {
    /// Inverts force direction for negated relations. The most commonly
    /// selected variant, but has no measurable impact on accuracy.
    Inversion,
    // ... etc
}
```

#### A23: `find_siblings()` — Low-Priority Improvement

Add a TODO comment noting the geometric alternative:
```rust
/// Find words sharing the same definition category as `word`.
/// Currently uses string comparison of definition_category() output.
///
/// TODO (geometric alternative): Find k-nearest neighbors in the space,
/// then filter by shared connector direction. This would discover siblings
/// that definitions don't explicitly mark.
fn find_siblings(&self, word: &str) -> Vec<String> {
```

No code change — just documentation.

---

## IMPLEMENTATION ORDER

1. **Batch 2 first** (EngineParams constants) — smallest, most mechanical, no behavior change
2. **Batch 1** (structural words) — the big one, multiple files
3. **Batch 3** (heuristics cleanup)
4. **Batch 4** (architecture cleanup, documentation)

## TESTING

After ALL fixes, run the full regression suite:

```bash
# Single-space dict5
cargo run --release -p yalm-eval

# Single-space dict12
cargo run --release -p yalm-eval -- --dict dictionaries/dict12.md --test dictionaries/dict12_test.md --grammar dictionaries/grammar18.md --genome results_v11/best_genome.json

# Multi-space unified test (45/50 baseline)
cargo run --release -p yalm-eval -- \
  --spaces content:dictionaries/dict5.md,math:dictionaries/dict_math5.md,grammar:dictionaries/dict_grammar5.md,task:dictionaries/dict_task5.md,self:dictionaries/dict_self5.md \
  --test dictionaries/unified_test.md \
  --genome results_v11/best_genome.json

# Bootstrap (should still find connectors and converge)
cargo run --release -p yalm-eval -- \
  --spaces content:dictionaries/dict5.md,math:dictionaries/dict_math5.md,grammar:dictionaries/dict_grammar5.md,task:dictionaries/dict_task5.md,self:dictionaries/dict_self5.md \
  --test dictionaries/unified_test.md \
  --genome results_v11/best_genome.json \
  --bootstrap 3
```

### Regression Targets (MUST NOT DROP)

| Test Suite | Baseline | Minimum |
|------------|----------|---------|
| dict5 | 20/20 | 20/20 |
| dict12 | 14/20 | 14/20 |
| unified_test (multi-space) | 45/50 | 44/50 |
| Bootstrap convergence | Level 2 | Level 3 |
| Bootstrap new connectors | 4 | ≥2 |

**One-point regression on unified_test is acceptable** if caused by removing TASK indicator fallbacks (A14). If regression is >1 point, revert that specific change and investigate.

---

## WHAT NOT TO DO

- Do NOT touch math space routing (A04, math part of A14)
- Do NOT touch definition_chain_check logic (A06)
- Do NOT touch definition_category first-word extraction logic (A07)
- Do NOT touch Why/When symbolic operations (A08)
- Do NOT touch entity fast path (A22)
- Do NOT implement Phase 15 property extraction (A21) — separate phase
- Do NOT refactor resolver into separate interface layer (A13 full fix) — too large
- Do NOT create new test files or dictionaries
- Do NOT change any dictionary content

## SUCCESS CRITERIA

| Metric | Target |
|--------|--------|
| Hardcoded word lists eliminated | A02, A03, A20, A24 use discovered data |
| Constants moved to EngineParams | A10, A11, A12 (5 new params) |
| SELF triggers derived from dict | A05 fixed |
| TASK grammar/content indicators removed | A14 partially fixed |
| Negation models documented | A15 commented |
| All regressions pass | dict5 20/20, dict12 14/20, unified ≥44/50 |
| Existing genomes still load | `#[serde(default)]` on all new params |
| Code compiles without warnings | `cargo build --release` clean |

## OUTPUT

When complete, produce `reports/19c_code_audit_fixes.md` with:
- Per-finding status: FIXED / PARTIAL / SKIPPED (with reason)
- Before/after test results
- Any unexpected regressions and how they were handled
- Updated severity summary (how many 🔴 remain?)

The goal: reduce 🔴 VIOLATION from 4 to ≤1 (A01 partial fix + A13 partial fix should eliminate 3, A14 partial = eliminate 1).
