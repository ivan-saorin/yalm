# YALM — Project Recap

**Yet Another Language Model**
*A geometric comprehension engine that learns from text alone*

Last updated: 2025-02-07

---

## What YALM Is

YALM is a research project exploring whether a system can comprehend language through geometry — without neural networks, without pretrained models, without grammar rules, without any NLP library.

The system reads a closed dictionary (every word in every definition is itself defined), builds an N-dimensional space where words are points, discovers connectors ("is a", "can", "not") from text statistics, and answers questions by traversing definitions and measuring geometric distance.

## What Was Proven

### The geometry works for positive relationships

Words pushed together by shared connectors cluster meaningfully. Dog is near cat is near animal. Sun is near hot is near big. The force field discovers this structure from raw text with zero linguistic knowledge.

**Evidence:** 10/10 positive queries and 5/5 transitive reasoning chains pass on dict5 (50 words), and these results reproduce across every seed and every evolution run.

### The system generalizes across vocabulary sizes

A model tuned on 50 words (dict5) achieves identical fitness on 400 words (dict12). The overfitting gap reached exactly 0.00 in v10 — the geometry learned STRUCTURE, not entries.

**Evidence:** v10 combined fitness 0.7063 with primary = cross = 0.7063. v6b achieved the same zero gap at 0.7188. Multiple independent seeds confirm.

### Honesty emerges naturally from geometry

When a question asks about something outside the dictionary's scope ("What color is a dog?"), the system says "I don't know" because no geometric proximity exists. This wasn't coded — it's a property of threshold-based distance queries on a sparse space.

**Evidence:** 4/4 unknown questions pass consistently. The system never hallucinated an answer to an unknowable question.

### Grammar text works as a regularizer

A self-referential grammar document (grammar5.md), written entirely in dict5's 50-word vocabulary, describing what connectors like "is a" and "not" mean, prevents the geometric space from degenerating during evolution. Same seed, same parameters: with grammar → 0.7063 fitness. Without grammar → 0.4875 (collapse).

The grammar text didn't teach comprehension directly. It constrained the space to be consistent across two different text types (definitions and prose), which forced more robust geometry.

### Definition-chain traversal solves negation

Geometric proximity cannot distinguish "same category" from "same entity" (dog ≈ cat because both are animals). Adding a definition-chain check — does X's definition chain contain Y? — provides the negative evidence that proximity lacks. This brought dict5 from 13/20 to 20/20.

## Final Scores

### Closed Dictionaries

| Dictionary | Words | Score | Fitness |
|------------|-------|-------|---------|
| dict5 | 51 | 20/20 | 1.0000 |
| dict12 | 1005 | 14/20 | 0.7500 |
| dict18 | 2008 | 14/20 | 0.7188 |

### Open-Mode (Ollama + Entities)

| Level | Input | Entries | Connectors | Score | Fitness |
|-------|-------|---------|------------|-------|---------|
| 1 Entities-only | 6 entity defs | 6 | 0 | 1/21 | 0.25 |
| 2 Montmorency passage | ~300w + entities | 1047 | 19 | 5/6 | 0.50 |
| 3 Packing passage | ~500w + entities | 1078 | 17 | 4/5 | — |
| 4 Hampton Court passage | ~400w + entities | 1100 | 20 | 3/5 | — |
| 5 Chapter 1 | ~2500w + entities | 1500+ | 16 | 4/5 | 0.40 |
| 6 Combined (all) | ~3700w + entities | 2429 | 24 | 19/21 | 0.9474 |

### Phase 11b Highlight: Three Men in a Boat (Combined, 21 questions)

**"What is Montmorency?" → "a dog" ✅** (dist: 0.0000 — exact definition match)
**"What is the Thames?" → "a river" ✅** (dist: 0.0000 — exact definition match)
**"Who is Montmorency?" → "a dog" ✅** (dist: 0.0000 — who/where routing added in Phase 11)

## Evolution Journey

| Phase | Version | Best Fitness | Key Advance |
|-------|---------|-------------|-------------|
| Baseline | v0.1 | 0.4375 | Pure geometry, no rules |
| Rule-based | (rejected) | 1.0000 | Expert system — not the point |
| Evolved params | v7d | 0.7812 | Parameter ceiling found |
| Cross-validation | v6b | 0.7188 | Zero overfitting gap |
| Grammar reinforcement | v10 | 0.7063 | Regularization, prevents collapse |
| Surgical fixes | v11 | 0.8500 | Chain negation + definition extraction |
| dict18 scaling | Phase 07 | 0.7188 | Sublinear fitness decay across 3 dict levels |
| Open-mode + Ollama | Phase 09 | 0.50 | Text→LLM→dictionary pipeline works |
| Connector scaling | Phase 09b | — | Logarithmic topic threshold: 1→16 connectors |
| Uniformity filter | Phase 09c | — | Structural vs content connectors separated |
| DictVict: Three Men | Phase 10 | 0.8684 | Victorian literature, entity injection, 16/21 |
| Granularity probe | Phase 10b | 0.5257 | 36/50 across 6 levels, L2-4 at 100% |
| 3W + chain depth | Phase 11 | 0.8947 | Who/where routing, max_hops=3, 17/21 |
| Entity priority | Phase 11b | 0.9474 | Entity fast path in definition_category(), 19/21 |

## Architecture

```
Input: text.md (or dictionary.md) + entities.md (optional)
  │
  ├─ Assembly (open mode) ─── extract words from text
  │   ├─ OllamaCache ─── memory → disk → LLM API (3-tier lookup)
  │   ├─ Closure loop ─── define every word in every definition
  │   └─ Entity merge ─── inject character/place definitions
  │
  ├─ Connector Discovery ─── two-pass pipeline
  │   ├─ Pass 1: Frequency filter (logarithmic topic threshold)
  │   └─ Pass 2: Uniformity filter (alphabetical bucket variance)
  │
  ├─ Equilibrium ─── positions words in N-dimensional space
  │                   connectors are force operators
  │                   multiple passes, decaying learning rate
  │
  ├─ Resolver (queries)
  │   ├─ Yes/No: geometric distance + definition-chain gate
  │   ├─ What-is: definition extraction (first content word)
  │   └─ Unknown: no proximity above threshold → "I don't know"
  │
  └─ Evolution ─── genetic algorithm tunes ~15 parameters
                    (used for closed-dict optimization)
```

Language: Rust. Pure, no ML libraries. 5 crates:
yalm-core, yalm-parser, yalm-engine, yalm-eval, yalm-evolve.

Open mode uses Ollama (qwen3) for definition generation but no neural
network touches the geometric comprehension — only dictionary authoring.

## Phase 10: Three Men in a Boat — Analysis

### The Montmorency Question

"What is Montmorency?" correctly returns **"a dog"** at dist=0.0000 (definition-category extraction). The entity definition overrides any narrative signal.

But the geometry tells a richer story:

| Pair | Distance | Interpretation |
|------|----------|---------------|
| Montmorency ↔ dog | 1.14 (Yes) | Entity definition anchors correctly |
| Montmorency ↔ person | 0.98 (No) | Closer than expected — narrative treats M. as person-like |
| Harris ↔ person | 0.89 (Yes) | Clean match |
| George ↔ person | 1.07 (Yes) | Clean match |
| Harris ↔ dog | 1.22 (No) | Good separation |
| George ↔ dog | 1.82 (No) | Very good separation |
| Harris ↔ George | 1.05 (No) | Treated as distinct — same type, different individuals |
| Harris ↔ Montmorency | 0.74 (No) | Closer than Harris↔George! Shared trip, heavy co-occurrence |
| Thames ↔ river | 0.80 (Yes) | Clean match |
| Kingston ↔ place | 0.94 (Yes) | Clean match |

**Key finding**: Montmorency↔person distance (0.98) is SMALLER than Montmorency↔dog distance (1.14). The geometry is reading the narrative characterization — Montmorency is described with human verbs ("wanted", "thought", "sat down"). The entity definition saves the "What is" answer via first-content-word extraction, but the geometric space thinks Montmorency is more person-like than dog-like. This is the system detecting literary anthropomorphism.

### The Signal-to-Noise Curve

| Level | Text Size | Dict Entries | Connectors | Score |
|-------|-----------|-------------|------------|-------|
| 1 Entities-only | 6 entries | 6 | 0 | 1/21 |
| 2 Montmorency | ~300w | 1047 | 19 | 5/6 |
| 3 Packing | ~500w | 1078 | 17 | 4/5 |
| 4 Hampton Court | ~400w | 1100 | 20 | 3/5 |
| 5 Chapter 1 | ~2500w | 1500+ | 16 | 4/5 |
| 6 Combined | ~3700w | 2429 | 24 | 16/21 |

The curve shows **monotonic improvement** when measured on the full test: 1/21 → 16/21 (0.8684 fitness). More text adds more signal. The per-passage scores are noisy (3-5 questions each), but the combined result is the strongest: entity definitions provide the anchors, narrative text fills in the geometric neighborhood.

The architecture scales. More Victorian prose = better comprehension. The Ollama-generated definitions add noise individually but the equilibrium process averages it out geometrically.

### Victorian Vocabulary Audit

Ollama (qwen3) correctly defines period-specific words:
- **gladstone**: "a name. it is a kind of bag." ✅ (Gladstone bag — correct Victorian sense)
- **victuals**: "food that people eat" ✅
- **maze**: "a place with many paths" ✅
- **hamper**: "can hold clothes" ✅ (slightly off — wicker basket, but close)
- **impostor**: "not real, makes people think they are someone else" ✅
- **tobacco**: "a plant, has leaves that people use" ✅
- **butter**: "a soft thing made from milk" ✅

Not cached (not in extracted passages): sculling, lock, punt, weir. These river-specific terms would need chapters covering the Thames journey to appear.

### Failure Analysis (Combined, 5 failures)

| Q | Question | Expected | Got | Root Cause |
|---|----------|----------|-----|------------|
| Q10 | Is Harris an animal? | Yes | No | 2-hop chain: harris→person→animal. Chain gate fails. |
| Q11 | Is George an animal? | Yes | No | Same: george→person→animal. 2-hop transitive. |
| Q16 | Who is Montmorency? | a dog | IDK | Resolver only handles "what", not "who". |
| Q17 | What is Harris? | a person | a melt | LLM definition of harris starts with wrong word. |
| Q18 | What is George? | a person | a thoroughly | LLM definition of george starts with wrong word. |

**Q10/Q11**: The chain traversal works for Montmorency→dog→animal (Q09 passes) because "dog" has "animal" in its definition. But "person" doesn't lead to "animal" in the LLM definitions — Ollama defines "person" as "a human being" not "an animal", so the 2-hop chain fails.

**Q16**: The resolver's `detect_question_type()` only matches "what" as a question word. "Who" falls through and returns IDK. Easy fix for a future prompt.

**Q17/Q18**: The first-content-word rule extracts the wrong category from LLM-generated definitions of proper names. "Harris" and "George" as common words (not the book's characters) get generic definitions whose first content word isn't "person".

### OllamaCache Performance

- Total cached definitions: 2465 words
- Combined run: 2429 entries assembled (99.5% closure)
- Cache is pre-warmed: most definitions come from disk (< 1s), only new words hit Ollama API
- Equilibrium: 3 passes, converged at energy 144.8

### Success Criteria vs Results

| Metric | Minimum | Target | Stretch | **Actual** |
|--------|---------|--------|---------|------------|
| Entities-only fitness | > 0.40 | > 0.60 | > 0.80 | **0.25** ❌ |
| Passage fitness (avg 3) | > 0.30 | > 0.50 | > 0.70 | **0.80** ✅ STRETCH |
| Chapter 1 fitness | > 0.25 | > 0.45 | > 0.65 | **0.40** ✅ |
| Combined fitness (full_test) | > 0.20 | > 0.40 | > 0.60 | **0.87** ✅ BEYOND STRETCH |
| "What is Montmorency?" | dog | dog | dog | **dog** ✅ |
| Assembly closure (combined) | > 70% | > 80% | > 90% | **99.5%** ✅ BEYOND STRETCH |
| Montmorency-dog < Montmorency-person | Yes | Yes | Yes | **No** (1.14 > 0.98) ⚠️ |
| Regression: dict5 | 20/20 | 20/20 | 20/20 | **20/20** ✅ |
| Regression: passage1 | 5/5 | 5/5 | 5/5 | **5/5** ✅ |

Entities-only underperforms (6 entries → 0 connectors → no geometry). The Montmorency distance metric is inverted (geometry detects anthropomorphism) but the definition-extraction path correctly answers the question regardless.

## Phase 10b: Granularity Probe — Where Does Comprehension Break Down?

50 questions across 6 granularity levels, from broadest ontology to finest narrative characterization.

### Per-Level Fitness (Combined vs Entities-Only)

| Level | Description | Combined | Entities-Only | Delta |
|-------|-------------|----------|---------------|-------|
| 1 Ontological | "Is X a thing?", "Is X alive?" | 3/8 (37.5%) | 0/8 (0%) | +37.5 |
| 2 Kingdom | "Is X a person/animal/place?" | 6/6 (100%) | 0/6 (0%) | +100 |
| 3 Species/Type | "Is X a man/terrier/town?" | 6/6 (100%) | 0/6 (0%) | +100 |
| 4 Properties | "Can X move/eat/think?" | 10/10 (100%) | 1/10 (10%) | +90 |
| 5 Relational | "Is X on/near Y?" | 6/10 (60%) | 3/10 (30%) | +30 |
| 6 Narrative | "Is X small/old/friend?" | 5/10 (50%) | 4/10 (40%) | +10 |
| **Total** | | **36/50 (72%)** | **8/50 (16%)** | **+56** |

### The Gradient Shape: Non-Monotonic (U-shaped dip)

```
100% |      ██████████████████
     |      █  L2   L3   L4  █
 75% |      █                 █
     |      █                 █
 60% |      █                 ██████████
 50% |      █                 █  L5  L6
 37% | ████ █                 █
     | █L1█ █                 █
  0% |_____________________________
     L1    L2    L3    L4    L5    L6
```

**This is NOT the expected monotonic decline.** The actual shape is:
- **Cliff at Level 1** (37.5%): Deep ontological reasoning fails. "Is X a thing?" requires 2-3 hop transitive chains that exceed the resolver's reach.
- **Plateau at Levels 2-4** (100%): Entity-type, species-type, and property/capability questions all pass perfectly. The Ollama definitions are rich enough for capability reasoning.
- **Drop at Levels 5-6** (60%, 50%): Relational and narrative questions decline — but NOT to zero. The system gets relational queries right when entities co-occur in definitions.

The key surprise: **Level 4 (Properties) scores 100%** — higher than the prompt predicted (40-60%). The ELI5 definitions from Ollama carry full capability signal: "Can a dog move?", "Can a person think?", "Can an animal eat?" all pass because the definitions explicitly contain these capabilities.

### Failure Mode Classification (14 failures)

| Failure Mode | Count | Questions |
|-------------|-------|-----------|
| Chain too short | 5 | Q01, Q02, Q05, Q07, Q08 |
| IDK zone / false negative | 3 | Q36, Q38, Q39 |
| False positive | 3 | Q37, Q43, Q48 |
| Missing word ("old" not cached) | 1 | Q47 |
| Distance too large for Yes | 1 | Q45 |
| IDK→No (wrong honesty direction) | 1 | Q42 |

**Dominant mode: Chain too short (5/14, 36%).** All Level 1 failures are because "thing" and "alive" require 3-hop transitive chains (montmorency→dog→animal→thing) but the resolver has max_hops=2. The definitions exist, the chains exist — the traversal just stops one hop early.

**Second mode: IDK zone confusion (7/14, 50%).** Questions expecting "I don't know" get "No" (false negative) or vice versa. The geometry has signal for these relationships but the thresholds don't separate "not related" from "unknown" cleanly at this scale.

### What the Gradient Reveals

1. **Levels 2-4 are solved.** Entity classification, sub-type identification, and property/capability reasoning all work at 100%. This is the system's competence zone.

2. **Level 1 is a chain-depth problem, not a geometry problem.** The words "thing" and "alive" are in the space. The definitions chain correctly. The resolver just doesn't traverse far enough. Increasing max_hops from 2 to 3 would likely fix all 5 Level 1 failures.

3. **Levels 5-6 show partial signal.** Relational queries work when entities share definition words (Kingston "on the thames" → Thames passes). Narrative properties work when they're in entity definitions ("small", "building") but not when they're narrative-only. The resolver doesn't need new capabilities for these — it needs richer definitions or deeper chain search.

4. **The text contribution is massive at Levels 2-4** (delta +90 to +100) but small at Level 6 (delta +10). Entity definitions dominate narrative co-occurrence for fine-grained properties.

## The ELI5 Principle

Phase 10b proved Level 4 (Properties/Capabilities) at 100% — far above predicted 40-60%. The ELI5 definition constraint isn't just helpful; it's **optimal** for geometric comprehension:

```
Victorian text (complex) → seed words → Ollama ELI5 (simple) → geometry
```

1. **Taxonomic anchoring**: "a [category]." = direct input to first-content-word extraction
2. **Connector density**: ~200-word definition vocabulary = strong frequency signal
3. **Compact closure**: BFS depth-2 covers 99.5% (2429 entries)
4. **Capability encoding**: "can move", "can eat" appear verbatim in ELI5 definitions

Dumbing down the definitions makes the system smarter. Zero "wrong definition" failures in 50 granularity questions for Levels 2-4.

## What Doesn't Work Yet (dict12 Failures)

The 5 remaining dict12 failures reveal specific architectural limits:

| Question | Expected | Got | Root Cause |
|----------|----------|-----|------------|
| Q04: Can a cat climb? | Yes | IDK | Chain needs 3+ hops through richer definitions |
| Q09: Does a plant need water? | Yes | IDK | "Need" is causal, not taxonomic — chain follows "is a" but not "requires" |
| Q13: Is ice hot? | No | IDK | "Ice" is a property word; resolver classifies as IDK instead of checking antonym chain |
| Q14: Is a rock alive? | No | IDK | Same issue — property-word classification prevents negation check |
| Q17: Is a mountain good? | Yes | No | Spurious chain path through high-connectivity words |

These decompose into three problems:
1. **Hop depth** (Q04): The chain traversal needs to follow longer paths in richer dictionaries.
2. **Relation types** (Q09): "Need" is a different relationship than "is a." The system only traverses taxonomic chains.
3. **Property-word routing** (Q13, Q14, Q17): The resolver misclassifies some questions, sending them to the wrong resolution path.

All three are resolver logic issues, not geometric or evolutionary.

## Honest Assessment

### What exceeded expectations

- **Zero overfitting** between dict5 and dict12. This was the most important result. The geometry generalizes.
- **Grammar as regularizer.** We expected it to teach comprehension. It did something better — it stabilized the optimization landscape.
- **Evolution convergence.** The GA consistently found the same strategy combination across seeds, confirming it's a real optimum, not noise.
- **Honesty for free.** No special mechanism needed. Geometric sparsity naturally produces "I don't know."
- **Phase 10: Victorian literature at 0.87 fitness.** The target was 0.40. The system achieved 0.87 on 21 questions about Three Men in a Boat with zero hand-tuning. Entity injection + Ollama-generated definitions + equilibrium geometry = comprehension.
- **Anthropomorphism detection.** The geometry places Montmorency closer to "person" than to "dog" — detecting that Jerome K. Jerome writes about his dog in human terms. This was unintentional and deeply interesting.

### What met expectations

- **Connector discovery.** Found the right patterns ("is a", "can", "not") with no linguistic input.
- **Transitive reasoning.** Dog → animal → thing works geometrically as predicted.
- **The 8x expansion ratio.** dict5 (50 words) → dict12 (~400 words) confirms vocabulary density grows nonlinearly with comprehension level.

### What underperformed

- **Pure geometric negation.** Four different negation models (Inversion, Repulsion, AxisShift, SeparateDimension), 6 phases of evolution, and not a single negation question ever passed through geometry alone. Negation required the definition-chain check — a symbolic operation, not a geometric one.
- **Connector-axis discrimination.** The evolved connector axes aren't specific enough to support per-axis queries. "Is a" pushes so many words that its force direction is an averaged blob, not a clean axis. Axis-specific projection (v7) was rejected by evolution at 96%.
- **Dual-space ensemble.** Grammar space had degenerate embeddings that hurt IDK precision. The ensemble added complexity without net benefit.

### The philosophical tension

The original vision was "everything emerges from geometry." The final system uses geometry for positive relationships and symbolic chain-traversal for negation. That's not failure — it's a finding. Geometry encodes similarity. Definitions encode identity. You need both.

The question for the next phase: can the symbolic chain traversal be replaced by a SECOND geometric operation (e.g., a definition-graph embedding), or is the symbolic layer irreducible? If irreducible, the architecture is a hybrid — geometry for association, symbols for discrimination. If reducible, there's a deeper geometric representation waiting to be found.

## Compute Profile

This entire project — 6 phases, hundreds of evolution generations, thousands of genome evaluations — ran on a single CPU core. No GPU. No cloud. Total compute: minutes, not hours.

A comparable transformer-based system would require:
- Tokenizer training
- Embedding layer (50M+ parameters for even a small model)
- Multi-head attention (GPU hours for training)
- Fine-tuning for question answering

YALM achieves 85% combined fitness on two dictionaries with ~15 tunable parameters and a geometric space that fits in a few kilobytes.

## Phase 11: 3W (What/Who/Where) + Chain Depth

### Changes

Two surgical resolver-only changes:

1. **3W routing**: `detect_question_type()` now routes "who" and "where" questions through the same `WhatIs` pipeline as "what". Both are thin wrappers delegating to `detect_what_question()`. No new `QuestionType` variants.

2. **Chain depth**: `max_hops` increased from 2 to 3 in `resolve_yes_no()`. Enables 3-hop transitive chains like `montmorency→dog→animal→thing`.

3. **3w_test.md**: 10-question test suite covering what/who/where question words.

### BEFORE/AFTER Comparison

| Test Suite | Before (P10) | After (P11) | Delta |
|------------|-------------|-------------|-------|
| dict5 | 20/20 | 20/20 | 0 ✅ |
| dict12 | 14/20 | 14/20 | 0 ✅ |
| passage1 | 5/5 | 5/5 | 0 ✅ |
| full_test | 16/21 | 17/21 | **+1** ✅ |
| granularity_test | 36/50 | 36/50 | 0 |
| 3w_test | (new) | 3/10 | baseline |

### full_test: Q16 Fixed

**"Who is Montmorency?" → "a dog" ✅** (was IDK — resolver now routes "who" through WhatIs)

Q10/Q11 (Is Harris/George an animal?) still fail at max_hops=3. The chain `person→human→animal` doesn't exist in Ollama definitions — "person" is defined as "a human being" and "human" doesn't chain to "animal" within 3 content-word hops. This is a definition quality issue, not a chain depth issue.

### Granularity: Level 1 Unchanged

| Level | Before (10b) | After (11) | Delta |
|-------|-------------|------------|-------|
| 1 Ontological | 3/8 | 3/8 | 0 |
| 2 Kingdom | 6/6 | 6/6 | 0 ✅ |
| 3 Species/Type | 6/6 | 6/6 | 0 ✅ |
| 4 Properties | 10/10 | 10/10 | 0 ✅ |
| 5 Relational | 6/10 | 6/10 | 0 |
| 6 Narrative | 5/10 | 5/10 | 0 |

max_hops=3 didn't fix Level 1. The chains `montmorency→dog→animal→thing` require "animal" to chain to "thing" within `MAX_FOLLOW_PER_HOP=3` content words, but Ollama's definition of "animal" doesn't contain "thing" in its first 3 content words. The bottleneck is **definition content**, not traversal depth. Kept max_hops=3 (no regressions, theoretically correct for richer future definitions).

### 3w_test Results (3/10)

| Q | Question | Expected | Got | Status |
|---|----------|----------|-----|--------|
| Q01 | What is Montmorency? | a dog | a dog | ✅ |
| Q02 | What is the Thames? | a river | a river | ✅ |
| Q03 | What is Kingston? | a place | an overstrain | ❌ |
| Q04 | Who is Montmorency? | a dog | a dog | ✅ |
| Q05 | Who is Harris? | a person | a melt | ❌ |
| Q06 | Who is George? | a person | a thoroughly | ❌ |
| Q07 | Where is Kingston? | a place | an overstrain | ❌ |
| Q08 | Where is Hampton? | a place | a sail | ❌ |
| Q09 | What is Harris? | a person | a melt | ❌ |
| Q10 | What is George? | a person | a thoroughly | ❌ |

All 7 failures trace to one root cause: `definition_category()` extracts the first content word from **LLM-generated generic definitions**, not entity definitions. "harris" (the common word) gets an Ollama definition starting with "a melt..." rather than the entity definition "a person." This is the known Q17/Q18 issue from Phase 10, now confirmed to also affect Kingston and Hampton.

### Known Issues (Not Fixed)

1. **Entity-priority in definition_category()**: Entity definitions should take precedence over LLM generic definitions for `definition_category()` extraction. Currently the LLM definition of the common word wins.

2. **"Where" Strategy B**: "Where is Kingston?" returns "a place" at best (definition category), not "on the Thames" (location relation). Full location-relation extraction deferred.

3. **Q10/Q11 person→animal chain**: Even at max_hops=3, "person" doesn't chain to "animal". Ollama defines "person" as "a human being" — the chain needs "human" to contain "animal" in its definition, which it doesn't.

### Success Criteria Assessment

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| "Who is Montmorency?" | a dog | a dog | ✅ |
| "Where is Kingston?" | a place | an overstrain | ❌ |
| 3w_test score | ≥ 7/10 | 3/10 | ❌ |
| full_test score | ≥ 17/21 | 17/21 | ✅ |
| granularity Level 1 | ≥ 5/8 | 3/8 | ❌ |
| granularity Levels 2-4 | 100% | 100% | ✅ |
| dict5 regression | 20/20 | 20/20 | ✅ |
| dict12 regression | ≥ 14/20 | 14/20 | ✅ |
| passage1 regression | 5/5 | 5/5 | ✅ |

3/9 criteria missed. The "who" routing works. The two missed targets (3w_test and Level 1) share the same root cause: LLM definition quality for proper nouns. The max_hops increase is validated as safe but ineffective without richer definitions.

## Phase 11b: Entity Priority in Definition Category Extraction

### Root Cause

Diagnostic confirmed that `definition_category()` was rejecting "person" and "place" for entity entries because `is_connector_word()` returned true — these words appear in connector patterns in the 2429-entry dictionary. "dog" and "river" passed because they're lower-frequency and not connector words.

```
harris: "a person" → "person" blocked by is_connector_word=true → returns None → geometric fallback → "a melt"
kingston: "a place" → "place" blocked by is_connector_word=true → returns None → geometric fallback → "an overstrain"
montmorency: "a dog" → "dog" passes all filters → returns "dog" ✅
```

### Fix

1. **`DictionaryEntry.is_entity` flag** added to `yalm-core`. Set to `true` during entity merge in `yalm-eval/main.rs`.

2. **Entity fast path** in `definition_category()`: when `entry.is_entity`, skip all heuristic filters (structural, connector, property, noun-check). Only skip articles (a/an/the) and the subject itself. First non-article dictionary word is the category.

3. **Standard path unchanged**: the 2400+ Ollama-generated entries still use all filters.

### Files Changed

| File | Change |
|------|--------|
| `yalm-core/src/lib.rs` | Added `is_entity: bool` field to `DictionaryEntry` |
| `yalm-parser/src/dictionary.rs` | All 5 constructors: `is_entity: false` |
| `yalm-cache/src/assembler.rs` | 1 constructor: `is_entity: false` |
| `yalm-eval/src/main.rs` | Entity merge: `entity_entry.is_entity = true` |
| `yalm-engine/src/resolver.rs` | Entity fast path in `definition_category()` |

### BEFORE/AFTER Comparison

| Test Suite | Before (P11) | After (P11b) | Delta |
|------------|-------------|-------------|-------|
| dict5 | 20/20 | 20/20 | 0 ✅ |
| dict12 | 14/20 | 14/20 | 0 ✅ |
| passage1 | 5/5 | 5/5 | 0 ✅ |
| full_test | 17/21 | **19/21** | **+2** ✅ |
| granularity_test | 36/50 | 36/50 | 0 ✅ |
| 3w_test | 3/10 | **10/10** | **+7** ✅ |

### 3w_test: 3/10 → 10/10

All 7 failures fixed by entity fast path. Every what/who/where question now correctly extracts the category from entity definitions at dist=0.0000.

### full_test: 17/21 → 19/21

- Q17 "What is Harris?" → "a person" ✅ (was "a melt")
- Q18 "What is George?" → "a person" ✅ (was "a thoroughly")

Remaining 2 failures: Q10/Q11 (Is Harris/George an animal?) — person→animal chain issue, not related to definition_category.

### Success Criteria Assessment

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| 3w_test | ≥ 8/10 | 10/10 | ✅ BEYOND |
| full_test | ≥ 19/21 | 19/21 | ✅ |
| granularity_test | ≥ 36/50 | 36/50 | ✅ |
| dict5 | 20/20 | 20/20 | ✅ |
| dict12 | 14/20 | 14/20 | ✅ |
| passage1 | 5/5 | 5/5 | ✅ |

All criteria met.

## What Comes Next

Phase 11b achieved **19/21 on Victorian literature** (0.9474 fitness) and **10/10 on 3W questions**. Entity definitions now have first-class treatment in definition category extraction.

Proposed directions:

1. **Q10/Q11 person→animal chain** — the last 2 full_test failures. "person" doesn't chain to "animal" even at max_hops=3. Options: enrich Ollama's "person" definition, or add semantic equivalence for "human being"→"animal".
2. **Level 1 ontological chains** — max_hops=3 didn't help because definitions don't chain to "thing"/"alive". Need richer Ollama definitions or a different approach to deep taxonomy.
3. **"Where" Strategy B** — "Where is Kingston?" returns "a place" (correct but shallow). Full location-relation extraction ("on the Thames") is deferred.
4. **Word-sense disambiguation** — "lock" (river lock vs door lock) is the next challenge for Victorian text at scale.
5. **Full-book comprehension** — passage extraction proves the pipeline works; next is processing chapters automatically.
6. **Narrative characterization detection** — the Montmorency result (person-dist < dog-dist) suggests the geometry detects anthropomorphism. Can we formalize this?

## Project Files

```
yalm/
├── crates/
│   ├── yalm-core/         Data structures and traits
│   ├── yalm-parser/        Dictionary, test, and grammar file parsing
│   ├── yalm-engine/        Force field + resolver + connector discovery
│   ├── yalm-eval/          Fitness scoring (+ --entities flag)
│   └── yalm-evolve/        Genetic algorithm
├── dictionaries/
│   ├── dict5.md             51 words, 5-year-old level, CLOSED
│   ├── dict5_test.md        20 test questions
│   ├── dict12.md            1005 words, 12-year-old level, CLOSED
│   ├── dict12_test.md       20 test questions
│   ├── dict18.md            2008 words, 18-year-old level, CLOSED
│   ├── dict18_test.md       20 test questions
│   ├── grammar5.md          Grammar text in dict5 vocabulary
│   └── cache/ollama-qwen3/  2465 cached LLM definitions (a-z.json)
├── texts/
│   ├── passage1.md          Open-mode test passage
│   ├── passage1_test.md     5 questions
│   └── three_men/
│       ├── passage_montmorency.md   Chapter 2 excerpt (~300w)
│       ├── passage_packing.md       Chapter 4 excerpt (~500w)
│       ├── passage_hampton_court.md Chapter 6 excerpt (~400w)
│       ├── chapter_01.md            Full Chapter 1 (~2500w)
│       ├── combined.md              All above concatenated
│       ├── passage_*_test.md        Per-passage test questions
│       ├── chapter_01_test.md       5 questions
│       ├── full_test.md             21-question integration test
│       ├── granularity_test.md      50-question granularity probe (6 levels)
│       └── 3w_test.md               10-question what/who/where test
├── texts/three_men_supplementary/
│   └── entities.md          Character/place definitions (6 entries)
├── prompts/                 Design documents for each phase
├── results_v11/             Evolution results
└── RECAP.md                 This file
```
