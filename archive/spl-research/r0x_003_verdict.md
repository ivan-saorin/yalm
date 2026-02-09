# r0x-003 Verdict: DEAD

## Summary

SPL predator-prey dynamics do not produce meaningful word geometry when applied to YALM's constraint-satisfaction problem. The fundamental mismatch: SPL is designed for **function approximation** (prey converge to f(x)=y surfaces), while word positioning requires **multi-constraint satisfaction** (each word must simultaneously satisfy dozens of pairwise distance constraints). The predator-prey metaphor does not map cleanly.

## Scores

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| dict5 mean score (population) | 9.8/20 | >=18 | **FAIL** |
| dict5 best-of-pop | 10/20 | =20 | **FAIL** |
| dict5 mean positions | 10/20 | -- | -- |
| dict5 best positions | 10/20 | -- | -- |
| Distance Spearman vs YALM | 0.082 | >0.6 | **FAIL** |
| Union queryable | 71% | >85% | **FAIL** |
| Criteria passed | 1/5 | >=4 | **FAIL** |

## Phase-by-Phase Analysis

### Phase A: SPL Engine

- Training: 170.7s for 300 steps, 51 words x 30 prey each
- **Critical observation**: Predators die off rapidly (30 -> 3 by step 200). The violation energy landscape is too flat -- there's no clear "worse" prey to hunt. SPL predators work by chasing individual prey agents, but word positioning violations are *relational* (between pairs), not *individual* (per agent).
- Mean violation stabilizes at ~0.36 but doesn't decrease -- SPL dynamics are not optimizing the violation energy effectively.
- Discovered 10 connectors and 42 relations (matching YALM's output for dict5).

### Phase B: Dict5 Evaluation

- The 10/20 score comes almost entirely from **definition-chain lookups** (Q01-Q05, Q08, Q10, Q11, Q12, Q14), not geometry.
- Geometric distance is essentially random (Spearman 0.082 vs YALM) -- it provides no useful signal for yes/no/unknown discrimination.
- "What is a dog?" returns "a thing" instead of "an animal" -- the definition parser finds "thing" (from "a thing that lives") rather than following the chain to "animal".
- Q06/Q07 "Is a dog a thing?" fails because the BFS follows only content words and "thing" is structural (>20% doc frequency).
- Q13 "Is a ball an animal?" gives Yes because geometric fallback fires (distances are meaningless).
- Q17/Q18 should be "I don't know" but geometric proximity gives false positives.

### Phase C: Bimodality

- Some words (dog, cat, sun) show non-normal distributions (normaltest p < 0.05).
- However, this is likely due to random initialization spread rather than meaningful multi-modal convergence.
- Without Montmorency in dict5, we cannot test the true bimodality hypothesis.
- The non-normality observed is not evidence of SPL capturing multiple semantic interpretations -- it's just noise.

### Phase D: Set Operations

- **Science5 discovered 0 connectors** -- the science dictionary is too small for frequency-based pattern discovery (no pattern appears >= 2 times).
- This means the science5 SPL engine has no forces at all -- prey just drift randomly.
- Union produces 61 words correctly (17 bridge terms detected).
- Intersection finds 10/17 bridge terms (position matching with radius=2.0 misses words whose populations drifted differently).
- Union queryable at 71% -- below the 85% threshold. The 2 failures are: "Is energy a thing?" (BFS can't traverse from energy to thing with 0 connectors) and "What is a dog?" (definition parsing issue).

## Root Cause Analysis

The fundamental failure has three components:

1. **Wrong optimization landscape**: SPL optimizes per-agent fitness (each prey independently tries to be on the "solution surface"). But word positioning is a **coupled constraint system** -- you can't evaluate a single word's position without knowing all its partners' positions. This creates a chicken-and-egg problem that SPL's local dynamics can't solve.

2. **Predator-prey mismatch**: In SPL, predators hunt individual prey. In word positioning, the "predator" (a violated constraint) targets a *pair* of words. There's no natural way to assign blame -- is word A wrong, or word B? SPL's capture mechanics can't express this.

3. **No gradient signal**: SPL prey move by fleeing predators + drifting toward solutions. But the violation gradient in word space is weak and noisy (it depends on mean positions of partner populations, which are themselves moving). YALM's force-field directly computes displacement vectors along connector axes -- this is a much stronger signal.

## Comparison with YALM Force-Field

| Aspect | YALM Force-Field | SPL Predator-Prey |
|--------|------------------|-------------------|
| Optimization | Direct force vectors | Indirect flee/drift |
| Coupling | Pairwise (explicit) | Individual (implicit) |
| Signal strength | Strong (projection on axis) | Weak (mean of partners) |
| Convergence | 50 passes, deterministic | 300 steps, stochastic |
| Score | 20/20 | 10/20 |
| Speed | ~0.1s | ~170s |
| Predator survival | N/A | Die off rapidly |

## Set Operations: Still Potentially Useful?

The set operations (union, intersection, difference) are conceptually sound and work correctly at the population level. However, they only add value if the underlying positions are meaningful. With SPL producing essentially random geometry, the set operations produce correctly-structured but semantically empty results.

If YALM's force-field were wrapped to produce **multiple equilibria** (different random seeds, different parameter perturbations), population-level set operations could still be useful -- but that doesn't require SPL dynamics.

## Conclusion

**DEAD.** SPL's predator-prey dynamics are not a viable replacement for YALM's force-field equilibrium. The predator-prey metaphor maps poorly to multi-constraint word positioning.

The experiment confirms that YALM's geometric structure is **not** an artifact of the optimization method -- it requires explicit pairwise force vectors, not population-based individual fitness. The force-field's direct coupling (forces between word pairs projected onto connector axes) is essential, not replaceable by ecological dynamics.

### Implications for r0x-004/005

- r0x-004 (SPL as cognitive layer) is **not affected** -- it uses SPL for embedding space organization, not word positioning.
- r0x-005 (SPL steering) is **not affected** -- it operates on LLM hidden states, not YALM geometry.
- The set operations concept remains valid for future work with multiple force-field runs.
