# PROMPT 15 — Rich Descriptions: Property Extraction from First Sentences

> **STATUS: PLACEHOLDER** — To be expanded after Prompt 14 (When/Why) is complete.

## GOAL

Extend `describe()` to extract **properties and modifiers** from the first sentence of definitions, not just the category noun.

Currently, "a big hot thing that is up" extracts only "thing" (category). Phase 15 adds:
- **Adjective extraction**: "big", "hot" → "the sun is big. the sun is hot."
- **Relative clause extraction**: "that is up" → "the sun is up."
- **Compound categories**: "a small animal" → "a cat is small."

This fills the gap between category (Phase 11) and capabilities (Phase 13), producing richer self-descriptions.

## PREREQUISITES

- Prompt 14 complete (When/Why reasoning works)
- `describe()` function stable with category + capability + negation sentences
- `definition_category()` and `is_property_word()` infrastructure available

## KEY DESIGN QUESTIONS

1. **How to distinguish adjectives from nouns in the first sentence?**
   - Property words: `is_property_word()` already identifies adjectives/verbs by definition shape
   - Structural words: already filtered
   - Category noun: already extracted by `definition_category()`
   - Remaining content words in first sentence = properties/modifiers

2. **"that is" / "which is" relative clauses:**
   - "a big hot thing that is up" → split on "that" → extract "is up"
   - "an animal that can make things" → split on "that" → extract "can make things" (already a capability)

3. **Sentence templates for properties:**
   - Adjective: "{subject} is {adjective}."
   - Relative clause with "is": "{subject} is {rest}."
   - Relative clause with "can": "{subject} can {rest}." (overlaps with capability extraction)

4. **Ordering: category → properties → capabilities → negations?**
   - Current: category, then definition sentences, then negations
   - Proposed: category, then first-sentence properties, then capability sentences, then negations

## SCOPE (TENTATIVE)

- Extend `describe()` with property extraction from first sentence
- New helper: `extract_first_sentence_properties()`
- Template-based sentence generation for properties
- Test: dict5 describe output quality (sun, ball, cat should show properties)
- Metric: sentence count increase, self-consistency maintained
- No engine/equilibrium changes

## TEST EXAMPLES

```
Describe sun (current):
  the sun is a thing.
  the sun makes things hot.
  the sun is not a person.

Describe sun (Phase 15):
  the sun is a thing.
  the sun is big.
  the sun is hot.
  the sun is up.
  the sun makes things hot.
  the sun is not a person.

Describe cat (current):
  a cat is an animal.
  a cat can live with a person.
  a cat is not a dog.

Describe cat (Phase 15):
  a cat is an animal.
  a cat is small.
  a cat can move with not-sound.
  a cat can live with a person.
  a cat is not a dog.
```

## ESTIMATED EFFORT

- Resolver describe() extension: ~50-80 lines
- New helper function: ~30 lines
- No new test files needed (reuse dict5 describe, verify richer output)
- Self-consistency should remain 100% (property sentences are verifiable)
