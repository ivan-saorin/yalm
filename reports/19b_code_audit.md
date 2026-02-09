# DAPHNE Code Audit Report (Phase 19b)

> **Date**: 2026-02-09
> **Scope**: All 8 engine source files across dafhne-core and dafhne-engine
> **Methodology**: Line-by-line review against the 7 founding principles
> **Updated**: Post-Phase 19c fixes — 16 of 24 findings addressed

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
**Severity**: 🔴 VIOLATION → 🟡 DOCUMENTED (Phase 19c)
**File**: `crates/dafhne-engine/src/resolver.rs`, `detect_question_type()`
**Finding**: Question type detection relies on a hardcoded set: `["is", "can", "does", "do", "has"]`. The resolver pattern-matches the first token of a question against these English words to determine whether it's a Yes/No question.
**Spirit Violation**: Principle 1 — "Everything emerges from text." Question verbs are hardcoded English knowledge, not discovered from the dictionary.
**Justification**: The dictionary defines "is", "can", "does" etc., but there is no mechanism to discover that these words introduce questions. Question structure is meta-linguistic — it describes how humans use language, not what words mean.
**19c Fix**: Yes/No question verb detection now uses `structural.contains(&tokens[0])` — these words are discovered as structural by the 20% doc-frequency threshold. The 5W question words (what, who, where, when, why) remain hardcoded English, documented with a `LANGUAGE-SPECIFIC LAYER` comment block.
**Recommendation**: A future phase could introduce a `question_grammar.md` file (written in the dictionary's own vocabulary) that teaches DAPHNE what question patterns look like. The GRAMMAR space could learn to detect question structure geometrically.

---

### A02. Hardcoded Articles
**Severity**: 🟡 PRAGMATIC → 🟢 FIXED (Phase 19c)
**File**: `crates/dafhne-engine/src/resolver.rs`, multiple functions; `crates/dafhne-engine/src/multispace.rs`, `is_structural()`
**Finding**: Articles `["a", "an", "the"]` are hardcoded in at least 6 locations: `definition_category()`, `resolve_what_is()`, `describe()`, `detect_what_question()`, `detect_why_question()`, and `is_structural()`.
**Spirit Violation**: Principle 1 — these are English-specific function words.
**19c Fix**: Entity fast path and subject extraction replaced with `structural.contains()`. The `is_structural()` hardcoded function replaced with `structural_words_cache` (union of per-space `classify_word_roles()` output). Skip-word fallbacks in question detectors retained as definition-shape patterns (see 19c report, Design Note).

---

### A03. Hardcoded Structural Word List in MultiSpace
**Severity**: 🔴 VIOLATION → 🟢 FIXED (Phase 19c)
**File**: `crates/dafhne-engine/src/multispace.rs`, `is_structural()` function
**Finding**: A hardcoded list of ~30 English words used to filter tokens during query routing.
**Spirit Violation**: Principle 1 — This is a comprehensive English function-word list baked into the code.
**19c Fix**: Replaced with `structural_words_cache` on the `MultiSpace` struct, computed at construction time as the union of all constituent spaces' `classify_word_roles()` output, plus a small set of question-syntax meta-words (what, who, where, when, why, how, which, yes, no, you, are, be, do, does) needed for cross-space routing. All 8+ call sites updated to `self.is_structural_cached()`. Old hardcoded function retained as `#[allow(dead_code)]` fallback.

---

### A04. Hardcoded Number-to-Word Mapping
**Severity**: 🟡 PRAGMATIC
**File**: `crates/dafhne-engine/src/multispace.rs`, `count_to_word()` function
**Finding**: Hardcoded mapping `{0: "zero", 1: "one", 2: "two", ..., 10: "ten"}` used for arithmetic answer formatting.
**Spirit Violation**: Principle 1 — English number words are hardcoded.
**Justification**: The MATH space contains definitions for these number words, but the mapping from integer result to word is needed at the presentation layer, not the comprehension layer. The system computes `2 + 3 = 5` and needs to say "five". This is output formatting, not understanding.
**Recommendation**: Could be extracted from the MATH dictionary at load time by parsing definitions like `"five — the number after four"`, but the complexity/benefit ratio is poor. Mark as accepted pragmatic debt.

---

### A05. Hardcoded SELF-Space Triggers
**Severity**: 🟡 PRAGMATIC → 🟢 FIXED (Phase 19c)
**File**: `crates/dafhne-engine/src/multispace.rs`, SELF routing logic
**Finding**: `self_triggers = ["dafhne"]` and `self_patterns = [("are", "you"), ("can", "you"), ("do", "you")]` are hardcoded for routing queries to the SELF space.
**Spirit Violation**: Principle 1 — The system's own name and second-person pronoun patterns are hardcoded.
**19c Fix**: Trigger words now derived from vocabulary at MultiSpace construction: words unique to SELF space (not in any other non-task space, not structural) become `self_trigger_words`. Pronoun patterns `("are","you")` etc. retained with comment explaining they're structural-word patterns not derivable from vocabulary alone.

---

### A06. Definition Chain Check — Symbolic Gate on Geometry
**Severity**: 🟡 PRAGMATIC
**File**: `crates/dafhne-engine/src/resolver.rs`, `definition_chain_check()`
**Finding**: The core Yes/No resolver uses geometric distance as primary signal but gates the result through a symbolic definition-chain traversal: follow the definition of word X up to `max_hops` steps, checking if Y appears. This is a string-matching BFS through definition text — not a geometric operation.
**Spirit Violation**: Principle 3 — "Geometry IS the knowledge." Here, geometry proposes but definitions dispose.
**Justification**: This is the most important pragmatic compromise in DAPHNE. Without the chain gate, dict5 scored 13/20. With it: 20/20. The problem: geometric proximity cannot distinguish "same category" (dog ≈ cat, both near animal) from "is-a" (dog IS animal). The chain provides identity evidence that proximity lacks. As RECAP.md states: "Geometry encodes similarity. Definitions encode identity. You need both."
**Recommendation**: This is the deepest architectural tension. Two paths forward: (a) Accept the hybrid — geometry for association, symbols for discrimination. (b) Research whether a second geometric operation (e.g., a definition-graph embedding or a connector-direction query) could replace the chain check. Phase 20+ research.

---

### A07. Definition Category Extraction — First-Content-Word Heuristic
**Severity**: 🟡 PRAGMATIC
**File**: `crates/dafhne-engine/src/resolver.rs`, `definition_category()`
**Finding**: "What is X?" answers are produced by extracting the first content word from X's definition, after skipping articles and the subject itself. This is string processing, not geometric lookup.
**Spirit Violation**: Principle 3 — The answer comes from parsing text, not measuring distance.
**Justification**: Geometric nearest-neighbor could answer "What is a dog?" (nearest content word in space), but it would return whichever word happens to be closest — potentially "cat" instead of "animal". The definition's first content word is more reliable because definitions are structured as "X — a [category]." The ELI5 format guarantees this structure.
**Recommendation**: Consider a hybrid: use definition-category as primary, fall back to geometric nearest-neighbor when definition extraction fails. The geometry would serve as a backup, not the primary source.

---

### A08. Why/When Resolution — Pure Symbolic Operations
**Severity**: 🟡 PRAGMATIC
**File**: `crates/dafhne-engine/src/resolver.rs`, `resolve_why()`, `trace_chain_path()`, `resolve_when()`, `extract_condition_clause()`
**Finding**: "Why" answers trace definition chains symbolically and format them as "because X is Y, and Y is Z." "When" answers scan definition text for "to"/"when"/"if" clauses. Neither operation uses geometric distance.
**Spirit Violation**: Principle 3 — These are purely string-based operations.
**Justification**: "Why" is inherently an explanation question — the answer IS the definition chain. Geometry tells you THAT dog is related to thing, but not WHY. The chain path is the explanation. "When" requires extracting temporal/conditional phrases from definitions — a parsing operation that geometry cannot perform (geometry has no notion of clause structure).
**Recommendation**: These operations are irreducibly symbolic. Geometry cannot explain WHY (it has no causal model) or WHEN (it has no temporal model). Accept as architectural features, not debt.

---

### A09. `is_property_word()` and `is_connector_word()` — Definition-Shape Heuristics
**Severity**: ⚪ TECHNICAL DEBT → 🟢 DOCUMENTED (Phase 19c)
**File**: `crates/dafhne-engine/src/resolver.rs`
**Finding**: `is_property_word()` checks whether a word's definition starts with patterns like "a way to" or "having". `is_connector_word()` checks whether a word appears in any connector pattern.
**19c Fix**: Added comprehensive doc comments documenting that `is_connector_word` is fully data-driven (scans discovered connectors) and `is_property_word` uses ELI5 definition-shape heuristics ("to" prefix, "-ing" suffix, "not X" pattern) that are format conventions, not English grammar rules.
**Recommendation**: Replace `is_property_word()` with a geometric test in a future phase.

---

### A10. MAX_FOLLOW_PER_HOP and max_hops — Fixed Constants
**Severity**: ⚪ TECHNICAL DEBT → 🟢 FIXED (Phase 19c)
**File**: `crates/dafhne-engine/src/resolver.rs`
**Finding**: `MAX_FOLLOW_PER_HOP = 3` and `max_hops = 3` were hardcoded constants.
**19c Fix**: Externalized to `EngineParams.max_follow_per_hop` and `EngineParams.max_chain_hops` with `#[serde(default)]` for backward compatibility. Added to `ParamRanges`, `random_genome()`, `mutate()`, and `crossover()` in dafhne-evolve. All 4+ call sites updated across resolver.rs and multispace.rs.

---

### A11. alpha = 0.2 in Connector Axis Emphasis
**Severity**: ⚪ TECHNICAL DEBT → 🟢 FIXED (Phase 19c)
**File**: `crates/dafhne-engine/src/resolver.rs`, `resolve_what_is()`
**Finding**: The `alpha` parameter was hardcoded at 0.2.
**19c Fix**: Externalized to `EngineParams.weighted_distance_alpha` with `#[serde(default)]`. Added to evolution infrastructure (ParamRanges, mutation, crossover). Range: (0.05, 0.5).

---

### A12. Uniformity Filter Constants
**Severity**: ⚪ TECHNICAL DEBT → 🟢 FIXED (Phase 19c)
**File**: `crates/dafhne-engine/src/connector_discovery.rs`, `connector_pipeline()`
**Finding**: `num_buckets = 10` and `uniformity_threshold = 0.75` were hardcoded.
**19c Fix**: Externalized to `EngineParams.uniformity_num_buckets` and `EngineParams.uniformity_threshold` with `#[serde(default)]`. Added to evolution infrastructure. Ranges: num_buckets (5, 20), threshold (0.5, 0.95). The topic frequency formula's constants (0.25 and 50) remain hardcoded — they're derived from analysis, not arbitrary.

---

### A13. Question-Type Detection is English-Specific
**Severity**: 🔴 VIOLATION → 🟡 DOCUMENTED (Phase 19c)
**File**: `crates/dafhne-engine/src/resolver.rs`, `detect_question_type()`
**Finding**: Question type detection matches against English question words: "what", "who", "where", "when", "why" as first token.
**Spirit Violation**: Principle 1 — The question parser hardcodes English syntax.
**19c Fix**: Yes/No detection now uses discovered structural words (`structural.contains(&tokens[0])`) instead of hardcoded verb lists. The 5W question words remain hardcoded English, documented with a `LANGUAGE-SPECIFIC LAYER` comment block describing the refactoring path (move to language-adapter config).
**Recommendation**: Move question-type detection out of the engine and into the evaluation/interface layer.

---

### A14. Task Classification Indicators in MultiSpace
**Severity**: 🔴 VIOLATION → 🟡 PARTIALLY FIXED (Phase 19c)
**File**: `crates/dafhne-engine/src/multispace.rs`, `resolve_task_classification()`
**Finding**: The TASK space routing used hardcoded indicator word lists for grammar, content, and math.
**Spirit Violation**: Principle 1 — These are handcrafted routing rules, not geometric routing.
**19c Fix**: Replaced `grammar_indicators` and `content_indicators` arrays with vocabulary membership checks against grammar and content space dictionaries (`grammar_vocab`, `content_vocab`). Math indicators (`["plus", "minus", "count", ...]`) retained — these are operator words consistent across dictionary sizes.
**Remaining**: Math indicators are still hardcoded. Full geometric routing (TASK space proximity) deferred to Phase 20.

---

### A15. Negation Models — All Four Failed
**Severity**: ⚪ TECHNICAL DEBT → 🟢 DOCUMENTED (Phase 19c)
**File**: `crates/dafhne-engine/src/strategy.rs`, `NegationModel` enum
**Finding**: Four NegationModel variants are implemented. Evolution explored all four across hundreds of generations.
**19c Fix**: Added comprehensive research-result documentation to the `NegationModel` enum and each variant. AxisShift documented as the consistent winner (96%+ convergence rate). Other variants retained for evolution diversity with explanatory comments.

---

### A16. Connector Discovery Correctly Follows Principle 2
**Severity**: 🟢 ALIGNED
**File**: `crates/dafhne-engine/src/connector_discovery.rs`
**Finding**: The full connector pipeline — `classify_word_roles()`, `extract_all_sentences()`, `extract_relations()`, `connector_pipeline()` — discovers connectors purely from text statistics. No linguistic knowledge is used to identify "is a" or "can" — they emerge from frequency analysis and the uniformity filter.
**Spirit Violation**: None — this is the system working as designed.
**Recommendation**: None needed. This is DAPHNE's strongest alignment with its principles.

---

### A17. Equilibrium Space Construction Follows Principle 3
**Severity**: 🟢 ALIGNED
**File**: `crates/dafhne-engine/src/equilibrium.rs`, `build_space_equilibrium()`
**Finding**: Sequential equilibrium places words by computing centroid of already-placed neighbors, then applies local relaxation with force-based perturbation and damping. The resulting positions encode meaning — related words cluster together. The geometry IS the knowledge for positive relationships.
**Spirit Violation**: None.
**Recommendation**: None needed. The equilibrium process is the heart of DAPHNE's geometric principle.

---

### A18. Bootstrap Loop Respects Immutable Dictionaries
**Severity**: 🟢 ALIGNED
**File**: `crates/dafhne-engine/src/bootstrap.rs`
**Finding**: The bootstrap loop generates describe() text, feeds it through connector discovery, and rebuilds the geometric space — but never modifies the dictionary. Only connectors evolve. This follows the Phase 19 design: "grammar evolves without changing any dictionary."
**Spirit Violation**: None.
**Recommendation**: None needed. This is the bootstrap loop working as designed.

---

### A19. Force Field Implements All Strategy Variants
**Severity**: 🟢 ALIGNED
**File**: `crates/dafhne-engine/src/force_field.rs`
**Finding**: The force field correctly implements all 4 ForceFunction variants (Linear, InverseDistance, Gravitational, Spring), all 3 SpaceInitialization variants, and all 4 MultiConnectorHandling strategies. Each is selectable by the `StrategyConfig` and evolvable by the genetic algorithm.
**Spirit Violation**: None — Principle 7 is satisfied. Strategies are evolved, not hand-tuned.
**Recommendation**: None needed.

---

### A20. `preceded_by_not()` — Literal String Matching for Negation
**Severity**: 🟡 PRAGMATIC → 🟢 FIXED (Phase 19c)
**File**: `crates/dafhne-engine/src/resolver.rs`
**Finding**: The function checks if "not" appears before a word, using literal string matching.
**19c Fix**: Added connector-existence guard at both call sites: negation check only fires when `space.connectors` contains a "not" pattern. The `preceded_by_not` function also now takes `structural` param (replaces hardcoded articles with structural set for skip-word detection). If the space hasn't discovered "not" as a connector, negation is not assumed.

---

### A21. Phase 15 Gap — describe() Produces Thin Output
**Severity**: ⚪ TECHNICAL DEBT
**File**: `crates/dafhne-engine/src/resolver.rs`, `describe()` function
**Finding**: Phase 15 (Rich Description: Property Extraction) was stubbed but never implemented. The current `describe()` produces: (1) category sentence ("X is a Y"), (2) definition sentence rewriting, (3) sibling negation. It does NOT extract embedded properties from definitions like "a big hot thing that is up" → separate sentences for "big", "hot", "up". The bootstrap loop (Phase 19) depends on describe() output for connector re-discovery, but receives thin signal without property extraction.
**Spirit Violation**: Not a principle violation — this is missing functionality.
**Justification**: Phases 16-19 proceeded without Phase 15 because the bootstrap loop found 4 new connectors even with thin describe output. The system works, but sub-optimally.
**Recommendation**: Implement Phase 15 property extraction before Phase 20 (per-space evolution). Richer describe output → more connector discovery signal → better bootstrap convergence.

---

### A22. Entity Fast Path Bypasses All Heuristics
**Severity**: 🟡 PRAGMATIC
**File**: `crates/dafhne-engine/src/resolver.rs`, `definition_category()`
**Finding**: When `entry.is_entity == true`, the function skips all heuristic filters (structural word check, connector word check, property word check, noun check) and returns the first non-article word from the definition. This was added in Phase 11b to fix entity category extraction.
**Spirit Violation**: Mild — entities get special treatment outside the normal pipeline.
**Justification**: Entity definitions are hand-crafted (e.g., "harris — a person") and follow a strict "X — a [category]" format. The heuristic filters were designed for LLM-generated definitions which are noisier. Applying the same filters to clean entity definitions caused false rejections (e.g., "person" blocked by `is_connector_word()`). The fast path is correct for its input class.
**Recommendation**: Long-term, the heuristic filters should be robust enough to handle both entity and LLM definitions without a separate code path. Short-term, the fast path is a clean solution.

---

### A23. `find_siblings()` — Category Comparison, Not Geometry
**Severity**: ⚪ TECHNICAL DEBT → 🟢 DOCUMENTED (Phase 19c)
**File**: `crates/dafhne-engine/src/resolver.rs`
**Finding**: `find_siblings()` uses string comparison on definition categories.
**19c Fix**: Added TODO comment documenting the geometric alternative (nearest-neighbor spatial lookup with category filtering). Blocked on spatial indexing (not yet implemented). Behavioral code unchanged.

---

### A24. skip_words List in Question Parsing
**Severity**: 🟡 PRAGMATIC → 🟢 PARTIALLY FIXED (Phase 19c)
**File**: `crates/dafhne-engine/src/resolver.rs`, question detection functions
**Finding**: Multiple functions used hardcoded skip_words and question_syntax lists.
**19c Fix**: `question_syntax` filter replaced with `structural.contains()`. Question verb detection uses `structural.contains(&tokens[0])`. The `structural` parameter was already threaded through all resolver functions. Two `skip_words` fallback sets retained as definition-shape patterns (see 19c report, Design Note) — replacing them with full structural causes regression when content-significant words like "thing" are structurally classified.

---

---

## Summary

### Severity Counts (Post-19c)

| Severity | Original | After 19c | Findings |
|----------|----------|-----------|----------|
| 🔴 VIOLATION | 4 | 1 | A06 remains |
| 🟡 PRAGMATIC/DOCUMENTED | 8 | 4 | A01→documented, A04, A06, A07, A08, A13→documented, A14→partially fixed, A22 |
| 🟢 ALIGNED/FIXED | 4 | 13 | A02✅, A03✅, A05✅, A09✅, A10✅, A11✅, A12✅, A15✅, A16, A17, A18, A19, A20✅, A23✅, A24✅ |
| ⚪ TECHNICAL DEBT | 6 | 1 | A21 remains |

**Total findings: 24. Fixed/documented: 16. Remaining: 8.**
**Regression impact: zero** — dict5 20/20, unified_test 45/50 (both match pre-19c baselines).

*See `reports/19c_code_audit_fixes.md` for detailed per-finding fix descriptions.*

---

### The Honest Assessment

#### How much of DAPHNE is truly geometric vs. symbolic?

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
| Task routing | Partially hardcoded → improved | Grammar/content indicators now from space vocab; math indicators remain |
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

But geometry cannot distinguish "similar" from "identical" — dog ≈ cat (both animals) is indistinguishable from dog → animal (is-a relationship) in pure distance. This is not a DAPHNE-specific failure. It's a fundamental property of metric spaces: distance is symmetric and doesn't encode direction.

The definition-chain check adds asymmetric, directional evidence: "dog's definition contains 'animal'" is a directed relationship that distance cannot express. This is the same distinction between:
- **Embedding spaces** (Word2Vec, GloVe): symmetric similarity
- **Knowledge graphs** (TransE): directed relations

DAPHNE discovered empirically what the field knew theoretically: you need both.

**The philosophical answer**: DAPHNE is a hybrid system that uses geometry for WHAT (similarity, association, proximity) and symbolic chain traversal for WHY and WHETHER (identity, causation, negation). The geometry is not the whole knowledge — but it IS half the knowledge, and the half that scales.

---

### The 4 Original Violations: Post-19c Status

| ID | Violation | Pre-19c | Post-19c | Status |
|----|-----------|---------|----------|--------|
| A01 | Question verbs | 🔴 | 🟡 | Yes/No verbs use discovered structural set. 5W words documented as language-specific. |
| A03 | Structural word list | 🔴 | 🟢 | Replaced with `structural_words_cache` from `classify_word_roles()`. |
| A13 | English question syntax | 🔴 | 🟡 | Yes/No uses structural. 5W words remain (documented with refactoring path). |
| A14 | Task routing indicators | 🔴 | 🟡 | Grammar/content indicators use space vocabulary. Math indicators remain. |

Three of four violations downgraded to documented pragmatic debt. A03 fully resolved. The remaining hardcoded English knowledge (5W question words, math indicators) is documented with clear refactoring paths.

---

### The Founding Principles: Scorecard (Updated Post-19c)

| Principle | Pre-19c | Post-19c | Evidence |
|-----------|---------|----------|----------|
| 1. Everything emerges from text | **Partial** | **Improved** | Structural words, SELF triggers, task indicators now derived from text. 5W question words remain hardcoded (documented). |
| 2. Connectors discovered, not defined | **Full** | **Full** | connector_discovery.rs is purely statistical. |
| 3. Geometry IS the knowledge | **Partial** | **Partial** | For similarity and association: yes. For identity and negation: no. (Fundamental — see A06.) |
| 4. No NLP/neural networks | **Full** | **Full** | Pure Rust, zero ML libraries. |
| 5. Transitive reasoning is free | **Full** | **Full** | Geometric proximity is inherently transitive. Works on 3+ hops. |
| 6. Honesty is free | **Full** | **Full** | No proximity = "I don't know". Emerges naturally. |
| 7. Parameters evolved | **Partial** | **Improved** | 5 new evolvable params (max_follow_per_hop, max_chain_hops, weighted_distance_alpha, uniformity_num_buckets, uniformity_threshold). All resolver constants now in EngineParams. |

**Score: 4/7 fully satisfied, 3/7 partially satisfied (2 improved), 0/7 violated outright.**

The partial satisfactions are honest: they reveal where geometry ends and symbols begin. This IS the finding.
