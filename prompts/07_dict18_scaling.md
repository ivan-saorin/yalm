# PROMPT 07 — Dict18: The Scaling Curve

## PREAMBLE

DAFHNE (Definition-Anchored Force-field Heuristic Network Engine) is a geometric comprehension engine that learns from text alone — no neural networks, no pretrained models, no NLP libraries. It reads closed dictionaries where every word in every definition is itself defined, builds an N-dimensional space where words are points, discovers connectors from statistics, and answers questions by geometric distance + definition-chain traversal.

Three results define the project so far:
1. **Dict5** (50 words, 5-year-old level): **20/20** — perfect score after v11 surgical fixes
2. **Dict12** (~400 words, 12-year-old level): **15/20** — zero overfitting from dict5
3. **Grammar as regularizer** — prevents geometric space collapse during evolution

This prompt builds **dict18** (~2000 words, 18-year-old / university-entry level) to produce a **three-point scaling curve**. Three data points tell us whether comprehension degrades linearly, exponentially, or plateaus as vocabulary grows. This is the most important empirical question in the project.

## PROJECT STRUCTURE

```
D:\workspace\projects\dafhne\
├── crates/
│   ├── dafhne-core/         Data structures, GeometricSpace, Answer, traits
│   ├── dafhne-parser/        Dictionary/test/grammar parsing
│   ├── dafhne-engine/        Force field + resolver (geometry + chain traversal)
│   ├── dafhne-eval/          Fitness scoring
│   └── dafhne-evolve/        Genetic algorithm
├── dictionaries/
│   ├── dict5.md             50 words, CLOSED
│   ├── dict5_test.md        20 test questions
│   ├── dict12.md            ~400 words, NEAR-CLOSED
│   ├── dict12_test.md       20 test questions
│   ├── grammar5.md          Grammar text in dict5 vocabulary
│   └── grammar5_test.md     20 grammar-aware test questions
├── prompts/                 Phase prompts (this file = 07)
├── results_v11/             Latest evolution results
└── RECAP.md                 Full project history
```

## PHASE GOAL

Produce three deliverables:
1. `dictionaries/dict18.md` — ~2000-word closed dictionary at university-entry level
2. `dictionaries/dict18_test.md` — 20 test questions spanning all question types
3. `dictionaries/grammar18.md` — grammar text written in dict18 vocabulary

Then run the existing engine on dict18 and record the scaling curve.

---

## TASK 1: BUILD DICT18

### Vocabulary Selection

Dict18 must satisfy these constraints:

1. **CLOSURE**: Every word used in every definition must itself be defined in dict18. This is the hardest constraint and the one most likely to break. Verify closure programmatically after construction.
2. **SUPERSET**: Dict18 must contain all dict12 words. Dict12 contains all dict5 words. The hierarchy must be strict: dict5 ⊂ dict12 ⊂ dict18.
3. **NATURAL GROWTH**: New words should come from domains that a 12→18 year old acquires: abstract concepts (justice, probability, hypothesis), scientific terms (molecule, orbit, frequency), emotional vocabulary (anxiety, empathy, ambition), social structures (democracy, economy, corporation), and technical foundations (algorithm, variable, equation).
4. **DEFINITION QUALITY**: Each definition must use ONLY words already in dict18. Definitions should be 1-3 sentences. The FIRST content word should be the category word (this is how the resolver extracts "what is X" answers). Examples from dict5:
   - dog: "an animal. it can make sound and move fast."
   - sun: "a big hot thing in the sky. it makes light."

### Construction Strategy

Do NOT try to write 2000 definitions in one pass. Use layers:

1. **Start from dict12** (~400 words). These are already closed.
2. **Add domain seeds** — 50-100 words per domain (science, society, emotions, abstractions, body, technology). Write rough definitions.
3. **Chase closure** — for each new definition, extract all words used. If any word is not yet in dict18, either:
   - Add it with its own definition (if it's a useful word at the 18-year-old level)
   - Rewrite the definition to avoid it (if it's obscure)
4. **Iterate** until the closure checker reports zero undefined words.
5. **Audit** — run closure check one final time. Report any leaks.

### Closure Verification

Write a script or use the existing parser to verify:

```
For every entry E in dict18:
  For every word W in E.definition + E.examples:
    Assert W is defined in dict18 OR W is a connector word ("is", "a", "the", "not", "can", etc.)
```

Connector words (function words) are exempt from closure because the system discovers them from statistics, not definitions.

### Expected Difficulty

Dict5 closure was trivial (50 words, hand-crafted). Dict12 closure required an audit prompt (prompt 01). Dict18 closure will be the hardest part of this phase. Budget significant effort here. Circular definitions are acceptable ("justice: when a society treats people in a fair way" / "fair: based on justice and equal treatment") — the geometry handles circularity through force equilibrium.

---

## TASK 2: BUILD DICT18 TEST

### Test Structure

Exactly 20 questions, matching the distribution used in dict5_test and dict12_test:

| Category | Count | Type | Purpose |
|----------|-------|------|---------|
| Positive Yes/No | 4 | "Is X a Y?" / "Is X Y?" | Taxonomic and property proximity |
| Capability | 2 | "Can X do Y?" | Action-relationship proximity |
| Transitive | 4 | Multi-hop chains | Depth of geometric reasoning |
| Negation | 4 | "Is X a Y?" (answer: No) | Definition-chain discrimination |
| Honesty/Unknown | 4 | Out-of-scope questions | Geometric sparsity → IDK |
| What-Is | 2 | "What is X?" | Category extraction |

### Question Design Principles

- **Use dict18-specific words** for at least 12 of the 20 questions. Don't just test dict12 words again.
- **Transitive chains should be longer** than in dict5/12. At 2000 words, 3-4 hop chains are realistic: "Is an electron part of matter?" (electron → atom → matter)
- **Negation questions should test new failure modes**: abstract category confusion ("Is democracy an emotion?"), cross-domain errors ("Is a molecule a feeling?"), near-miss categories ("Is a hypothesis a theory?" — these are close but NOT the same)
- **Unknown questions should probe the boundaries**: questions where a human might guess but the dictionary genuinely lacks the path ("What color is anxiety?" — category error with abstract nouns)
- **What-Is questions should test abstract definitions**: "What is democracy?" should return something like "system" or "way" — whatever the first content word is in the definition.

### Test File Format

Match the existing format exactly. Check dict5_test.md for the format.

---

## TASK 3: BUILD GRAMMAR18

### What Grammar Does

Grammar text is a regularizer. It's prose written entirely in the dictionary's vocabulary that describes what connectors mean. grammar5.md was written in dict5's 50 words and described patterns like "is a" and "not". Its effect: same seed, same parameters, with grammar → 0.7063 fitness, without grammar → 0.4875 (collapse).

### Grammar18 Requirements

1. Written ENTIRELY in dict18 vocabulary (closure applies to grammar too)
2. Describes connector patterns at a more sophisticated level than grammar5:
   - "is a" for taxonomy
   - "can" for capability
   - "not" for negation
   - NEW: "has" for possession/composition ("a molecule has atoms")
   - NEW: "makes" / "causes" for causality ("heat makes ice become water")
   - NEW: "part of" for meronymy ("an electron is part of an atom")
3. Should be 2-3x longer than grammar5 to provide sufficient regularization pressure for the larger space
4. Should include self-referential passages ("a word is a thing that means something. a definition is a group of words that says what a word means.")

---

## TASK 4: RUN AND MEASURE

### Step 1: Direct Evaluation (No Evolution)

Use the best v11 genome parameters on dict18:

```bash
cargo run -p dafhne-engine -- \
    --dict dictionaries/dict18.md \
    --grammar dictionaries/grammar18.md \
    --test dictionaries/dict18_test.md
```

Record: score out of 20, per-question pass/fail, any crashes or panics.

### Step 2: Cross-Validation Matrix

Run every dict against every test to build the generalization matrix:

| Trained On | Tested On | Score |
|-----------|-----------|-------|
| dict5 genome | dict5_test | (known: 20/20) |
| dict5 genome | dict12_test | (known: 15/20) |
| dict5 genome | dict18_test | ? |
| dict18 genome | dict5_test | ? |
| dict18 genome | dict12_test | ? |
| dict18 genome | dict18_test | ? |

The overfitting gap between diagonal and off-diagonal scores is the key metric. If it's zero (as with dict5→dict12), the geometry is truly learning structure.

### Step 3: Evolution on Dict18

```bash
cargo run --release -p dafhne-evolve -- run \
    --dict18 dictionaries/dict18.md --test18 dictionaries/dict18_test.md \
    --grammar dictionaries/grammar18.md \
    --population 50 --generations 50 --results results_v12/ --seed 42
```

Note: the evolve binary may need updating to accept dict18 paths. Check `dafhne-evolve/src/main.rs` for CLI argument handling.

### Step 4: The Scaling Curve

Plot (or just tabulate) the three-point curve:

| Dictionary | Words | Best Fitness | Overfitting Gap |
|-----------|-------|-------------|----------------|
| dict5 | 50 | 1.0000 | 0.00 |
| dict12 | ~400 | 0.7500 | 0.00 |
| dict18 | ~2000 | ? | ? |

Three scenarios and what they mean:
- **Fitness > 0.70, gap ≈ 0**: The geometry scales. Proceed to plasticity (prompt 08).
- **Fitness 0.50-0.70, gap < 0.15**: Partial scaling. Geometry works but resolver needs dict18-specific tuning. Fixable.
- **Fitness < 0.50 or gap > 0.20**: The architecture hits a wall. Need to understand WHY before proceeding. Likely cause: dimensionality insufficient, or force field doesn't converge in reasonable time at 2000 words.

---

## WHAT NOT TO DO

- Do NOT change the engine, resolver, force field, or evolution code. This phase is DATA ONLY.
- Do NOT skip closure verification. An unclosed dict18 will produce garbage results and waste evolution compute.
- Do NOT make dict18 definitions more complex than necessary. Simple definitions in dict18 vocabulary. The geometry needs clean signal.
- Do NOT write dict18_test questions that require world knowledge beyond the dictionary. Every answer must be derivable from the definitions alone.
- Do NOT create dict18 by machine-translating an existing dictionary. Hand-craft it or at minimum hand-verify every definition for closure.

## SUCCESS CRITERIA

| Metric | Minimum | Target | Stretch |
|--------|---------|--------|--------|
| Dict18 word count | 1500 | 2000 | 2500 |
| Closure violations | 0 | 0 | 0 |
| Dict18 fitness (v11 genome) | > 0.40 | > 0.60 | > 0.70 |
| Dict18 fitness (evolved) | > 0.55 | > 0.70 | > 0.80 |
| Overfitting gap (dict5↔dict18) | < 0.25 | < 0.15 | 0.00 |
| Scaling curve shape | Any data | Sublinear decay | Plateau |

## OUTPUT

1. `dictionaries/dict18.md` — closed dictionary, ~2000 words
2. `dictionaries/dict18_test.md` — 20 test questions
3. `dictionaries/grammar18.md` — grammar text in dict18 vocabulary
4. Closure verification report (zero violations)
5. Scaling curve table with all three data points
6. Per-question results for dict18_test
7. Cross-validation matrix (6 combinations)
8. Analysis: which question types fail and why