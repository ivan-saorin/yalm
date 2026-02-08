# PROMPT 13 — Basic Writing: Geometric Expression

> **STATUS: PLACEHOLDER** — To be expanded after Prompt 12 (Boolean) is complete.

## GOAL

YALM flips from **comprehension** to **generation**. Walk the geometric space outward from a word along connectors and produce descriptive sentences.

```
"Describe a dog" → traverse dog's geometric neighborhood →
  "a dog is an animal. it can move. it can eat. it is not a cat."
```

The system expresses what it knows by reading its own geometry.

## PREREQUISITES

- Prompt 12 complete (boolean operators enable compound sentence generation)
- Stable resolver for 3W + boolean queries
- Definition-chain traversal working for category extraction

## KEY DESIGN QUESTIONS

1. **Traversal strategy: nearest neighbors or definition walk?**
   - Nearest neighbors: find the N closest content words to the subject, generate "X is [near] Y" sentences
   - Definition walk: follow the subject's definition chain, generate sentences from each hop
   - Hybrid: definition walk for category sentences, nearest neighbors for property/capability sentences
   - Recommendation: hybrid (definition walk gives "is a" sentences, neighbors give "can" and "has" sentences)

2. **Sentence templates driven by connector type:**
   - "is" connector + noun neighbor → "X is a Y" (category)
   - "can" connector + verb neighbor → "X can Y" (capability)
   - "not" connector + antonym → "X is not Y" (negation)
   - "has" connector + part neighbor → "X has Y" (composition)
   - Templates are filled from geometric data, not generated freely

3. **Quality control: how to avoid garbage sentences?**
   - Distance threshold: only include neighbors within yes_threshold distance
   - Connector filtering: only use connectors that actually appear in the subject's definition relations
   - Deduplication: don't repeat the same relationship in different phrasings

4. **Output format:**
   - Simple prose: sentences joined with periods
   - Structured: category sentence first, then capabilities, then negations
   - YALM-style: output matches the ELI5 definition format (self-consistency test)

## SCOPE

- New `describe()` function in resolver (or separate module)
- Input: a word + the geometric space + dictionary
- Output: 3-5 sentences describing the word using geometric relationships
- Test: generate descriptions for dict5 words, compare to actual definitions
- Self-consistency test: feed generated descriptions back as text → does the geometry reconstruct?
- No engine/equilibrium changes

## TEST EXAMPLES

```
Describe dog:
  "a dog is an animal. a dog can move. a dog can eat. a dog is not a cat."

Describe sun:
  "the sun is a big hot thing. the sun is up. the sun makes light."

Describe Montmorency:
  "montmorency is a dog. montmorency can [X]. montmorency is not a person."
```

## THE SELF-CONSISTENCY TEST

The most interesting test: take YALM's generated description of X, feed it back as input text, rebuild the geometry, and ask "What is X?". If the answer matches the original, the system's comprehension and expression are consistent. This is the geometric equivalent of a round-trip test.

## ESTIMATED EFFORT

- New module or function: ~150-200 lines
- Sentence templates: ~5 connector-based templates
- Test file: ~10 words to describe, quality scoring
- Self-consistency harness: ~50 lines
- No engine changes