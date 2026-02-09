# DAFHNE — Definition-Anchored Force-field Heuristic Network Engine

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

> O(n log n) learning · O(1) inference · Zero GPU · Zero training corpus

**A geometric comprehension engine that understands language through physics, not statistics.**

**TL;DR** — DAFHNE constructs word vector spaces from closed ELI5 dictionary definitions using typed force-field equilibrium, without neural networks. The pipeline: closed dictionary → automatic connector discovery → typed force-field equilibrium → multi-space architecture → bootstrap self-improvement. 51 words → 20/20 perfect. 2008 words → 14/20 with sublinear decay. 5 geometric spaces → 45/50 (90%). The honest finding: geometry encodes similarity, definitions encode identity, absence encodes uncertainty. You need all three.

---

DAFHNE reads a closed dictionary, places every word as a point in N-dimensional space, and lets sentences act as physical forces that push related words together. After the force field reaches equilibrium, the geometry *is* the knowledge — questions become distance measurements, reasoning becomes chain traversal, and "I don't know" falls out naturally when nothing is close enough.

No neural networks. No embeddings. No training corpus. Just definitions, forces, and geometry.

## Try It — Quick Demo

```bash
cargo build --release
cargo run --release -p dafhne-demo -- -k dictionaries/dict5.md -q questions_demo.md
```

Output:

```
  ╔════════════════════════════════════════════════════╗
  ║           DAFHNE — Geometric Comprehension           ║
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

## How It Works

```
Dictionary → Connector Discovery → Force Field → Equilibrium → Geometric Space
                                                                      ↓
                                                          Question → Resolver → Answer
```

1. **Read** a closed dictionary (every word in every definition is itself defined)
2. **Discover connectors** — recurring patterns like "is a", "can", "not" — from text statistics alone
3. **Build** an N-dimensional space where each connector is a physical force pushing related words together
4. **Answer** questions by measuring geometric distance, traversing definition chains, and detecting absence

**Transitive reasoning is free**: if dog is near animal and animal is near thing, dog is near thing.
**Honesty is free**: no proximity = "I don't know" instead of hallucinating.

For the full step-by-step explanation, see [docs/architecture.md](docs/architecture.md).

## Results

| Dictionary | Words | Score | Notes |
|-----------|------:|------:|-------|
| dict5 | 51 | **20/20** | Perfect score, closed dictionary |
| dict12 | 1005 | **14/20** | 20x vocabulary, sublinear decay |
| dict18 | 2008 | **14/20** | 40x vocabulary, near-zero additional loss |
| Three Men in a Boat | 2429 | **19/21** | Victorian literature via Ollama |
| Multi-space (5 spaces) | 50 Qs | **45/50** | CONTENT + MATH + GRAMMAR + TASK + SELF |

All 5W question types covered: What, Who, Where, When, Why + Yes/No + Boolean AND/OR + Describe.

For comprehensive results across all phases and test suites, see [docs/results.md](docs/results.md).

## Multi-Space Architecture (Phase 16-19)

Five independent geometric spaces ("thought domains"):

| Space | Dictionary | Domain |
|-------|-----------|--------|
| CONTENT | dict5.md | Physical world — animals, properties, actions |
| MATH | dict_math5.md | Numbers, operations, counting |
| GRAMMAR | dict_grammar5.md | Language structure — nouns, verbs, sentences |
| TASK | dict_task5.md | Dispatcher — routes queries to correct domain(s) |
| SELF | dict_self5.md | Identity — what DAFHNE is, can do, cannot do |

Each space runs its own equilibrium independently. Connected at query time through bridge terms and geometric routing.

A **bootstrap loop** (Phase 19) enables self-improvement: describe() generates text, connector discovery finds new patterns, the space rebuilds with richer grammar — all without changing the dictionary.

## Using dafhne-demo

### Closed dictionary (default)

```bash
cargo run --release -p dafhne-demo -- \
  -k dictionaries/dict5.md \
  -q questions_demo.md
```

### Open mode (free text + Ollama)

```bash
ollama pull qwen3:8b && ollama serve

cargo run --release -p dafhne-demo -- \
  -k texts/three_men/combined.md \
  --entities texts/three_men_supplementary/entities.md \
  --cache-type ollama \
  --cache dictionaries/cache/ollama-qwen3 \
  -q questions_three_men_demo.md
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

Seven Rust crates, ~9300 lines total, zero ML dependencies:

| Crate | Role |
|-------|------|
| `dafhne-core` | Shared types — Dictionary, GeometricSpace, EngineParams |
| `dafhne-parser` | Markdown dictionary and test file parsing |
| `dafhne-engine` | Core engine — connector discovery, force field, equilibrium, resolver, multispace, bootstrap |
| `dafhne-eval` | Evaluation harness — test suites, fitness scoring |
| `dafhne-evolve` | Genetic algorithm — parameter and strategy evolution |
| `dafhne-demo` | Interactive demo with pretty-printing |
| `dafhne-cache` | Dictionary assembly from free text via LLM (Ollama) |
| `dafhne-server` | HTTP server — Ollama, OpenAI, MCP APIs + web chat UI |

Strategy evolution explores 5 algorithmic dimensions: force function, connector detection, space initialization, multi-connector handling, and negation model — plus ~15 continuous parameters, all co-evolved by genetic algorithm.

For the full architecture deep-dive, see [docs/architecture.md](docs/architecture.md).

## Dictionaries

| File | Words | Description |
|------|------:|-------------|
| `dictionaries/dict5.md` | 51 | Core vocabulary — animals, colors, basic concepts |
| `dictionaries/dict12.md` | 1005 | General knowledge — science, geography, everyday life |
| `dictionaries/dict18.md` | 2008 | Abstract concepts, social structures, academic domains |

All are **closed dictionaries** — every word in every definition is itself defined. This creates a self-consistent universe of meaning with no undefined symbols.

## DAFHNE Server

Run DAFHNE as an API server with web chat, Ollama/OpenAI compatibility, and MCP tools:

```bash
cargo run --release -p dafhne-server -- \
  --data-dir ./dictionaries \
  --multi-genome ./results_multi/gen_029/best_genome.json
```

Then open `http://localhost:3000/chat` for the web UI, or connect any Ollama/OpenAI client.

For full instructions — CLI reference, curl examples, MCP setup for Claude Code, Docker, and how to connect chat clients — see **[docs/server.md](docs/server.md)**.

## Other Commands

```bash
# Evaluation harness
cargo run --release -p dafhne-eval

# Run evolution
cargo run --release -p dafhne-evolve -- run \
  --dict5 dictionaries/dict5.md --test5 dictionaries/dict5_test.md \
  --population 50 --generations 50 --results results
```

## Project Structure

```
dafhne/
├── crates/                     # 7 Rust crates
│   ├── dafhne-core/              # Shared types
│   ├── dafhne-parser/            # Dictionary parsing
│   ├── dafhne-engine/            # Core engine (connector discovery, force field, resolver, multispace)
│   ├── dafhne-eval/              # Evaluation and fitness
│   ├── dafhne-evolve/            # Genetic algorithm
│   ├── dafhne-demo/              # Interactive demo
│   ├── dafhne-cache/             # LLM dictionary assembly
│   └── dafhne-server/            # HTTP server (Ollama, OpenAI, MCP, web chat)
├── dictionaries/               # Closed dictionaries + tests + grammar files
├── texts/                      # Open-mode texts (Three Men in a Boat)
├── docs/                       # Deep-dive documentation
├── reports/                    # Audit and analysis reports
├── prompts/                    # Phase design documents
└── RECAP.md                    # Full project history
```

## Deep-Dive Documentation

| Document | Description |
|----------|-------------|
| [docs/server.md](docs/server.md) | Server setup — running, testing, MCP, Docker |
| [docs/architecture.md](docs/architecture.md) | Full pipeline walkthrough: dictionary to answers |
| [docs/results.md](docs/results.md) | Comprehensive scores across all test suites and phases |
| [docs/design_decisions.md](docs/design_decisions.md) | Key choices and alternatives considered |
| [docs/prior_art.md](docs/prior_art.md) | Relationship to existing work (20 citations) |
| [docs/limitations.md](docs/limitations.md) | Known limitations and honest assessment |
| [docs/roadmap.md](docs/roadmap.md) | Future directions and open research questions |
| [reports/19b_code_audit.md](reports/19b_code_audit.md) | 24-finding code audit against founding principles |

## What This Proves

DAFHNE demonstrates that **geometric structure from definitions** can produce meaningful comprehension without neural networks. The system achieves perfect scores on small dictionaries, scales sublinearly, produces honest uncertainty, self-improves through evolution and bootstrap, and handles all 5W question types plus Boolean operators and text generation.

The honest finding: geometry encodes similarity (what's related), definitions encode identity (what IS what), and absence encodes uncertainty (what's unknown). You need all three. The hybrid geometry + symbolic chain traversal is not a compromise — it's the architecture.

## Requirements

- Rust 1.70+ (2021 edition)
- Python 3.10+ (only for `closure_check.py`)
- [Ollama](https://ollama.ai) (only for open-mode text processing)

No external data, no API keys, no GPU needed for closed-dictionary mode.

## Citation

If you use DAFHNE in your research, please cite:

```bibtex
@software{dafhne2026,
  title     = {DAFHNE: Definition-Anchored Force-field Heuristic Network Engine --- A Geometric Comprehension Engine},
  author    = {Ivan Saorin},
  year      = {2026},
  url       = {https://github.com/ivan-saorin/dafhne},
  note      = {Geometric comprehension from closed dictionaries via typed force-field equilibrium}
}
```

### Related Work

DAFHNE builds on ideas from conceptual spaces, knowledge graph embeddings, and dictionary-based learning. For a full analysis with 20 citations, see [docs/prior_art.md](docs/prior_art.md). Key references:

- Gardenfors, P. (2000). *Conceptual Spaces: The Geometry of Thought*. MIT Press.
- Bordes, A. et al. (2013). "Translating Embeddings for Modeling Multi-relational Data." *NeurIPS 2013*.
- Mikolov, T. et al. (2013). "Efficient Estimation of Word Representations in Vector Space." *arXiv:1301.3781*.
- Tissier, J. et al. (2017). "Dict2Vec: Learning Word Embeddings using Lexical Dictionaries." *EMNLP 2017*.
- Fruchterman, T. & Reingold, E. (1991). "Graph Drawing by Force-Directed Placement." *Software: Practice and Experience 21(11)*.
