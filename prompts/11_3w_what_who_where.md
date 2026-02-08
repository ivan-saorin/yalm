# PROMPT 11 — 3W: What, Who, Where

## PREAMBLE

YALM's resolver currently handles two question types:
- **Yes/No**: "Is X a Y?" — geometric distance + definition-chain gate
- **What-Is**: "What is X?" — definition-category extraction + geometric nearest neighbor

Phase 10 proved the pipeline works on Victorian literature (0.87 fitness, 16/21), but exposed a gap: **"Who is Montmorency?" returns IDK** because the resolver only recognizes "what" as a question word. Q16 is a free point.

This prompt extends the resolver to handle the **3W** identity questions: What, Who, Where. All three are classification queries — they ask "which category does X belong to?" — and the geometry already supports them. The work is resolver routing, not engine changes.

### The ELI5 Insight

Phase 10 revealed a key architectural principle worth documenting: the pipeline decouples text complexity from geometric complexity.

```
Victorian text (complex) → seed words → Ollama ELI5 (simple) → geometry
```

The ELI5 definition constraint maximizes algorithm efficacy in three ways:
1. **Taxonomic anchoring**: "a [category]." as first sentence = direct input to `resolve_what_is()` first-content-word extraction
2. **Connector density**: ~200-word definition vocabulary = high repetition = strong frequency signal for connector discovery
3. **Compact closure**: BFS depth-2 covers nearly everything (99.5% closure at 2429 entries)

Dumbing down the definitions makes the system smarter. The intelligence is in the geometric structure, not the definitions. ELI5 definitions are maximally transparent to the force field.

This principle should be added to RECAP.md as a standalone section: **"The ELI5 Principle"**.

---

## ARCHITECTURE

### Current State

`detect_question_type()` in `crates/yalm-engine/src/resolver.rs`:

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

That's it. The resolver already extracts the first content word from the definition. If Montmorency's definition starts with "a dog", both "What is Montmorency?" and "Who is Montmorency?" return "a dog". The answer is correct regardless of question word because the definition determines the category, not the question.

### Why Not Enforce Person-Category?

"Who is Montmorency?" should answer "a dog" — that's the correct answer even though "who" implies personhood. Enforcing person-category would break this. The question word is a social convention, not a semantic constraint. If a future prompt wants to add "who expects person" logic, that's a resolver refinement, not a blocker.

### Test

Add to `texts/three_men/full_test.md` (or create a new test file `texts/three_men/3w_test.md`):

```markdown
## WHO QUESTIONS

---

**Q01**: Who is Montmorency?
**A**: a dog
**Chain**: montmorency -> dog (definition category)

---

**Q02**: Who is Harris?
**A**: a person
**Chain**: harris -> person (entity definition)

---

**Q03**: Who is George?
**A**: a person
**Chain**: george -> person (entity definition)
```

Expected: Q01 passes (was Q16 failure in phase 10). Q02/Q03 may fail due to the known LLM-definition-override issue (common-noun "harris"/"george" definitions from Ollama override entity definitions in `definition_category()` — see phase 10 failure analysis Q17/Q18). Document but don't block on this.

---

## TASK 2: "WHERE" HANDLER

### Semantics

"Where is X?" is different from "What is X?". It asks for a **location relationship**, not a category.

- "Where is Kingston?" → should answer with a place ("on the thames", "in england")
- "Where is Montmorency?" → should answer with a location if available ("on the boat") or IDK

This is harder than "who" because the answer isn't the first content word of the definition — it's a **location relation** extracted from the definition.

### Implementation: Two Strategies

**Strategy A (Simple — recommended for this prompt):**

Treat "where" like "what" for now. "Where is Kingston?" → extracts definition category → "a place" or "a town". Not ideal ("a town" doesn't answer WHERE), but it works with existing machinery and scores points on tests like "Where is Kingston?" → "a place" if the test expects that.

**Strategy B (Full — future prompt):**

Add a new `QuestionType::WhereIs` that scans the definition for location prepositions ("in", "on", "near", "at") followed by place-category words. "Kingston: a town on the Thames" → extract "on the thames" as the location. This requires new resolver logic.

### For This Prompt: Use Strategy A

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

### Test

Add to the 3W test file:

```markdown
## WHERE QUESTIONS

---

**Q04**: Where is Kingston?
**A**: a place
**Chain**: kingston -> place (definition category)

---

**Q05**: Where is Hampton?
**A**: a place
**Chain**: hampton -> place (definition category)
```

Note: we test "Where is X?" → "a place" (category extraction), not "Where is X?" → "on the Thames" (location relation). The latter requires Strategy B.

---

## TASK 3: TEST SUITE

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
# 3W test with combined text + entities
cargo run -p yalm-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/3w_test.md \
    --mode equilibrium
```

### Also re-run full_test.md

The original full_test.md includes Q16 ("Who is Montmorency?") which should now pass:

```bash
cargo run -p yalm-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/full_test.md \
    --mode equilibrium
```

Expected: 17/21 (was 16/21, +1 from Q16 who-handler).

---

## TASK 4: REGRESSION

Verify zero regression:

```bash
# dict5 closed mode
cargo run -p yalm-eval -- \
    --dict dictionaries/dict5.md \
    --test dictionaries/dict5_test.md \
    --mode equilibrium
# Expected: 20/20

# passage1 open mode
cargo run -p yalm-eval -- \
    --text texts/passage1.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/passage1_test.md \
    --mode equilibrium
# Expected: 5/5
```

---

## TASK 5: KNOWN ISSUES TO DOCUMENT (DO NOT FIX)

These are known issues from Phase 10. Document their current status but do NOT fix them in this prompt:

1. **Q17/Q18 (What is Harris/George?)**: LLM definitions for common nouns "harris" and "george" override entity definitions in `definition_category()`. The entity merge puts the correct definition in the dictionary, but `definition_category()` may pick up the wrong word. Root cause: entity definitions say "a person" but the resolver's first-content-word extraction might not land on "person" if the definition format differs from dict5 style.

2. **Q10/Q11 (Is Harris/George an animal?)**: 2-hop transitive chain person→animal fails because Ollama defines "person" as "a human being" not "an animal". The chain traversal finds person→human but not human→animal within 2 hops.

3. **"Where" Strategy B**: Full location-relation extraction ("on the Thames", "in England") is deferred to a future prompt.

---

## WHAT NOT TO DO

- Do NOT modify the engine, equilibrium, force field, or connector discovery code.
- Do NOT add new QuestionType variants. Both "who" and "where" reuse `WhatIs`.
- Do NOT enforce category constraints ("who" expects person, "where" expects place). Let the definition determine the answer.
- Do NOT fix the Harris/George LLM-definition issue in this prompt. That's a separate problem.

## SUCCESS CRITERIA

| Metric | Expected |
|--------|----------|
| "Who is Montmorency?" | a dog |
| "Where is Kingston?" | a place |
| 3w_test score | ≥ 7/10 |
| full_test score (re-run) | ≥ 17/21 (was 16/21) |
| dict5 regression | 20/20 |
| passage1 regression | 5/5 |
| Code changes | resolver.rs only (detect_question_type + 2 new functions) |

## OUTPUT CHECKLIST

1. ☐ `detect_question_type()` updated with "who" and "where" routing
2. ☐ `detect_who_question()` implemented
3. ☐ `detect_where_question()` implemented
4. ☐ `texts/three_men/3w_test.md` created (10 questions)
5. ☐ 3w_test results recorded
6. ☐ full_test re-run results recorded (expect 17/21)
7. ☐ Regression: dict5 20/20, passage1 5/5
8. ☐ ELI5 Principle section added to RECAP.md
9. ☐ RECAP.md updated with Phase 11 results