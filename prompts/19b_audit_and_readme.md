# PROMPT 19b — Code Audit, README Overhaul, and Prior Art Analysis

> **STATUS: REPORTING ONLY — No code changes. Produce reports and documentation.**

## CONTEXT

DAFHNE has reached a milestone: 19 phases of development, 45/50 on unified_test.md, 5 geometric spaces, a self-improvement bootstrap loop, and zero neural networks. Before moving to Phase 20 (per-space parameter evolution), we need to pause, breathe, and honestly assess what we've built.

This prompt has three deliverables:
1. **Code Audit Report** — find everything that violates DAFHNE's architectural spirit
2. **README Overhaul** — restructure with deep-dive pages
3. **Prior Art Analysis** — honest assessment of novelty vs. reinvention

**CRITICAL: This is a reporting phase. Do NOT modify any source files. Produce markdown reports only.**

Project location: `D:\workspace\projects\dafhne`

Use the filesystem MCP with path `D:\workspace\projects\dafhne` to access files.

## PREREQUISITE READING

Before writing anything, read these files in order. They define the soul of the project:

1. `prompts/02_geometric_comprehension_engine.md` — the original vision
2. `prompts/03_evolution_self_improvement.md` — genetic algorithm design
3. `prompts/06_surgical_fixes.md` — where symbolic chain-traversal was introduced
4. `prompts/08_sequential_equilibrium.md` — force field physics
5. `prompts/09c_connector_quality.md` — uniformity filter
6. `prompts/13_basic_writing.md` — describe mode
7. `prompts/15_description_enrichment.md` — property extraction stub (never fully implemented)
8. `prompts/16_multispace_architecture.md` — multi-space design
9. `prompts/18_self_space.md` — SELF space
10. `prompts/19_bootstrap_loop.md` — bootstrap loop

Then read the source code:
- `crates/dafhne-core/src/lib.rs` — all shared types
- `crates/dafhne-engine/src/strategy.rs` — evolvable strategies
- `crates/dafhne-engine/src/connector_discovery.rs` — connector pipeline
- `crates/dafhne-engine/src/resolver.rs` — question answering (largest file)
- `crates/dafhne-engine/src/multispace.rs` — multi-space routing and SELF patterns
- `crates/dafhne-engine/src/bootstrap.rs` — self-improvement loop
- `crates/dafhne-engine/src/equilibrium.rs` — sequential equilibrium
- `crates/dafhne-engine/src/force_field.rs` — force simulation

And the current documentation:
- `README.md`
- `RECAP.md`

---

## DELIVERABLE 1: CODE AUDIT REPORT

Write a file: `reports/19b_code_audit.md`

### The Spirit of DAFHNE (from Prompt 02)

These are the founding principles. Every line of code should be measured against them:

1. **Everything emerges from text** — no hardcoded linguistic knowledge
2. **Connectors discovered, not defined** — statistical patterns only
3. **Geometry IS the knowledge** — positions in space encode meaning
4. **No NLP libraries, no neural networks** — pure Rust, pure physics
5. **Transitive reasoning is free** — geometric proximity is inherently transitive
6. **Honesty is free** — no proximity = "I don't know"
7. **Parameters evolved, not hand-tuned** — genetic algorithm explores the space

### Audit Categories

For each finding, classify as:

| Severity | Meaning |
|----------|--------|
| 🔴 VIOLATION | Directly contradicts the spirit. Should be reconsidered. |
| 🟡 PRAGMATIC | Bends the spirit for good reasons. Document the trade-off. |
| 🟢 ALIGNED | Correctly follows the spirit. |
| ⚪ TECHNICAL DEBT | Not a spirit violation, but engineering cleanup needed. |

### What to Look For

#### A. Hardcoded Linguistic Knowledge
Scan all source files for:
- Hardcoded word lists (articles, question verbs, pronouns, etc.)
- Hardcoded syntactic patterns ("is a", "can", "not", "that")
- String matching on specific words rather than geometric lookup
- Any knowledge that "knows" English rather than discovering it from text

Examples to investigate:
- `question_verbs: HashSet<&str> = ["is", "can", "does", "do", "has"]` in resolver.rs
- `articles: HashSet<&str> = ["a", "an", "the"]` in multiple places
- `preceded_by_not()` checking for literal "not" string
- SELF space triggers: `self_triggers = ["dafhne"]`, `self_patterns = [("are", "you"), ...]`

For each: is this discoverable from the dictionary, or is it genuinely hardcoded English?

#### B. Symbolic Operations Masquerading as Geometry
Find every place where the resolver does string/definition operations instead of geometric distance:
- `definition_chain_check()` — symbolic chain traversal
- `definition_category()` — first-content-word extraction
- `resolve_why()` / `trace_chain_path()` — symbolic chain as explanation
- `resolve_when()` / `extract_condition_clause()` — string pattern matching
- `is_property_word()` — definition-shape heuristics
- `is_connector_word()` — pattern membership check
- `find_siblings()` — definition category comparison

For each: could this be replaced with a geometric operation? If not, is the symbolic operation justified?

#### C. Fixed Constants That Should Be Parameters
Find every numeric/string constant that isn't in `EngineParams` or `StrategyConfig`:
- `MAX_FOLLOW_PER_HOP = 3` in resolver.rs
- `max_hops = 3` in resolve_yes_no
- `alpha = 0.2` in resolve_what_is (connector axis emphasis)
- `uniformity_threshold = 0.75` in connector_discovery.rs
- `num_buckets = 10` in connector_discovery.rs
- `topic_threshold` formula (25% / ln(n/50))
- Any threshold in multispace.rs for SELF routing

For each: should it be evolvable? Or is there a principled reason for the constant?

#### D. Dead Code and Unused Paths
- Strategy enum variants that evolution never selects
- Negation models that don't work (known: all four failed on pure geometry)
- Unused functions or struct fields
- Code paths that are unreachable

#### E. Architecture Coherence
- Does `multispace.rs` maintain the "each space is an independent DAFHNE instance" principle?
- Does `bootstrap.rs` respect immutable dictionaries?
- Does the SELF space fit as a peer (not meta) space?
- Are bridge terms working symmetrically?
- Is the resolver's question-type detection generalizable or English-specific?

#### F. Phase 15 Gap
Phase 15 (Rich Description: Property Extraction) was stubbed but never implemented before Phase 19. The bootstrap loop depends on rich describe output. Assess:
- What does describe() currently produce?
- Is the bootstrap loop getting enough signal without Phase 15?
- The 4 new connectors found in bootstrap — did they come from existing describe output or would Phase 15 have found more?

### Report Format

For each finding:
```
### [ID] Short Title
**Severity**: 🔴/🟡/🟢/⚪
**File**: path/to/file.rs, lines X-Y
**Finding**: What was found
**Spirit Violation**: Which principle it violates (if any)
**Justification**: Why it exists (if pragmatic)
**Recommendation**: What could be done (in a future phase, not now)
```

End the audit with summary counts per severity and an honest assessment:
- How much of DAFHNE is truly geometric vs. symbolic?
- What percentage of correct answers come from geometry vs. chain traversal?
- Is the hybrid (geometry + symbols) a pragmatic success or a philosophical failure?

---

## DELIVERABLE 2: README OVERHAUL

The current README.md is good but monolithic. Restructure into:

### Main README.md (keep concise)
- Quick demo (keep as-is, it's great)
- "How It Works" summary (shorter than current)
- Architecture diagram (keep)
- Quick results table
- Links to deep-dive pages

### Deep-dive pages (new directory: `docs/`)

Create these files:

#### `docs/architecture.md` — How DAFHNE Works
The full step-by-step explanation (expanded from current README). Include:
- Dictionary format and closure property
- Connector discovery pipeline (frequency + uniformity)
- Force field and equilibrium physics
- Question resolution strategies
- Multi-space architecture (5 spaces)
- Bootstrap loop

#### `docs/results.md` — Comprehensive Results
All scores, all test suites, all phases. Tables for:
- Closed dictionaries (dict5, dict12, dict18)
- Open mode (Three Men in a Boat, all levels)
- Multi-space (unified_test.md, 50 questions)
- Bootstrap results
- Per-question-type breakdown

#### `docs/design_decisions.md` — Why Things Are the Way They Are
Key decisions with alternatives considered:
- Why closed dictionaries? (vs. corpus-based)
- Why force fields? (vs. co-occurrence matrices)
- Why genetic evolution? (vs. gradient descent)
- Why definition-chain gate? (geometry alone failed for negation)
- Why multi-space? (vs. single merged space)
- Why ELI5? (dumbing down = smarter)
- Why SELF as peer space? (vs. meta-space)

#### `docs/prior_art.md` — Relationship to Existing Work
**This is the most important deep-dive.** See Deliverable 3 below.

#### `docs/limitations.md` — Known Limitations and Honest Assessment
- What DAFHNE can't do (and why)
- The geometry-vs-symbols tension
- Scale limitations
- English-specific assumptions
- What would break if applied to a different language?

#### `docs/roadmap.md` — Where It's Going
- Phase 20: Per-space parameter evolution
- Phase 21: Open mode multi-space
- Long-term: Can the symbolic operations become geometric?

### RECAP.md
Update to include Phases 15-19. The current RECAP stops at Phase 14. Add:
- Phase 15 (stub status — not implemented)
- Phase 16: Multi-space architecture (MATH + GRAMMAR + TASK)
- Phase 17: CONTENT space integration
- Phase 18: SELF space (45/50, identity as geometry)
- Phase 19: Bootstrap loop (4 new connectors, convergence at Level 2)

---

## DELIVERABLE 3: PRIOR ART ANALYSIS

Write a file: `docs/prior_art.md`

This addresses the elephant in the room: **"Did we reinvent the wheel? Has no one thought to teach a machine like a kid before?"**

### Research Questions to Answer

1. **Conceptual Spaces (Gärdenfors, 2000, 2014)**
   - Peter Gärdenfors proposed geometric semantic spaces 25 years ago
   - His framework uses quality dimensions, convex regions, and betweenness
   - How does DAFHNE relate? Is DAFHNE an implementation of Gärdenfors' theory?
   - Key difference candidates: DAFHNE discovers dimensions from text, Gärdenfors assumes them

2. **Word Embeddings (Word2Vec, GloVe, FastText)**
   - All create vector spaces where similar words are close
   - Word2Vec uses neural networks + massive corpus
   - GloVe uses co-occurrence matrix factorization
   - How does DAFHNE differ? Same output (word vectors), different input (dictionary vs. corpus)?
   - Key difference candidates: DAFHNE uses directional forces with connector semantics, not statistical co-occurrence; DAFHNE works from 51 words, not billions

3. **Knowledge Graph Embeddings (TransE, TransR, RotatE)**
   - TransE: `head + relation ≈ tail` in vector space
   - This is remarkably similar to DAFHNE's connector forces
   - TransE learns from (h, r, t) triples; DAFHNE discovers them from text
   - Key difference candidates: DAFHNE discovers the relations, TransE is given them

4. **Dictionary-Based Learning**
   - Has anyone learned from dictionaries specifically?
   - Search for: "learning from dictionary definitions", "definitional embeddings"
   - Projects like: Dict2Vec, Definition-based word embeddings
   - The "teach like a kid" intuition — has it been formalized?

5. **Self-Contained/Closed Vocabulary Learning**
   - The closure property (every word defined in terms of other defined words)
   - This is similar to "grounding" in AI — can meaning be self-contained?
   - The Chinese Room argument (Searle) is relevant here
   - Symbol grounding problem (Harnad, 1990)

6. **Force-Directed Graph Layouts**
   - Force-directed placement (Fruchterman-Reingold, 1991)
   - Spring-electrical models for graph visualization
   - DAFHNE's equilibrium is literally a force-directed layout with semantic forces
   - Difference: DAFHNE's forces encode typed relations, not just connectivity

7. **Bootstrap/Self-Play**
   - AlphaGo Zero learns by self-play
   - DAFHNE's bootstrap loop: reads own output → discovers new patterns
   - Self-distillation in neural networks
   - How does DAFHNE's approach compare?

8. **Children's Language Acquisition**
   - The "teach like a kid" principle
   - Nativist (Chomsky) vs. empiricist (Tomasello) debate
   - Conceptual primitives and semantic bootstrapping
   - Does DAFHNE's ELI5 principle map to developmental psychology?

### What to Produce

A structured document covering:

**Section 1: The Landscape**
- Brief taxonomy of approaches to machine language understanding
- Where DAFHNE fits (or doesn't fit) in this taxonomy

**Section 2: Closest Relatives**
- For each related work: what's shared, what's different, what's novel
- Be brutally honest — if something is reinvention, say so

**Section 3: What's Actually Novel**
- The specific combination of ideas that (as far as we can tell) hasn't been done:
  - Closed dictionary → connector discovery → force field → equilibrium → geometric QA
  - The full pipeline from definitions to comprehension without neural networks
  - Multi-space with domain separation from ELI5 dictionaries
  - Bootstrap loop using describe-then-rediscover
- Or: if this HAS been done, document it

**Section 4: What's NOT Novel (and that's fine)**
- Word vectors: well-established since 2013
- Force-directed layouts: well-established since 1991
- Genetic algorithms: well-established since 1975
- Question answering from knowledge bases: decades of work
- The individual pieces are known; the combination may be new

**Section 5: The Hard Question**
> "If everything is as it seems, we basically rewrote ML from scratch.
> No one thought to teach a machine like a kid before? How could it be?"

Answer this honestly. Possibilities:
- **Yes, people tried, and it didn't scale** — dictionary-based approaches were explored in the 1980s-90s (knowledge-based AI, CYC project) and abandoned when statistical methods won
- **The ELI5 closure trick is the innovation** — not the geometry, not the forces, but the constraint that definitions use only defined words, creating a self-consistent universe
- **It's a rediscovery with modern tooling** — the ideas are old but the implementation pathway (Rust + genetic evolution + LLM-generated definitions for open mode) is new
- **It works on toy problems but wouldn't scale** — 51-2008 words is not "rewriting ML"; the real test is 100K+ words
- **The hybrid nature is the real finding** — geometry for similarity, symbols for identity; this tension IS the result

Use web search to find relevant papers and cite them properly. The analysis should be research-quality, not hand-waving.

---

## DELIVERABLE 4 (optional): PHASE 15 STATUS

If time permits, produce: `reports/19b_phase15_gap.md`

Phase 15 (Rich Description: Property Extraction) was planned as:
```
"a big hot thing that is up" → extract "big", "hot", "up" as separate sentences
```

It was never implemented. Phases 16-19 proceeded without it. Assess:
- What does describe() currently produce for key test words?
- Run describe on dict5 content words (dog, cat, sun, ball, water, food, person, animal)
- How would Phase 15 change the bootstrap loop results?
- Is Phase 15 still needed, or has the architecture moved past it?

---

## OUTPUT FILES

| File | Content |
|------|---------|
| `reports/19b_code_audit.md` | Complete code audit with findings |
| `docs/prior_art.md` | Prior art analysis |
| `docs/architecture.md` | Deep-dive: how DAFHNE works |
| `docs/results.md` | Comprehensive results |
| `docs/design_decisions.md` | Key decisions and alternatives |
| `docs/limitations.md` | Known limitations |
| `docs/roadmap.md` | Future directions |
| `README.md` | **UPDATED** (restructured, links to docs/) |
| `RECAP.md` | **UPDATED** (Phases 15-19 added) |
| `reports/19b_phase15_gap.md` | Phase 15 status assessment (optional) |

**IMPORTANT**: The README.md and RECAP.md updates are the only modifications to existing files. All other output is NEW files in `reports/` and `docs/`.

---

## HOW TO START

1. Read all 10 prompts listed in prerequisite reading
2. Read all engine source files
3. Start with the code audit (Deliverable 1) — this grounds everything
4. Do the prior art analysis (Deliverable 3) — this answers the big question
5. Write the deep-dive docs (Deliverable 2)
6. Update README and RECAP last

The code audit is first because it forces you to understand every line of code before making claims about novelty.

---

## SUCCESS CRITERIA

| Metric | Target |
|--------|--------|
| Code audit: every source file reviewed | All 8 engine .rs files |
| Code audit: findings classified by severity | At least 15 findings |
| Prior art: concrete references to published work | At least 8 citations |
| Prior art: honest answer to "did we reinvent ML?" | Clear, evidence-based |
| README: links to all deep-dive pages | 6+ docs/ files |
| RECAP: Phases 15-19 documented | 5 new sections |
| No source code modified | Zero .rs changes |

---

## THE REAL QUESTION

At the end of this exercise, we should be able to answer:

**"DAFHNE is a _____ that differs from existing approaches because _____. The individual components (word vectors, force layouts, genetic algorithms) are established techniques. The novel contribution is _____. The main limitation is _____."**

Fill in those blanks. With evidence.
