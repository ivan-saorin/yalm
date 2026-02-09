# Phase 15 Gap Assessment

> What was planned, what was skipped, and how it affects the current system.

---

## What Phase 15 Was

Phase 15 (Rich Description: Property Extraction) was planned to extract embedded adjective and relational properties from definition sentences, producing richer describe() output.

**Planned transformation**:
```
Input:  "sun — a big hot thing that is up in the sky"
Current output:
  "a sun is a thing."

Phase 15 output:
  "a sun is a thing."
  "the sun is big."
  "the sun is hot."
  "the sun is up."
  "the sun is in the sky."
```

The prompt (`prompts/15_description_enrichment.md`) exists as a placeholder/stub. It was never fully specified or implemented.

---

## What describe() Currently Produces

Based on the code in `resolver.rs`, the describe() function generates three types of sentences:

### 1. Category sentence
Extracted via `definition_category()`: first content word from definition.
```
"a dog is an animal."
"a cat is an animal."
"a person is an animal."
```

### 2. Definition sentence rewriting
Rewrites each definition sentence, replacing "it" with the subject:
```
"a dog can make sound."       (from "it can make sound")
"a dog can live with a person."  (from "it can live with a person")
```

### 3. Sibling negation
Finds words sharing the same category, generates "is not" sentences:
```
"a dog is not a food."
"a dog is not a cat."
```

### What's MISSING

- **Adjective extraction**: "a big hot thing" → properties "big", "hot" are lost
- **Prepositional phrase extraction**: "up in the sky" → location/state lost
- **Relative clause extraction**: "that can move" → capability embedded in category sentence lost
- **Multi-property definitions**: Any definition with more than the category word loses all embedded qualifiers

---

## Impact on dict5 Content Words

Analyzing key dict5 entries and what describe() produces vs. what Phase 15 would produce:

### dog
- **Current**: "a dog is an animal.", "a dog can make sound.", "a dog can live with a person.", "a dog is not a food.", "a dog is not a cat."
- **Phase 15 would add**: Nothing significant — dog's definition is already sentence-structured. (**No impact**)

### cat
- **Current**: "a cat is an animal.", "a cat can move with not-sound.", "a cat can live with a person.", "a cat is not a food.", "a cat is not a dog."
- **Phase 15 would add**: Nothing significant. (**No impact**)

### sun
- **Current**: "a sun is a thing." (from "a big hot thing that is up")
- **Phase 15 would add**: "the sun is big.", "the sun is hot.", "the sun is up." (**HIGH IMPACT** — 3 new property sentences)

### ball
- **Current**: "a ball is a thing." (from "a round thing")
- **Phase 15 would add**: "a ball is round." (**Medium impact** — 1 new property)

### water
- **Current**: Depends on definition structure
- **Phase 15 would add**: Properties embedded in definition (**Medium impact**)

### food
- **Current**: "food is a thing."
- **Phase 15 would add**: Depends on definition structure (**Low-medium impact**)

### person
- **Current**: "a person is an animal."
- **Phase 15 would add**: Properties from definition (**Medium impact**)

### animal
- **Current**: "an animal can move.", "an animal can eat.", "an animal can feel."
- **Phase 15 would add**: Nothing — capabilities already extracted as separate sentences. (**No impact**)

**Summary**: Phase 15 has HIGH impact for definitions with embedded adjectives in the category sentence (sun, ball). LOW impact for definitions that are already well-structured with separate capability sentences (dog, cat, animal).

---

## Impact on Bootstrap Loop

### Current Bootstrap (without Phase 15)

Phase 19 bootstrap results:
- Level 1: 87 generated sentences → 4 new connectors discovered
- Level 2: 91 generated sentences → 0 new connectors → converged

The 87 sentences at Level 1 come from:
- ~20-25 category sentences ("X is a Y")
- ~15-20 rewritten definition sentences ("X can Z")
- ~40-50 sibling negation sentences ("X is not a Y")

### With Phase 15

Property extraction would add ~15-25 additional sentences per bootstrap iteration:
- "the sun is big" → new (adjective, subject, copula) triple
- "the sun is hot" → new triple
- "a ball is round" → new triple
- etc.

These property sentences would provide:
1. **More connector variety**: "is big", "is hot", "is round" are different patterns from "is a" and "can". The connector discovery pipeline might find "is [adjective]" as a new connector pattern.
2. **More relation triples**: More forces in the equilibrium → richer geometry.
3. **Better adjective placement**: Currently adjectives like "big" and "hot" have weak force signal because they appear embedded inside category sentences rather than as standalone relations.

**Estimated impact**: 2-4 additional connectors at Level 1 (doubling the current 4). Potentially 1 more bootstrap level before convergence.

---

## Is Phase 15 Still Needed?

**Yes, but for different reasons than originally planned.**

Original motivation (Phase 15 design): Richer describe() output for self-consistency verification.

Current motivation: Richer describe() output for the bootstrap loop. Phase 19 demonstrated that the bootstrap works even with thin signal. Phase 15 would amplify the signal, potentially finding connectors that distinguish properties from categories.

### Priority

Phase 15 should be implemented before:
- **Phase 20 (per-space evolution)**: Evolution would benefit from richer connector sets to optimize against.
- **Phase 21 (open-mode multi-space)**: LLM-generated definitions are MORE likely to have embedded adjectives, so property extraction is even more important at scale.

### Complexity

Phase 15 is a moderate-complexity change:
- **Scope**: Only `describe()` in `resolver.rs` needs modification
- **Algorithm**: Scan definition sentence for adjectives before the category word; emit each as a separate property sentence
- **Challenge**: Distinguishing adjectives from other pre-category words. In ELI5 definitions, adjectives typically appear between the article and the category: "a [adj1] [adj2] [category]". A simple heuristic (skip articles, collect words until category word) would handle 80%+ of cases.
- **Testing**: Self-consistency verification (--describe-verify) already exists from Phase 13.

**Estimated effort**: Small. The change is localized to one function in one file.

---

## Conclusion

Phase 15 was skipped, and Phases 16-19 succeeded without it. But the bootstrap loop (Phase 19) would produce significantly stronger signal with property extraction enabled. The change is small, localized, and testable. Recommend implementing as Phase 22 (after Phase 20-21) or earlier if bootstrap improvement is a priority.
