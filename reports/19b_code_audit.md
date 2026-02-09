# YALM Code Audit Report (Phase 19b)

> **Date**: 2026-02-09
> **Scope**: All 8 engine source files across yalm-core and yalm-engine
> **Methodology**: Line-by-line review against the 7 founding principles

---

## The 7 Founding Principles (from Prompt 02)

1. **Everything emerges from text** — no hardcoded linguistic knowledge
2. **Connectors discovered, not defined** — statistical patterns only
3. **Geometry IS the knowledge** — positions in space encode meaning
4. **No NLP libraries, no neural networks** — pure Rust, pure physics
5. **Transitive reasoning is free** — geometric proximity is inherently transitive
6. **Honesty is free** — no proximity = "I don't know"
7. **Parameters evolved, not hand-tuned** — genetic algorithm explores the space

---

## Findings

---

### A01. Hardcoded Question Verbs
**Severity**: 🔴 VIOLATION
**File**: `crates/yalm-engine/src/resolver.rs`, `detect_question_type()`
**Finding**: Question type detection relies on a hardcoded set: `["is", "can", "does", "do", "has"]`. The resolver pattern-matches the first token of a question against these English words to determine whether it's a Yes/No question.
**Spirit Violation**: Principle 1 — "Everything emerges from text." Question verbs are hardcoded English knowledge, not discovered from the dictionary.
**Justification**: The dictionary defines "is", "can", "does" etc., but there is no mechanism to discover that these words introduce questions. Question structure is meta-linguistic — it describes how humans use language, not what words mean.
**Recommendation**: A future phase could introduce a `question_grammar.md` file (written in the dictionary's own vocabulary) that teaches YALM what question patterns look like. The GRAMMAR space could learn to detect question structure geometrically.

---

### A02. Hardcoded Articles
**Severity**: 🟡 PRAGMATIC
**File**: `crates/yalm-engine/src/resolver.rs`, multiple functions; `crates/yalm-engine/src/multispace.rs`, `is_structural()`
**Finding**: Articles `["a", "an", "the"]` are hardcoded in at least 6 locations: `definition_category()`, `resolve_what_is()`, `describe()`, `detect_what_question()`, `detect_why_question()`, and `is_structural()`.
**Spirit Violation**: Principle 1 — these are English-specific function words.
**Justification**: Articles carry no semantic content and must be stripped to reach content words. The connector discovery pipeline independently identifies these as structural words (>20% document frequency), so the information IS available from text. The hardcoded lists are shortcuts to the same result.
**Recommendation**: Replace hardcoded article lists with a lookup against `classify_word_roles()` output. Structural words identified by frequency should serve as the article/function-word filter everywhere.

---

### A03. Hardcoded Structural Word List in MultiSpace
**Severity**: 🔴 VIOLATION
**File**: `crates/yalm-engine/src/multispace.rs`, `is_structural()` function
**Finding**: A hardcoded list of ~30 English words: `["is", "a", "an", "the", "of", "in", "on", "at", "to", "for", "and", "or", "not", "it", "its", "this", "that", "with", "from", "by", "as", "be", "are", "was", "were", "has", "have", "had", "do", "does", "did", "can", "will", "would", "should", "could", "may", "might"]`. Used to filter tokens during query routing.
**Spirit Violation**: Principle 1 — This is a comprehensive English function-word list baked into the code.
**Justification**: This function is only used in multi-space query routing (Phase 16+), where speed matters and the structural/content distinction must be available before any space is consulted.
**Recommendation**: At MultiSpace construction time, compute the union of structural words from all constituent spaces' connector discovery results. Cache as a `HashSet<String>` on the MultiSpace struct. Remove the hardcoded list entirely.

---

### A04. Hardcoded Number-to-Word Mapping
**Severity**: 🟡 PRAGMATIC
**File**: `crates/yalm-engine/src/multispace.rs`, `count_to_word()` function
**Finding**: Hardcoded mapping `{0: "zero", 1: "one", 2: "two", ..., 10: "ten"}` used for arithmetic answer formatting.
**Spirit Violation**: Principle 1 — English number words are hardcoded.
**Justification**: The MATH space contains definitions for these number words, but the mapping from integer result to word is needed at the presentation layer, not the comprehension layer. The system computes `2 + 3 = 5` and needs to say "five". This is output formatting, not understanding.
**Recommendation**: Could be extracted from the MATH dictionary at load time by parsing definitions like `"five — the number after four"`, but the complexity/benefit ratio is poor. Mark as accepted pragmatic debt.

---

### A05. Hardcoded SELF-Space Triggers
**Severity**: 🟡 PRAGMATIC
**File**: `crates/yalm-engine/src/multispace.rs`, SELF routing logic
**Finding**: `self_triggers = ["yalm"]` and `self_patterns = [("are", "you"), ("can", "you"), ("do", "you")]` are hardcoded for routing queries to the SELF space.
**Spirit Violation**: Principle 1 — The system's own name and second-person pronoun patterns are hardcoded.
**Justification**: The SELF space dictionary (`dict_self5.md`) defines "yalm" and "you" as entries. The triggers could be extracted from the SELF dictionary's vocabulary intersection with second-person patterns. However, the SELF space needs to be identified *before* query routing can consult any space's geometry, creating a bootstrapping problem.
**Recommendation**: Extract triggers from the SELF dictionary at MultiSpace construction: any word defined in SELF but not in any other space is a self-trigger. For patterns, the SELF dictionary's example sentences could be parsed for recurring pronoun patterns.

---

### A06. Definition Chain Check — Symbolic Gate on Geometry
**Severity**: 🟡 PRAGMATIC
**File**: `crates/yalm-engine/src/resolver.rs`, `definition_chain_check()`
**Finding**: The core Yes/No resolver uses geometric distance as primary signal but gates the result through a symbolic definition-chain traversal: follow the definition of word X up to `max_hops` steps, checking if Y appears. This is a string-matching BFS through definition text — not a geometric operation.
**Spirit Violation**: Principle 3 — "Geometry IS the knowledge." Here, geometry proposes but definitions dispose.
**Justification**: This is the most important pragmatic compromise in YALM. Without the chain gate, dict5 scored 13/20. With it: 20/20. The problem: geometric proximity cannot distinguish "same category" (dog ≈ cat, both near animal) from "is-a" (dog IS animal). The chain provides identity evidence that proximity lacks. As RECAP.md states: "Geometry encodes similarity. Definitions encode identity. You need both."
**Recommendation**: This is the deepest architectural tension. Two paths forward: (a) Accept the hybrid — geometry for association, symbols for discrimination. (b) Research whether a second geometric operation (e.g., a definition-graph embedding or a connector-direction query) could replace the chain check. Phase 20+ research.

---

### A07. Definition Category Extraction — First-Content-Word Heuristic
**Severity**: 🟡 PRAGMATIC
**File**: `crates/yalm-engine/src/resolver.rs`, `definition_category()`
**Finding**: "What is X?" answers are produced by extracting the first content word from X's definition, after skipping articles and the subject itself. This is string processing, not geometric lookup.
**Spirit Violation**: Principle 3 — The answer comes from parsing text, not measuring distance.
**Justification**: Geometric nearest-neighbor could answer "What is a dog?" (nearest content word in space), but it would return whichever word happens to be closest — potentially "cat" instead of "animal". The definition's first content word is more reliable because definitions are structured as "X — a [category]." The ELI5 format guarantees this structure.
**Recommendation**: Consider a hybrid: use definition-category as primary, fall back to geometric nearest-neighbor when definition extraction fails. The geometry would serve as a backup, not the primary source.

---

### A08. Why/When Resolution — Pure Symbolic Operations
**Severity**: 🟡 PRAGMATIC
**File**: `crates/yalm-engine/src/resolver.rs`, `resolve_why()`, `trace_chain_path()`, `resolve_when()`, `extract_condition_clause()`
**Finding**: "Why" answers trace definition chains symbolically and format them as "because X is Y, and Y is Z." "When" answers scan definition text for "to"/"when"/"if" clauses. Neither operation uses geometric distance.
**Spirit Violation**: Principle 3 — These are purely string-based operations.
**Justification**: "Why" is inherently an explanation question — the answer IS the definition chain. Geometry tells you THAT dog is related to thing, but not WHY. The chain path is the explanation. "When" requires extracting temporal/conditional phrases from definitions — a parsing operation that geometry cannot perform (geometry has no notion of clause structure).
**Recommendation**: These operations are irreducibly symbolic. Geometry cannot explain WHY (it has no causal model) or WHEN (it has no temporal model). Accept as architectural features, not debt.

---

### A09. `is_property_word()` and `is_connector_word()` — Definition-Shape Heuristics
**Severity**: ⚪ TECHNICAL DEBT
**File**: `crates/yalm-engine/src/resolver.rs`
**Finding**: `is_property_word()` checks whether a word's definition starts with patterns like "a way to" or "having". `is_connector_word()` checks whether a word appears in any connector pattern. Both are used in `definition_category()` to filter false positives.
**Spirit Violation**: Mild violation of Principle 1 — "a way to" and "having" are English patterns.
**Justification**: These heuristics prevent `definition_category()` from returning connector words or property words as categories. They improve answer quality at the cost of hardcoded English assumptions.
**Recommendation**: Replace `is_property_word()` with a geometric test: property words should cluster in specific regions of the GRAMMAR space. Replace `is_connector_word()` with a lookup against the discovered connector set (which it already partially does).

---

### A10. MAX_FOLLOW_PER_HOP and max_hops — Fixed Constants
**Severity**: ⚪ TECHNICAL DEBT
**File**: `crates/yalm-engine/src/resolver.rs`
**Finding**: `MAX_FOLLOW_PER_HOP = 3` (max content words to follow per chain step) and `max_hops = 3` (max chain depth) are hardcoded constants, not in `EngineParams`.
**Spirit Violation**: Principle 7 — "Parameters evolved, not hand-tuned." These directly affect answer quality and should be evolvable.
**Justification**: These were tuned by hand during Phase 06 and Phase 11. They work for dict5-dict18, but different dictionary structures might need different values.
**Recommendation**: Move to `EngineParams`. The genetic algorithm should explore chain depth and breadth along with other parameters.

---

### A11. alpha = 0.2 in Connector Axis Emphasis
**Severity**: ⚪ TECHNICAL DEBT
**File**: `crates/yalm-engine/src/resolver.rs`, `resolve_what_is()`
**Finding**: The `alpha` parameter controlling connector-axis emphasis in nearest-neighbor search is hardcoded at 0.2.
**Spirit Violation**: Principle 7 — should be evolvable.
**Justification**: Evolution in v7 rejected axis-specific projection at 96%, so this parameter has limited impact. The value 0.2 was hand-selected as a compromise.
**Recommendation**: Add to `EngineParams` if per-space parameter evolution (Phase 20) is implemented. Low priority since evolution consistently ignores it.

---

### A12. Uniformity Filter Constants
**Severity**: ⚪ TECHNICAL DEBT
**File**: `crates/yalm-engine/src/connector_discovery.rs`, `connector_pipeline()`
**Finding**: `num_buckets = 10` and `uniformity_threshold = 0.75` are hardcoded in the connector pipeline. The topic frequency threshold formula `n * 0.25 / ln(n / 50)` also has hardcoded magic numbers (0.25 and 50).
**Spirit Violation**: Principle 7 — these directly control which connectors are discovered.
**Justification**: The uniformity filter was designed in Phase 09c with principled reasoning: 10 buckets for alphabetical distribution, 0.75 as a coefficient-of-variation threshold. The topic formula scales logarithmically with dictionary size, anchored to dict5's 50 words. These are not arbitrary — they're engineering constants derived from the data.
**Recommendation**: The uniformity parameters could be added to `EngineParams` for evolution, but the current values were derived from analysis, not guessing. Consider making them configurable but documenting the derivation.

---

### A13. Question-Type Detection is English-Specific
**Severity**: 🔴 VIOLATION
**File**: `crates/yalm-engine/src/resolver.rs`, `detect_question_type()`
**Finding**: Question type detection matches against English question words: "what", "who", "where", "when", "why" as first token, plus verb patterns "is", "can", "does" etc. This entire classification system assumes English question syntax.
**Spirit Violation**: Principle 1 — The question parser hardcodes English syntax.
**Justification**: YALM's dictionaries are in English. The question format is part of the evaluation protocol, not the comprehension engine. A French YALM would need different question words but the same geometric engine.
**Recommendation**: Move question-type detection out of the engine and into the evaluation/interface layer. The engine should expose primitives: `nearest(word)`, `distance(a, b)`, `chain(a, b)`, `describe(word)`. The question parser is language-specific glue.

---

### A14. Task Classification Indicators in MultiSpace
**Severity**: 🔴 VIOLATION
**File**: `crates/yalm-engine/src/multispace.rs`, `resolve_task_classification()`
**Finding**: The TASK space routing uses hardcoded indicator word lists: `math_indicators = ["add", "subtract", "plus", "minus", "sum", "count", "number", "how many", ...]`, `grammar_indicators = ["noun", "verb", "sentence", "grammar", "word", "language", ...]`, `content_indicators = ["animal", "dog", "cat", "person", "thing", ...]`.
**Spirit Violation**: Principle 1 — These are handcrafted routing rules, not geometric routing.
**Justification**: The TASK space is designed to route queries geometrically — it has a dictionary where "math" is defined in terms of number-related concepts. The indicator lists are a fallback when geometric routing is uncertain. In practice, they dominate routing because the TASK dictionary is small.
**Recommendation**: Remove the indicator fallbacks. The TASK space should route purely by geometric proximity: compute distance from the query's content words to "math", "grammar", "content", "self" in the TASK space. If TASK geometry can't decide, admit uncertainty rather than falling back to word lists.

---

### A15. Negation Models — All Four Failed
**Severity**: ⚪ TECHNICAL DEBT
**File**: `crates/yalm-engine/src/force_field.rs`, `apply_force()` with NegationModel variants
**Finding**: Four NegationModel variants are implemented (Inversion, Repulsion, AxisShift, SeparateDimension). Evolution explored all four across hundreds of generations. None successfully handles negation through geometry alone. The winning strategy is always the definition-chain check (A06).
**Spirit Violation**: None (the code correctly implements the strategies). But the presence of four unused-in-practice strategies is dead code.
**Justification**: These represent genuine research attempts to solve negation geometrically. They document the negative result: geometric negation doesn't work in this architecture. The code serves as research documentation.
**Recommendation**: Keep the code but add comments documenting that evolution consistently prefers definition-chain negation. Consider marking specific variants as `#[deprecated]` if they're provably never selected.

---

### A16. Connector Discovery Correctly Follows Principle 2
**Severity**: 🟢 ALIGNED
**File**: `crates/yalm-engine/src/connector_discovery.rs`
**Finding**: The full connector pipeline — `classify_word_roles()`, `extract_all_sentences()`, `extract_relations()`, `connector_pipeline()` — discovers connectors purely from text statistics. No linguistic knowledge is used to identify "is a" or "can" — they emerge from frequency analysis and the uniformity filter.
**Spirit Violation**: None — this is the system working as designed.
**Recommendation**: None needed. This is YALM's strongest alignment with its principles.

---

### A17. Equilibrium Space Construction Follows Principle 3
**Severity**: 🟢 ALIGNED
**File**: `crates/yalm-engine/src/equilibrium.rs`, `build_space_equilibrium()`
**Finding**: Sequential equilibrium places words by computing centroid of already-placed neighbors, then applies local relaxation with force-based perturbation and damping. The resulting positions encode meaning — related words cluster together. The geometry IS the knowledge for positive relationships.
**Spirit Violation**: None.
**Recommendation**: None needed. The equilibrium process is the heart of YALM's geometric principle.

---

### A18. Bootstrap Loop Respects Immutable Dictionaries
**Severity**: 🟢 ALIGNED
**File**: `crates/yalm-engine/src/bootstrap.rs`
**Finding**: The bootstrap loop generates describe() text, feeds it through connector discovery, and rebuilds the geometric space — but never modifies the dictionary. Only connectors evolve. This follows the Phase 19 design: "grammar evolves without changing any dictionary."
**Spirit Violation**: None.
**Recommendation**: None needed. This is the bootstrap loop working as designed.

---

### A19. Force Field Implements All Strategy Variants
**Severity**: 🟢 ALIGNED
**File**: `crates/yalm-engine/src/force_field.rs`
**Finding**: The force field correctly implements all 4 ForceFunction variants (Linear, InverseDistance, Gravitational, Spring), all 3 SpaceInitialization variants, and all 4 MultiConnectorHandling strategies. Each is selectable by the `StrategyConfig` and evolvable by the genetic algorithm.
**Spirit Violation**: None — Principle 7 is satisfied. Strategies are evolved, not hand-tuned.
**Recommendation**: None needed.

---

### A20. `preceded_by_not()` — Literal String Matching for Negation
**Severity**: 🟡 PRAGMATIC
**File**: `crates/yalm-engine/src/resolver.rs`
**Finding**: The function checks if "not" appears before a word in the question text, using literal string matching. Used to detect negated queries like "Is a dog NOT an animal?"
**Spirit Violation**: Principle 1 — "not" is hardcoded English.
**Justification**: The connector discovery pipeline independently discovers "not" as a high-frequency structural pattern. The literal check is a shortcut to the same information.
**Recommendation**: Replace with a check against discovered connector patterns. If "not" is in the connector set, use it; if not, the language might express negation differently.

---

### A21. Phase 15 Gap — describe() Produces Thin Output
**Severity**: ⚪ TECHNICAL DEBT
**File**: `crates/yalm-engine/src/resolver.rs`, `describe()` function
**Finding**: Phase 15 (Rich Description: Property Extraction) was stubbed but never implemented. The current `describe()` produces: (1) category sentence ("X is a Y"), (2) definition sentence rewriting, (3) sibling negation. It does NOT extract embedded properties from definitions like "a big hot thing that is up" → separate sentences for "big", "hot", "up". The bootstrap loop (Phase 19) depends on describe() output for connector re-discovery, but receives thin signal without property extraction.
**Spirit Violation**: Not a principle violation — this is missing functionality.
**Justification**: Phases 16-19 proceeded without Phase 15 because the bootstrap loop found 4 new connectors even with thin describe output. The system works, but sub-optimally.
**Recommendation**: Implement Phase 15 property extraction before Phase 20 (per-space evolution). Richer describe output → more connector discovery signal → better bootstrap convergence.

---

### A22. Entity Fast Path Bypasses All Heuristics
**Severity**: 🟡 PRAGMATIC
**File**: `crates/yalm-engine/src/resolver.rs`, `definition_category()`
**Finding**: When `entry.is_entity == true`, the function skips all heuristic filters (structural word check, connector word check, property word check, noun check) and returns the first non-article word from the definition. This was added in Phase 11b to fix entity category extraction.
**Spirit Violation**: Mild — entities get special treatment outside the normal pipeline.
**Justification**: Entity definitions are hand-crafted (e.g., "harris — a person") and follow a strict "X — a [category]" format. The heuristic filters were designed for LLM-generated definitions which are noisier. Applying the same filters to clean entity definitions caused false rejections (e.g., "person" blocked by `is_connector_word()`). The fast path is correct for its input class.
**Recommendation**: Long-term, the heuristic filters should be robust enough to handle both entity and LLM definitions without a separate code path. Short-term, the fast path is a clean solution.

---

### A23. `find_siblings()` — Category Comparison, Not Geometry
**Severity**: ⚪ TECHNICAL DEBT
**File**: `crates/yalm-engine/src/resolver.rs`
**Finding**: `find_siblings()` finds words sharing the same definition category (e.g., all words whose first content word is "animal"). Used for negation inference in describe(). This is a string operation — comparing extracted categories, not geometric neighborhoods.
**Spirit Violation**: Mild — siblings could be discovered geometrically as the nearest words in the same connector direction.
**Justification**: Category comparison is precise: "dog" and "cat" are siblings because their definitions both start with "animal". Geometric nearest-neighbors might include "food" or "ball" if they're close in space. The definition-based approach is more accurate for this specific task.
**Recommendation**: Consider a geometric alternative for describe() Phase 2: find k-nearest neighbors, then filter by shared connector direction. This would be more principled and would discover "siblings" that definitions don't explicitly mark.

---

### A24. skip_words List in Question Parsing
**Severity**: 🟡 PRAGMATIC
**File**: `crates/yalm-engine/src/resolver.rs`, question detection functions
**Finding**: Multiple functions use `skip_words` sets like `["is", "a", "the", "it", "not"]` and `question_syntax` lists like `["is", "a", "an", "the", "of", "do", "does", "can", "has"]` to strip function words from questions before extracting content.
**Spirit Violation**: Principle 1 — hardcoded English function words.
**Justification**: These overlap almost entirely with the structural words discovered by `classify_word_roles()`. The hardcoded lists exist because the resolver needs them during question parsing, before it has access to the space's structural word classification.
**Recommendation**: Pass the structural word set (from connector discovery) into the resolver at construction time. Use it in place of hardcoded skip lists.

---

---

## Summary

### Severity Counts

| Severity | Count | Description |
|----------|-------|-------------|
| 🔴 VIOLATION | 4 | A01, A03, A13, A14 |
| 🟡 PRAGMATIC | 8 | A02, A04, A05, A06, A07, A08, A20, A22, A24 |
| 🟢 ALIGNED | 4 | A16, A17, A18, A19 |
| ⚪ TECHNICAL DEBT | 6 | A09, A10, A11, A12, A15, A21, A23 |

**Total findings: 24**

*Note: A02 and A24 overlap (both concern hardcoded function word lists). Counted separately because they appear in different contexts.*

---

### The Honest Assessment

#### How much of YALM is truly geometric vs. symbolic?

The answer depends on which question type:

| Component | Nature | Contribution |
|-----------|--------|-------------|
| Connector discovery | Text-statistical | Discovers structure from text (Principle 2 ✅) |
| Equilibrium / force field | Geometric | Positions words, encodes relationships (Principle 3 ✅) |
| Yes/No (positive) | Geometric + symbolic gate | Distance proposes, chain gate confirms |
| Yes/No (negative) | Symbolic | Chain traversal is the only working negation |
| What/Who/Where | Symbolic | First-content-word extraction from definitions |
| Why | Symbolic | Chain path traced and formatted |
| When | Symbolic | Clause extraction from definition text |
| Describe | Symbolic | Definition rewriting + sibling comparison |
| Boolean (AND/OR) | Meta | Decomposes into sub-queries, combines |
| Task routing | Partially hardcoded | Indicator word fallbacks override geometry |
| Bootstrap loop | Geometric + text | describe() → connector discovery → re-equilibrium |

#### What percentage of correct answers come from geometry vs. chain traversal?

On dict5 (20/20):
- **10 positive Yes/No questions**: Geometry (distance < threshold) provides the signal, chain gate confirms. ~50% geometric.
- **4 negative Yes/No questions**: Chain traversal does all the work. 0% geometric.
- **4 unknown questions**: Geometry (distance > threshold → IDK). 100% geometric.
- **2 What-Is questions**: Definition extraction. 0% geometric.

**Rough split: ~35% of dict5 answers rely primarily on geometry, ~40% on symbolic chain operations, ~25% on geometric absence (honesty).**

On the full multi-space unified test (45/50):
- Geometry is more important for cross-space routing (bridge terms)
- Symbolic operations dominate within-space answer extraction
- Estimated: ~30% geometric, ~50% symbolic, ~20% honest-absence

#### Is the hybrid a pragmatic success or a philosophical failure?

**It's a pragmatic success that reveals a genuine finding.**

The original vision ("geometry IS the knowledge") was partially right: geometry correctly encodes similarity, proximity, and association. Dog IS near animal IS near thing. Transitive reasoning IS free. Honesty IS free.

But geometry cannot distinguish "similar" from "identical" — dog ≈ cat (both animals) is indistinguishable from dog → animal (is-a relationship) in pure distance. This is not a YALM-specific failure. It's a fundamental property of metric spaces: distance is symmetric and doesn't encode direction.

The definition-chain check adds asymmetric, directional evidence: "dog's definition contains 'animal'" is a directed relationship that distance cannot express. This is the same distinction between:
- **Embedding spaces** (Word2Vec, GloVe): symmetric similarity
- **Knowledge graphs** (TransE): directed relations

YALM discovered empirically what the field knew theoretically: you need both.

**The philosophical answer**: YALM is a hybrid system that uses geometry for WHAT (similarity, association, proximity) and symbolic chain traversal for WHY and WHETHER (identity, causation, negation). The geometry is not the whole knowledge — but it IS half the knowledge, and the half that scales.

---

### The 4 Hardcoded Violations: Are They Fixable?

| ID | Violation | Fixable? | How |
|----|-----------|----------|-----|
| A01 | Question verbs | Yes | Question grammar dictionary |
| A03 | Structural word list | Yes | Union of spaces' structural words |
| A13 | English question syntax | Yes | Move parser to interface layer |
| A14 | Task routing indicators | Yes | Remove fallbacks, trust TASK geometry |

All 4 are fixable without changing the architecture. They are engineering shortcuts, not fundamental limits. A "pure" YALM that discovers its question syntax from a grammar dictionary is achievable — it would just need richer dictionaries and a boot-strapping step for the question parser itself.

---

### The Founding Principles: Scorecard

| Principle | Status | Evidence |
|-----------|--------|----------|
| 1. Everything emerges from text | **Partial** | Core comprehension: yes. Question parsing: no (English-specific). |
| 2. Connectors discovered, not defined | **Full** | connector_discovery.rs is purely statistical. |
| 3. Geometry IS the knowledge | **Partial** | For similarity and association: yes. For identity and negation: no. |
| 4. No NLP/neural networks | **Full** | Pure Rust, zero ML libraries. |
| 5. Transitive reasoning is free | **Full** | Geometric proximity is inherently transitive. Works on 3+ hops. |
| 6. Honesty is free | **Full** | No proximity = "I don't know". Emerges naturally. |
| 7. Parameters evolved | **Partial** | Core params: yes. Resolver constants: hand-tuned. |

**Score: 4/7 fully satisfied, 3/7 partially satisfied, 0/7 violated outright.**

The partial satisfactions are honest: they reveal where geometry ends and symbols begin. This IS the finding.
