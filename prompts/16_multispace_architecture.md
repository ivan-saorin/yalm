# PROMPT 16 — Multi-Space Architecture: Parallel Thought Domains

## GOAL

Build the ability to run multiple independent geometric spaces ("thought domains") and compose their outputs via a TASK dispatcher space. This is DAPHNE's path toward SLM-level abstraction.

Two domain spaces (MATH, GRAMMAR) + one dispatcher space (TASK). Each space runs its own equilibrium independently. The TASK space routes queries to the correct domain(s) and composes results.

## PREREQUISITE

- Phase 14 complete (all 5W question types working)
- dict_math5.md, dict_grammar5.md, dict_task5.md in dictionaries/
- multispace_test.md in dictionaries/
- All existing regression tests pass

## KEY ARCHITECTURAL DECISIONS

### 1. Each space = independent DAPHNE instance

Each dictionary gets its own equilibrium. No shared state during equilibrium computation. The spaces are connected ONLY at query time via bridge terms.

```
dict_math5.md  → equilibrium → Space_MATH   (46 words, 8D)
dict_grammar5.md → equilibrium → Space_GRAMMAR (48 words, 8D)
dict_task5.md  → equilibrium → Space_TASK   (40 words, 8D)
```

Implementation: run `dafhne-eval` three times, one per dictionary. Store three separate spaces.

### 2. Bridge terms identified by vocabulary intersection

Bridge terms = words that appear in multiple dictionaries. Their definitions may differ per space (that's a feature, not a bug).

```rust
let bridge_math_grammar: HashSet<String> = 
    space_math.vocabulary()
    .intersection(&space_grammar.vocabulary())
    .cloned()
    .collect();
```

Expected bridges:
- MATH ↔ GRAMMAR: thing, is, a, not, name, one, all, make, do, how, many, order, first, after, ...
- MATH ↔ TASK: number, count, result, plus, minus, how, many, ...
- GRAMMAR ↔ TASK: word, sentence, how, many, ...
- ALL THREE: thing, is, a, not, what, can, you, ...

### 3. TASK space as geometric dispatcher

Query arrives. Parse it. Identify key terms. Measure distances in Space_TASK:

```
Query: "What is two plus three?"
  │
  ├─ "plus" in query → dist(plus, number_task) in Space_TASK → CLOSE
  ├─ "plus" in query → dist(plus, word_task) in Space_TASK → FAR
  │
  └─ Route to: MATH
```

```
Query: "Is dog a noun?"
  │
  ├─ "noun" in query → dist(noun, word_task) in Space_TASK → CLOSE
  ├─ "noun" not in Space_MATH → skip MATH
  │
  └─ Route to: GRAMMAR
```

```
Query: "Two plus three. Write the answer as a sentence."
  │
  ├─ "plus" → number_task → MATH
  ├─ "sentence" → word_task → GRAMMAR
  │
  └─ Route to: MATH + GRAMMAR (compose)
```

The routing algorithm:

```
1. Tokenize query into words
2. For each non-structural word in query:
   a. Is it in Space_TASK vocabulary? If yes, measure dist to "number" and "word"
   b. Is it ONLY in Space_MATH? → activate MATH
   c. Is it ONLY in Space_GRAMMAR? → activate GRAMMAR
   d. Is it in both? → activate both, use TASK distances to rank
3. If no domain activated: fall back to the space where most query words exist
4. Activated spaces process the query independently
5. Results composed (see section 4)
```

### 4. Both equilibrium modes: separate AND joint

Test BOTH:

**Mode A: Separate** (primary)
- Each space has its own equilibrium
- Query routing selects space(s)
- Bridge terms connect via vocabulary overlap
- Cross-space queries resolved by chain: word in Space_A → bridge term → word in Space_B

**Mode B: Joint** (experimental)
- Merge all three dictionaries into one large dictionary
- Single equilibrium over ~80 unique words
- No routing needed — everything in one space
- Compare scores: does separation help or hurt?

The comparison tells us if domain separation is an architectural advantage or just organization.

### 5. Result composition

When multiple spaces are activated, results must be composed:

**Case: Both answer the same type (Yes/No)**
- Both say Yes → Yes (high confidence)
- Both say No → No (high confidence)
- Disagree → report both with confidence (distance)

**Case: One computes, other formats**
- MATH produces: "five" (raw result)
- GRAMMAR provides: sentence structure → "two plus three is five"
- Composition: MATH result inserted into GRAMMAR template

**Case: Chain across spaces**
- Q: "Is five a noun?"
- MATH: five is a number → number is a thing
- GRAMMAR: noun is a name for a thing
- Bridge: "thing" exists in both → five chains to thing in MATH, noun chains from thing in GRAMMAR → Yes

## IMPLEMENTATION PLAN

### Phase A: Infrastructure (≈1 day)

**New module: `multispace.rs` in dafhne-engine**

```rust
pub struct Space {
    pub name: String,           // "math", "grammar", "task"
    pub vocabulary: HashSet<String>,
    pub positions: HashMap<String, Vec<f64>>,
    pub definitions: HashMap<String, String>,
    pub connectors: Vec<Connector>,
}

pub struct MultiSpace {
    pub spaces: HashMap<String, Space>,
    pub bridges: HashMap<(String, String), HashSet<String>>,  // (space_a, space_b) -> bridge terms
}

impl MultiSpace {
    pub fn load(configs: Vec<SpaceConfig>) -> Self;
    pub fn identify_bridges(&mut self);
    pub fn route_query(&self, query: &str) -> Vec<String>;  // returns activated space names
    pub fn resolve(&self, query: &str) -> QueryResult;
}
```

**Changes to dafhne-eval:**
- New flag: `--spaces math:dict_math5.md,grammar:dict_grammar5.md,task:dict_task5.md`
- Loads multiple spaces, builds MultiSpace
- Routes and resolves queries through MultiSpace

### Phase B: Single-Space Validation (≈1 day)

Before any cross-space work, validate that each dictionary works independently:

1. Run dict_math5 through standard dafhne-eval with v11 params
   - Does equilibrium converge?
   - Do basic math queries work? (Q01-Q05)
   - Distance matrix: are numbers near each other? Are operations near operations?

2. Run dict_grammar5 through standard dafhne-eval
   - Does equilibrium converge?
   - Do grammar queries work? (Q06-Q10)
   - Distance matrix: are nouns near nouns? Are verbs near verbs?

3. Run dict_task5 through standard dafhne-eval
   - Does equilibrium converge?
   - Do routing queries work? (Q11-Q15)
   - Distance matrix: are number-related terms clustered? Word-related terms?

If ANY single-space fails: fix the dictionary first. Don't proceed to cross-space.

### Phase C: Cross-Space Resolution (≈2 days)

1. Implement bridge term identification
2. Implement TASK routing (geometric distance-based)
3. Implement cross-space chain traversal
4. Test Q16-Q20 (cross-space)
5. Test Q21-Q25 (task-routed)

### Phase D: Joint Mode Comparison (≈half day)

1. Merge all three dicts into one: dict_joint.md
2. Run standard dafhne-eval on dict_joint
3. Score multispace_test.md on joint space
4. Compare: separate vs joint scores

## TEST FILE

`dictionaries/multispace_test.md` (25 questions, already created)

## SUCCESS CRITERIA

| Metric | Minimum | Target |
|--------|---------|--------|
| MATH only (Q01-Q05) | 4/5 | 5/5 |
| GRAMMAR only (Q06-Q10) | 4/5 | 5/5 |
| TASK routing (Q11-Q15) | 3/5 | 4/5 |
| Cross-space (Q16-Q20) | 2/5 | 3/5 |
| Task-routed (Q21-Q25) | 1/5 | 2/5 |
| **Total** | **14/25** | **19/25** |
| Separate vs Joint | Separate ≥ Joint on cross-space | Separate > Joint |
| Existing regression: dict5 | 20/20 | 20/20 |
| Existing regression: dict12 | 14/20 | 14/20 |
| Existing regression: full_test | 19/21 | 19/21 |

## KILL CRITERIA

- Phase B fails: dictionaries don't produce meaningful geometry → rewrite dictionaries
- Phase C routing wrong >50%: TASK space doesn't separate domains → redesign dispatcher
- Joint mode scores HIGHER on cross-space than separate: domain separation hurts → rethink architecture
- Any existing regression fails: back out changes to dafhne-engine

## CODE CHANGES SCOPE

| File | Change |
|------|--------|
| `crates/dafhne-engine/src/multispace.rs` | NEW: MultiSpace, routing, cross-space resolution |
| `crates/dafhne-engine/src/lib.rs` | Add multispace module |
| `crates/dafhne-eval/src/main.rs` | Add --spaces flag, MultiSpace mode |
| `dictionaries/dict_math5.md` | NEW (already created) |
| `dictionaries/dict_grammar5.md` | NEW (already created) |
| `dictionaries/dict_task5.md` | NEW (already created) |
| `dictionaries/multispace_test.md` | NEW (already created) |

**No changes to**: dafhne-core, dafhne-parser, dafhne-evolve, resolver.rs (existing), equilibrium engine

The resolver.rs is REUSED inside each Space. multispace.rs WRAPS the existing resolver, it doesn't replace it.

## THE BIGGER PICTURE

Phase 16 validates one thing: **can independent geometric spaces compose meaningful answers that no single space could produce alone?**

Q21 ("Two plus three. Write the answer as a sentence.") is the litmus test. MATH alone produces "five". GRAMMAR alone can't compute. Together they produce "two plus three is five". If that works, the multi-space architecture is real.

Future spaces (not in scope for Phase 16):
- LOGIC: causation, implication, contradiction
- SELF: what DAPHNE is, what it can do, what it cannot do
- CONTENT: dict5/dict12/open-mode (existing capability)
- CAUSAL: why, because, therefore

Each is a new dictionary + equilibrium. The architecture scales by addition, not by growth.
