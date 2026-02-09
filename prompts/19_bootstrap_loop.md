# PROMPT 19 — Bootstrap Loop: Read What You Write, Write What You Read (STUB)

> **STATUS: STUB** — To be expanded after Phase 18 is complete.

## GOAL

Implement the self-improvement loop: YALM reads its own output, builds richer geometry from it, and uses the richer geometry to produce better output. This is the path from ELI5 to SLM-level abstraction.

## THE LOOP

```
Level 0:
  CONTENT reads dict5 → knows dogs, cats, sun
  GRAMMAR reads dict_grammar5 → knows nouns, verbs, sentences
  YALM writes: "a dog is an animal" (ELI5 level)

Level 1:
  GRAMMAR generates sentence templates from its space
  CONTENT fills templates with knowledge from its space
  Output: "the dog is a small animal. it can move. it can live with a person."
  → GRAMMAR reads this output
  → Discovers new patterns: adjective placement, pronoun reference
  → Grammar geometry gets richer

Level 2:
  GRAMMAR with richer geometry generates better templates
  CONTENT fills with more nuanced selection
  Output: "a dog is a small animal that can make sound and live with a person."
  → Relative clauses, compound predicates emerge
  → GRAMMAR reads, geometry gets richer again

Level N:
  Grammar complexity approaches natural language
  CONTENT remains ELI5 (its dictionary doesn't change)
  But the OUTPUT is no longer ELI5 — it's structured by evolved grammar
```

## KEY CHALLENGES

1. **What exactly does "GRAMMAR reads its own output" mean?**
   - The output text is parsed as new input for connector discovery
   - New connectors emerge from patterns in generated text
   - Equilibrium is re-run with enriched connector set
   - Grammar geometry evolves without changing the dictionary

2. **Convergence or divergence?**
   - If each cycle produces strictly better output → convergence (desired)
   - If errors compound → divergence (failure mode)
   - Self-consistency check: output re-read must produce same answers as direct query
   - If consistency drops below threshold → stop loop, use previous level

3. **How to measure "better"?**
   - Level 0: simple sentences (subject-verb-object)
   - Level 1: adjectives, simple clauses
   - Level 2: relative clauses, compound sentences
   - Level 3: paragraph structure, topic coherence
   - Metric: syntactic complexity score (clause depth, connective count)
   - Hard constraint: semantic accuracy cannot decrease

4. **Which spaces participate in the loop?**
   - GRAMMAR: evolves (new connectors from generated text)
   - CONTENT: stable (dict5 doesn't change, provides facts)
   - MATH: stable (arithmetic doesn't evolve)
   - TASK: may need updates (new capabilities from richer grammar)
   - SELF: must update ("I can now write compound sentences")

## IMPLEMENTATION SKETCH

```
loop_iteration(level: usize, max_level: usize):
  1. Generate text using current GRAMMAR + CONTENT
     - Pick 5 topics from CONTENT (dog, cat, sun, person, food)
     - For each: generate description using describe mode
  2. Parse generated text as input
     - Run connector discovery on generated text
     - Compare connector set with previous level
     - New connectors = grammar has evolved
  3. Re-run GRAMMAR equilibrium with enriched connectors
  4. Validate:
     - Self-consistency: re-read generated text, verify answers
     - Regression: run unified_test.md, must not decrease
     - Complexity: measure syntactic complexity of new output
  5. If valid and level < max_level: recurse
     If regression or consistency drop: stop, use previous level
```

## PREREQUISITES

- Phase 18 complete (SELF space, 5 spaces integrated)
- Describe mode working across multi-space
- Self-consistency verification from Phase 13

## ESTIMATED EFFORT

- Design: 2-3 days (this is architecturally the hardest phase)
- Implementation: 3-5 days
- Testing and iteration: 2-3 days
- Unknown unknowns: high — this is genuinely novel territory

## SUCCESS CRITERIA (tentative)

- Loop runs at least 2 iterations without regression
- Output at Level 2 is measurably more complex than Level 0
- Self-consistency remains above 90% at each level
- At least one new connector discovered by the loop

## RISK ASSESSMENT

This is the highest-risk phase. The hypothesis (geometry improves from reading its own output) is unproven. Possible outcomes:
- **Best case**: grammar evolves, output quality improves measurably, path to SLM clear
- **Medium case**: loop converges after 1-2 iterations, marginal improvement, ceiling found
- **Worst case**: output is too simple to generate new connectors, loop is a no-op

All outcomes are informative. Even "worst case" tells us that grammar evolution requires external input (richer dictionaries), not self-improvement.
