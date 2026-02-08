# PROMPT 14 — 2W: When, Why (Basic Reasoning)

> **STATUS: PLACEHOLDER** — To be expanded after Prompt 13 (Writing) is complete.

## GOAL

Add the two hard question words: **When** and **Why**. These require capabilities beyond classification:

- **When**: temporal ordering — "When does a person eat?" (requires understanding of conditions/sequences)
- **Why**: causal reasoning — "Why is ice cold?" (requires tracing cause-effect chains)

Both may require architectural extensions beyond the current geometric proximity model.

## PREREQUISITES

- Prompt 13 complete (basic writing works — the system can express its knowledge)
- The system can describe words, compose boolean queries, and resolve 3W identity questions
- Definition-chain traversal is robust at 2+ hops

## WHY THIS IS HARD

### When (Temporal)

Geometric proximity encodes **similarity**, not **sequence**. "Before" and "after" are not distances — they're ordering relationships. The current space has no temporal dimension.

Possible approaches:
1. **Definition-based**: extract temporal markers from definitions ("X happens when Y", "X is before Y") and build a partial order
2. **Temporal dimension**: reserve one geometric dimension for temporal ordering, where earlier events have lower values
3. **Narrative position**: in open-mode texts, use sentence position as a temporal proxy (earlier in text = earlier in time)
4. **Punt**: answer "When" questions with conditions rather than times ("a person eats when they are hungry" → extract "hungry" from eat's definition chain)

Recommendation: start with approach 4 (condition extraction). True temporal reasoning is a research problem.

### Why (Causal)

Causality is directional: "A causes B" ≠ "B causes A". Geometric proximity is symmetric. The definition chain is directional (A's definition mentions B) but this encodes taxonomy, not causation.

Possible approaches:
1. **Definition-based**: extract causal markers from definitions ("because", "makes", "causes", "so that") and follow them
2. **Connector-type tagging**: discover causal connectors ("makes", "causes") separately from taxonomic ones ("is a")
3. **Reverse definition walk**: "Why is X Y?" → find Z where Z's definition says "Z makes Y" and X's definition mentions Z
4. **Property chain**: "Why is ice cold?" → ice's definition says "frozen water" → frozen = "very cold" → the definition IS the explanation

Recommendation: start with approach 4 (definition chain as explanation). Present the chain as the "why".

## SCOPE (TENTATIVE)

- **When**: condition extraction from definitions ("X does Y when Z")
- **Why**: definition-chain presentation as causal explanation ("X is Y because X's definition says Z which means Y")
- New QuestionType variants: `WhenIs` and `WhyIs`
- New resolver functions: `resolve_when()` and `resolve_why()`
- May require new connector types or relation annotations
- Test with dict5 + Three Men in a Boat

## TEST EXAMPLES (SPECULATIVE)

```
When does a person eat? -> when hungry / when they need food
When is it cold? -> when there is no heat / in winter
Why is ice cold? -> because ice is frozen water and frozen means very cold
Why is the sun hot? -> because the sun is a big hot thing that makes light
Why does a dog eat? -> because a dog is an animal and animals need food
```

These answers should be generated from definition chains, not from geometric proximity.

## THE FUNDAMENTAL QUESTION

Can temporal and causal reasoning emerge from geometry, or do they require a fundamentally different representation?

If the geometric space can be extended (temporal dimension, causal axes), the architecture stays unified. If temporal/causal reasoning requires symbolic graph traversal, the architecture confirms what Phase 10's RECAP already suggested: **geometry for association, symbols for discrimination** — and now, **symbols for reasoning**.

This prompt will answer that question empirically.

## ESTIMATED EFFORT

- Resolver extensions: ~200-300 lines (condition extraction + chain presentation)
- Possible new relation types in connector discovery: ~100 lines
- Test file: ~15 questions (when + why)
- May require architectural design document before implementation
- Potentially the first prompt that requires engine-level changes