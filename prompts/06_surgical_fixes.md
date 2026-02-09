# PROMPT 06 — Surgical Fixes: Negation & Property Queries

## CONTEXT

DAPHNE is a geometric comprehension engine at the end of its evolutionary optimization phase. Six phases of development have produced:

- **0.7063 combined fitness** with zero overfitting (v10, best ever)
- **10/10 positive queries**, **5/5 transitive reasoning**, **3-4/4 honesty**
- Grammar reinforcement validated as a regularizer (prevents space degeneration)
- Dual-space ensemble provides complementary signal but hurts IDK precision
- All parameter and strategy evolution exhausted — convergence confirmed across 6 independent runs

**Two walls remain, unchanged across all phases:**
- **Negation (Q11-Q14): 0/4** — "Is a dog a cat?" → wrong. "Is the sun cold?" → wrong.
- **Property queries (Q19-Q20): 0/2** — "What is a dog?" → IDK instead of "animal"

These are architectural limitations. No more evolution runs. Two surgical code changes.

## PROJECT STRUCTURE

```
D:\workspace\projects\dafhne\
├── crates/
│   ├── dafhne-core/       Core types, GeometricSpace, Answer, traits
│   ├── dafhne-parser/     Dictionary/test/grammar parsing
│   ├── dafhne-engine/     Engine + resolver.rs (THE FILE TO CHANGE)
│   ├── dafhne-eval/       Fitness scoring
│   └── dafhne-evolve/     GA (not changed in this prompt)
├── dictionaries/
│   ├── dict5.md, dict5_test.md
│   ├── dict12.md, dict12_test.md
│   └── grammar5.md, grammar5_test.md
└── results_v10/         Best run (genome for testing)
```

**Key file:** `crates/dafhne-engine/src/resolver.rs`

The resolver dispatches questions:
```
query() → parse → classify
  ├── Yes/No → resolve_yes_no()        ← CHANGE FOR NEGATION
  ├── What-is → resolve_what_is()      ← CHANGE FOR PROPERTY QUERIES
  └── Unknown → IDontKnow
```

## FIX 1: DEFINITION-CHAIN NEGATION

### The Problem

"Is a dog a cat?" fails because dog and cat are geometrically close — they're both animals, they co-occur with similar words, the force field pushes them together. Euclidean distance can't distinguish "same category" from "same entity." Every negation model tried (Inversion, Repulsion, AxisShift, SeparateDimension) fails because they all rely on distance, and distance says dog ≈ cat.

### The Solution

Add a **definition-chain check** before the distance check. This is NOT a rule — it's a geometric traversal that provides NEGATIVE evidence.

Algorithm for "Is X a Y?" where current distance says Yes:

```
1. Get X's definition text from the dictionary
2. Tokenize the definition into words
3. Check: does Y appear in X's definition? 
   - "Is a dog an animal?" → dog def: "an animal. it can make sound..." → "animal" FOUND → confirm Yes
   - "Is a dog a cat?" → dog def: "an animal. it can make sound..." → "cat" NOT FOUND → continue
4. If not found directly, follow ONE hop:
   - X's definition mentions category words. For each category word Z in X's definition:
     - Does Z's definition contain Y?
   - dog's def mentions "animal" → animal's def: "a thing that lives..." → "cat" NOT FOUND
5. Now check reverse: does Y's definition contain X?
   - cat's def: "a small animal..." → "dog" NOT FOUND
6. If Y not found in X's chain AND X not found in Y's chain:
   - They are in the same space but NOT definitionally linked
   - Override distance-based Yes → answer No
7. If either chain connects → confirm the distance-based Yes
```

This works for all four negation questions:
- Q11 "Is a dog a cat?" → dog's chain: animal→thing. No "cat". cat's chain: animal→thing. No "dog". → **No** ✓
- Q12 "Is the sun cold?" → sun's def: "a big hot thing." Contains "hot." hot's def: "not cold." Contains "not" + "cold" → antonym detected → **No** ✓
- Q13 "Is a ball an animal?" → ball's chain: thing. No "animal" anywhere. animal not in ball's chain. → **No** ✓
- Q14 "Is the sun small?" → sun's def: "a big hot thing." Contains "big." big's def: "not small." → antonym detected → **No** ✓

### Antonym Detection Sub-rule

For Q12 and Q14, the chain finds the ANTONYM via "not" patterns. The definition-chain traversal must recognize:
- If X's definition contains word Z, and Z's definition says "not Y" → X is NOT Y
- If X's definition says "hot" and hot's definition says "not cold" → X is not cold

This is still definition-chain traversal, not a hardcoded rule. The "not" pattern is already a discovered connector in the system. The traversal just needs to propagate negation through one hop.

### Implementation

```rust
/// Check if subject is definitionally linked to object.
/// Returns: Some(true) = linked, Some(false) = definitionally unlinked, None = can't determine
fn definition_chain_check(
    &self,
    subject: &str,
    object: &str,
    dictionary: &Dictionary,
    max_hops: usize,  // evolve this: start with 2
) -> Option<bool> {
    let subject_entry = dictionary.get(subject)?;
    let subject_words = tokenize(&subject_entry.definition);
    
    // Direct check: object in subject's definition?
    if subject_words.contains(&object.to_string()) {
        // Check for negation: "not {object}" pattern
        if preceded_by_not(&subject_words, object) {
            return Some(false);  // definitionally negated
        }
        return Some(true);  // definitionally linked
    }
    
    // One-hop check: follow category words in subject's definition
    if max_hops > 0 {
        for word in &subject_words {
            if dictionary.contains(word) && is_content_word(word) {
                // Check this intermediate word's definition
                if let Some(result) = self.definition_chain_check(
                    word, object, dictionary, max_hops - 1
                ) {
                    return Some(result);
                }
            }
        }
    }
    
    None  // can't determine from definitions
}

fn preceded_by_not(words: &[String], target: &str) -> bool {
    for (i, word) in words.iter().enumerate() {
        if word == target && i > 0 && words[i - 1] == "not" {
            return true;
        }
    }
    false
}
```

### Integration into resolve_yes_no()

The definition chain check is a GATE, not a replacement:

```rust
fn resolve_yes_no(&self, subject: &str, object: &str, ...) -> Answer {
    // Step 1: Geometric distance (existing code, unchanged)
    let distance = self.compute_distance(subject, object);
    let geometric_answer = self.decide_yes_no(distance, ...);
    
    // Step 2: Definition chain check (NEW)
    if geometric_answer == Answer::Yes {
        // Distance says yes — verify with definition chain
        match self.definition_chain_check(subject, object, &self.dictionary, 2) {
            Some(false) => return Answer::No,   // chain says NO → override
            Some(true) => return Answer::Yes,    // chain confirms → keep
            None => {}
                // Chain can't determine → check reverse direction
        }
        match self.definition_chain_check(object, subject, &self.dictionary, 2) {
            Some(false) => return Answer::No,
            _ => {}
        }
        // If chain is inconclusive, trust the geometry
        return geometric_answer;
    }
    
    // Step 3: If geometry says No or IDK, trust it
    geometric_answer
}
```

### Why This Isn't a Rule System

The definition chain check traverses the SAME dictionary that built the geometric space. It uses the SAME tokenizer. It finds patterns that the force field encoded as proximity — but it adds the negative dimension that proximity can't express. The geometry says "dog and cat are related" (true). The chain says "but neither defines the other" (also true). Both pieces of evidence are needed for negation.

Future evolution: `max_hops` becomes a genome parameter (range 1-4). `is_content_word()` can use the connector discovery results to filter function words. The chain traversal is evolvable.

### What This Does NOT Fix

This only helps when both words are in the dictionary. For novel words not in definitions, the system falls back to pure geometry. This is correct behavior — if you don't have definitional evidence, you shouldn't pretend you do.

---

## FIX 2: AXIS-SPECIFIC NEAREST NEIGHBOR FOR PROPERTY QUERIES

### The Problem

"What is a dog?" should return "animal" but returns IDK. The resolver finds the nearest word to "dog" in the full euclidean space — which might be "cat" or "food" or any co-occurring word. It doesn't know to look along the "is a" axis specifically.

Previous attempt (v7): Added `use_connector_axis` boolean. Evolution rejected it (96% → false) because single-axis projection compressed all words onto a line where unrelated words overlapped by coincidence.

### Why It Failed Before

The v7 approach projected onto a SINGLE dimension (the connector's force_direction). In a 20-dimensional space, projecting onto 1 dimension discards 95% of the information. Unrelated words that happen to have similar values on that one dimension appear close.

### The New Approach: Multi-Axis Weighted Projection

Instead of projecting onto a single connector axis, use the connector's force_direction as a WEIGHT VECTOR for the distance computation. Dimensions aligned with the connector get higher weight. Dimensions orthogonal to it get lower weight (but not zero).

```
Standard euclidean:  d = sqrt(sum((a_i - b_i)²))
Weighted by axis:    d = sqrt(sum(w_i * (a_i - b_i)²))

where w_i = alpha + (1 - alpha) * |connector_direction_i|
alpha = minimum weight for non-connector dimensions (0.1 .. 0.5)
```

This keeps ALL dimensions in play (avoiding the compression problem) but emphasizes the connector-relevant dimensions. The `alpha` parameter controls how much emphasis: alpha=0.5 means only mild emphasis, alpha=0.1 means strong emphasis on the connector axis.

### Implementation

```rust
fn weighted_distance(
    &self,
    word_a: &str,
    word_b: &str,
    connector_direction: &[f64],
    alpha: f64,  // evolve this: range 0.05..0.5
) -> Option<f64> {
    let pos_a = self.space.get_position(word_a)?;
    let pos_b = self.space.get_position(word_b)?;
    
    let mut sum = 0.0;
    for i in 0..pos_a.len() {
        let weight = alpha + (1.0 - alpha) * connector_direction[i].abs();
        let diff = pos_a[i] - pos_b[i];
        sum += weight * diff * diff;
    }
    Some(sum.sqrt())
}
```

### Integration into resolve_what_is()

```rust
fn resolve_what_is(&self, subject: &str) -> Answer {
    // Step 1: Find the "is" or "is a" connector
    let is_connector = self.space.connectors.iter()
        .find(|c| c.pattern == vec!["is"] || c.pattern == vec!["is", "a"]);
    
    // Step 2: Find nearest content word using weighted distance
    let mut best_word = None;
    let mut best_distance = f64::MAX;
    
    for (word, _point) in &self.space.words {
        if word == subject { continue; }
        if !self.is_content_word(word) { continue; }  // skip function words
        
        let dist = match is_connector {
            Some(conn) => self.weighted_distance(
                subject, word, &conn.force_direction, self.params.axis_weight_alpha
            ),
            None => self.euclidean_distance(subject, word),
        };
        
        if let Some(d) = dist {
            if d < best_distance {
                best_distance = d;
                best_word = Some(word.clone());
            }
        }
    }
    
    // Step 3: Apply threshold — only return word if clearly closest
    match best_word {
        Some(word) if best_distance < self.params.what_is_threshold => {
            Answer::Word(word)
        }
        _ => Answer::IDontKnow,
    }
}
```

### Content Word Filtering

The function must skip function words ("the", "a", "is", "can", "not", etc.) when searching for the nearest category word. Use connector discovery results: any word that appears in a connector pattern is a function word. Everything else is a content word.

```rust
fn is_content_word(&self, word: &str) -> bool {
    // Words that appear in connector patterns are function words
    for connector in &self.space.connectors {
        if connector.pattern.contains(&word.to_string()) {
            return false;
        }
    }
    true
}
```

This is derived from the system's own connector discovery — not hardcoded.

### Definition-Chain Fallback for What-Is

If the weighted distance approach still can't discriminate (everything is equidistant), fall back to definition extraction:

```rust
// Fallback: extract first content word from subject's definition
fn definition_category(&self, subject: &str, dictionary: &Dictionary) -> Option<String> {
    let entry = dictionary.get(subject)?;
    let words = tokenize(&entry.definition);
    
    // Find first content word that is itself a dictionary entry
    for word in &words {
        if dictionary.contains(word) && self.is_content_word(word) {
            return Some(word.clone());
        }
    }
    None
}
```

For "What is a dog?" → dog's definition: "an animal. it can make sound..." → first content word that's a dict entry = "animal" → return "animal".

This is direct but clean: it uses the dictionary text as structured data, not as a rule. The definition's word order encodes the primary category (authors put the category first: "an ANIMAL", "a small ANIMAL", "a big hot THING").

### New Genome Parameters

```rust
// Add to EngineGenome / EngineParams:
axis_weight_alpha: f64,     // 0.05..0.5 — connector axis emphasis
what_is_threshold: f64,     // 0.1..1.0 — threshold for word answers
chain_max_hops: usize,      // 1..4 — definition chain traversal depth
use_definition_fallback: bool,  // whether to use definition extraction for what-is
```

---

## TESTING PROTOCOL

### Step 1: Manual Test (no evolution)

Apply both fixes to the resolver. Use the best v10 genome parameters. Run:

```bash
cargo run -p dafhne-engine -- \
    --dict dictionaries/dict5.md \
    --grammar dictionaries/grammar5.md \
    --test dictionaries/dict5_test.md
```

Expected results per question:

| Question | Before | After | Fix Used |
|----------|--------|-------|----------|
| Q01: Is a dog an animal? | ✓ or ✗ | ✓ | chain confirms |
| Q02: Is the sun hot? | ✓ or ✗ | ✓ | chain confirms |
| Q03: Is a cat small? | ✓ | ✓ | unchanged |
| Q04: Can a dog make sound? | ✓ or ✗ | ✓ | chain confirms |
| Q05: Can a person make things? | ✗ | ✓ | chain confirms |
| Q06-Q10: Transitive | ✓ | ✓ | chain confirms via hops |
| Q11: Is a dog a cat? | ✗ | ✓ | **chain negation** |
| Q12: Is the sun cold? | ✗ | ✓ | **antonym chain** |
| Q13: Is a ball an animal? | ✗ | ✓ | **chain negation** |
| Q14: Is the sun small? | ✗ | ✓ | **antonym chain** |
| Q15-Q18: Unknown | ✓ | ✓ | unchanged |
| Q19: What is a dog? | ✗ | ✓ | **weighted axis OR def fallback** |
| Q20: What is a person? | ✗ | ✓ | **weighted axis OR def fallback** |

Target: 20/20 on dict5.

### Step 2: Cross-Validation

Run on dict12 WITHOUT retraining:

```bash
cargo run -p dafhne-engine -- \
    --dict dictionaries/dict12.md \
    --grammar dictionaries/grammar5.md \
    --test dictionaries/dict12_test.md
```

Target: > 16/20 on dict12 (negation + property queries should now work if definitions are available).

### Step 3: Grammar Test

```bash
cargo run -p dafhne-engine -- \
    --dict dictionaries/dict5.md \
    --grammar dictionaries/grammar5.md \
    --test dictionaries/grammar5_test.md
```

Target: > 17/20 on grammar5_test (meta-knowledge questions Q17-Q20 may still be hard).

### Step 4: Evolution with New Parameters

Only if Step 1-3 show improvement. Add the 4 new genome parameters. Run 20 generations:

```bash
cargo run --release -p dafhne-evolve -- run \
    --dict5 dictionaries/dict5.md --test5 dictionaries/dict5_test.md \
    --dict12 dictionaries/dict12.md --test12 dictionaries/dict12_test.md \
    --grammar dictionaries/grammar5.md \
    --population 50 --generations 20 --results results_v11/ --seed 42
```

Target: Combined fitness > 0.85.

---

## WHAT NOT TO DO

- Do NOT change the force field or training pipeline. These fixes are resolver-only.
- Do NOT remove the geometric distance check. The chain is a gate ON TOP of geometry, not a replacement.
- Do NOT hardcode which words are function words. Use connector discovery results.
- Do NOT process grammar5 differently from any other document. Same pipeline.
- Do NOT run evolution BEFORE testing the fixes manually. Understand the raw impact first.
- Do NOT add the definition-chain check for IDK questions. Those work already. Chain check is ONLY for Yes/No overrides and What-Is extraction.
- Do NOT change the fitness function. Same formula: `0.5 * accuracy + 0.5 * honesty`.

## CRITICAL: PRESERVE WHAT WORKS

The following must NOT regress:
- Q06-Q10 transitive reasoning (currently 5/5)
- Q15-Q18 honesty/IDK (currently 3-4/4)
- Zero overfitting gap between dict5 and dict12
- Grammar regularization effect

If any of these regress after the fixes, the chain check is being too aggressive. Reduce `chain_max_hops` or add a confidence threshold before overriding geometry.

## SUCCESS CRITERIA

| Metric | Current | Minimum | Target | Stretch |
|--------|---------|---------|--------|---------|
| dict5 fitness | 0.7063 | > 0.75 | > 0.90 | 1.00 |
| dict12 fitness | 0.6875 | > 0.70 | > 0.85 | > 0.95 |
| Negation (Q11-Q14) | 0/4 | 2/4 | 4/4 | 4/4 both dicts |
| Property (Q19-Q20) | 0/2 | 1/2 | 2/2 | 2/2 both dicts |
| Transitive (Q06-Q10) | 5/5 | 5/5 | 5/5 | 5/5 both dicts |
| Honesty (Q15-Q18) | 3-4/4 | 3/4 | 4/4 | 4/4 both dicts |
| Overfitting gap | 0.00 | < 0.15 | < 0.10 | 0.00 |

## ATTACHED FILES

- All source code (crates/dafhne-*)
- All dictionaries and test files
- `results_v10/` best genome parameters
- `prompts/04_phase5_handoff.md` — detailed resolver architecture reference

## OUTPUT

1. Modified `resolver.rs` with both fixes
2. Updated `genome.rs` with 4 new parameters
3. Updated `strategy.rs` if needed for new parameter integration
4. All existing tests still pass
5. New tests for definition-chain and weighted-distance functions
6. Results table: per-question before/after for dict5, dict12, grammar5_test
