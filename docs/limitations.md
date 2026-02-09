# DAFHNE Limitations

> An honest assessment of what DAFHNE cannot do, and why.

---

## Fundamental Limitations

### 1. Geometry Cannot Distinguish Similarity from Identity

The deepest limitation. Dog and cat are both geometrically close to animal (they are similar). But "Is a dog a cat?" should be No, and "Is a dog an animal?" should be Yes. Distance is symmetric — it cannot encode directed "is-a" relationships.

**Consequence**: Yes/No resolution requires a definition-chain gate on top of geometry. This makes DAFHNE a hybrid system (geometry + symbols), not a pure geometric engine.

**Why it's fundamental**: This is a property of metric spaces, not a DAFHNE-specific bug. Any system using distance as the sole knowledge representation will face this. TransE (head + relation = tail) addresses it by making relations directional vectors, not scalar distances.

### 2. No Causal Model

DAFHNE knows THAT things are related but not WHY in any deep sense. "Why is a dog an animal?" → "because a dog is an animal" (the definition IS the explanation). This is tautological — correct but shallow.

**Consequence**: DAFHNE cannot explain mechanisms, processes, or causal chains beyond what definitions state explicitly.

**Why it's fundamental**: The definitions are the only input. If a definition doesn't explain causation, there is no other source to draw from. Geometry encodes co-occurrence and taxonomic structure, not causal direction.

### 3. No Temporal Model

Geometry has no time axis. "When does a person eat?" is answered by extracting purpose/condition clauses from definitions ("to feel good"), not by temporal reasoning.

**Consequence**: Cannot answer genuine temporal questions ("When did X happen?", "What comes after Y?") unless the answer is explicitly in a definition.

### 4. Vocabulary Scale

DAFHNE has been tested up to 2008 words (dict18) and 2429 words (open mode). This is orders of magnitude smaller than real-world vocabulary (~100K active English words, 1M+ total).

**What we know**: The scaling curve is sublinear (1005→2008 words costs only 0.03 fitness). But we don't know if this holds at 10K, 50K, or 100K words.

**What would likely break at scale**:
- Connector discovery: More noise, harder to separate structural from content patterns
- Equilibrium convergence: More words = more forces = slower convergence
- Threshold tuning: Yes/No thresholds calibrated for ~1000 words may not transfer to 100K
- Definition chain depth: Longer chains = more spurious matches

---

## The Geometry-vs-Symbols Tension

The [code audit](../reports/19b_code_audit.md) found that approximately:
- **35%** of correct answers rely primarily on geometry (positive Yes/No, transitive chains)
- **40%** rely on symbolic chain operations (negative Yes/No, What/Why/When)
- **25%** rely on geometric absence (honesty — "I don't know")

This is not a failure — it's a finding. The hybrid architecture works because:
- Geometry handles **association** (what's related to what)
- Definitions handle **identity** (what IS what)
- Absence handles **uncertainty** (what's not in the space at all)

The open question: can the symbolic operations (chain traversal, clause extraction) be replaced with a SECOND geometric operation (e.g., a definition-graph embedding)? Or is the symbolic layer irreducible?

---

## English-Specific Assumptions

### Hardcoded English Knowledge

The following English-specific knowledge is hardcoded in the resolver and multispace modules:

| Component | English Assumption | Location |
|-----------|-------------------|----------|
| Question detection | "is", "can", "does", "what", "who", "where", "when", "why" | resolver.rs |
| Article stripping | "a", "an", "the" | resolver.rs, multispace.rs |
| Structural words | ~30 English function words | multispace.rs |
| Negation detection | literal "not" | resolver.rs |
| Number words | "zero" through "ten" | multispace.rs |
| Self-triggers | "dafhne", "you" patterns | multispace.rs |
| Task indicators | English domain keywords | multispace.rs |

### What Would Break in Another Language

- **Question type detection**: Completely English-specific. A French DAFHNE would need "est-ce que", "qu'est-ce que", "pourquoi", etc.
- **Article handling**: Languages without articles (Russian, Japanese, Chinese) would need different function-word filtering.
- **SVO word order**: The resolver assumes Subject-Verb-Object order for English questions. SOV languages (Japanese, Korean, Turkish) would need different parsing.
- **Connector patterns**: "is a", "can", "not" are English-specific. But the DISCOVERY process is language-independent — the frequency/uniformity pipeline would find "est un", "peut", "ne...pas" in French text.

### What Would Still Work

- **Connector discovery**: Purely statistical — works on any language with recurring patterns
- **Force field**: Language-independent physics
- **Geometric distance**: Universal
- **Honesty**: Geometric absence works regardless of language
- **Bootstrap loop**: Language-independent (describe→discover→rebuild)

**Assessment**: The engine core (connector discovery, force field, equilibrium) is language-independent. The interface layer (question parsing, answer formatting) is English-specific. A proper refactoring would move all English-specific code out of the engine into a language-specific adapter.

---

## Known Failure Modes

### 1. Category Confusion (Q08 in dict5_bool_test)

"Is a cat a dog?" → Yes (at max_hops=3, the chain cat→mammal→...→dog finds a connection through shared taxonomy). The chain gate doesn't distinguish "related via shared ancestor" from "is a."

**Root cause**: Chain traversal is too permissive at depth 3. Two words sharing a grandparent category can chain to each other.

### 2. Ordinal Comparison (Q13 in unified_test)

"Is three more than one?" → No. Ordinal comparison is not geometric — "more than" requires a learned ordering that distance doesn't encode.

### 3. WhatIs in Small Spaces (Q18, Q20 in unified_test)

"What is a sentence made of?" → IDK. The grammar space is small enough that `definition_category()` fails to extract the answer. The definition exists, but the extraction heuristic doesn't match.

### 4. Quoted Phrases (Q25 in unified_test)

"What kind of number is 'seven'?" → IDK. Quoted phrases confuse the token parser.

### 5. Multi-Part Context Pipelines (Q36 in unified_test)

"How many legs does a dog have?" → IDK. Requires routing through CONTENT (legs → dog) AND MATH (counting), with context passed between spaces.

### 6. LLM Definition Quality for Proper Nouns

"harris" as a common English word gets an Ollama definition about melting/smelting, not about a person. Entity definitions fix this, but only for explicitly injected entities.

---

## What DAFHNE is NOT

- **Not a general-purpose language model**: It cannot generate free text, translate, summarize, or have conversations.
- **Not scalable to arbitrary text**: Open mode works but requires an LLM preprocessor for definitions.
- **Not language-independent (yet)**: The engine core is, but the interface layer is English-only.
- **Not a replacement for neural networks**: At 2000 words, DAFHNE achieves 90% on structured questions. GPT-4 achieves 90%+ on free-form text about millions of concepts. Different tools, different scales.
- **Not a pure geometric engine**: The definition-chain gate is symbolic. The hybrid is the system, not a compromise.
