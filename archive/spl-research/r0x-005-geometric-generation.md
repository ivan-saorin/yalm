# r0x-005: Geometric Generation — Trajectory-Guided Writing

**Branch:** pure-research
**Prerequisite:** r0x-003 (SPL equilibrium) + r0x-004 Phase A (expert ELI5 pipeline)
**Goal:** Test whether SPL trajectories through geometric space can guide coherent text generation, using the expert-ELI5-reorder architecture.

## The Problem

DAPHNE comprehends but doesn't generate. Phase 13 (describe mode) rewrites definitions. Real generation requires producing *new* text that is geometrically coherent.

Direct generation from ELI5 geometry produces baby-talk. Direct retrieval from source text produces collage. Neither is acceptable.

## Prior Art Search (Step 0 — Do This First)

Before writing any code, search for existing work. Use `paper-search2` MCP server.

### Required Searches

```
search_arxiv("random walk knowledge graph text generation", max_results=10)
search_semantic("graph traversal coherent text generation", year="2018-", max_results=10)
search_arxiv("trajectory semantic space language generation", max_results=10)
search_semantic("geometric text generation non-autoregressive", max_results=10)
search_arxiv("knowledge graph to text linearization", max_results=10)
```

### Key Authors / Papers to Look For

- Koncel-Kedziorski et al. — graph-to-text generation
- Ribeiro et al. — investigating graph-to-text approaches
- Any work on planning-then-writing (content planning + surface realization)
- Diffusion models for text (Li et al., Gong et al.) — non-autoregressive generation
- Topic-guided generation via semantic space navigation

### What We Need to Know

1. Does random walk on semantic graphs produce coherent ordering? What's needed beyond walk?
2. Content planning literature: how is "what to say and in what order" typically solved?
3. Has anyone used geometric centrality as importance ranking for text?
4. Self-consistency verification: is round-trip validation (generate → comprehend → verify) studied?
5. Non-autoregressive generation: any geometric approaches that bypass token-by-token?

### Output

Log findings in `research/r0x-005_prior_art.md`.

Critical: if trajectory-based text ordering is known to fail (random walks produce incoherent sequences), document WHY and stop before Phase B. The prior art search should prevent wasted effort on the most speculative experiment.

---

## The Architecture

```
Query: "Describe Montmorency"
  │
  ├─ Step 1: SPL Trajectory
  │   Release agent at "montmorency" in combined population
  │   Record visited regions: [dog, animal, small, move, person-like]
  │   This is the SEMANTIC PLAN — what to say, in what order
  │
  ├─ Step 2: Expert Generation (per region)
  │   For each region in trajectory:
  │     Ollama generates a sentence about montmorency + region concept
  │     e.g., region "dog": "Montmorency is the group's fox-terrier."
  │     e.g., region "person-like": "He observes events with a critical eye."
  │
  ├─ Step 3: ELI5 Validation
  │   Each generated sentence → ELI5 rewrite → DAPHNE comprehension
  │   Does the ELI5 version land in the correct region?
  │   YES → keep sentence
  │   NO → sentence is hallucination, discard
  │
  └─ Step 4: Assembly
      Concatenate validated sentences in trajectory order
      = geometrically coherent paragraph
```

## What To Build

Script: `research/r0x_005_generation.py`

### Phase A: Trajectory Generation

```python
def generate_trajectory(spl_population, start_word, max_steps=5):
    """
    Release agent at start_word.
    At each step, move to the nearest unvisited region
    with highest predator pressure (= most semantic relevance).
    """
    visited = [start_word]
    current = start_word

    for _ in range(max_steps):
        neighbors = get_nearest_unvisited(
            spl_population, current, visited, k=10
        )
        # Pick neighbor with highest "information gain"
        # = most connections to other unvisited regions
        next_word = max(neighbors, key=lambda w:
            count_connections(spl_population, w, exclude=visited)
        )
        visited.append(next_word)
        current = next_word

    return visited
```

Test on dict5: trajectory from "dog" should visit meaningful regions, not random walk.

### Phase B: Expert Sentence Generation

```python
GENERATION_PROMPT = """
Write ONE sentence about {subject} that emphasizes its relationship to {concept}.
Use natural English. The sentence should be informative and specific.
Do not start with "It" or "{subject} is".

Subject: {subject}
Concept: {concept}
"""

# For each step in trajectory
sentences = []
for concept in trajectory[1:]:  # skip start word
    response = ollama_generate(
        GENERATION_PROMPT.format(subject=start_word, concept=concept)
    )
    sentences.append(response)
```

### Phase C: ELI5 Validation Loop

```python
def validate_sentence(sentence, expected_region, dafhne_engine):
    """
    Rewrite sentence to ELI5.
    Parse into micro-dictionary.
    Check if key terms land near expected_region in DAPHNE space.
    """
    eli5 = ollama_eli5_rewrite(sentence)
    micro_dict = dafhne_assemble(eli5)
    distances = dafhne_distances(micro_dict, expected_region)

    # The ELI5 version should be geometrically close to the target region
    return min(distances.values()) < THRESHOLD
```

Sentences that fail validation are discarded or regenerated (max 2 retries).

### Phase D: Comparative Test

Generate descriptions for 5 subjects using:
1. **DAPHNE Phase 13 describe mode** (baseline — definition rewriting)
2. **Trajectory-guided generation** (this experiment)
3. **Raw Ollama** (no geometric guidance — control)

Human evaluation (Ivan): rate each output 1-5 on:
- Coherence (does it make sense?)
- Informativeness (does it say useful things?)
- Ordering (are the most important things first?)
- No hallucination (everything verifiable from source?)

### Test Subjects

```
1. "montmorency"  — entity with rich narrative context
2. "dog"          — simple concept, dict5
3. "thames"       — entity with relational context
4. "energy"       — if dict_science5 from r0x-003 exists
5. "person"       — abstract concept, multiple facets
```

## Output

```
research/
├── r0x_005_generation.py
├── r0x_005_trajectories.json    # Raw trajectories for each subject
├── r0x_005_generated.json       # Expert sentences + validation results
├── r0x_005_comparison.md        # Side-by-side: describe vs trajectory vs raw
└── r0x_005_verdict.md
```

## Success Criteria

| Metric | Fail | Pass |
|--------|------|------|
| Trajectory visits meaningful regions | Random-looking | Semantically coherent path |
| Validation rate (sentences kept) | < 50% | > 70% |
| Human rating vs describe mode | Worse or equal | Better on ordering + info |
| Human rating vs raw Ollama | Worse | Better on no-hallucination |
| Self-consistency (re-read = same trajectory) | < 60% | > 80% |

If PASS: Geometric generation works. DAPHNE becomes a comprehension-AND-generation engine.
If PARTIAL (trajectory good, generation mediocre): The planning layer works but needs better expert prompts.
If FAIL: Generation requires capabilities beyond geometric structure. DAPHNE stays a comprehension engine.

## Critical Note

This is the most speculative experiment. The hypothesis that geometric trajectories produce *coherent* text ordering is unproven. If Phase A trajectories look random, STOP — don't proceed to Phase B-D. Document the trajectory patterns and close.
