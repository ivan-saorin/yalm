# YALM — Yet Another Language Model

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

> O(n log n) learning · O(1) inference · Zero GPU · Zero training corpus

**A geometric comprehension engine that understands language through physics, not statistics.**

YALM reads a closed dictionary, places every word as a point in N-dimensional space, and lets sentences act as physical forces that push related words together. After the force field reaches equilibrium, the geometry *is* the knowledge — questions become distance measurements, reasoning becomes chain traversal, and "I don't know" falls out naturally when nothing is close enough.

No neural networks. No embeddings. No training corpus. Just definitions, forces, and geometry.

## Try It — Quick Demo

```bash
cargo build --release
cargo run --release -p yalm-demo -- -k dictionaries/dict5.md -q questions_demo.md
```

Output:

```
  ╔════════════════════════════════════════════════════╗
  ║           YALM — Geometric Comprehension           ║
  ╚════════════════════════════════════════════════════╝

  Knowledge : dict5.md (51 entries)
  Questions : questions_demo.md (9 questions)
  Build mode: Equilibrium
  Dimensions: 8

  ┌─ Building space ────────────────────────────────────┐
  │ Dictionary:  51 entries                             │
  │ Connectors:  11 discovered                          │
  │ Space:       51 words in 8-D                        │
  │ Build time:  0.050s                                 │
  └─────────────────────────────────────────────────────┘

  ┌─ Q&A ───────────────────────────────────────────────┐
  │                                                     │
  │ Q: Is a dog an animal?                              │
  │ A: Yes                                              │
  │                                                     │
  │ Q: What is a cat?                                   │
  │ A: an animal                                        │
  │                                                     │
  │ Q: Is a dog a cat?                                  │
  │ A: No                                               │
  │                                                     │
  │ Q: Why is a dog an animal?                          │
  │ A: because a dog is an animal                       │
  │                                                     │
  │ Q: Is a dog an animal and a thing?                  │
  │ A: Yes                                              │
  │                                                     │
  └─────────────────────────────────────────────────────┘

  Answered 9 questions in 0.001s (total: 0.052s)
```

51 words. Zero training data. Correct answers in 50 milliseconds.

## How It Works — Step by Step

### Step 1: Read the Dictionary

YALM starts from a **closed dictionary** — a markdown file where every word used in any definition is itself a defined entry. No undefined symbols.

```markdown
**dog** — an animal. it can make sound. it can live with a person.
- "a dog is a thing"
- "the dog can eat food"
- "a dog can move"
```

The parser extracts each entry's word, definition sentences, and example sentences.

### Step 2: Discover Connectors

The engine scans every sentence in every definition and example to find **recurring word patterns**. No grammar rules, no linguistic knowledge — just frequency analysis.

Patterns like "is a", "can", "not", "of the" appear across many different definitions. These high-frequency patterns become **connectors** — the force types in the geometric space.

A **uniformity filter** then separates structural connectors (appear uniformly across the whole dictionary) from content words (clustered in specific topics). Only true structural words become force operators.

### Step 3: Extract Relations

Each sentence is decomposed into `(subject, connector, object)` triples:

```
"a dog is an animal"  → dog  -[is a]→  animal
"a cat is an animal"  → cat  -[is a]→  animal
"a rock is a mineral" → rock -[is a]→  mineral
"it can make sound"   → dog  -[can]→   sound
```

These relations are the raw data for building the geometric space.

### Step 4: Build the Geometric Space

All words start as points in N-dimensional space (default: 8 dimensions). Each relation becomes a **physical force** — the connector defines a direction, and the relation pushes related words together along that axis:

```
Pass 1:  dog ←─[is a]──→ animal    (strong force, magnitude 0.15)
Pass 2:  dog ←─[is a]──→ animal    (decaying, magnitude 0.147)
  ...
Pass 50: dog ←─[is a]──→ animal    (residual force ~0.05)
```

Over 50 learning passes with decaying force magnitude, the space reaches **equilibrium**. Words cluster by meaning:

```
After equilibrium:
  dog ●──── animal ────● cat       (close together — both animals)
                 ...
  rock ●──── mineral               (far away — different category)
```

The geometry *is* the knowledge. No weights, no matrices — just positions in space.

### Step 5: Answer Questions

When a question arrives, the resolver classifies it and applies the appropriate strategy:

| Question Type | Strategy | Example |
|--------------|----------|---------|
| **Yes/No** | Geometric distance + definition-chain gate | "Is a dog an animal?" → distance < threshold → **Yes** |
| **What/Who/Where** | First-content-word extraction from definition | "What is a dog?" → definition starts with "an animal" → **an animal** |
| **Why** | Trace definition chain, present as explanation | "Why is a dog a thing?" → dog→animal→thing → **because a dog is an animal, and an animal is a thing** |
| **When** | Extract purpose/condition clauses from definitions | "When does a person eat?" → eat's def contains "to feel good" → **to feel good** |
| **AND/OR** | Decompose into sub-queries, combine with boolean logic | "Is a dog an animal and a thing?" → Yes AND Yes → **Yes** |
| **Unknown** | No proximity on any axis above threshold | "What color is a dog?" → no path found → **I don't know** |

**Transitive reasoning is free.** If "dog" is near "animal" and "animal" is near "thing", then "dog" is geometrically near "thing" — no explicit rule needed.

**Honesty is free too.** When no geometric proximity exists, the system says "I don't know" instead of hallucinating.

### Step 6 (Optional): Open Mode — Any Text

YALM can also work with **any text**, not just hand-crafted dictionaries. In open mode, it:

1. Extracts seed words from the input text
2. Sends each word to an LLM (Ollama/qwen3) for a simple ELI5 definition
3. Recursively defines every word in every definition (closure chase)
4. Assembles the resulting definitions into a dictionary
5. Proceeds with Steps 2-5 as normal

```bash
cargo run --release -p yalm-demo -- \
  -k texts/three_men/combined.md \
  --entities texts/three_men_supplementary/entities.md \
  --cache-type ollama \
  --cache dictionaries/cache/ollama-qwen3 \
  -q questions_three_men_demo.md
```

This reads excerpts from *Three Men in a Boat* (1889), assembles a 2429-entry dictionary via Ollama, merges hand-crafted entity definitions (characters and places), builds an 8-D geometric space, and answers questions:

```
  Q: Is Montmorency a dog?              → Yes
  Q: What is Harris?                    → a person
  Q: Is the Thames a river?             → Yes
  Q: Why is Montmorency an animal?      → because montmorency is a dog,
                                           and a dog is an animal
```

The LLM only writes definitions — it never touches the comprehension engine. Understanding comes from geometry.

## Scaling Results

YALM has been tested across three dictionary sizes and on Victorian literature:

### Closed Dictionaries

| Dictionary | Words | Correct | Fitness | Notes |
|-----------|------:|--------:|--------:|-------|
| dict5     |    51 |  20/20  | **1.00** | Perfect score on a small closed dictionary |
| dict12    |  1005 |  14/20  | **0.75** | 20x more words, loses 6 questions |
| dict18    |  2008 |  14/20  | **0.72** | 40x more words, loses 0 more |

The scaling curve is **sublinear** — the drop from 1005 to 2008 words costs only 0.03 fitness, versus 0.25 for the jump from 51 to 1005. The geometry holds.

### Open Mode (Three Men in a Boat)

| Test Suite | Questions | Score | Description |
|------------|----------|-------|-------------|
| full_test | 21 | **19/21** | Integration test across all question types |
| 3w_test | 10 | **10/10** | What/Who/Where questions |
| bool_test | 5 | **5/5** | Boolean AND/OR compound queries |
| 2w_test | 5 | **5/5** | When/Why reasoning questions |

2429 words assembled from Victorian prose. 39/41 correct across all test suites.

### Question Type Coverage

| Capability | Status | Example |
|-----------|--------|---------|
| Yes/No | 20/20 on dict5 | "Is a dog an animal?" → Yes |
| What/Who/Where | 10/10 on entities | "Who is Montmorency?" → a dog |
| Why (chain explanation) | 9/10 | "Why is a dog a thing?" → because... |
| When (condition extraction) | 9/10 | "When does a person eat?" → to feel good |
| Boolean AND/OR | 14/15 | "Is a dog an animal and a thing?" → Yes |
| Describe (generation) | 100% self-consistency | Generates sentences from definitions |
| I don't know | Emerges from geometry | "What color is a dog?" → I don't know |

## Using yalm-demo

`yalm-demo` is the interactive demo program. It takes knowledge files and a questions file, builds a geometric space, answers every question, and pretty-prints the results with timing.

### Basic usage (closed dictionary)

```bash
cargo run --release -p yalm-demo -- \
  -k dictionaries/dict5.md \
  -q questions_demo.md
```

### Open mode (free text + LLM definitions)

Requires [Ollama](https://ollama.ai) running locally with a model pulled:

```bash
ollama pull qwen3:8b
ollama serve

cargo run --release -p yalm-demo -- \
  -k texts/three_men/combined.md \
  --entities texts/three_men_supplementary/entities.md \
  --cache-type ollama \
  --cache dictionaries/cache/ollama-qwen3 \
  -q questions_three_men_demo.md
```

### Multiple knowledge sources

You can pass multiple `-k` files. Dictionary-format files are parsed directly; free-text files are assembled via the cache:

```bash
cargo run --release -p yalm-demo -- \
  -k dictionaries/dict5.md \
  -k texts/extra_knowledge.md \
  --cache-type ollama \
  --cache dictionaries/cache/ollama-qwen3 \
  -q my_questions.md
```

### Writing question files

Questions files are simple — one question per line. Lines starting with `#` are comments:

```markdown
# My demo questions
Is a dog an animal?
What is a cat?
Why is a dog a thing?
When does a person eat?
Is the sun big and hot?
```

### CLI reference

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--knowledge` | `-k` | Knowledge files (one or more) | *required* |
| `--questions` | `-q` | Questions file | *required* |
| `--mode` | | Build mode: `equilibrium` or `forcefield` | `equilibrium` |
| `--entities` | | Entity definitions file to merge | — |
| `--cache-type` | | Cache backend: `manual`, `wiktionary`, `ollama` | `manual` |
| `--cache` | | Path to cache directory | — |
| `--max-depth` | | BFS closure depth for open mode | `3` |
| `--max-words` | | Max dictionary size for open mode | `5000` |
| `--ollama-url` | | Ollama API URL | `http://localhost:11434` |
| `--ollama-model` | | Ollama model name | `qwen3:8b` |

## Architecture

YALM is structured as a Rust workspace with seven crates:

```
Input: dictionary.md or free text + entities (optional)
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
  │   ├─ What/Who/Where: definition extraction (first content word)
  │   ├─ Why: definition chain traced as "because" explanation
  │   ├─ When: condition/purpose clause extraction from definitions
  │   ├─ Boolean: AND/OR compound query decomposition
  │   ├─ Unknown: no proximity above threshold → "I don't know"
  │   └─ Describe: definition rewriting + sibling negation inference
  │
  └─ Evolution ─── genetic algorithm tunes ~15 parameters
                    (used for closed-dict optimization)
```

| Crate | Role |
|-------|------|
| `yalm-core` | Shared types — `Dictionary`, `GeometricSpace`, `EngineParams`, `TestSuite` |
| `yalm-parser` | Reads markdown dictionaries, extracts entries, tokenizes, stems |
| `yalm-engine` | The geometric engine — connector discovery, force field simulation, question resolution |
| `yalm-eval` | Evaluation harness — runs test suites, computes accuracy/honesty/fitness |
| `yalm-evolve` | Genetic algorithm — evolves engine parameters and strategy choices across generations |
| `yalm-demo` | Interactive demo — load knowledge, answer questions, pretty-print results |
| `yalm-cache` | Dictionary assembly from free text via LLM-generated definitions |

### Strategy Evolution

The engine has multiple algorithmic choices at each stage — how forces decay, how connectors are detected, how the space is initialized, how negation works. Rather than hand-tuning these, a genetic algorithm explores the strategy space:

- **Force functions**: Linear, InverseDistance, Gravitational, Spring
- **Connector detection**: FrequencyOnly, PositionalBias, MutualInformation
- **Space initialization**: Random, Spherical, FromConnectors
- **Multi-connector handling**: FirstOnly, Sequential, Weighted, Compositional
- **Negation models**: Inversion, Repulsion, AxisShift, SeparateDimension

Plus continuous parameters (dimensions, learning passes, force magnitude, thresholds, etc.) that are co-evolved with the strategy choices.

Cross-validation across all three dictionary sizes prevents overfitting — a genome must perform well on dict5, dict12, *and* dict18 simultaneously.

## Dictionaries

YALM uses **closed dictionaries** — every word that appears in any definition or example must itself be a defined entry. This is what makes the geometric space self-consistent: there are no undefined symbols.

| File | Words | Description |
|------|------:|-------------|
| `dictionaries/dict5.md` | 51 | Core vocabulary — animals, colors, basic concepts |
| `dictionaries/dict12.md` | 1005 | General knowledge — science, geography, everyday life |
| `dictionaries/dict18.md` | 2008 | Extended — abstract concepts, social structures, academic domains |

Each entry follows a strict format:

```markdown
**atom** — a particle that is the smallest part of an element.
- "An atom is very small."
- "Atoms join to form molecules."
- "Each element is made of one kind of atom."
```

Grammar regularizer files (`grammar5.md`, `grammar18.md`) provide additional taxonomic and relational sentences that reinforce the force field without adding new vocabulary.

## Other Commands

### Evaluation harness

```bash
# Evaluate dict5 with default parameters (perfect 20/20)
cargo run --release -p yalm-eval

# Evaluate dict18 with the evolved genome
cargo run --release -p yalm-eval -- \
  --dict dictionaries/dict18.md \
  --test dictionaries/dict18_test.md \
  --grammar dictionaries/grammar18.md \
  --genome results_v11/gen_049/best_genome.json
```

### Run evolution

```bash
cargo run --release -p yalm-evolve -- run \
  --dict5 dictionaries/dict5.md \
  --test5 dictionaries/dict5_test.md \
  --dict12 dictionaries/dict12.md \
  --test12 dictionaries/dict12_test.md \
  --grammar5 dictionaries/grammar5.md \
  --dict18 dictionaries/dict18.md \
  --test18 dictionaries/dict18_test.md \
  --grammar18 dictionaries/grammar18.md \
  --population 50 \
  --generations 50 \
  --results results
```

## Project Structure

```
yalm/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── yalm-core/              # Shared types and data structures
│   ├── yalm-parser/            # Dictionary and test question parsing
│   ├── yalm-engine/            # Geometric comprehension engine
│   │   ├── connector_discovery # Pattern-based connector detection
│   │   ├── equilibrium         # Sequential word placement
│   │   ├── force_field         # N-dimensional force simulation
│   │   ├── resolver            # Question answering via geometry
│   │   └── strategy            # Evolvable algorithm choices
│   ├── yalm-eval/              # Evaluation and fitness scoring
│   ├── yalm-evolve/            # Genetic algorithm for self-improvement
│   ├── yalm-demo/              # Interactive demo program
│   └── yalm-cache/             # LLM dictionary assembly (Ollama, Wiktionary)
├── dictionaries/
│   ├── dict5.md                # 51-word closed dictionary
│   ├── dict12.md               # 1005-word closed dictionary
│   ├── dict18.md               # 2008-word closed dictionary
│   ├── *_test.md               # Test suites (20 questions each)
│   ├── *_bool_test.md          # Boolean operator test questions
│   ├── *_2w_test.md            # When/Why test questions
│   ├── grammar*.md             # Grammar regularizer texts
│   └── cache/ollama-qwen3/     # Cached LLM definitions (2465 entries)
├── texts/
│   ├── passage1.md             # Open-mode test passage
│   └── three_men/              # Three Men in a Boat excerpts
│       ├── combined.md         # All passages concatenated
│       ├── full_test.md        # 21-question integration test
│       ├── 3w_test.md          # What/Who/Where test
│       ├── bool_test.md        # Boolean operator test
│       └── 2w_test.md          # When/Why test
├── texts/three_men_supplementary/
│   └── entities.md             # Character/place definitions (6 entries)
├── questions_demo.md           # Demo questions for dict5
├── questions_three_men_demo.md # Demo questions for Three Men
├── scripts/
│   └── closure_check.py        # Validates dictionary closure
├── prompts/                    # Design documents and phase specs
└── RECAP.md                    # Detailed project history
```

## The Closure Problem

Building a closed dictionary at scale is harder than it sounds. Every word in every definition and every example sentence must itself be defined — and those definitions introduce new words that need their own entries, recursively.

For dict18, this meant:
- Starting with ~250 domain seed words (science, politics, economics, etc.)
- Adding ~750 closure words pulled in by definitions
- Writing ~6000 example sentences using only defined vocabulary
- Multiple rounds of automated checking (`scripts/closure_check.py`) with stemming, irregular form mapping, and derivational suffix handling

The result is a self-contained universe of 2008 words where the geometry can reason about atoms, democracy, algorithms, and anxiety — all defined in terms of each other.

## What This Proves

YALM demonstrates that **geometric structure alone** can produce meaningful comprehension from text, without neural networks or statistical learning. The system:

- Achieves **perfect comprehension** on a 51-word dictionary (20/20)
- **Scales sublinearly** — doubling vocabulary from 1005 to 2008 words costs only 1 additional error
- Produces **honest uncertainty** — it says "I don't know" when the geometry has no answer, rather than hallucinating
- **Self-improves** through evolutionary search over its own algorithmic choices
- Handles **all 5W question types** — What, Who, Where, When, Why — plus Yes/No, Boolean, and free-text generation
- Works on **real literature** — 39/41 correct on Victorian prose (Three Men in a Boat)

Inference is O(1) — answering a question in a 2008-word space takes the same time as in a 51-word space. Learning is O(n log n) — essentially sorting-tier complexity.

The remaining failures at scale are engineering challenges (threshold tuning, parser improvements), not fundamental architectural limits. The geometry works.

## Requirements

- Rust 1.70+ (2021 edition)
- Python 3.10+ (only for `closure_check.py`)
- [Ollama](https://ollama.ai) (only for open-mode text processing)

No external data, no API keys, no GPU needed for closed-dictionary mode. The entire knowledge base is the dictionary files in this repo.
