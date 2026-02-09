# DAPHNE Design Decisions

> Key architectural choices, the alternatives considered, and why we chose what we chose.

---

## 1. Why Closed Dictionaries? (vs. corpus-based learning)

**Decision**: DAPHNE learns from a closed dictionary where every word in every definition is itself defined. No external corpus.

**Alternatives considered**:
- **Corpus-based**: Learn from raw text (like Word2Vec, GloVe). Requires billions of tokens, statistical co-occurrence, and massive scale.
- **Hybrid**: Dictionary + corpus co-occurrence.

**Why this choice**:
- **Self-consistency**: A closed dictionary is a self-contained universe. Every symbol is defined in terms of other symbols. There are no undefined atoms.
- **Minimal data**: 51 words is enough for perfect comprehension. No training corpus needed.
- **Verifiable**: Every definition can be audited. Every answer can be traced to a specific definition chain. No black box.
- **The ELI5 insight**: Simple definitions (5-year-old level) produce better geometry than complex ones. Fewer unique words = higher connector density = stronger force signal.

**Trade-off**: Cannot learn from free text directly. Open mode requires an LLM (Ollama) to translate free text into ELI5 definitions — the LLM is a preprocessor, not a comprehension engine.

---

## 2. Why Force Fields? (vs. co-occurrence matrices)

**Decision**: Words are positioned by physical forces — connectors push related words together. The space reaches equilibrium through iterative force application.

**Alternatives considered**:
- **Co-occurrence matrix factorization** (GloVe-style): Count word co-occurrences, apply SVD.
- **Random walks** (DeepWalk/Node2Vec-style): Generate random walks on definition graph, learn embeddings.
- **Direct optimization**: Minimize a loss function over word positions.

**Why this choice**:
- **Typed forces**: Each connector is a different force type with its own direction. "is a" pushes differently than "can" or "not". Co-occurrence matrices collapse all relationships into a single similarity score.
- **Physical intuition**: The system literally feels like physics — springs connecting related words, equilibrium as a minimum-energy state. This makes the behavior interpretable.
- **Incremental**: New words can be added by placing them at the centroid of their neighbors and relaxing locally. No need to retrain the entire space.
- **Evolvable**: Force function, decay rate, connector handling — all are strategy choices that the genetic algorithm can explore.

**Trade-off**: Force fields can oscillate or fail to converge with poorly tuned parameters. Sequential equilibrium (Phase 08) replaced global optimization with incremental placement to avoid this.

---

## 3. Why Genetic Evolution? (vs. gradient descent)

**Decision**: Parameters and strategies are evolved by a genetic algorithm, not optimized by gradient descent.

**Alternatives considered**:
- **Gradient descent**: Backpropagation through the force field.
- **Bayesian optimization**: Model the parameter-fitness surface.
- **Grid search**: Exhaustive parameter sweep.
- **Manual tuning**: Expert selection.

**Why this choice**:
- **Discrete + continuous**: The genetic algorithm evolves both continuous parameters (force magnitude, thresholds) and discrete strategy choices (Spring vs. Gravitational force, AxisShift vs. Inversion negation). Gradient descent only handles continuous parameters.
- **No differentiable path**: The fitness function (accuracy + honesty on test questions) is not differentiable. The chain between parameters → force field → equilibrium → distance → answer → score has no smooth gradient.
- **Exploration**: A population of 50 genomes explores the search space in parallel. Gradient descent gets stuck in local optima.
- **Self-documenting**: Each generation's best genome, fitness stats, and strategy distribution are saved. The evolution trajectory IS the research log.

**Trade-off**: Slower convergence than gradient methods. 50 generations x 50 population = 2500 evaluations, each building a complete geometric space. Takes ~30 minutes on one core.

**Outcome**: The GA consistently converges to Spring force function + AxisShift negation + FromConnectors initialization across independent seeds, confirming a real optimum.

---

## 4. Why Definition-Chain Gate? (geometry alone fails for negation)

**Decision**: Yes/No questions use geometric distance as primary signal, then gate through definition-chain traversal to confirm.

**Alternatives considered**:
- **Pure geometry**: Distance below threshold = Yes, above = No.
- **Separate negation space**: A dedicated "not" dimension.
- **Learned threshold per pair**: Different thresholds for different word categories.
- **Graph-based reasoning**: Traverse the definition graph without geometry.

**Why this choice**:
- **The fundamental problem**: Dog and cat are both close to animal. Geometry correctly places them near each other (they ARE similar). But "Is a dog a cat?" should be No. Distance cannot distinguish "same category" from "is-a". This is inherent in metric spaces — distance is symmetric and cannot encode direction.
- **The chain adds asymmetry**: "dog's definition contains 'animal'" is a directed, asymmetric fact. The chain traversal provides identity evidence that proximity cannot.
- **Scores before/after**: dict5 13/20 (pure geometry) → 20/20 (with chain gate). The improvement is decisive.

**The philosophical tension**: The original vision was "geometry IS the knowledge." The chain gate means geometry is HALF the knowledge. This IS the finding — not a failure, but a discovery: geometric spaces encode similarity; directed definitions encode identity. You need both.

**Trade-off**: The system is now a hybrid (geometry + symbols), not a pure geometric engine. This limits the theoretical elegance but delivers practical results.

---

## 5. Why Multi-Space? (vs. single merged space)

**Decision**: Five independent geometric spaces (CONTENT, MATH, GRAMMAR, TASK, SELF), each with its own dictionary and equilibrium. Connected only at query time.

**Alternatives considered**:
- **Single merged space**: One dictionary with all words, one equilibrium.
- **Shared space with subspaces**: One space with designated dimension ranges per domain.
- **Hierarchical spaces**: Meta-space containing sub-spaces.

**Why this choice**:
- **Domain interference**: In a merged space, "number" (math concept) and "noun" (grammar concept) compete for position. Math forces pull "number" toward "add" and "count"; grammar forces pull it toward "word" and "sentence". The equilibrium averages them, satisfying neither.
- **Independent equilibria**: Each space finds its own optimal geometry. "number" in MATH is near "add"; "number" in GRAMMAR is near "word". Both are correct in their domains.
- **Bridge terms**: Words appearing in multiple spaces serve as handoff points. "number" bridges MATH and GRAMMAR naturally.
- **Composability**: New spaces can be added without retraining existing ones. SELF (Phase 18) was added without changing CONTENT, MATH, GRAMMAR, or TASK.

**Trade-off**: Cross-space queries require routing logic. The TASK space handles routing geometrically, but fallback indicator lists (hardcoded) are needed when TASK geometry is uncertain. See audit finding A14.

---

## 6. Why ELI5? (dumbing down = smarter)

**Decision**: All definitions are written at a 5-year-old level. Simple words, short sentences, direct relationships.

**Alternatives considered**:
- **Standard dictionary definitions**: Academic, precise, verbose.
- **Wikipedia-style**: Encyclopedic, detailed, context-rich.
- **Technical definitions**: Domain-specific precision.

**Why this choice**:
- **Connector density**: A 200-word definition vocabulary means every word appears many times across definitions. High frequency = strong force signal = better geometry.
- **Closure tractability**: ELI5 definitions use ~200 unique words. Standard definitions might use 5000+. Closure at scale requires ELI5.
- **Taxonomic anchoring**: ELI5 definitions start with "a [category]." This guarantees the first content word is the parent category — perfect for `definition_category()` extraction.
- **Capability encoding**: ELI5 definitions say "can move", "can eat" directly. Technical definitions assume capabilities implicitly.

**Evidence**: Phase 10b granularity probe — Level 4 (Properties/Capabilities) scored **100%**, far above the predicted 40-60%. The ELI5 definitions carry full capability signal.

**Trade-off**: Loses nuance. "Democracy" defined at ELI5 level becomes "a way for people to choose leaders by voting" — correct but shallow. This is acceptable because DAPHNE tests comprehension (does it understand?), not depth of knowledge.

---

## 7. Why SELF as Peer Space? (vs. meta-space)

**Decision**: The SELF space is a regular geometric space with its own dictionary, equilibrium, and connectors — not a privileged meta-space that monitors or controls other spaces.

**Alternatives considered**:
- **Meta-space**: A space that embeds the OTHER spaces as points. "MATH" would be a point near "numbers" and "counting".
- **Reflection layer**: A separate module (not a geometric space) that answers self-referential queries.
- **No self-knowledge**: Skip self-awareness entirely.

**Why this choice**:
- **Architectural consistency**: Every domain is a geometric space. Self-knowledge is just another domain.
- **Composability**: SELF participates in cross-space queries the same way MATH or GRAMMAR does. "Can DAPHNE do math?" routes to both SELF and MATH spaces.
- **Identity as geometry**: "DAPHNE" is a point in SELF space, near "system", "geometric", "comprehension". Its capabilities are connectors: "can answer", "can describe". Its limitations are distances: far from "image", "translate". The geometry IS the self-model.

**Trade-off**: SELF cannot introspect on its own accuracy or monitor its own failures. A meta-space could track which question types succeed/fail. A peer space only knows what its dictionary tells it about itself.

---

## 8. Why Sequential Equilibrium? (vs. batch force field)

**Decision**: Phase 08 introduced incremental word placement (place one word at a time at the centroid of its placed neighbors) replacing the batch force field (place all words, then iterate forces).

**Alternatives considered**:
- **Batch force field**: All words random → iterate forces → converge. (Original Phase 02 approach.)
- **Spectral methods**: Eigendecomposition of the relation graph.

**Why this choice**:
- **Determinism**: Sequential placement with a fixed seed produces the same space every time. Batch force fields are sensitive to random initialization.
- **Plasticity claim**: The same parameters work across dict5 (51 words), dict12 (1005 words), and dict18 (2008 words) without retuning. The equilibrium process adapts to dictionary scale. Batch force fields required per-scale tuning.
- **Speed**: Sequential placement is O(n log n). Batch force field is O(n² × passes).
- **Comprehension demonstration**: Sequential equilibrium proves that text PRODUCES comprehension, not just that a configuration EXISTS. Each word is placed based on what the system has learned so far — like a student learning one word at a time.

**Trade-off**: Sequential placement is order-dependent (mitigated by multiple passes with shuffling). Earlier-placed words may have suboptimal final positions because later words weren't yet placed when their forces were computed.

---

## 9. Why Grammar as Regularizer? (not teacher)

**Decision**: Grammar files (grammar5.md, grammar18.md) provide additional sentences in the dictionary's vocabulary that describe what connectors mean. They regularize the force field but don't teach comprehension directly.

**Evidence**: Same seed, same parameters: with grammar → 0.7063 fitness. Without grammar → 0.4875 (collapse).

**What grammar does**: Constrains the space to be consistent across two text types (definitions and prose). Forces more robust geometry by preventing the space from overfitting to definition structure alone.

**What grammar doesn't do**: It doesn't add new words, doesn't teach question syntax, doesn't define grammar rules. It's text about connectors ("'is a' shows what group a word belongs to") written in the dictionary's own vocabulary.
