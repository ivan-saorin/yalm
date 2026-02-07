# PROMPT 04 — Phase 5 Handoff: Property Queries, Negation, and Hierarchical Space

## CONTEXT

You are continuing development of YALM (Yet Another Language Model), a geometric comprehension engine written in pure Rust. YALM reads closed dictionaries, discovers connectors from text, builds N-dimensional geometric spaces, and answers questions via proximity queries. An evolutionary system tunes its parameters and strategy choices.

**Phase 4 is complete.** All targets met:

| Metric | Phase 4 Target | Achieved (v6b seed 123) |
|--------|---------------|------------------------|
| dict5 primary fitness | > 0.70 | 0.7188 |
| dict12 cross fitness | > 0.40 | 0.7188 |
| Combined fitness | > 0.65 | 0.7188 |
| Overfitting gap | < 0.25 | **0.00** (perfectly balanced) |
| Strategy convergence | meaningful | Gravitational + Spherical + MutualInfo + Weighted converged across 90%+ of population |

**The plateau is architectural, not parametric.** Evolution ran 50 generations × 50 population with adaptive mutation and cannot break past 0.72. The bottleneck is specific question categories that ALL genomes fail.

---

## PROJECT STRUCTURE

```
D:\workspace\projects\yalm\
├── crates/
│   ├── yalm-core/       Core types: EngineParams, GeometricSpace, Answer, TestQuestion
│   ├── yalm-parser/     Dictionary/test file parsing, tokenization, stemming
│   ├── yalm-engine/     Engine, resolver.rs (question answering), strategy.rs (18 variants)
│   ├── yalm-eval/       Evaluation, fitness scoring (accuracy × 0.5 + honesty × 0.5)
│   └── yalm-evolve/     Genetic algorithm: genome, population, mutation, crossover, runner
├── dictionaries/
│   ├── dict5.md          50 words, 5-year-old level, fully closed
│   ├── dict5_test.md     20 questions: Q01-Q05 direct, Q06-Q10 transitive, Q11-Q14 negation, Q15-Q18 unknown, Q19-Q20 property
│   ├── dict12.md         Larger dictionary, 12-year-old level
│   └── dict12_test.md    20 questions, same categories
├── prompts/
│   ├── 02_geometric_comprehension_engine.md   Engine specification
│   ├── 03_evolution_self_improvement.md        Evolution system specification
│   └── 04_phase5_handoff.md                   THIS FILE
├── results_v6/           Seed 42 run: best 0.7063
└── results_v6b/          Seed 123 run: best 0.7188 (peak genome ID 2034)
```

---

## WHAT WORKS (DO NOT BREAK)

### Converged Strategy Configuration
Evolution consistently selects:
- **ForceFunction::Gravitational** (F = m/d², 96% of population)
- **SpaceInitialization::Spherical** (unit sphere, 92%)
- **ConnectorDetection::MutualInformation** (92%)
- **MultiConnectorHandling::Weighted** (94%)
- **NegationModel**: Mixed — Inversion wins on combined fitness, AxisShift dominates population count (27/50)

### Best Genome Parameters (ID 2034, gen 44, fitness 0.7188)
```json
{
  "dimensions": 30,
  "learning_passes": 28,
  "force_magnitude": 0.997,
  "force_decay": 0.99,
  "connector_min_frequency": 2,
  "connector_max_length": 4,
  "yes_threshold": 0.321,
  "no_threshold": 0.586,
  "negation_inversion": -0.220,
  "bidirectional_force": 0.158
}
```

### Question Results (best genome, dict5)
**PASS (10/20):** Q03, Q06, Q07, Q08, Q09, Q10, Q15, Q16, Q17, Q18
**FAIL (10/20):** Q01, Q02, Q04, Q05, Q11, Q12, Q13, Q14, Q19, Q20

Pattern: Transitive reasoning (Q06-Q10) and unknown detection (Q15-Q18) work well. Direct lookup (Q01-Q05), negation (Q11-Q14), and property queries (Q19-Q20) fail.

### Evolution Mechanics (recently improved, working well)
- Fitness formula: `0.6 × primary + 0.4 × cross` with overfitting penalty `0.5 × (gap - 0.15)` when gap > 0.15
- Adaptive mutation: stall detection (< 0.005 improvement), rate escalation 1.0× → 1.5× → 2.5×
- Elitism 4, tournament size 5, cross-validation threshold >= 0.4
- Tightened parameter ranges: yes_threshold (0.05, 0.35), no_threshold (0.15, 0.6)

---

## THREE PRIORITIES (in order)

### Priority 1: Property Queries (Q19-Q20) — Axis-Specific Nearest Neighbor

**Impact:** 2 questions × 2 dicts = potential +0.10 fitness

**The problem:**
- Q19: "What is a dog?" → expected "an animal" → currently returns "I don't know"
- Q20: "What is a person?" → expected "an animal" → currently returns "I don't know"
- dict12 equivalents: "What is a cat?" (→ "a mammal"), "What is a wolf?" (→ "an animal")

**Root cause:** `resolve_what_is()` in `resolver.rs` (lines 621-682) finds the nearest content word to the subject using plain euclidean distance. But "dog" is not necessarily closest to "animal" in the full space — it might be closer to "cat" or "food" or other words it co-occurs with. The function needs to understand that "What is X?" asks for the *category* of X, which is encoded in the "is a" connector axis.

**Suggested approach:**
Instead of finding the nearest word in full euclidean space, project the subject word onto the "is a" connector axis and find the nearest content word *along that axis*. The "is a" connector should encode the category relationship.

```
Current: nearest_word(dog, all_dims) → might return "cat" (co-occurrence neighbor)
Desired: nearest_word(dog, projected_onto_is_a_axis) → should return "animal"
```

**Key code location:** `resolve_what_is()` at `crates/yalm-engine/src/resolver.rs:621-682`

The function already has axis-projection infrastructure for Repulsion model (lines 634-640, 649-650). The same pattern can be adapted: find the "is a" connector, project onto its force_direction, find nearest along that axis.

**Critical detail about how connectors work:**
Connectors have a `force_direction: Vec<f64>` that encodes the geometric axis of the relationship. The "is" connector (pattern: `["is"]` or `["is", "a"]`) should have a force_direction that separates categories. Use `projected_distance()` (already implemented) to measure distance along this axis.

**What NOT to do:**
- Don't change the yes/no resolver — it works for 10/20 questions
- Don't change how distances are normalized for threshold comparison in the non-Repulsion path — raw distances work (this was a painful debugging session; ratio normalization broke what-is)

---

### Priority 2: Negation (Q11-Q14) — SeparateDimension or Identity Approach

**Impact:** 4 questions × 2 dicts = potential +0.20 fitness (biggest payoff, hardest)

**The problem:**
- Q11: "Is a dog a cat?" → No (currently: wrong answer)
- Q12: "Is the sun cold?" → No (currently: wrong answer)
- Q13: "Is a ball an animal?" → No (currently: wrong answer)
- Q14: "Is the sun small?" → No (currently: wrong answer)

These require understanding that:
- dog ≠ cat (different entities, same category)
- sun is hot, cold = not hot, therefore sun is not cold
- ball = "a small thing" with no path to "animal"
- sun = "a big hot thing", big = not small

**Current negation models (all 4 exist in strategy.rs):**

1. **Inversion** — flips force direction vector. Best combined fitness but fails negation.
2. **Repulsion** — pushes "not X" away from X. Has specialized resolver paths in resolve_yes_no (lines 497-522) with axis projection. Works mechanically but doesn't produce good fitness.
3. **AxisShift** — moves to opposite end of same axis. Dominates population (27/50) but not in best genome.
4. **SeparateDimension** — reserves dim 0 for negation signal. Has specialized resolver at line 533: `resolve_yes_no_separate_dimension()`. Uses dim0 for negated questions, dims 1..N for non-negated.

**Key insight from evolution:** The population is split between AxisShift (27/50) and Inversion (best genome). Neither solves Q11-Q14. This suggests the resolver logic needs improvement, not just the training strategy.

**Suggested approaches:**

*Approach A: Fix SeparateDimension resolver*
The resolver exists (line 533+) but may have threshold issues. The dim0 signal might be too weak relative to other dimensions. Consider: instead of using only dim0 for negation, use a threshold on dim0 combined with full-space distance.

*Approach B: Semantic distance for "Is X a Y?"*
For "Is a dog a cat?" — the system needs to distinguish "dog and cat are both animals" from "dog is a cat". Currently, if dog is close to cat in space (they're both animals), the system says Yes. Fix: require the path through the "is a" connector axis specifically, not just proximity.

*Approach C: Identity dimension*
Add a mechanism where each entity has a unique identity. "Is a dog a cat?" fails because dog's identity ≠ cat's identity, even though they're geometrically close. This could be a reserved dimension or a lookup table.

**Key code locations:**
- `resolve_yes_no()`: `crates/yalm-engine/src/resolver.rs:470-531`
- `resolve_yes_no_separate_dimension()`: `crates/yalm-engine/src/resolver.rs:533-587`
- `decide_yes_no()`: `crates/yalm-engine/src/resolver.rs:589-608`
- NegationModel enum: `crates/yalm-engine/src/strategy.rs:122-149`
- Force application for negation: `crates/yalm-engine/src/force_field.rs`

---

### Priority 3: Hierarchical Space — Train Core → Expand

**Impact:** Not needed yet (overfitting gap is 0.0), but essential for dict18 (university level, 2000+ words)

**Concept:** Train on dict5 (50 words) first to establish core relationships, then freeze those positions and expand to dict12 (or dict18) around them. This prevents the larger vocabulary from disrupting fundamental relationships.

**Not urgent because:** Cross-validation gap is literally zero. The current system generalizes perfectly from dict5 to dict12. But this will matter when scaling to much larger dictionaries where the geometric space gets crowded.

---

## RESOLVER ARCHITECTURE (key file: resolver.rs)

The resolver is the most important file for Priorities 1 and 2. Here's the dispatch flow:

```
query() → parse question → classify type
  ├── Yes/No question → resolve_yes_no()
  │     ├── SeparateDimension → resolve_yes_no_separate_dimension()
  │     ├── Repulsion → axis-projected distance with inverted thresholds
  │     └── Default → euclidean + ratio_normalize + decide_yes_no()
  ├── What-is question → resolve_what_is()
  │     ├── Repulsion → euclidean_distance_excluding_axis + ratio threshold
  │     └── Default → plain euclidean + raw distance threshold
  └── Unknown → check if subject+object exist, answer IDontKnow if appropriate
```

**Distance functions available (all in resolver.rs):**
- `euclidean_distance(a, b)` — standard L2
- `euclidean_distance_excluding_axis(a, b, axis)` — L2 after projecting out one axis
- `projected_distance(a, b, axis)` — distance along a single axis only
- `ratio_normalize(distance, mean)` — d/mean, gives scale-invariant comparison
- `compute_distance_stats_excluding_axis(space, axis)` → (mean, std_dev)
- `compute_axis_distance_stats(space, axis)` → (mean, std_dev)
- `find_negation_connector(space)` — finds connector with "not" in pattern

**Threshold logic (decide_yes_no, line 589-608):**
```
if normalized < yes_threshold → Yes
if normalized > no_threshold → IDontKnow
else → No (in between = uncertain = negative)
```
For negated questions, Yes/No are swapped.

---

## CRITICAL LESSONS LEARNED (avoid repeating these mistakes)

### 1. What-is threshold: use RAW distances for non-Repulsion models
The what-is resolver for non-Repulsion models uses `best_distance` (raw euclidean) for threshold comparison, NOT ratio-normalized. Changing this to ratio-normalized caused a severe regression (fitness 0.5625 → 0.3125) because dividing by mean≈22 made all distances ≈0.05, causing everything to get word answers instead of "I don't know". This was a multi-hour debugging session. **Do not normalize what-is distances for non-Repulsion paths.**

### 2. Z-score normalization doesn't work for small spaces
Attempted `(d - mean) / std_dev` normalization. Failed because with only 50 words, the distribution is too narrow and scores compress to a tiny range near 0. Ratio normalization (`d / mean`) works much better.

### 3. Evolution can't fix resolver bugs
If the resolver logic is fundamentally wrong (e.g., nearest-neighbor in full space for "what is" queries), no amount of parameter tuning will fix it. The bottleneck questions (Q19-Q20, Q11-Q14) require architectural changes to the resolver, not more evolution runs.

### 4. Cached DistanceStats are correct
`GeometricSpace::compute_distance_stats()` is called after training and cached in `distance_stats: Option<DistanceStats>`. These match on-the-fly computation. No need to debug this again.

---

## HOW TO BUILD AND TEST

```bash
# Build everything
cargo build --workspace

# Run all tests
cargo test --workspace

# Quick fitness check (default params, no evolution)
cargo run -p yalm-engine -- --dict dictionaries/dict5.md --test dictionaries/dict5_test.md

# Run evolution
cargo run --release -p yalm-evolve -- run \
  --dict5 dictionaries/dict5.md --test5 dictionaries/dict5_test.md \
  --dict12 dictionaries/dict12.md --test12 dictionaries/dict12_test.md \
  --population 50 --generations 50 --results results_v7/ --seed 42

# Evaluate best genome from a run
cargo run --release -p yalm-evolve -- run-best results_v6b/ \
  --dict dictionaries/dict5.md --test dictionaries/dict5_test.md

# Analyze a generation
cargo run -p yalm-evolve -- analyze results_v6b/gen_049/ --dict5 dictionaries/dict5.md
```

---

## DICT5 KEY DEFINITIONS (relevant to failing questions)

For property queries (Q19-Q20):
- **dog** — "an animal. it can make sound. it can live with a person."
- **person** — "an animal that can make things and give names."
- **animal** — "a thing that lives. it can move. it can eat. it can feel."

For negation (Q11-Q14):
- **not** — "not yes is no. not good is bad. not big is small."
- **cold** — "you feel cold. not hot. water is cold."
- **small** — "not big. the cat is small."
- **big** — "not small. the sun is big."
- **ball** — "a small thing. it can move. you can give it."
- **sun** — "a big hot thing that is up."
- **cat** — "a small animal. it can move with not-sound."

The connector "is a" (pattern: ["is"] or ["is", "a"]) should encode the category axis: dog→animal, person→animal, cat→animal, ball→thing, sun→thing.

---

## EVOLUTION RESULTS SUMMARY

| Run | Seed | Best Fitness | Best Genome | Primary | Cross | Gap |
|-----|------|-------------|-------------|---------|-------|-----|
| v4 | 42 | 0.6562 | — | 0.6562 | 0.5312 | 0.125 |
| v5 | 42 | 0.6594 | 535 | 0.7812 | 0.3750 | 0.406 |
| v6 | 42 | 0.7063 | 1881 | ~0.69 | ~0.72 | ~-0.03 |
| v6b | 123 | **0.7188** | **2034** | **0.7188** | **0.7188** | **0.000** |

Key progression: v4→v5 improved primary but overfit badly. v5→v6/v6b fixed overfitting via 60/40 weighting + penalty. The system now generalizes perfectly but is architecturally limited.

---

## PHASE 5 SUCCESS CRITERIA

| Metric | Current | Target | Stretch |
|--------|---------|--------|---------|
| dict5 primary | 0.7188 | > 0.80 | > 0.90 |
| dict12 cross | 0.7188 | > 0.75 | > 0.85 |
| Q19-Q20 (property) | 0/2 | 2/2 | 2/2 both dicts |
| Q11-Q14 (negation) | 0/4 | 2/4 | 4/4 both dicts |
| Overfitting gap | 0.00 | < 0.15 | < 0.10 |

Getting Q19-Q20 right adds +0.0625 to accuracy per question (1/16 of the answerable pool). Getting Q11-Q14 right adds +0.0625 each. Fixing all 6 could push fitness from 0.72 to ~0.90+.
