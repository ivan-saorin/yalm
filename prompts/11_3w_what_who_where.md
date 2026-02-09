# PROMPT 11 — 3W: What, Who, Where + Chain Depth

## PREAMBLE

DAPHNE's resolver currently handles two question types:
- **Yes/No**: "Is X a Y?" — geometric distance + definition-chain gate
- **What-Is**: "What is X?" — definition-category extraction + geometric nearest neighbor

Phase 10 proved the pipeline works on Victorian literature (0.87 fitness, 16/21). Phase 10b mapped the comprehension gradient across 50 questions at 6 granularity levels:

| Level | Description | Score | Rate |
|-------|-------------|-------|------|
| 1 Ontological | "Is X a thing/alive?" | 3/8 | 37.5% |
| 2 Kingdom | "Is X a person/animal/place?" | 6/6 | 100% |
| 3 Species/Type | "Is X a man/terrier/town?" | 6/6 | 100% |
| 4 Properties | "Can X move/eat/think?" | 10/10 | 100% |
| 5 Relational | "Is X on/near Y?" | 6/10 | 60% |
| 6 Narrative | "Is X small/old/friend?" | 5/10 | 50% |

Two clear findings drive this prompt:

1. **"Who is Montmorency?" returns IDK** because the resolver only recognizes "what" as a question word. Q16 is a free point.

2. **Level 1 ontological failures are ALL chain-depth limited.** "Is Montmorency a thing?" needs 3 hops (montmorency→dog→animal→thing) but `max_hops=2`. The geometry and definitions are correct; the traversal stops too early. Five of 14 total failures across the granularity probe trace to this single parameter.

This prompt makes two surgical changes: **3W routing** (who/where handlers) and **chain depth experiment** (max_hops 2→3). Both are resolver-only.

### The ELI5 Principle (confirmed by 10b)

Phase 10b proved Level 4 (Properties/Capabilities) at 100% — the prompt predicted 40-60%. The ELI5 definition constraint doesn't just help; it's **optimal** for geometric comprehension:

```
Victorian text (complex) → seed words → Ollama ELI5 (simple) → geometry
```

1. **Taxonomic anchoring**: "a [category]." = direct input to first-content-word extraction
2. **Connector density**: ~200-word definition vocabulary = strong frequency signal
3. **Compact closure**: BFS depth-2 covers 99.5% (2429 entries)
4. **Capability encoding**: "can move", "can eat" appear verbatim in ELI5 definitions

Dumbing down the definitions makes the system smarter. Zero "wrong definition" failures in 50 questions.

This principle should be added to RECAP.md as a standalone section: **"The ELI5 Principle"**.

---

## ARCHITECTURE

### Current State

`detect_question_type()` in `crates/dafhne-engine/src/resolver.rs`:

```rust
fn detect_question_type(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    if tokens.is_empty() { return None; }
    if tokens[0] == "what" {
        return detect_what_question(tokens, dictionary, content, structural);
    }
    detect_yes_no_question(tokens, dictionary, content, structural)
}
```

Only "what" triggers `WhatIs` resolution. Everything else falls through to `YesNo` detection, and if that fails → `None` → IDK.

### Target State

```rust
fn detect_question_type(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    if tokens.is_empty() { return None; }
    match tokens[0].as_str() {
        "what" => detect_what_question(tokens, dictionary, content, structural),
        "who"  => detect_who_question(tokens, dictionary, content, structural),
        "where" => detect_where_question(tokens, dictionary, content, structural),
        _ => detect_yes_no_question(tokens, dictionary, content, structural),
    }
}
```

---

## TASK 1: "WHO" HANDLER

### Semantics

"Who is X?" is identical to "What is X?" except:
- It expects a **person-category** answer (person, man, woman, character)
- If the answer is NOT a person-category, it should still return the definition category (the geometry doesn't care about the question word)

### Implementation

`detect_who_question()` is a thin wrapper around the existing `detect_what_question()`:

```rust
/// Detect "Who is X?" questions.
/// Semantically identical to "What is X?" — the resolver doesn't
/// enforce person-category constraints. The question word is just routing.
fn detect_who_question(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    // Reuse what-question detection: "who" behaves like "what" for resolution.
    // The geometric space and definition-category extraction handle the rest.
    detect_what_question(tokens, dictionary, content, structural)
}
```

That's it. The resolver already extracts the first content word from the definition. If Montmorency's definition starts with "a dog", both "What is Montmorency?" and "Who is Montmorency?" return "a dog".

### Why Not Enforce Person-Category?

"Who is Montmorency?" should answer "a dog" — that's the correct answer even though "who" implies personhood. Enforcing person-category would break this. The question word is a social convention, not a semantic constraint.

---

## TASK 2: "WHERE" HANDLER

### Semantics

"Where is X?" asks for a **location relationship**, not a category. This is harder than "who".

### Implementation: Strategy A (Simple)

Treat "where" like "what" for now. "Where is Kingston?" → extracts definition category → "a place" or "a town". Not ideal ("a town" doesn't answer WHERE), but it works with existing machinery.

```rust
/// Detect "Where is X?" questions.
/// Currently uses same resolution as "What is X?" (definition-category extraction).
/// Future: dedicated location-relation extraction from definitions.
fn detect_where_question(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
    structural: &HashSet<String>,
) -> Option<QuestionType> {
    detect_what_question(tokens, dictionary, content, structural)
}
```

Full location-relation extraction ("on the Thames", "in England") is Strategy B, deferred.

---

## TASK 3: CHAIN DEPTH EXPERIMENT

### The Problem

Phase 10b found that 5 of 14 failures (36%) across the granularity probe are caused by `max_hops=2` in `definition_chain_check()`. All Level 1 ontological questions require 3 hops:

```
Is Montmorency a thing?   montmorency → dog → animal → thing     (3 hops)
Is Montmorency alive?     montmorency → dog → animal → alive     (3 hops)
Is Harris a thing?        harris → person → human → thing        (3 hops)
Is Harris alive?          harris → person → human → alive        (3 hops)
Is the Thames alive?      thames → river → ... (expected No, but chain can't verify)
```

### The Risk

Increasing `max_hops` from 2 to 3 means longer chains, which means more connections, which means more false positives. At depth 3, nearly everything might connect to everything through high-connectivity words like "thing".

Specifically, the risk is that **negation questions start failing**. "Is a dog a river?" currently returns No because the 2-hop chain finds no connection. At 3 hops, dog→animal→thing and river→water→thing might BOTH reach "thing", creating a false chain link.

The `MAX_FOLLOW_PER_HOP=3` limit (only follow first 3 content words per definition) provides some protection, but it's untested at depth 3.

### Implementation

Change `max_hops` from 2 to 3 in `resolve_yes_no()` in `crates/dafhne-engine/src/resolver.rs`:

```rust
// In resolve_yes_no(), the definition-chain gate section:
let max_hops = 3; // was: 2
```

This is a ONE-LINE change. But the testing is critical.

### Testing Protocol

Run ALL test suites before and after the change. Record a comparison table:

```bash
# BEFORE (max_hops=2) — record baseline for all test suites

# 1. dict5 (must stay 20/20)
cargo run -p dafhne-eval -- \
    --dict dictionaries/dict5.md \
    --test dictionaries/dict5_test.md \
    --mode equilibrium

# 2. dict12 (baseline: 14/20)
cargo run -p dafhne-eval -- \
    --dict dictionaries/dict12.md \
    --test dictionaries/dict12_test.md \
    --mode equilibrium

# 3. passage1 (must stay 5/5)
cargo run -p dafhne-eval -- \
    --text texts/passage1.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/passage1_test.md \
    --mode equilibrium

# 4. full_test (baseline: 16/21)
cargo run -p dafhne-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/full_test.md \
    --mode equilibrium

# 5. granularity_test (baseline: 36/50)
cargo run -p dafhne-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/granularity_test.md \
    --mode equilibrium
```

```bash
# AFTER (max_hops=3) — same runs, compare every question
# (same commands as above)
```

### Decision Matrix

| Outcome | Action |
|---------|--------|
| Level 1 improves, no regressions | Keep max_hops=3 ✔ |
| Level 1 improves, dict5 regresses | Revert. Protection too weak at depth 3. |
| Level 1 improves, dict12 regresses | Investigate which questions broke. May be acceptable if net gain. |
| Level 1 unchanged | Chain still too short for some paths. Revert, investigate definitions. |
| Widespread false positives | Revert. Need per-question-type depth control (ontological=3, negation=2). |

If regression is found, consider **selective depth**: increase max_hops only for forward (positive) chain checks while keeping max_hops=2 for the "both-in-dict → No" fallback path. The forward check benefits from depth; the absence check doesn't.

---

## TASK 4: TEST SUITE

Create `texts/three_men/3w_test.md` with 10 questions covering all three question words:

```markdown
# 3w_test — What/Who/Where Questions for Three Men in a Boat

## WHAT QUESTIONS

---

**Q01**: What is Montmorency?
**A**: a dog
**Chain**: montmorency -> dog

---

**Q02**: What is the Thames?
**A**: a river
**Chain**: thames -> river

---

**Q03**: What is Kingston?
**A**: a place
**Chain**: kingston -> place

---

## WHO QUESTIONS

---

**Q04**: Who is Montmorency?
**A**: a dog
**Chain**: montmorency -> dog

---

**Q05**: Who is Harris?
**A**: a person
**Chain**: harris -> person

---

**Q06**: Who is George?
**A**: a person
**Chain**: george -> person

---

## WHERE QUESTIONS

---

**Q07**: Where is Kingston?
**A**: a place
**Chain**: kingston -> place

---

**Q08**: Where is Hampton?
**A**: a place
**Chain**: hampton -> place

---

## MIXED

---

**Q09**: What is Harris?
**A**: a person
**Chain**: harris -> person

---

**Q10**: What is George?
**A**: a person
**Chain**: george -> person
```

### Run

```bash
cargo run -p dafhne-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/3w_test.md \
    --mode equilibrium
```

---

## TASK 5: FULL REGRESSION + COMPARISON

After implementing both changes (3W routing + max_hops=3), run all test suites and record:

| Test Suite | Before (P10) | After (P11) | Delta |
|------------|-------------|-------------|-------|
| dict5 | 20/20 | ?/20 | must be 0 |
| dict12 | 14/20 | ?/20 | watch for regression |
| passage1 | 5/5 | ?/5 | must be 0 |
| full_test | 16/21 | ?/21 | expect +1 (Q16 who) |
| granularity_test | 36/50 | ?/50 | expect +3–5 (Level 1) |
| 3w_test | (new) | ?/10 | baseline |

### Also re-run granularity_test per-level

| Level | Before (10b) | After (11) | Delta | Notes |
|-------|-------------|------------|-------|-------|
| 1 Ontological | 3/8 | ?/8 | ? | max_hops=3 target |
| 2 Kingdom | 6/6 | ?/6 | ? | must hold |
| 3 Species/Type | 6/6 | ?/6 | ? | must hold |
| 4 Properties | 10/10 | ?/10 | ? | must hold |
| 5 Relational | 6/10 | ?/10 | ? | watch for false positives |
| 6 Narrative | 5/10 | ?/10 | ? | watch for false positives |

The critical check: do Levels 2-4 stay at 100%? Does Level 5-6 get WORSE (false positives from deeper chains)? If Level 1 improves without regression elsewhere, the change is validated.

---

## TASK 6: KNOWN ISSUES TO DOCUMENT (DO NOT FIX)

These are known issues from Phase 10. Document their current status but do NOT fix them:

1. **Q17/Q18 (What is Harris/George?)**: LLM definitions for common nouns "harris" and "george" override entity definitions in `definition_category()`. The entity merge puts the correct definition in the dictionary, but `definition_category()` may pick up the wrong word.

2. **Q10/Q11 (Is Harris/George an animal?)**: 2-hop transitive chain person→animal fails because Ollama defines "person" as "a human being" not "an animal". At max_hops=3, this MAY now succeed if the chain finds human→being→animal or human→animal at depth 3. Record whether this changes.

3. **"Where" Strategy B**: Full location-relation extraction deferred.

---

## WHAT NOT TO DO

- Do NOT modify the engine, equilibrium, force field, or connector discovery code.
- Do NOT add new QuestionType variants. Both "who" and "where" reuse `WhatIs`.
- Do NOT enforce category constraints ("who" expects person, "where" expects place).
- Do NOT fix the Harris/George LLM-definition issue in this prompt.
- Do NOT increase max_hops beyond 3 without testing. The false-positive risk grows exponentially.

## SUCCESS CRITERIA

| Metric | Expected |
|--------|----------|
| "Who is Montmorency?" | a dog |
| "Where is Kingston?" | a place |
| 3w_test score | ≥ 7/10 |
| full_test score (re-run) | ≥ 17/21 (was 16/21) |
| granularity Level 1 | ≥ 5/8 (was 3/8) |
| granularity Levels 2-4 | stay at 100% (no regression) |
| dict5 regression | 20/20 |
| dict12 regression | ≥ 14/20 (no regression) |
| passage1 regression | 5/5 |
| Code changes | resolver.rs only |

## OUTPUT CHECKLIST

1. ☐ `detect_question_type()` updated with "who" and "where" routing
2. ☐ `detect_who_question()` implemented
3. ☐ `detect_where_question()` implemented
4. ☐ `max_hops` changed from 2 to 3 in `resolve_yes_no()`
5. ☐ `texts/three_men/3w_test.md` created (10 questions)
6. ☐ BEFORE/AFTER comparison table for all test suites
7. ☐ Per-level granularity comparison (10b vs 11)
8. ☐ Decision on max_hops=3: keep or revert (with evidence)
9. ☐ 3w_test results recorded
10. ☐ full_test re-run results recorded
11. ☐ Regression: dict5 20/20, dict12 ≥14/20, passage1 5/5
12. ☐ Q10/Q11 (person→animal) status documented — did max_hops=3 fix it?
13. ☐ ELI5 Principle section added to RECAP.md
14. ☐ RECAP.md updated with Phase 11 results