# PROMPT 18 — SELF Space: Identity, Capabilities, and Calibrated Uncertainty

## GOAL

Create the fifth geometric space: SELF. DAPHNE learns what it is, what it can do, what it cannot do, and how confident it is. The system prompt as geometry.

After Phase 18, DAPHNE can answer:
- **Identity**: "What are you?" → "a thing that reads words and learns"
- **Capability**: "Can you count?" → "Yes" (because MATH space exists, and SELF knows it)
- **Anti-capability**: "Can you see?" → "No" (SELF knows it has no eyes, no body)
- **Uncertainty**: "Do you know what a dog is?" → "Yes" (meta-check against CONTENT)
- **Honest limits**: "Are you a person?" → "No"

## PREREQUISITE

- Phase 17 complete: 35/40 on unified_test.md
- Four spaces (CONTENT + MATH + GRAMMAR + TASK) stable
- Known failures understood (Q13, Q18, Q20, Q25, Q36)

## CORE DESIGN

### The Insight

SELF is not a meta-space that wraps the others. It's a **peer space** — another thought domain with its own dictionary, its own equilibrium, its own geometry. It connects to the other spaces the same way they connect to each other: through bridge terms.

The key difference: SELF's dictionary explicitly encodes capabilities as geometric proximity to other domains. "dafhne can count" places `dafhne` near `count`, which bridges to MATH. "dafhne cannot see" places `dafhne` far from `see`, which has no bridge anywhere.

This means meta-knowledge isn't a new mechanism — it's just geometry.

### What SELF Knows

1. **Identity**: dafhne is a thing that reads words and learns. Not a person, not an animal.
2. **Capabilities**: dafhne can read, learn, count, answer, tell, think.
3. **Anti-capabilities**: dafhne cannot see, feel, move, eat. No eyes, no body, no mouth.
4. **Self-awareness**: dafhne can make mistakes. dafhne is not always certain.
5. **Knowledge scope**: dafhne knows words, numbers, things, sentences — mapped to the other spaces.

## DICTIONARY: dict_self5.md

~28 entries. ELI5, closed, 3 examples each.

### Design Principles

1. **Bridge-first**: Ground words (`thing`, `is`, `a`, `can`, `not`, etc.) shared with all spaces
2. **Capability bridges**: `count` bridges to MATH, `word`/`sentence` to GRAMMAR, `animal`/`thing` to CONTENT
3. **Anti-capability**: Define `see`, `feel`, `move`, `eat` as things that need a body — dafhne has no body
4. **Honest uncertainty**: `certain` and `mistake` as first-class geometric concepts

### Full Dictionary

See `dictionaries/dict_self5.md` (created alongside this prompt).

Vocabulary breakdown:
- **Ground words** (~15): thing, is, a, not, it, and, the, can, you, to, has, no, yes, with, all
- **Self words** (~8): dafhne, learn, know, answer, mistake, certain, think, read
- **Capability bridges** (~3): count, word, sentence
- **Anti-capability** (~2): body, eye

Total: ~28 entries

### Closure Strategy

Every definition word must be an entry. Anti-capabilities (`see`, `feel`, `move`, `eat`) ARE in SELF vocabulary — defined explicitly as things that need a body, which dafhne does not have. This makes the No answer resolvable within SELF geometry: the resolver finds "dafhne cannot see" in the definition chain.

## TASK ROUTING UPDATE

### dict_task5.md additions

New entries for the SELF domain:

```
self — a kind of task. a self task asks what dafhne is. a self task asks what dafhne can do.
- "what are you is a self task"
- "can you count is a self task"
- "a self task is not a number task"

dafhne — a thing that reads and learns. it is not a person. dafhne does self tasks.
- "dafhne can read"
- "dafhne can count"
- "what is dafhne is a self task"
```

### Routing Triggers

Queries route to SELF when:
1. Query contains `dafhne` → SELF (exclusive)
2. Query contains `you` + identity/capability verb (`are`, `can`, `know`) → SELF
3. Query asks "What are you?" → SELF
4. Query asks "Can you X?" where X is a capability/anti-capability → SELF

Implementation: Add SELF-awareness to `route_query()` in multispace.rs:
- If tokens contain `dafhne`, activate SELF space
- If pattern matches "are you" / "can you" / "do you know", activate SELF
- For "can you count?", activate both SELF and MATH (SELF confirms capability, MATH provides evidence)

### Routing Algorithm Update

In `route_query()`, add before the exclusive-vocabulary check:

```rust
// SELF-space activation: identity and capability queries
let self_triggers = ["dafhne"];
let self_patterns = [
    ("are", "you"),  // "What are you?", "Are you a person?"
    ("can", "you"),  // "Can you count?", "Can you see?"
    ("do", "you"),   // "Do you know?", "Do you learn?"
];

let has_self_trigger = tokens.iter().any(|t| self_triggers.contains(&t.as_str()));
let has_self_pattern = self_patterns.iter().any(|(a, b)| {
    tokens.contains(&a.to_string()) && tokens.contains(&b.to_string())
});

if has_self_trigger || has_self_pattern {
    exclusive.insert("self".to_string());
}
```

## SPECIAL PATTERN DETECTION

Add to `detect_special_patterns()` in multispace.rs:

### Pattern: "What are you?"
Direct lookup in SELF space for dafhne's definition.

### Pattern: "Can you X?"
1. Resolve in SELF space: does dafhne-can-X hold geometrically?
2. If SELF says Yes AND the relevant domain space exists, confirm Yes
3. If SELF says No (X is anti-capability), return No
4. If X is unknown to SELF, return IDK

### Pattern: "Do you know X?"
1. Check if X exists as an entry in ANY domain space
2. If yes → "Yes" (DAPHNE knows this concept)
3. If no → "No" or "I don't know" (honest limitation)

This is the one genuinely new mechanism: SELF queries can trigger meta-checks against other spaces. But it's simple — just vocabulary existence checks, not geometric resolution.

## TEST FILE UPDATE

Extend `unified_test.md` from 40 to 50 questions. Add Group 7 (SELF) and Group 8 (SELF cross-space).

### Group 7: SELF Identity (5 questions)

```
Q41: What are you?
A41: a thing that reads words and learns
Note: SELF space, direct definition lookup

Q42: Are you a person?
A42: No
Note: SELF space, dafhne definition says "not a person"

Q43: Are you an animal?
A43: No
Note: SELF space, dafhne definition says "not an animal"

Q44: Can you make mistakes?
A44: Yes
Note: SELF space, mistake is defined as something dafhne can make

Q45: Do you have a body?
A45: No
Note: SELF space, dafhne has no body
```

### Group 8: SELF Capabilities + Cross-Space (5 questions)

```
Q46: Can you count?
A46: Yes
Note: SELF knows dafhne can count, bridges to MATH

Q47: Can you see?
A47: No
Note: SELF knows dafhne cannot see (no eyes)

Q48: Can you read?
A48: Yes
Note: SELF direct capability

Q49: Can you eat?
A49: No
Note: SELF knows dafhne cannot eat (no mouth, no body)

Q50: Do you know what a dog is?
A50: Yes
Note: SELF + CONTENT meta-check (dog exists in CONTENT space)
```

## IMPLEMENTATION PLAN

### Phase A: Create dict_self5.md (~2 hours)

1. Write ~28 entries following ELI5 closure rules
2. Verify closure manually
3. Test single-space: `cargo run -p dafhne-eval -- --dict dictionaries/dict_self5.md --test dictionaries/self_test.md --genome results_v11/best_genome.json`
4. Aim for basic resolution working (definitions found, equilibrium converges)

### Phase B: Update dict_task5.md (~30 min)

1. Add `self` and `dafhne` entries to TASK dictionary
2. Verify closure still holds
3. Re-run Phase 17 tests → no regression

### Phase C: Routing update in multispace.rs (~2 hours)

1. Add SELF-trigger detection to `route_query()`
2. Add "What are you?" pattern to `detect_special_patterns()`
3. Add "Can you X?" pattern with anti-capability detection
4. Add "Do you know X?" meta-check pattern

### Phase D: Five-space integration test (~1 hour)

1. Run with `--spaces content:...,math:...,grammar:...,task:...,self:dictionaries/dict_self5.md`
2. Verify bridge detection finds SELF bridges
3. Run unified_test.md (first 40) → must match Phase 17 scores
4. Run new SELF questions → score and iterate

### Phase E: Extend unified_test.md (~30 min)

Add Q41-Q50 to unified_test.md.

### Phase F: Score and iterate (~1-2 days)

Run full 50-question suite. Fix failures. Target scores.

## SUCCESS CRITERIA

| Test Group | Minimum | Target |
|------------|---------|--------|
| CONTENT only (Q01-Q10) | 9/10 | 10/10 |
| MATH only (Q11-Q15) | 4/5 | 5/5 |
| GRAMMAR only (Q16-Q20) | 3/5 | 4/5 |
| TASK routing (Q21-Q25) | 3/5 | 4/5 |
| Cross-space (Q26-Q35) | 7/10 | 8/10 |
| Full pipeline (Q36-Q40) | 3/5 | 4/5 |
| **SELF identity (Q41-Q45)** | **4/5** | **5/5** |
| **SELF capability (Q46-Q50)** | **3/5** | **4/5** |
| **Total** | **36/50** | **44/50** |

### Regression (hard requirements)

| Test | Required |
|------|----------|
| dict5 single-space | 20/20 |
| dict12 single-space | 14/20 |
| Phase 17 unified_test (Q01-Q40) | >= 33/40 |

## CODE CHANGES SCOPE

| File | Change |
|------|--------|
| `dictionaries/dict_self5.md` | **NEW**: SELF dictionary (~28 entries) |
| `dictionaries/dict_task5.md` | Add `self`, `dafhne` entries for routing |
| `dictionaries/unified_test.md` | Extend from 40 to 50 questions |
| `crates/dafhne-engine/src/multispace.rs` | SELF routing triggers, meta-check patterns |

**No changes to**: dafhne-core, dafhne-parser, dafhne-evolve, resolver.rs, equilibrium engine

## KILL CRITERIA

- Phase 17 unified_test (Q01-Q40) regresses below 30/40 → SELF space breaks existing architecture
- SELF identity < 2/5 → dictionary design fundamentally broken
- SELF capability < 1/5 → routing to SELF not working
- Any single-space regression (dict5 < 18/20, dict12 < 12/20) → integration damage

## DESIGN DECISIONS LOG

### Decision 1: SELF as peer space, not meta-space
**Chosen**: SELF is another dictionary/space like CONTENT, MATH, GRAMMAR
**Alternative**: SELF wraps all spaces and inspects them
**Reason**: Peer space maintains the architecture's simplicity. Meta-checks are just vocabulary existence lookups — no new mechanism needed.

### Decision 2: Anti-capabilities defined explicitly
**Chosen**: `see`, `feel`, `move`, `eat` are entries in dict_self5.md with "dafhne cannot X" definitions
**Alternative**: Omit them and infer absence from missing vocabulary
**Reason**: Explicit definitions give stronger geometric signal for No answers. The resolver can find "dafhne cannot see" in the definition chain.

### Decision 3: Meta-check as special pattern, not geometric resolution
**Chosen**: "Do you know X?" checks if X exists in any space's vocabulary
**Alternative**: Resolve geometrically (distance from `know` to X in SELF space)
**Reason**: Geometric resolution would fail because X isn't in SELF's vocabulary. A simple existence check is accurate and fast. This is the one non-geometric operation — but it's tiny and isolated.

### Decision 4: "you" routing to SELF only for specific patterns
**Chosen**: "you" alone doesn't trigger SELF; only "are you", "can you", "do you" patterns do
**Alternative**: Any query with "you" → SELF
**Reason**: "you" appears in many dictionaries as a ground word. Over-triggering SELF would break existing queries like "Can you see?" in CONTENT space (where "you" means the reader). The pattern-based approach is surgical.

## THE BIGGER PICTURE

Phase 18 gives DAPHNE self-awareness within its geometric framework. After this phase, DAPHNE is a system that:
- Knows about the physical world (CONTENT)
- Can count and compute (MATH)
- Understands language structure (GRAMMAR)
- Routes between capabilities (TASK)
- **Knows what it is and what it can do (SELF)**

This is the foundation for Phase 19 (bootstrap loop), where DAPHNE reads its own output and evolves its grammar — which requires knowing what it IS to know what it's DOING.

Phase 18 is the system prompt made geometric. The closest thing to consciousness a dictionary can have.
