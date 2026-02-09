# PROMPT 21 — Hard Questions

> **STATUS: Implementation phase. Fix the 5 remaining unified_test failures from Phase 20.**

## CONTEXT

Phase 20 achieved 45/50 (90%) on unified_test with per-space evolution. The 5 failures fall into 4 distinct categories. This prompt addresses them in order of difficulty (easiest first).

Project location: `D:\workspace\projects\dafhne`

Use the filesystem MCP with path `D:\workspace\projects\dafhne` to access files.

## PREREQUISITE READING

Read these before writing any code:
1. `crates/dafhne-engine/src/resolver.rs` — main resolution logic, `definition_category()`, `resolve_what_is()`, `resolve_yes_no()`
2. `crates/dafhne-engine/src/multispace.rs` — space routing, `route_question()`, multi-step dispatch
3. `dictionaries/unified_test.md` — the 50-question test file (check Q13, Q18, Q20, Q25, Q36 specifically)
4. `dictionaries/dict_grammar5.md` — grammar space definitions (Q18/Q20 answers live here)
5. `dictionaries/dict_math5.md` — math space definitions (Q13 needs ordinal comparison)
6. `dictionaries/dict_task5.md` — task space definitions (Q25 routing)
7. `crates/dafhne-core/src/lib.rs` — `EngineParams`, `GeometricSpace`
8. `RECAP.md` — phase history for context

## THE 5 FAILURES

| Q | Question | Expected | Got | Category |
|---|----------|----------|-----|----------|
| Q18 | What is a sentence? | words in order that tell a thing | a word | Definition truncation |
| Q20 | What is a subject? | the thing in a sentence that does the action | a thing | Definition truncation |
| Q13 | Is three more than one? | Yes | No | Ordinal comparison |
| Q25 | What kind of task is "how many animals"? | a number task | I don't know | Quoted phrase routing |
| Q36 | Two plus three. The answer is a number. Is it big? | No | Yes | Multi-step pipeline |

---

## FIX 1: Definition Truncation (Q18, Q20) — LOWEST HANGING FRUIT

### Problem

`definition_category()` extracts the first content word from a definition to identify the hypernym/category:
- "sentence — words in order that tell a thing" → category = "word" (first content word after skipping structural)
- "subject — the thing in a sentence that does the action" → category = "thing"

For "What is X?" questions, `resolve_what_is()` returns `definition_category()` output formatted as "a [category]". This works when you want "What kind of thing is a dog?" → "an animal". But it FAILS when the question is asking for the actual DEFINITION, not just the category.

### Root Cause

The resolver has no "full definition" mode. Every "What is X?" question gets the same truncated hypernym answer.

### Fix

Modify `resolve_what_is()` (or the function it calls) to return the **full definition text** when the question is literally "What is [X]?" and X is a known word. The category extraction should only be used when the question implies classification ("What kind of thing is X?", "What is X — an animal or a food?").

**Approach A — Full Definition Fallback** (recommended):

In `resolve_what_is()`, when the question is "What is [word]?" and [word] is in the dictionary:
1. Look up the word's definition text from the dictionary
2. Return the full definition text (everything after the "—") as the answer
3. Only fall back to `definition_category()` if full definition is not available

The definition text IS the answer for "What is a sentence?" — it's "words in order that tell a thing".

**Implementation sketch**:
```rust
fn resolve_what_is(&self, subject: &str) -> Option<String> {
    // If the word is in our dictionary, return its full definition
    if let Some(definition) = self.get_definition_text(subject) {
        return Some(definition);
    }
    // Fallback: use category from geometric nearest neighbor
    // (existing logic for unknown words)
    self.resolve_what_is_by_geometry(subject)
}
```

You'll need a method to retrieve the raw definition text from the dictionary. Check how the dictionary is stored — definitions are parsed during `Engine::train()` from the markdown format:
```
word — definition text here
  - sentence one
  - sentence two
```

The definition text (the part after "—") should be accessible from the `GeometricSpace` or dictionary store. If it's not currently stored, you need to store it during parsing.

**Important**: The answer format for Q18 expects "words in order that tell a thing" — the raw definition text, NOT "a words in order that tell a thing". So don't prepend an article.

**Also important**: This must NOT break existing "What is a dog?" → "an animal" behavior. For dict5, "What is a dog?" should still return "an animal" because that IS the definition category and the test expects it. Check the dict5 test expectations to make sure. The fix should prefer the full definition for grammar-space words (where definitions are longer descriptive phrases) while still working for content-space words (where the category IS the answer).

**Strategy**: Return full definition text when it's more than ~3 words. Return category when the definition is short (e.g., "dog — an animal that..." → category = "animal" is sufficient). This heuristic naturally separates content words (short category definitions) from grammar words (long descriptive definitions).

### Testing

After this fix:
- Q18: "What is a sentence?" → "words in order that tell a thing" ✅
- Q20: "What is a subject?" → "the thing in a sentence that does the action" ✅
- Existing: "What is a dog?" → "an animal" ✅ (MUST NOT REGRESS)

---

## FIX 2: Ordinal Comparison (Q13) — MEDIUM DIFFICULTY

### Problem

"Is three more than one?" requires comparing ordinal values. The math space has numbers but no relational operators. "more than" is not arithmetic (not addition/subtraction) — it's a comparison.

### Root Cause

The math space handles `X plus Y`, `X minus Y` but has no `X more than Y` / `X less than Y` operators. These are relational comparisons that need ordinal knowledge.

### Fix

Add ordinal comparison to the math resolver. Numbers in the math dictionary already have implicit ordinal positions (one=1, two=2, etc. from the number-to-word mapping). The fix needs to:

1. **Detect comparison questions**: Patterns like "is X more than Y?", "is X less than Y?", "is X bigger than Y?"
2. **Extract the two operands**: X and Y (number words)
3. **Compare their ordinal values**: Return Yes/No

**Implementation sketch**:

In the math resolver (or `resolve_yes_no()` when routed to math space):
```rust
// Detect "more than" / "less than" patterns
if question_contains_comparison(tokens) {
    let (a, b) = extract_comparison_operands(tokens);
    let val_a = number_word_to_value(a)?;  // "three" → 3
    let val_b = number_word_to_value(b)?;  // "one" → 1
    
    if is_more_than {
        return Some(if val_a > val_b { "yes" } else { "no" });
    }
    if is_less_than {
        return Some(if val_a < val_b { "yes" } else { "no" });
    }
}
```

The `number_word_to_value()` mapping already exists (A04 in the audit — it's hardcoded but works). Use it.

**Comparison triggers**: "more than", "less than", "bigger than", "smaller than", "greater than". These are fixed English phrases. Since math routing already uses hardcoded indicators (accepted until math rework), adding comparison triggers is consistent.

### Testing

- Q13: "Is three more than one?" → "Yes" ✅
- Existing math questions must still pass

---

## FIX 3: Quoted Phrase Routing (Q25) — MEDIUM DIFFICULTY

### Problem

"What kind of task is 'how many animals'?" expects "a number task". The TASK space should recognize "how many" inside the quoted phrase as a math indicator and route accordingly. But the quoted phrase is treated as opaque text.

### Root Cause

The task routing in `multispace.rs` (or wherever task classification happens) checks the full question for math indicators. But "how many" appears inside quotes, and the question itself is a meta-question about classification, not a direct math question.

### Fix

This is a two-part problem:
1. **Extract the quoted phrase**: Parse out the content between quotes
2. **Classify the extracted phrase**: Run task classification on the extracted phrase content

**Implementation sketch**:

In the task classifier (likely in `multispace.rs`):
```rust
fn classify_task(&self, question: &str) -> Option<String> {
    // Check for quoted phrases — meta-classification questions
    if let Some(quoted) = extract_quoted_phrase(question) {
        // Classify the quoted content, not the wrapper question
        return self.classify_task_content(&quoted);
    }
    // Normal classification...
}

fn extract_quoted_phrase(text: &str) -> Option<String> {
    // Match "..." or '...' or "..." (smart quotes)
    // Return the content between quotes
}
```

The key insight: when someone asks "What kind of task is [X]?", they're asking you to classify X. So strip the meta-question and classify the inner content.

For "how many animals", the math indicators should match "how many" → returns "a number task" (or however the expected answer maps to the existing task vocabulary).

**Check the expected answer format**: Q25 expects "a number task". Look at how `dict_task5.md` defines task types and how the task space answers classification questions. The answer needs to match the test format exactly.

### Testing

- Q25: "What kind of task is 'how many animals'?" → "a number task" ✅
- Existing task routing must still pass

---

## FIX 4: Multi-Step Pipeline (Q36) — HARDEST

### Problem

"Two plus three. The answer is a number. Is it big?" requires:
1. Compute: 2 + 3 = 5
2. Recognize "the answer" refers to the computation result
3. Retrieve: "Is five big?" → property lookup
4. Answer: No (five is not big in the dictionary's definition)

### Root Cause

The architecture processes one question at a time with no state between sentences. Multi-sentence queries with anaphoric references ("the answer") and cross-space reasoning (math → content property) are not supported.

### Fix — Option A: Sentence Pipeline (recommended)

Split multi-sentence input into individual steps, carry context forward:

```rust
fn resolve_pipeline(&self, input: &str) -> String {
    let sentences = split_sentences(input);  // Split on ". " 
    let mut context: HashMap<String, String> = HashMap::new();
    let mut last_result = String::new();
    
    for sentence in sentences {
        // Substitute anaphoric references
        let resolved = substitute_anaphora(&sentence, &context);
        
        // Process the resolved sentence
        let result = self.resolve(&resolved);
        
        // Store result for next sentence
        context.insert("the answer".to_string(), result.clone());
        context.insert("it".to_string(), result.clone());
        last_result = result;
    }
    
    last_result
}
```

**Anaphora resolution**: Replace "the answer", "it", "the result" with the previous computation's output. This is a minimal coreference resolution.

For Q36:
1. "Two plus three" → compute → "five"
2. "The answer is a number" → substitute "the answer" → "five is a number" → confirm (Yes, five is a number) → context: answer = "five"
3. "Is it big?" → substitute "it" → "Is five big?" → resolve → "No"

**Implementation location**: This belongs in `multispace.rs` since it's a multi-step dispatcher that orchestrates across spaces. The `answer()` method should detect multi-sentence input and invoke the pipeline.

### Fix — Option B: Minimal Two-Pass (simpler but less general)

If Option A is too large, detect the specific pattern "X op Y. ... Is it [property]?":
1. If input contains multiple sentences AND a math operation in the first sentence
2. Compute the math result
3. Replace "it"/"the answer" in subsequent sentences with the result
4. Process the final question

This handles Q36 specifically but doesn't generalize.

### Testing

- Q36: "Two plus three. The answer is a number. Is it big?" → "No" ✅
- Ensure single-sentence questions are unaffected

---

## IMPLEMENTATION ORDER

1. **Fix 1 (Q18, Q20)** — definition truncation. Smallest change, biggest win (+2 points). Localized to resolver.
2. **Fix 2 (Q13)** — ordinal comparison. Small addition to math resolver (+1 point).
3. **Fix 3 (Q25)** — quoted phrase routing. Small addition to task classifier (+1 point).
4. **Fix 4 (Q36)** — multi-step pipeline. Largest change, but biggest architectural value (+1 point).

**If time-constrained**: Fix 1 alone gets 47/50. Fixes 1+2+3 get 49/50. Fix 4 gets 50/50 but is the riskiest.

## TESTING

After ALL fixes, run the full regression suite:

```bash
# Single-space dict5 (MUST stay 20/20)
cargo run --release -p dafhne-eval

# Multi-space unified test (target: 50/50)
cargo run --release -p dafhne-eval -- \
  --spaces content:dictionaries/dict5.md,math:dictionaries/dict_math5.md,grammar:dictionaries/dict_grammar5.md,task:dictionaries/dict_task5.md,self:dictionaries/dict_self5.md \
  --test dictionaries/unified_test.md \
  --genome results_v11/best_genome.json

# dict12 (MUST stay ≥14/20)
cargo run --release -p dafhne-eval -- \
  --dict dictionaries/dict12.md \
  --test dictionaries/dict12_test.md \
  --grammar dictionaries/grammar18.md \
  --genome results_v11/best_genome.json

# Bootstrap (should still converge)
cargo run --release -p dafhne-eval -- \
  --spaces content:dictionaries/dict5.md,math:dictionaries/dict_math5.md,grammar:dictionaries/dict_grammar5.md,task:dictionaries/dict_task5.md,self:dictionaries/dict_self5.md \
  --test dictionaries/unified_test.md \
  --genome results_v11/best_genome.json \
  --bootstrap 3
```

**NOTE**: The binary may still be `yalm-eval` if crate rename isn't complete. Check `Cargo.toml` workspace for actual binary names.

### Regression Targets

| Test Suite | Baseline | Target | Minimum |
|------------|----------|--------|---------|
| dict5 | 20/20 | 20/20 | 20/20 |
| unified_test | 45/50 | 50/50 | 47/50 |
| dict12 | 14/20 | 14/20 | 14/20 |
| Bootstrap | Level 2 | Level 2 | Level 2 |

## WHAT NOT TO DO

- Do NOT change any dictionary content (the definitions are the input, not the code)
- Do NOT change test expectations (the answers are correct, the engine needs to produce them)
- Do NOT change EngineParams or genome structure (no evolution changes this phase)
- Do NOT create new test files
- Do NOT add new dependencies
- Do NOT restructure the resolver — make surgical additions to existing methods
- Do NOT break single-space dict5 behavior (the "What is a dog?" → "an animal" path)

## SUCCESS CRITERIA

| Metric | Target |
|--------|--------|
| Q18 fixed | "What is a sentence?" → "words in order that tell a thing" |
| Q20 fixed | "What is a subject?" → "the thing in a sentence that does the action" |
| Q13 fixed | "Is three more than one?" → "Yes" |
| Q25 fixed | 'What kind of task is "how many animals"?' → "a number task" |
| Q36 fixed | "Two plus three. The answer is a number. Is it big?" → "No" |
| Zero regression | dict5 20/20, dict12 ≥14/20 |
| Clean build | `cargo build --release` no warnings |

## OUTPUT

When complete, produce `reports/21_hard_questions.md` with:
- Per-fix status: FIXED / PARTIAL / SKIPPED
- Before/after test results
- Description of each code change (which function, what was added)
- Any unexpected regressions and how they were handled
- Updated unified_test score

Update `RECAP.md` with Phase 21 entry.
Update `STATUS.md` with new score.
