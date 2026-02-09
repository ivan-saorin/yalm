# DAPHNE Architecture Deep-Dive

> How a geometric comprehension engine works, from dictionary to answers.

---

## Overview

DAPHNE transforms text into geometry. It reads definitions, discovers recurring patterns (connectors), uses those patterns as physical forces to position words in N-dimensional space, and then answers questions by measuring distances and traversing definitions.

```
Dictionary → Connector Discovery → Force Field → Equilibrium → Geometric Space
                                                                      ↓
                                                          Question → Resolver → Answer
```

No neural networks. No embeddings layer. No training corpus. The dictionary IS the training data, the geometry IS the model, and distance IS the inference.

---

## Stage 1: The Closed Dictionary

DAPHNE starts from a **closed dictionary** — a markdown file where every word used in any definition is itself a defined entry.

```markdown
**dog** — an animal. it can make sound. it can live with a person.
- "a dog is a thing"
- "the dog can eat food"
- "a dog can move"
```

Each entry has:
- A **headword** (the word being defined)
- A **definition** (one or more sentences)
- **Example sentences** (optional, prefixed with `- "..."`)

The **closure property** means there are no undefined symbols. Every word in "an animal. it can make sound." — "an", "animal", "it", "can", "make", "sound" — has its own entry. This creates a self-consistent universe of meaning.

### Three dictionary scales

| Dictionary | Words | Level | Description |
|-----------|------:|-------|-------------|
| dict5.md | 51 | 5-year-old | Animals, colors, basic concepts |
| dict12.md | 1005 | 12-year-old | Science, geography, everyday life |
| dict18.md | 2008 | 18-year-old | Abstract concepts, social structures, academic domains |

### Open mode

For arbitrary text, DAPHNE assembles a dictionary on-the-fly:
1. Extract seed words from input text
2. Query Ollama (qwen3) for ELI5 definitions
3. Recursively define every word in every definition (closure chase)
4. Assemble into a closed dictionary
5. Proceed with the normal pipeline

The LLM writes definitions only — it never touches comprehension. Understanding comes from geometry.

---

## Stage 2: Connector Discovery

The engine scans every sentence in every definition and example to find **recurring word patterns**. No grammar rules, no linguistic knowledge — just frequency analysis.

### Pass 1: Frequency filter

Count all word sequences (1-4 tokens) that appear between content words. Apply a **topic-scaling threshold**: patterns must appear across enough different entries, scaled by dictionary size:

```
topic_threshold = entries * 0.25 / ln(entries / 50)
```

This logarithmic scaling ensures dict5 finds ~11 connectors while dict18 finds ~24, without drowning in noise.

### Pass 2: Uniformity filter

High frequency alone isn't enough — a pattern like "the" might appear 1000 times but only in definitions about geography. True structural connectors (like "is a") appear **uniformly** across the whole dictionary.

The filter:
1. Sort all entries alphabetically
2. Split into 10 equal buckets
3. Count pattern occurrences per bucket
4. Compute coefficient of variation: `CV = std_dev / mean`
5. Uniformity = `1 - CV`. If uniformity < 0.75, reject.

Result: connectors like "is a", "can", "not", "of the" pass. Topic-specific patterns like "an element" or "in the water" fail.

### What gets discovered

Typical dict5 connectors (11 total):
- "is a" — taxonomic membership
- "can" — capability
- "not" — negation
- "a" — article (structural)
- "is" — copula (structural)
- "the" — article (structural)
- "of" — possession/relation
- "it" — pronoun (structural)

The system doesn't KNOW these are articles or copulas. It knows they're high-frequency, uniformly distributed patterns. The semantic labels are our interpretation.

---

## Stage 3: Force Field and Equilibrium

### Word placement

Each connector becomes a **force type**. Each relation (extracted from sentences) becomes a **force instance**:

```
"a dog is an animal" → dog -[is a]→ animal    (push together along "is a" axis)
"a dog can make sound" → dog -[can]→ sound     (push together along "can" axis)
```

### Sequential equilibrium (default)

Words are placed one at a time:
1. **Centroid initialization**: Position each new word at the centroid of its already-placed neighbors
2. **Local relaxation**: Apply force-based perturbation with damping (strength 0.1, damping 0.95)
3. **Multiple passes**: Repeat 3 times with shuffled order to reduce placement-order bias

This is `O(n log n)` — essentially sorting-tier complexity.

### Force field (alternative)

All words placed simultaneously, then iteratively pushed by forces:
- 50 learning passes with decaying force magnitude (decay 0.98/pass)
- 4 force functions available: Linear, InverseDistance, Gravitational, Spring
- 4 multi-connector strategies: FirstOnly, Sequential, Weighted, Compositional
- 4 negation models: Inversion, Repulsion, AxisShift, SeparateDimension

The genetic algorithm selects which strategy combination works best.

### The result

After equilibrium, related words cluster:

```
dog ●── animal ──● cat       (both animals → close)
         |
       thing                 (animal is a thing → nearby)
         |
sun ●── hot                  (properties cluster)
         |
       rock ●── mineral      (different category → far)
```

The geometry IS the knowledge. No weights, no matrices — just positions in 8-dimensional space.

---

## Stage 4: Question Resolution

When a question arrives, the resolver classifies it and applies the appropriate strategy.

### Yes/No questions

Two-stage process:
1. **Geometric distance**: Compute distance between subject and object in the space. Below threshold → candidate Yes. Above threshold → candidate No or IDK.
2. **Definition-chain gate**: Traverse the subject's definition chain (up to 3 hops) to verify the object appears. This confirms identity relationships that proximity alone can't distinguish from mere similarity.

Example: "Is a dog an animal?" → distance(dog, animal) = 0.42 (below threshold) + chain: dog's definition contains "animal" → **Yes**.

Example: "Is a dog a cat?" → distance(dog, cat) = 0.35 (below threshold, both animals!) BUT chain: dog's definition doesn't contain "cat" → **No**.

### What/Who/Where questions

Extract the first content word from the subject's definition:
```
"dog — an animal. it can make sound."
  → skip "an" (article) → "animal" → "an animal"
```

### Why questions

Trace the definition chain from subject to object, format as explanation:
```
"Why is a dog a thing?"
  → dog's definition contains "animal"
  → animal's definition contains "thing"
  → "because a dog is an animal, and an animal is a thing"
```

### When questions

Extract conditional/purpose clauses ("to", "when", "if") from definitions:
```
"When does a person eat?"
  → eat's definition: "to take in food to feel good"
  → extracted clause: "to feel good"
```

### Boolean (AND/OR)

Decompose compound queries into sub-queries, resolve each, combine with three-valued logic:
```
"Is a dog an animal and a thing?"
  → "Is a dog an animal?" → Yes
  → "Is a dog a thing?" → Yes
  → Yes AND Yes → Yes
```

### Honesty

When no geometric proximity exists above threshold, the system says "I don't know." This is not a special case — it's what happens when nothing is close enough. Honesty is free.

---

## Stage 5: Multi-Space Architecture

Phase 16+ introduced **multiple independent geometric spaces** ("thought domains"):

| Space | Dictionary | Domain |
|-------|-----------|--------|
| CONTENT | dict5.md | Physical world — animals, properties, actions |
| MATH | dict_math5.md | Numbers, operations, counting |
| GRAMMAR | dict_grammar5.md | Language structure — nouns, verbs, sentences |
| TASK | dict_task5.md | Dispatcher — routes queries to correct domain(s) |
| SELF | dict_self5.md | Identity — what DAPHNE is, can do, cannot do |

Each space runs its own connector discovery and equilibrium independently. They connect only at query time through:

### Bridge terms

Words that appear in multiple spaces serve as bridges. "number" exists in both MATH and GRAMMAR — it's the geometric handoff point for cross-domain queries like "Is 'number' a noun?"

### TASK routing

The TASK space dictionary defines meta-concepts ("math is about numbers", "grammar is about words"). When a query arrives, the system computes geometric distance from query content words to each domain label in the TASK space. Closest domain(s) handle the query.

### Cross-space chains

For queries that span domains: resolve in each relevant space, compose results. "How many legs does a dog have?" → CONTENT knows legs are a body part, MATH knows "four", TASK routes to both.

---

## Stage 6: Bootstrap Loop

Phase 19 introduced self-improvement without changing dictionaries:

```
loop {
  1. describe() each content word → generated sentences
  2. Feed sentences through connector discovery → new connector patterns
  3. Rebuild geometric space with enriched connectors
  4. If no new connectors found → converged, stop
}
```

Typical result on dict5:
- **Level 1**: 4 new connectors discovered (e.g., "is not" emerges from negation sentences)
- **Level 2**: 0 new connectors → converged

The bootstrap loop discovers grammar that was implicit in the original definitions. The dictionary never changes — only the connector set evolves.

---

## Crate Structure

| Crate | Lines | Role |
|-------|------:|------|
| `dafhne-core` | ~312 | Shared types: Dictionary, GeometricSpace, EngineParams, Connector, Answer |
| `dafhne-parser` | ~400 | Markdown parser for dictionaries, tests, grammar files |
| `dafhne-engine` | ~5600 | Core engine: connector discovery, equilibrium, force field, resolver, multispace, bootstrap |
| `dafhne-eval` | ~600 | Evaluation harness: runs test suites, computes fitness |
| `dafhne-evolve` | ~1200 | Genetic algorithm: genome, population, mutation, crossover, selection |
| `dafhne-demo` | ~400 | Interactive demo: load knowledge, answer questions, pretty-print |
| `dafhne-cache` | ~800 | Dictionary assembly from free text via LLM (Ollama, Wiktionary) |

Total: ~9300 lines of Rust. Zero external ML dependencies.
