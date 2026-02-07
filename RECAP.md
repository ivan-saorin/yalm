# YALM — Project Recap

**Yet Another Language Model**
*A geometric comprehension engine that learns from text alone*

Last updated: 2025-02-06

---

## What YALM Is

YALM is a research project exploring whether a system can comprehend language through geometry — without neural networks, without pretrained models, without grammar rules, without any NLP library.

The system reads a closed dictionary (every word in every definition is itself defined), builds an N-dimensional space where words are points, discovers connectors ("is a", "can", "not") from text statistics, and answers questions by traversing definitions and measuring geometric distance.

## What Was Proven

### The geometry works for positive relationships

Words pushed together by shared connectors cluster meaningfully. Dog is near cat is near animal. Sun is near hot is near big. The force field discovers this structure from raw text with zero linguistic knowledge.

**Evidence:** 10/10 positive queries and 5/5 transitive reasoning chains pass on dict5 (50 words), and these results reproduce across every seed and every evolution run.

### The system generalizes across vocabulary sizes

A model tuned on 50 words (dict5) achieves identical fitness on 400 words (dict12). The overfitting gap reached exactly 0.00 in v10 — the geometry learned STRUCTURE, not entries.

**Evidence:** v10 combined fitness 0.7063 with primary = cross = 0.7063. v6b achieved the same zero gap at 0.7188. Multiple independent seeds confirm.

### Honesty emerges naturally from geometry

When a question asks about something outside the dictionary's scope ("What color is a dog?"), the system says "I don't know" because no geometric proximity exists. This wasn't coded — it's a property of threshold-based distance queries on a sparse space.

**Evidence:** 4/4 unknown questions pass consistently. The system never hallucinated an answer to an unknowable question.

### Grammar text works as a regularizer

A self-referential grammar document (grammar5.md), written entirely in dict5's 50-word vocabulary, describing what connectors like "is a" and "not" mean, prevents the geometric space from degenerating during evolution. Same seed, same parameters: with grammar → 0.7063 fitness. Without grammar → 0.4875 (collapse).

The grammar text didn't teach comprehension directly. It constrained the space to be consistent across two different text types (definitions and prose), which forced more robust geometry.

### Definition-chain traversal solves negation

Geometric proximity cannot distinguish "same category" from "same entity" (dog ≈ cat because both are animals). Adding a definition-chain check — does X's definition chain contain Y? — provides the negative evidence that proximity lacks. This brought dict5 from 13/20 to 20/20.

## Final Scores

| Dictionary | Words | Score | Fitness |
|------------|-------|-------|---------|
| dict5 | 50 | 20/20 | 1.0000 |
| dict12 | ~400 | 15/20 | 0.7500 |
| Combined (v11) | — | — | 0.8500 |

## Evolution Journey

| Phase | Version | Best Fitness | Key Advance |
|-------|---------|-------------|-------------|
| Baseline | v0.1 | 0.4375 | Pure geometry, no rules |
| Rule-based | (rejected) | 1.0000 | Expert system — not the point |
| Evolved params | v7d | 0.7812 | Parameter ceiling found |
| Cross-validation | v6b | 0.7188 | Zero overfitting gap |
| Grammar reinforcement | v10 | 0.7063 | Regularization, prevents collapse |
| Surgical fixes | v11 | 0.8500 | Chain negation + definition extraction |

## Architecture (Final)

```
Input: dictionary.md + grammar.md
  │
  ├─ Parser ─── extracts entries, definitions, examples, sentences
  │
  ├─ Connector Discovery ─── finds ["is"], ["is","a"], ["can"], ["not"], etc.
  │                           from word co-occurrence statistics
  │
  ├─ Force Field ─── positions words in N-dimensional space
  │                   connectors are force operators that push words
  │                   multiple passes, decaying force magnitude
  │
  ├─ Resolver (queries)
  │   ├─ Yes/No: geometric distance + definition-chain gate
  │   │          chain confirms (linked → Yes)
  │   │          chain denies (not linked + antonym detected → No)
  │   │          chain inconclusive → trust geometry
  │   ├─ What-is: definition extraction (first content word in definition)
  │   └─ Unknown: no proximity above threshold → "I don't know"
  │
  └─ Evolution ─── genetic algorithm tunes ~15 parameters
                    50 population × 50 generations
                    converged strategies: Gravitational, MutualInformation,
                    Spherical, Weighted, Inversion/SeparateDimension
```

Language: Rust. Pure, no ML libraries. Workspace with 5 crates:
yalm-core, yalm-parser, yalm-engine, yalm-eval, yalm-evolve.

## What Doesn't Work Yet (dict12 Failures)

The 5 remaining dict12 failures reveal specific architectural limits:

| Question | Expected | Got | Root Cause |
|----------|----------|-----|------------|
| Q04: Can a cat climb? | Yes | IDK | Chain needs 3+ hops through richer definitions |
| Q09: Does a plant need water? | Yes | IDK | "Need" is causal, not taxonomic — chain follows "is a" but not "requires" |
| Q13: Is ice hot? | No | IDK | "Ice" is a property word; resolver classifies as IDK instead of checking antonym chain |
| Q14: Is a rock alive? | No | IDK | Same issue — property-word classification prevents negation check |
| Q17: Is a mountain good? | Yes | No | Spurious chain path through high-connectivity words |

These decompose into three problems:
1. **Hop depth** (Q04): The chain traversal needs to follow longer paths in richer dictionaries.
2. **Relation types** (Q09): "Need" is a different relationship than "is a." The system only traverses taxonomic chains.
3. **Property-word routing** (Q13, Q14, Q17): The resolver misclassifies some questions, sending them to the wrong resolution path.

All three are resolver logic issues, not geometric or evolutionary.

## Honest Assessment

### What exceeded expectations

- **Zero overfitting** between dict5 and dict12. This was the most important result. The geometry generalizes.
- **Grammar as regularizer.** We expected it to teach comprehension. It did something better — it stabilized the optimization landscape.
- **Evolution convergence.** The GA consistently found the same strategy combination across seeds, confirming it's a real optimum, not noise.
- **Honesty for free.** No special mechanism needed. Geometric sparsity naturally produces "I don't know."

### What met expectations

- **Connector discovery.** Found the right patterns ("is a", "can", "not") with no linguistic input.
- **Transitive reasoning.** Dog → animal → thing works geometrically as predicted.
- **The 8x expansion ratio.** dict5 (50 words) → dict12 (~400 words) confirms vocabulary density grows nonlinearly with comprehension level.

### What underperformed

- **Pure geometric negation.** Four different negation models (Inversion, Repulsion, AxisShift, SeparateDimension), 6 phases of evolution, and not a single negation question ever passed through geometry alone. Negation required the definition-chain check — a symbolic operation, not a geometric one.
- **Connector-axis discrimination.** The evolved connector axes aren't specific enough to support per-axis queries. "Is a" pushes so many words that its force direction is an averaged blob, not a clean axis. Axis-specific projection (v7) was rejected by evolution at 96%.
- **Dual-space ensemble.** Grammar space had degenerate embeddings that hurt IDK precision. The ensemble added complexity without net benefit.

### The philosophical tension

The original vision was "everything emerges from geometry." The final system uses geometry for positive relationships and symbolic chain-traversal for negation. That's not failure — it's a finding. Geometry encodes similarity. Definitions encode identity. You need both.

The question for the next phase: can the symbolic chain traversal be replaced by a SECOND geometric operation (e.g., a definition-graph embedding), or is the symbolic layer irreducible? If irreducible, the architecture is a hybrid — geometry for association, symbols for discrimination. If reducible, there's a deeper geometric representation waiting to be found.

## Compute Profile

This entire project — 6 phases, hundreds of evolution generations, thousands of genome evaluations — ran on a single CPU core. No GPU. No cloud. Total compute: minutes, not hours.

A comparable transformer-based system would require:
- Tokenizer training
- Embedding layer (50M+ parameters for even a small model)
- Multi-head attention (GPU hours for training)
- Fine-tuning for question answering

YALM achieves 85% combined fitness on two dictionaries with ~15 tunable parameters and a geometric space that fits in a few kilobytes.

## What Comes Next

Proposed directions (not yet implemented):

1. **Fix the 5 dict12 failures** — resolver routing + deeper chain traversal + relation-type awareness
2. **Build dict18** (university level, ~2000 words) — tests scaling to genuine complexity
3. **Grammar-level progression** — grammar12 written in dict12 vocabulary, self-referential at each level
4. **Word-sense disambiguation** — split polysemous words into separate geometric points
5. **Open text comprehension** — read a children's story, answer questions about characters and events
6. **Self-authoring meta-knowledge** — the system describes its own discovered patterns as grammar text
7. **Adversarial reader/writer** — co-evolving generation and comprehension systems

The foundation is solid. The architecture is clean. The question is whether geometric comprehension scales beyond closed dictionaries into open text.

## Project Files

```
yalm/
├── crates/
│   ├── yalm-core/         Data structures and traits
│   ├── yalm-parser/        Dictionary, test, and grammar file parsing
│   ├── yalm-engine/        Force field + resolver (the heart)
│   ├── yalm-eval/          Fitness scoring
│   └── yalm-evolve/        Genetic algorithm
├── dictionaries/
│   ├── dict5.md             50 words, 5-year-old level, CLOSED
│   ├── dict5_test.md        20 test questions
│   ├── dict12.md            ~400 words, 12-year-old level, NEAR-CLOSED
│   ├── dict12_test.md       20 test questions
│   ├── grammar5.md          Grammar text in dict5 vocabulary
│   └── grammar5_test.md     20 grammar-aware test questions
├── prompts/
│   ├── 01_dict12_closure_audit.md
│   ├── 02_geometric_comprehension_engine.md
│   ├── 03_evolution_self_improvement.md
│   ├── 04_phase5_handoff.md
│   ├── 05_grammar_reinforcement.md
│   └── 06_surgical_fixes.md
├── results_v11/             Latest evolution results
└── RECAP.md                 This file
```
