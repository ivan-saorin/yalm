# r0x-004: MoE Geometric Gating — Expert Populations

**Branch:** pure-research
**Prerequisite:** r0x-003 Phase D (set operations working)
**Goal:** Build a Mixture-of-Experts architecture where the gating mechanism is geometric (SPL set operations), not neural.

## Core Idea

Multiple expert populations (general, scientific, literary) produce responses. Each response is rewritten in ELI5 by Ollama and ingested into a DAPHNE geometric space. SPL set operations determine agreement, disagreement, and relevance ranking — replacing the neural gating network in traditional MoE.

## Prior Art Search (Step 0 — Do This First)

Before writing any code, search for existing work. Use `paper-search2` MCP server.

### Required Searches

```
search_arxiv("mixture of experts gating mechanism geometric", max_results=10)
search_semantic("hyperbolic embedding hierarchical representation", year="2017-", max_results=10)
search_arxiv("Poincare embedding taxonomy knowledge graph", max_results=10)
search_semantic("mixture of experts non-neural gating routing", max_results=10)
search_arxiv("word sense disambiguation geometric multi-space", max_results=10)
```

### Key Authors / Papers to Look For

- Nickel & Kiela — Poincaré embeddings for hierarchical data
- Shazeer et al. — original MoE gating architectures
- Fedus et al. — Switch Transformer (sparse MoE)
- Any work on geometric or distance-based expert routing (not learned gates)
- Multi-sense embeddings (Neelakantan, Reisinger, Camacho-Collados)

### What We Need to Know

1. Has anyone replaced neural gating in MoE with geometric/distance-based routing?
2. Do hyperbolic embeddings handle taxonomic reasoning better than Euclidean (DAPHNE is Euclidean)?
3. Multi-sense embeddings: how do they handle polysemy? Compare with population-based approach
4. What's the state of the art for automatic ambiguity detection from embeddings?
5. ELI5 as inter-expert protocol: is there precedent for "simplified representation" as interoperability layer?

### Output

Log findings in `research/r0x-004_prior_art.md`.

Special attention: hyperbolic embeddings. If Poincaré space handles "is a" hierarchies natively, it might be a better metric space for DAPHNE than Euclidean. This could change the entire equilibrium engine.

---

## What To Build

Script: `research/r0x_004_moe_gating.py` (orchestrator) + Rust components

### Architecture

```
Query: "What is energy?"
  │
  ├─ Expert 1 (General): Ollama prompt "explain simply"
  │   → "Energy is the ability to do things and make things happen."
  │   → ELI5 rewrite (may be identity if already simple)
  │   → DAPHNE assembly → Population A
  │
  ├─ Expert 2 (Scientific): Ollama prompt "explain technically"  
  │   → "Energy is a scalar quantity, conserved in isolated systems..."
  │   → ELI5 rewrite → "energy is a number that stays the same..."
  │   → DAPHNE assembly → Population B
  │
  ├─ Expert 3 (Literary/Contextual): Ollama prompt with context
  │   → Uses Three Men vocabulary/style
  │   → ELI5 rewrite
  │   → DAPHNE assembly → Population C
  │
  └─ Geometric Gating:
      ├─ Intersection(A, B, C) → high-confidence core answer
      ├─ Intersection(A, B) \ C → factual but not contextual
      ├─ C \ Intersection(A, B) → context-specific additions
      └─ Symmetric Difference → disagreements / ambiguity flags
```

### Phase A: Single-Query Multi-Expert

Start minimal:

1. Pick 5 test queries from dict5_test.md
2. For each, generate 2 expert responses (general + scientific) via Ollama
3. ELI5 rewrite each response
4. Parse each ELI5 response into a micro-dictionary (5-15 words)
5. Run DAPHNE assembly + equilibrium on each micro-dictionary
6. Compute pairwise distances within each expert's space
7. Compare the two distance matrices

**Key measurement:** For the query "Is a dog an animal?":
- Expert 1 micro-dict distances: dog↔animal
- Expert 2 micro-dict distances: dog↔animal
- Do they agree? (correlation of overlapping word pairs)

### Phase B: Set Operations as Gating

Using r0x-003's SPL populations:

```python
# Intersection: words where BOTH experts place them
# in similar relative positions
def geometric_intersection(pop_a, pop_b, threshold=0.3):
    """Find word pairs where relative distances agree."""
    agreements = []
    for (w1, w2) in common_pairs(pop_a, pop_b):
        d_a = mean_distance(pop_a, w1, w2)
        d_b = mean_distance(pop_b, w1, w2)
        if abs(d_a - d_b) < threshold:
            agreements.append((w1, w2, (d_a + d_b) / 2))
    return agreements

# Difference: where experts disagree
def geometric_difference(pop_a, pop_b, threshold=0.3):
    """Find word pairs where distances diverge."""
    disagreements = []
    for (w1, w2) in common_pairs(pop_a, pop_b):
        d_a = mean_distance(pop_a, w1, w2)
        d_b = mean_distance(pop_b, w1, w2)
        if abs(d_a - d_b) >= threshold:
            disagreements.append((w1, w2, d_a, d_b))
    return disagreements
```

### Phase C: Ambiguity Detection

The "cell" test:
- Expert General: "cell" near "room", "small"
- Expert Scientific: "cell" near "organism", "alive"
- Intersection on "cell": empty or near-empty → DAPHNE *knows* the word is ambiguous

Compare with "energy":
- Expert General: "energy" near "do", "move"
- Expert Scientific: "energy" near "work", "move"
- Intersection on "energy": non-empty ("move" is bridge) → partial agreement

### Phase D: Importance Ranking

```python
def rank_by_centrality(population):
    """Words closer to more other words = more important."""
    centrality = {}
    for word in population.words:
        mean_dist = np.mean([
            distance(word, other)
            for other in population.words if other != word
        ])
        centrality[word.name] = 1.0 / (1.0 + mean_dist)
    return sorted(centrality.items(), key=lambda x: -x[1])
```

Intersection words ranked by centrality = the answer ordered by importance.

## Ollama Prompts

```
EXPERT_GENERAL = """
Define '{word}' in one sentence. Use only simple, common words.
Do not use technical terms. A 5-year-old should understand.
"""

EXPERT_SCIENTIFIC = """
Define '{word}' precisely in one sentence using scientific terminology.
Be accurate and specific.
"""

EXPERT_ELI5_REWRITE = """
Rewrite this definition using only very simple words.
Every word you use must be a common word a 5-year-old knows.
Keep the same meaning. One sentence only.

Definition: {text}
"""
```

## Output

```
research/
├── r0x_004_moe_gating.py
├── r0x_004_results.json
│   ├── per_query_expert_responses
│   ├── intersection_scores
│   ├── ambiguity_detection (cell vs energy)
│   └── importance_rankings
├── r0x_004_gating.png          # Visualization of agreement/disagreement
└── r0x_004_verdict.md
```

## Success Criteria

| Metric | Fail | Pass |
|--------|------|------|
| Expert distance correlation (agreeable terms) | < 0.3 | > 0.5 |
| Ambiguity detection (cell=ambiguous, energy=not) | Wrong | Correct |
| Importance ranking matches human intuition | Random | Meaningful |
| Ollama round-trips per query | > 10 | <= 6 |

If PASS: Geometric MoE gating works. This is a new architecture.
If PARTIAL: Gating works but ranking doesn't. Still useful for ambiguity detection.
If FAIL: Expert responses don't produce comparable geometries through ELI5. The protocol doesn't unify them.
