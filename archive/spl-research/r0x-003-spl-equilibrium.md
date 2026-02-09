# r0x-003: SPL Equilibrium — Population-Based Word Positioning

**Branch:** pure-research
**Prerequisite:** None (independent path from r0x-001/002)
**Goal:** Replace DAPHNE's force-field equilibrium with SPL predator-prey dynamics and verify it produces equivalent or better geometry.

## Hypothesis

DAPHNE's equilibrium engine uses iterative force application with decaying learning rate — essentially gradient descent. SPL uses predator-prey population dynamics with O(n) scaling. If the geometric structure is fundamental (not an artifact of the optimization method), both should converge to equivalent spaces.

Bonus: SPL maintains a *population* of configurations, not a single point. This could resolve ambiguities (Montmorency as dog vs person-like).

## Prior Art Search (Step 0 — Do This First)

Before writing any code, search for existing work. Use `paper-search2` MCP server.

### Required Searches

```
search_arxiv("predator prey optimization algorithm convergence", max_results=10)
search_semantic("particle swarm optimization semantic space word positioning", max_results=10)
search_arxiv("population-based optimization word embedding", max_results=10)
search_semantic("evolutionary game theory representation learning", year="2018-", max_results=10)
search_arxiv("set operations learned representations model merging", max_results=10)
```

### Key Authors / Papers to Look For

- Lotka-Volterra dynamics applied to optimization (any field)
- Particle Swarm Optimization for NLP / embedding learning
- Model merging literature (Wortsman, Ilharco et al. — model soups, task arithmetic)
- Ecological dynamics as computation (Hopfield-adjacent work)
- Any work on maintaining population diversity in learned representations

### What We Need to Know

1. Do predator-prey dynamics converge to stable equilibria for high-dimensional positioning?
2. What convergence guarantees exist (or provably don't)?
3. Has anyone done set operations (union/intersection) on learned representations?
4. Model merging in NN: what works, what breaks? (Task arithmetic, TIES, DARE)
5. Is bimodal distribution maintenance studied in population-based optimization?

### Output

Log findings in `research/r0x-003_prior_art.md`.

Critical focus: model merging / task arithmetic literature. If weight-space arithmetic works for NNs, the equivalent in geometric space should be even cleaner. Look for theoretical justification.

---

## What To Build

New crate: `crates/dafhne-spl/` (or standalone `research/r0x_003_spl_equilibrium.rs`)

### Phase A: Minimal SPL Engine

Implement the core SPL loop adapted to DAPHNE's domain:

```rust
struct Word {
    id: usize,
    name: String,
    positions: Vec<Vec<f64>>,  // POPULATION of positions, not single point
}

struct Connector {
    name: String,        // "is a", "can", "not"
    direction: Vec<f64>, // force direction in N-dim space
    strength: f64,
}

struct SPLEngine {
    words: Vec<Word>,
    connectors: Vec<Connector>,
    population_size: usize,  // e.g., 50 configurations per word
}
```

**Predator-prey mapping:**
- **Prey** = word positions. They move to minimize energy (same as current equilibrium).
- **Predators** = connectors. They "hunt" word pairs that violate their relationship.
  - "is a" hunts pairs where definition says "X is a Y" but distance(X,Y) is large
  - "not" hunts pairs that are too close despite negative relationship
  - "can" hunts capability mismatches
- **Prey movement** = words shift away from predator pressure (adjust position)
- **Predator movement** = connectors adjust their direction/strength based on how many violations they find

### Phase B: Run on dict5

```
Input: dict5.md + grammar5.md (same as current DAPHNE)
Parameters: Same connector discovery output
Output: Population of 50 equilibrium configurations for 51 words
```

Compare with current DAPHNE equilibrium:
1. For each configuration in the population, compute dict5_test scores (20 questions)
2. Compute mean score across population
3. Compute best-of-population score
4. Compare distances: mean-population distances vs single-equilibrium distances (Spearman)

### Phase C: The Montmorency Test

The key test for population advantage:

```
In current DAPHNE:
  Montmorency ↔ dog = 1.14
  Montmorency ↔ person = 0.98
  (inverted — geometry sees anthropomorphism)

In SPL population:
  - Some configurations: Montmorency near dog (definition-driven)
  - Some configurations: Montmorency near person (narrative-driven)
  - Distribution of distances should be BIMODAL
```

If bimodal: SPL captures both interpretations simultaneously. The resolver can query the distribution mode instead of a single point.

### Phase D: Set Operations Proof

The killer feature. After Phase B produces SPL populations for dict5:

1. Run SPL on a SECOND dictionary (dict5 + 10 science words, manually added)
2. Compute Union(dict5_spl, science_spl)
3. Compute Intersection(dict5_spl, science_spl)
4. Verify: Intersection should contain only the bridge terms
5. Verify: Union should be queryable on both vocabularies

For the science words, create a minimal `dict_science5.md`:
```
energy: the ability to do work. it can make things move or change.
cell: a very small part of a living thing. all animals and plants have cells.
force: a push or a pull. it can make things move.
matter: anything that takes up space. all things are made of matter.
wave: a way that energy moves. light and sound move in waves.
atom: a very very small thing. all matter is made of atoms.
molecule: a group of atoms. water is a molecule.
gravity: a force that pulls things down. it keeps us on the ground.
temperature: how hot or cold something is.
oxygen: a gas in the air. animals need it to live.
```

Note: "energy", "force", "move", "thing", "animal", "live" are bridge terms (exist in dict5 too).

## Output

```
research/
├── r0x_003_spl_equilibrium.rs  # or crates/dafhne-spl/
├── dict_science5.md            # Test dictionary
├── r0x_003_results.json
│   ├── dict5_scores (per-configuration and aggregate)
│   ├── distance_correlation (SPL vs original equilibrium)
│   ├── montmorency_distribution (bimodal test)
│   └── set_operations (union/intersection verification)
├── r0x_003_montmorency.png     # Distance distribution histogram
├── r0x_003_sets.png            # Visualization of union/intersection
└── r0x_003_verdict.md
```

## Success Criteria

| Metric | Fail | Pass |
|--------|------|------|
| dict5 mean score | < 16/20 | >= 18/20 |
| dict5 best-of-pop | < 20/20 | = 20/20 |
| Distance Spearman vs original | < 0.4 | > 0.6 |
| Montmorency bimodal | Unimodal | Bimodal |
| Set Union queryable | < 70% | > 85% |
| O(n) scaling | Worse than original | Equal or better |

If PASS: SPL equilibrium is a viable replacement. Merge candidate.
If FAIL on scores but PASS on sets: SPL is useful as a layer above, not replacement.
If FAIL on everything: SPL dynamics don't map to word positioning. Document and close.
