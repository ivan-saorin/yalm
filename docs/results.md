# DAPHNE Results

> Comprehensive scores across all test suites, dictionary sizes, and phases.

---

## Closed Dictionaries

### Single-Space Performance

| Dictionary | Words | Score | Fitness | Build Mode |
|-----------|------:|------:|--------:|------------|
| dict5 | 51 | **20/20** | 1.00 | Equilibrium |
| dict12 | 1005 | **14/20** | 0.75 | Equilibrium |
| dict18 | 2008 | **14/20** | 0.72 | Equilibrium |

The scaling curve is **sublinear**: 51 → 1005 words costs 6 questions, but 1005 → 2008 costs only 0 more. The geometry holds under 40x vocabulary expansion.

### dict5 Question Breakdown (20/20)

| # | Question | Type | Answer | Status |
|---|----------|------|--------|--------|
| Q01 | Is a dog an animal? | Yes/No | Yes | Pass |
| Q02 | Is a cat an animal? | Yes/No | Yes | Pass |
| Q03 | Is the sun hot? | Yes/No | Yes | Pass |
| Q04 | Is a ball a thing? | Yes/No | Yes | Pass |
| Q05 | Is a dog a thing? | Yes/No (transitive) | Yes | Pass |
| Q06 | Can a dog eat? | Yes/No | Yes | Pass |
| Q07 | Can a person move? | Yes/No | Yes | Pass |
| Q08 | Is a dog a cat? | Yes/No (negative) | No | Pass |
| Q09 | Is the sun cold? | Yes/No (negative) | No | Pass |
| Q10 | Is a ball an animal? | Yes/No (negative) | No | Pass |
| Q11 | Is a cat a dog? | Yes/No (negative) | No | Pass |
| Q12 | What is a dog? | WhatIs | an animal | Pass |
| Q13 | What is a cat? | WhatIs | an animal | Pass |
| Q14 | Is water food? | Yes/No (negative) | No | Pass |
| Q15 | What color is a dog? | Unknown | I don't know | Pass |
| Q16 | How tall is a cat? | Unknown | I don't know | Pass |
| Q17 | What is a dog's name? | Unknown | I don't know | Pass |
| Q18 | Where does a dog live? | Unknown | I don't know | Pass |
| Q19 | Is a dog an animal and a thing? | Boolean AND | Yes | Pass |
| Q20 | Is a dog a cat or an animal? | Boolean OR | Yes | Pass |

---

## Open Mode: Three Men in a Boat

### Scaling by Text Input

| Level | Input | Dict Entries | Connectors | Score | Fitness |
|-------|-------|-------------|-----------|------:|--------:|
| 1 | 6 entity defs only | 6 | 0 | 1/21 | 0.25 |
| 2 | Montmorency passage (~300w) | 1047 | 19 | 5/6 | 0.50 |
| 3 | Packing passage (~500w) | 1078 | 17 | 4/5 | — |
| 4 | Hampton Court passage (~400w) | 1100 | 20 | 3/5 | — |
| 5 | Chapter 1 (~2500w) | 1500+ | 16 | 4/5 | 0.40 |
| 6 | Combined (all passages) | 2429 | 24 | **19/21** | 0.95 |

More text = more signal. Combined score (19/21) exceeds any individual passage. Entity definitions anchor proper nouns; Ollama-generated ELI5 definitions fill in the geometry.

### Test Suite Breakdown (Open Mode)

| Test Suite | Questions | Score | Description |
|-----------|----------|------:|-------------|
| full_test | 21 | **19/21** | Integration test across all question types |
| 3w_test | 10 | **10/10** | What/Who/Where questions |
| bool_test | 5 | **5/5** | Boolean AND/OR compound queries |
| 2w_test | 5 | **5/5** | When/Why reasoning questions |

**Total open-mode: 39/41 correct** across all test suites.

### Remaining Open-Mode Failures

| Q | Question | Expected | Got | Root Cause |
|---|----------|----------|-----|------------|
| Q10 | Is Harris an animal? | Yes | No | person → human → animal chain fails at 3 hops |
| Q11 | Is George an animal? | Yes | No | Same: person → animal chain not in LLM definitions |

Both failures trace to Ollama defining "person" as "a human being" — the chain from "human" to "animal" doesn't exist within 3 content-word hops.

---

## Multi-Space Results (Phase 16-19)

### Phase 16: Three-Space (MATH + GRAMMAR + TASK)

25 questions, 3 spaces (no CONTENT space yet):

| Category | Score | Questions |
|----------|------:|-----------|
| MATH queries | 8/10 | Arithmetic, counting, ordinal |
| GRAMMAR queries | 10/10 | Part-of-speech, sentence structure |
| Cross-space | 4/5 | Queries spanning math + grammar |
| **Total** | **22/25** | 88% |

### Phase 17: Four-Space (+ CONTENT)

40 questions, 4 spaces:

| Category | Score | Questions |
|----------|------:|-----------|
| CONTENT queries | 16/20 | Animals, properties, classic dict5 |
| MATH queries | 5/5 | Arithmetic operations |
| Cross-space | 10/10 | Content+math, content+grammar |
| Full pipeline | 4/5 | Complex multi-domain routing |
| **Total** | **35/40** | 87.5% |

### Phase 18: Five-Space (+ SELF)

50 questions, 5 spaces (unified_test.md):

| Category | Score | Questions |
|----------|------:|-----------|
| CONTENT | 16/20 | Physical world queries |
| MATH | 5/5 | Number operations |
| GRAMMAR | 5/5 | Language structure |
| SELF | 10/10 | Identity queries ("What is DAPHNE?") |
| Cross-space | 5/5 | Multi-domain queries |
| Full pipeline | 4/5 | Complex routing |
| **Total** | **45/50** | 90% |

### Known Multi-Space Failures (5/50)

| # | Question | Expected | Got | Root Cause |
|---|----------|----------|-----|------------|
| Q13 | Is three more than one? | Yes | No | Ordinal comparison is not geometric |
| Q18 | What is a sentence made of? | words | IDK | WhatIs extraction fails in small grammar space |
| Q20 | What does a verb describe? | an action | IDK | Same: grammar space WhatIs extraction |
| Q25 | What kind of number is "seven"? | an odd number | IDK | Quoted phrases in kind queries |
| Q36 | How many legs does a dog have? | four | IDK | 3-part context pipeline (stretch goal) |

---

## Bootstrap Loop Results (Phase 19)

| Level | New Connectors | Lost Connectors | Generated Sentences |
|-------|---------------|----------------|--------------------:|
| Level 1 | 4 | 0 | 87 |
| Level 2 | 0 | 0 | 91 |

Converged at Level 2. The 4 new connectors discovered at Level 1:
- "is not" — emerged from negation sentences in describe() output
- 3 additional patterns from enriched sentence structure

Dictionaries remain immutable. Only the connector set evolved.

---

## Granularity Probe (Phase 10b)

50 questions across 6 levels of granularity, from broadest ontology to finest narrative characterization:

| Level | Description | Score | Status |
|-------|-------------|------:|--------|
| L1 | Ontological ("Is X a thing?") | 3/8 (37.5%) | Chain too short |
| L2 | Kingdom ("Is X a person?") | 6/6 (100%) | Solved |
| L3 | Species/Type ("Is X a terrier?") | 6/6 (100%) | Solved |
| L4 | Properties ("Can X move?") | 10/10 (100%) | Solved |
| L5 | Relational ("Is X on/near Y?") | 6/10 (60%) | Partial |
| L6 | Narrative ("Is X small/old?") | 5/10 (50%) | Partial |
| **Total** | | **36/50 (72%)** | |

The shape is NOT monotonic — it's a U-shaped dip at L1 with a perfect plateau at L2-L4. L1 failures are chain-depth issues (not geometry issues). L5-L6 require richer definitions or deeper search.

---

## Per-Question-Type Coverage

| Question Type | Best Score | Test Suite | Notes |
|--------------|-----------|------------|-------|
| Yes/No (positive) | 10/10 | dict5 | Pure geometry works |
| Yes/No (negative) | 4/4 | dict5 | Definition-chain gate required |
| Yes/No (transitive) | 5/5 | dict5 | Geometry is inherently transitive |
| What/Who/Where | 10/10 | 3w_test | Definition extraction + entity fast path |
| Why (chain explanation) | 9/10 | dict5_2w_test | Chain trace as "because" explanation |
| When (condition extraction) | 4/5 | dict5_2w_test + 2w_test | Clause extraction from definitions |
| Boolean AND/OR | 14/15 | dict5_bool + bool_test | Three-valued boolean decomposition |
| Describe (generation) | 100% | self-consistency | Every generated sentence verifies as true |
| I don't know (honesty) | 4/4 | dict5 | Emerges from geometric absence |

---

## Evolution History

| Phase | Version | Best Fitness | Key Advance |
|-------|---------|-------------|-------------|
| Baseline | v0.1 | 0.4375 | Pure geometry, no rules |
| Evolved params | v7d | 0.7812 | Parameter ceiling found |
| Cross-validation | v6b | 0.7188 | Zero overfitting gap (dict5 = dict12) |
| Grammar regularizer | v10 | 0.7063 | Prevents space collapse |
| Surgical fixes | v11 | 0.8500 | Chain negation + definition extraction |
| Open-mode | Phase 10 | 0.8684 | Victorian literature comprehension |
| Entity priority | Phase 11b | 0.9474 | Entity fast path, 19/21 |
| Boolean operators | Phase 12 | — | AND/OR compound queries |
| Basic writing | Phase 13 | — | describe() mode, 100% self-consistency |
| When/Why | Phase 14 | — | Chain-as-explanation, condition extraction |
| Multi-space | Phase 16-17 | 87.5% | 4 independent geometric spaces |
| SELF space | Phase 18 | 90% | Identity as geometry, 45/50 |
| Bootstrap | Phase 19 | — | Self-improvement, 4 new connectors |

---

## Compute Profile

The entire project runs on a single CPU core:
- **dict5 build**: ~50ms (51 words, 11 connectors, 8 dimensions)
- **dict12 build**: ~2s (1005 words, 16 connectors)
- **dict18 build**: ~5s (2008 words, 24 connectors)
- **Three Men build**: ~3s (2429 words from cache)
- **Question answering**: <1ms per question (any dictionary size)
- **Evolution (50 gen, 50 pop)**: ~30 minutes on single core

Learning is O(n log n). Inference is O(1) — answering a question in a 2008-word space takes the same time as in a 51-word space. No GPU. No cloud. Total compute for the entire project: minutes, not hours.
