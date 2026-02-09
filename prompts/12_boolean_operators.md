# PROMPT 12 — Boolean Operators: AND/OR Compound Queries

## PREAMBLE

Phase 11b achieved 19/21 on full_test and 10/10 on 3w_test. The resolver now handles what/who/where questions and definition-category extraction reliably. All Yes/No and What-Is single-predicate questions work.

Phase 12 adds compound queries: questions with two predicates joined by AND or OR. This is a query-level decomposition — the engine, equilibrium, and connector discovery are untouched.

## GOAL

Extend the resolver to handle:
- **AND**: "Is a dog an animal and a thing?" → decompose into two sub-queries, both must be Yes
- **OR**: "Is a cat a dog or an animal?" → decompose into two sub-queries, either can be Yes

Existing NOT negation is already handled by the negated flag in Yes/No detection and works correctly. No changes needed for NOT.

## ROOT CAUSE: WHY THIS DOESN'T WORK TODAY

### What happens now

```
Question: "Is a dog an animal and a thing?"
  → tokenize: ["is", "a", "dog", "an", "animal", "and", "a", "thing"]
  → detect_yes_no_question:
    → content_entries: [(2, "dog"), (4, "animal"), (7, "thing")]
    → subject = content_entries[0] = "dog"
    → object  = content_entries[last] = "thing"
    → "and" is structural → skipped in content scan
    → "animal" is silently lost
  → Resolved as: "Is a dog a thing?" (ignoring "and an animal")
  → Answer: Yes (happens to be right, but by accident)
```

For OR:
```
Question: "Is a dog a cat or an animal?"
  → content_entries: [(2, "dog"), (4, "cat"), (7, "animal")]
  → subject = "dog", object = "animal" (last content word)
  → "cat" is silently lost
  → Resolved as: "Is a dog an animal?" → Yes
  → Correct by accident, but "Is a dog a person or a cat?" → "Is a dog a cat?" → No
  → Should be No (both are No), and it IS No, but by luck not logic
```

The current parser throws away intermediate content words between subject and object. For single-predicate questions this is fine (there's only one object). For compound queries, the first predicate is lost.

## THE FIX

### Architecture: Query-Level Decomposition

Detect AND/OR at the **top of `resolve_question()`**, BEFORE any question-type detection. Split into two independent sub-questions. Resolve each recursively. Combine with boolean truth table.

This avoids changing `QuestionType`, `detect_yes_no_question()`, or any existing resolution path.

### Step 1: Add BoolOp enum

In `crates/dafhne-engine/src/resolver.rs`, add near the top:

```rust
/// Boolean operator for compound queries.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BoolOp {
    And,
    Or,
}
```

### Step 2: Add compound detection function

This function scans tokens for "and"/"or", determines the question prefix (question verb + subject), and constructs two complete sub-question strings.

```rust
/// Detect AND/OR compound queries and split into sub-question strings.
///
/// For "Is a dog an animal and a thing?":
///   → prefix = "is a dog" (question verb + articles + subject)
///   → left  = "is a dog an animal?"
///   → right = "is a dog a thing?"
///
/// Only fires for Yes/No questions (starting with is/can/does/do/has).
/// Returns None for What/Who/Where questions and single-predicate questions.
fn detect_compound(
    tokens: &[String],
    dictionary: &Dictionary,
    content: &HashSet<String>,
) -> Option<(BoolOp, String, String)> {
    // Only split Yes/No questions (question-verb-first).
    // What/Who/Where compound ("What is a dog and what is a cat?") is
    // two separate questions, not a boolean compound.
    let question_verbs: HashSet<&str> = ["is", "can", "does", "do", "has"]
        .iter().copied().collect();
    if tokens.is_empty() || !question_verbs.contains(tokens[0].as_str()) {
        return None;
    }

    // Find boolean operator (first occurrence only).
    // Using first occurrence handles single-operator compounds.
    // Multi-operator ("A and B and C") resolves left-to-right:
    // the right sub-question still contains "and", which triggers
    // recursive compound detection.
    let (op, op_idx) = tokens.iter().enumerate()
        .find_map(|(i, t)| match t.as_str() {
            "and" => Some((BoolOp::And, i)),
            "or" => Some((BoolOp::Or, i)),
            _ => None,
        })?;

    // Boolean operator must be AFTER the subject (at least position 2)
    // to avoid false positives on compound-noun subjects like
    // "bread and butter". The subject is at minimum position 1
    // (position 0 is the question verb).
    if op_idx < 3 {
        return None;
    }

    // Extract question prefix: everything up to and including the subject.
    // Pattern: [question_verb] [articles...] [subject]
    // Subject = first content word after the question verb.
    let articles: HashSet<&str> = ["a", "an", "the"].iter().copied().collect();
    let mut prefix_end = 0; // exclusive index past the subject
    for (i, token) in tokens.iter().enumerate().skip(1) {
        // Skip articles
        if articles.contains(token.as_str()) {
            continue;
        }
        // First non-article token after question verb = subject
        if let Some(stemmed) = stem_to_entry(token, &dictionary.entry_set) {
            if dictionary.entry_set.contains(&stemmed) {
                prefix_end = i + 1;
                break;
            }
        }
        // If token isn't in dictionary, it might still be the subject
        // (e.g., a proper noun not in the entry set). Include it.
        prefix_end = i + 1;
        break;
    }

    if prefix_end == 0 || prefix_end >= op_idx {
        return None; // no subject found or subject is past the operator
    }

    let prefix: Vec<&str> = tokens[..prefix_end].iter().map(|s| s.as_str()).collect();
    let left_pred: Vec<&str> = tokens[prefix_end..op_idx].iter().map(|s| s.as_str()).collect();
    let right_pred: Vec<&str> = tokens[op_idx + 1..].iter().map(|s| s.as_str()).collect();

    if left_pred.is_empty() || right_pred.is_empty() {
        return None; // malformed: nothing on one side of the operator
    }

    let left_question = format!("{} {}?", prefix.join(" "), left_pred.join(" "));
    let right_question = format!("{} {}?", prefix.join(" "), right_pred.join(" "));

    Some((op, left_question, right_question))
}
```

### Step 3: Add boolean combiner

Three-valued boolean logic with IDK as the unknown:

```rust
/// Combine two Yes/No/IDK answers with boolean AND or OR.
///
/// Truth tables:
///   AND: Yes∧Yes=Yes, Yes∧No=No, Yes∧IDK=IDK, No∧anything=No, IDK∧IDK=IDK
///   OR:  Yes∨anything=Yes, No∨No=No, No∨IDK=IDK, IDK∨IDK=IDK
///
/// Word answers in boolean context are treated as IDK (compounds are
/// for Yes/No questions only).
fn combine_boolean(
    op: BoolOp,
    left: &Answer,
    right: &Answer,
) -> Answer {
    // Normalize Word answers to IDK for boolean context
    let l = match left {
        Answer::Yes | Answer::No | Answer::IDontKnow => left.clone(),
        Answer::Word(_) => Answer::IDontKnow,
    };
    let r = match right {
        Answer::Yes | Answer::No | Answer::IDontKnow => right.clone(),
        Answer::Word(_) => Answer::IDontKnow,
    };

    match op {
        BoolOp::And => match (&l, &r) {
            (Answer::No, _) | (_, Answer::No) => Answer::No,
            (Answer::Yes, Answer::Yes) => Answer::Yes,
            _ => Answer::IDontKnow,
        },
        BoolOp::Or => match (&l, &r) {
            (Answer::Yes, _) | (_, Answer::Yes) => Answer::Yes,
            (Answer::No, Answer::No) => Answer::No,
            _ => Answer::IDontKnow,
        },
    }
}
```

### Step 4: Wire into resolve_question()

At the **top** of `resolve_question()`, before existing question-type detection:

```rust
pub fn resolve_question(
    question: &str,
    space: &GeometricSpace,
    dictionary: &Dictionary,
    structural: &HashSet<String>,
    content: &HashSet<String>,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> (Answer, Option<f64>, Option<String>) {
    let tokens = tokenize(question);

    // ── Compound query detection (AND/OR) ──────────────────────
    if let Some((op, left_q, right_q)) = detect_compound(&tokens, dictionary, content) {
        let (left_ans, left_dist, left_conn) =
            resolve_question(&left_q, space, dictionary, structural, content, params, strategy);
        let (right_ans, right_dist, right_conn) =
            resolve_question(&right_q, space, dictionary, structural, content, params, strategy);

        let combined = combine_boolean(op, &left_ans, &right_ans);

        // Distance: use the sub-query that determined the result.
        // AND→No: distance from the No sub-query (the bottleneck).
        // AND→Yes: max distance (both had to pass).
        // OR→Yes: distance from the Yes sub-query (the winner).
        // OR→No: max distance (both failed).
        let dist = match (&combined, &op) {
            (Answer::Yes, BoolOp::And) => match (left_dist, right_dist) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (d, None) | (None, d) => d,
            },
            (Answer::Yes, BoolOp::Or) => match (left_dist, right_dist) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (d, None) | (None, d) => d,
            },
            (Answer::No, _) => match (left_dist, right_dist) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (d, None) | (None, d) => d,
            },
            _ => match (left_dist, right_dist) {
                (Some(a), Some(b)) => Some((a + b) / 2.0),
                (d, None) | (None, d) => d,
            },
        };

        let op_str = match op { BoolOp::And => "AND", BoolOp::Or => "OR" };
        let conn = match (left_conn, right_conn) {
            (Some(l), Some(r)) => Some(format!("{} [{}] {}", l, op_str, r)),
            (Some(c), None) | (None, Some(c)) => Some(c),
            _ => None,
        };

        return (combined, dist, conn);
    }

    // ── Existing single-question path (unchanged) ──────────────
    let question_type = detect_question_type(&tokens, dictionary, content, structural);
    // ... rest of existing code ...
}
```

**Important**: The recursive call to `resolve_question()` means multi-operator compounds ("A and B and C") work automatically via left-to-right splitting. "Is a dog an animal and a thing and a cat?" splits into:
- Left: "Is a dog an animal?" → resolved normally
- Right: "Is a dog a thing and a cat?" → triggers compound detection again → splits into "Is a dog a thing?" + "Is a dog a cat?" → combined with AND
- Final: AND(Yes, AND(Yes, No)) = AND(Yes, No) = No

### Step 5: No changes elsewhere

- **dafhne-core**: No changes. Answer types are unchanged.
- **dafhne-parser**: No changes. Tokenizer already produces "and"/"or" as tokens.
- **dafhne-eval**: No changes. `evaluate()` calls `resolve_question()` which now handles compounds internally.
- **Engine/equilibrium/connectors**: No changes.

## TESTING

### Test File: dict5_bool_test.md

Create `dictionaries/dict5_bool_test.md` with 10 questions using dict5's 51-word vocabulary.

```markdown
# dict5_bool — Boolean Operator Tests (10)

## AND — both must be Yes

---

**Q01**: Is a dog an animal and a thing?
**A**: Yes
**Chain**: dog→animal=Yes AND dog→animal→thing=Yes → AND→Yes

---

**Q02**: Is a dog an animal and a cat?
**A**: No
**Chain**: dog→animal=Yes AND dog→cat=No → AND→No

---

**Q03**: Is the sun big and hot?
**A**: Yes
**Chain**: sun→big=Yes AND sun→hot=Yes → AND→Yes

---

**Q04**: Is the sun hot and cold?
**A**: No
**Chain**: sun→hot=Yes AND sun→cold=No → AND→No

---

**Q05**: Is a ball an animal and a thing?
**A**: No
**Chain**: ball→animal=No AND ball→thing=Yes → AND→No

---

## OR — either can be Yes

---

**Q06**: Is a dog a cat or an animal?
**A**: Yes
**Chain**: dog→cat=No OR dog→animal=Yes → OR→Yes

---

**Q07**: Is the sun hot or cold?
**A**: Yes
**Chain**: sun→hot=Yes OR sun→cold=No → OR→Yes

---

**Q08**: Is a cat a dog or a ball?
**A**: No
**Chain**: cat→dog=No OR cat→ball=No → OR→No

---

**Q09**: Is a dog an animal or a person?
**A**: Yes
**Chain**: dog→animal=Yes OR dog→person=No → OR→Yes

---

**Q10**: Can a dog eat and move?
**A**: Yes
**Chain**: dog→eat=Yes AND dog→move=Yes → AND→Yes
```

### Test File: bool_test.md (Three Men in a Boat)

Create `texts/three_men/bool_test.md` with 5 questions using entities.

```markdown
# bool_test — Boolean Operators for Three Men in a Boat (5)

---

**Q01**: Is Montmorency a dog and an animal?
**A**: Yes
**Chain**: montmorency→dog=Yes AND montmorency→animal=Yes → AND→Yes

---

**Q02**: Is Harris a person or a dog?
**A**: Yes
**Chain**: harris→person=Yes OR harris→dog=No → OR→Yes

---

**Q03**: Is Harris a person and a dog?
**A**: No
**Chain**: harris→person=Yes AND harris→dog=No → AND→No

---

**Q04**: Is the Thames a river or a person?
**A**: Yes
**Chain**: thames→river=Yes OR thames→person=No → OR→Yes

---

**Q05**: Is Kingston a place and a person?
**A**: No
**Chain**: kingston→place=Yes AND kingston→person=No → AND→No
```

### Run order

```bash
# 1. dict5 boolean test (closed mode)
cargo run -p dafhne-eval -- \
    --dict dictionaries/dict5.md \
    --test dictionaries/dict5_bool_test.md \
    --mode equilibrium
# Expected: ≥9/10 (Q05 might be tricky: ball→animal chain depends on dict5 definitions)

# 2. Three Men boolean test (open mode + entities)
cargo run -p dafhne-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/bool_test.md \
    --mode equilibrium
# Expected: 5/5

# 3. dict5 regression
cargo run -p dafhne-eval -- \
    --dict dictionaries/dict5.md \
    --test dictionaries/dict5_test.md \
    --mode equilibrium
# Expected: 20/20

# 4. dict12 regression
cargo run -p dafhne-eval -- \
    --dict dictionaries/dict12.md \
    --test dictionaries/dict12_test.md \
    --mode equilibrium
# Expected: 14/20

# 5. passage1 regression
cargo run -p dafhne-eval -- \
    --text texts/passage1.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/passage1_test.md \
    --mode equilibrium
# Expected: 5/5

# 6. full_test regression
cargo run -p dafhne-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/full_test.md \
    --mode equilibrium
# Expected: 19/21

# 7. 3w_test regression
cargo run -p dafhne-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/3w_test.md \
    --mode equilibrium
# Expected: 10/10
```

## EXPECTED IMPACT

### dict5_bool_test: ≥9/10

| Q | Question | Sub1 | Sub2 | Bool | Expected |
|---|----------|------|------|------|----------|
| Q01 | dog: animal AND thing? | Yes | Yes | AND→Yes | ✅ |
| Q02 | dog: animal AND cat? | Yes | No | AND→No | ✅ |
| Q03 | sun: big AND hot? | Yes | Yes | AND→Yes | ✅ |
| Q04 | sun: hot AND cold? | Yes | No | AND→No | ✅ |
| Q05 | ball: animal AND thing? | No | Yes | AND→No | ✅ |
| Q06 | dog: cat OR animal? | No | Yes | OR→Yes | ✅ |
| Q07 | sun: hot OR cold? | Yes | No | OR→Yes | ✅ |
| Q08 | cat: dog OR ball? | No | No | OR→No | ✅ |
| Q09 | dog: animal OR person? | Yes | No | OR→Yes | ✅ |
| Q10 | dog: eat AND move? | Yes | Yes | AND→Yes | ✅ |

Q10 depends on "Can a dog eat?" and "Can a dog move?" both resolving to Yes. Both pass in dict5_test (Q08 and chain from Q06-Q10 area), so this should work.

### bool_test (Three Men): 5/5

All sub-queries are known-passing questions from full_test and 3w_test.

### Regressions: zero

Existing tests contain no "and"/"or" tokens in questions, so the compound detection path never fires. All existing questions go through the unchanged single-question path.

## WHAT NOT TO DO

- Do NOT change `QuestionType` enum. Compound detection is a wrapper, not a new variant.
- Do NOT modify `detect_yes_no_question()` or `detect_what_question()`. Existing parsing is correct for single-predicate questions.
- Do NOT modify engine, equilibrium, force field, or connector discovery.
- Do NOT handle What-Is compounds ("What is X and what is Y?"). These are two separate questions, not a boolean compound. Out of scope.
- Do NOT apply boolean splitting to questions starting with "what"/"who"/"where".
- Do NOT try geometric-level composition (intersection/union of proximity regions). Query-level decomposition is simpler and correct.

## KNOWN LIMITATIONS

1. **Compound-noun subjects**: "Is bread and butter good?" would be incorrectly split. Mitigated by `op_idx < 3` guard (requires operator at position 3+). DAFHNE's test vocabulary doesn't include compound nouns.

2. **Mixed negation + boolean**: "Is a dog not a cat and not a person?" splits into "Is a dog not a cat?" + "Is a dog not a person?". Each sub-question's negation detection handles the "not" independently. Should work but is untested in Phase 12.

3. **Multi-operator chains**: "Is a dog an animal and a thing and a cat?" works via recursive splitting (right sub-question contains another "and"). Not explicitly tested but architecturally sound.

4. **OR with What-Is**: "Is a dog a cat or what?" is nonsensical. Safeguarded by question-verb check.

## SUCCESS CRITERIA

| Metric | Expected |
|--------|----------|
| dict5_bool_test | ≥9/10 |
| bool_test (Three Men) | 5/5 |
| dict5 regression | 20/20 |
| dict12 regression | 14/20 |
| passage1 regression | 5/5 |
| full_test regression | 19/21 |
| 3w_test regression | 10/10 |
| Code changes | resolver.rs only (BoolOp + detect_compound + combine_boolean + wiring) |

## OUTPUT CHECKLIST

1. ☐ `BoolOp` enum added
2. ☐ `detect_compound()` function implemented
3. ☐ `combine_boolean()` function implemented
4. ☐ `resolve_question()` wired with compound detection
5. ☐ `dict5_bool_test.md` created (10 questions)
6. ☐ `bool_test.md` created (5 questions, Three Men)
7. ☐ dict5_bool_test results (expect ≥9/10)
8. ☐ bool_test results (expect 5/5)
9. ☐ Regression: dict5 20/20
10. ☐ Regression: dict12 14/20
11. ☐ Regression: passage1 5/5
12. ☐ Regression: full_test 19/21
13. ☐ Regression: 3w_test 10/10
14. ☐ RECAP.md updated with Phase 12 results
