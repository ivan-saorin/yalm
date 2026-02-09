# YALM Roadmap

> Where the project is going, from concrete next steps to open research questions.

---

## Near-Term: Phases 20-22

### Phase 20: Per-Space Parameter Evolution

**Status**: Next in queue
**Goal**: Each geometric space gets its own evolved parameters, rather than sharing global defaults.

Currently all spaces use the same `EngineParams`. But MATH (51 entries, numeric patterns) likely needs different dimensions, force magnitudes, and thresholds than CONTENT (51 entries, taxonomic structure) or GRAMMAR (language structure).

Approach:
- Run independent GA evolution for each space's parameters
- Fitness function: per-space test suites
- Constraint: must not regress overall unified_test score

Expected impact: +2-3 questions on unified_test (currently 45/50). The 5 failures include several that might respond to per-space tuning.

### Phase 21: Open-Mode Multi-Space

**Status**: Planned
**Goal**: Apply the multi-space architecture to open-mode text (Ollama-assembled dictionaries).

Currently, multi-space only works with hand-crafted domain dictionaries. Open mode produces a single merged dictionary. Phase 21 would automatically partition open-mode vocabulary into domain spaces.

Approach:
- Use connector patterns to identify topic clusters
- Automatically separate math-related, grammar-related, and content words
- Build per-domain spaces from the partitioned vocabulary

Challenge: Automatic domain detection is hard. The TASK space dispatcher would need to generalize beyond its current 4-domain hardcoding.

### Phase 22: Phase 15 Implementation (Rich Property Extraction)

**Status**: Overdue (was supposed to be Phase 15, now critical for bootstrap quality)
**Goal**: Extract embedded properties from definitions.

Currently describe() misses adjective properties:
```
"sun — a big hot thing that is up in the sky"
  Current: "a sun is a thing."
  Phase 15: "a sun is a thing.", "the sun is big.", "the sun is hot.", "the sun is up."
```

Impact on bootstrap loop: Richer describe() output → more connector discovery signal → better convergence. The bootstrap loop (Phase 19) found only 4 new connectors. With property extraction, the signal would be much stronger.

---

## Medium-Term: Phases 23-25

### Phase 23: Remove Hardcoded English

Replace all hardcoded word lists with text-derived equivalents:
- Structural words from `classify_word_roles()` output
- Question patterns from a grammar dictionary
- Task routing from pure TASK-space geometry (remove indicator fallbacks)
- Number mapping from MATH dictionary parsing

Goal: A version of YALM where the engine crate contains zero English strings. All language knowledge comes from dictionaries.

### Phase 24: Multi-Language Test

Validate the architecture on a non-English closed dictionary:
- French dict5 equivalent (51 words, ELI5 definitions in French)
- Test connector discovery: does it find "est un", "peut", "ne pas"?
- Test equilibrium: does geometry cluster correctly?
- Test question answering: with a French question parser adapter

Success criterion: Same percentage accuracy as English dict5 (20/20) on equivalent French questions.

### Phase 25: Scaling Test (10K words)

Build dict25.md with ~10,000 entries. Test:
- Connector discovery quality at scale
- Equilibrium convergence time
- Threshold transfer from smaller dictionaries
- Fitness degradation curve

This is the critical scale test. If fitness holds above 0.60 at 10K words with evolved parameters, the architecture scales. If it collapses, the force-field approach has a ceiling.

---

## Long-Term: Research Questions

### Can the symbolic operations become geometric?

The definition-chain gate (Finding A06 in the audit) is the core hybrid component. Two paths to pure geometry:

1. **Definition-graph embedding**: Embed the definition graph (not just word co-occurrence) into the space. "dog → animal" becomes a geometric DIRECTION, not just proximity. This is essentially TransE inside YALM.

2. **Connector-direction queries**: Instead of scalar distance, query along a specific connector's axis. "Is a dog an animal?" would check the projection of `dog - animal` onto the "is a" axis direction, not just the magnitude. Connector axes already exist but evolution consistently ignores them (alpha=0.2 emphasis was rejected at 96%).

3. **Asymmetric distance**: Replace Euclidean distance with an asymmetric metric where `d(dog, animal) ≠ d(animal, dog)`. This would naturally encode directed "is-a" relationships. Research frontier — not clear if force-field equilibrium can produce asymmetric spaces.

### Can YALM learn from conversation?

Currently YALM learns from dictionaries (static) and describe() output (generated). Could it learn from:
- Questions it was asked (what topics do users care about?)
- Answers it got wrong (what went wrong in the geometry?)
- New definitions provided at runtime ("Actually, a whale is a mammal")

This would make YALM a continuously learning system, not a batch-build system.

### Can multiple YALMs communicate?

If two YALM instances have different dictionaries (one has dict_biology, another has dict_physics), can they share knowledge through bridge terms? The multi-space architecture already supports this within one process. Extending to separate processes (or separate machines) would create a distributed comprehension network.

### What happens with curved spaces?

The current space is Euclidean (flat). Hierarchical relationships (dog → animal → living thing → thing) are better represented in **hyperbolic** space, where trees embed naturally with low distortion. Replacing Euclidean distance with Poincare ball distance could improve taxonomic reasoning.

---

## The Honest Forecast

YALM works at the 1000-2000 word scale. The architecture is sound, the principles are (mostly) respected, and the results are real.

The open questions are:
1. **Scale**: Does it work at 10K? 100K? This is testable.
2. **Language independence**: Does the engine core work for non-English? This is testable.
3. **Hybrid resolution**: Can the symbolic operations become geometric? This is a research question.

The most likely outcome: YALM demonstrates that geometric comprehension from text IS possible at small-to-medium scale, that the hybrid geometry+symbols architecture is the natural endpoint, and that the individual techniques (force-directed layout, closed dictionaries, genetic evolution) compose well. It will NOT replace neural language models at scale, but it will contribute the insight that **definitions + physics + evolution = comprehension** at a scale where neural networks are overkill.
