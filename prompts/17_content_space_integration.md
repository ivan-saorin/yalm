# PROMPT 17 — Content Space Integration: dict5 as Fourth Domain

## GOAL

Integrate dict5 (the original 51-word content dictionary) as a fourth space in the multi-space architecture alongside MATH, GRAMMAR, and TASK. This unifies DAPHNE's existing comprehension capabilities with the new multi-space framework.

After Phase 17, DAPHNE can answer questions about dogs, cats, and the sun (CONTENT), do arithmetic (MATH), reason about parts of speech (GRAMMAR), and route automatically between them (TASK).

## PREREQUISITE

- Phase 16 complete (22/25 on multispace_test)
- MultiSpace infrastructure in `multispace.rs` working
- `--spaces` CLI flag functional
- dict5: 20/20 in single-space mode (regression baseline)

## WHAT CHANGES

### 1. dict_task5.md gains CONTENT routing

Currently dict_task5 routes between `number` tasks and `word` tasks. It needs a third domain: `content` tasks — questions about things, animals, properties.

Add to dict_task5.md:

```
content — things you know. animals, the sun, colors. a content task asks about things.
- "what is a dog is a content task"
- "is the sun hot is a content task"
- "a content task asks about things"
```

And update domain bridge words section to include:

```
animal — a thing that lives. it can move. it can eat. animal tasks are content tasks.
- "is a dog an animal is a content task"
- "animal is a content word"
- "animal is not a number"

hot — you feel hot. hot is a property. asking about hot is a content task.
- "is the sun hot"
- "hot is a content word"
- "hot is not a number word"

cold — you feel cold. not hot. asking about cold is a content task.
- "is water cold"
- "cold is a content word"
- "cold is not a number word"
```

This must remain CLOSED — verify all new words are defined.

### 2. Four-space invocation

```bash
cargo run -p dafhne-eval -- \
  --spaces content:dictionaries/dict5.md,math:dictionaries/dict_math5.md,grammar:dictionaries/dict_grammar5.md,task:dictionaries/dict_task5.md \
  --test dictionaries/unified_test.md \
  --genome results_v11/best_genome.json
```

Note: `--genome` applies to ALL spaces. The v11 params were tuned on dict5 but should be a reasonable starting point for all ELI5 dictionaries. If math/grammar scores regress with v11 params vs defaults, log the difference but don't tune — param tuning for multi-space is Phase 20.

### 3. Bridge terms CONTENT ↔ others

Expected bridges:
- CONTENT ↔ MATH: `thing`, `big`, `small`, `one`, `all`, `part`, `make`, `see`, `feel`, `can`, ...
- CONTENT ↔ GRAMMAR: `thing`, `name`, `see`, `feel`, `good`, `bad`, `animal`, `person`, ...
- CONTENT ↔ TASK: `thing`, `animal`, `hot`, `cold`, `content`, ...

Key insight: dict5's vocabulary is heavily overlapping with all other spaces because it defines the ELI5 ground vocabulary. It IS the bridge.

### 4. TASK routing for CONTENT

The router must distinguish:
- "Is a dog an animal?" → CONTENT (dog, animal both in CONTENT, not in MATH/GRAMMAR exclusively)
- "Is dog a noun?" → GRAMMAR (noun only in GRAMMAR)
- "What is two plus three?" → MATH (plus only in MATH)
- "Is the sun hot? Write a sentence." → CONTENT + GRAMMAR
- "How many animals can move?" → CONTENT + MATH (count + animals)

The routing algorithm from Phase 16 should handle this via exclusive-vocabulary detection:
- Words ONLY in CONTENT: `dog`, `cat`, `sun`, `ball`, `water`, `food`, `color`, `sound`, `place`, `live`, `eat`, `move`, `feel`, `hot`, `cold`, `up`, `down`
- Words ONLY in MATH: `zero`, `two`, `three`, `four`, `five`, `ten`, `plus`, `minus`, `equal`, `more`, `less`, `result`
- Words ONLY in GRAMMAR: `word`, `sentence`, `noun`, `verb`, `subject`, `action`, `property`, `meaning`, `tell`, `say`

If a query contains words exclusive to ONE space, route there. If mixed, activate multiple.

## TEST FILE

Create `dictionaries/unified_test.md` with 40 questions:

### Group 1: CONTENT only (10 questions)
Reuse dict5_test.md questions Q01-Q10 (first 10). These MUST pass — they're the regression baseline.

### Group 2: MATH only (5 questions)
Reuse multispace_test Q01-Q05.

### Group 3: GRAMMAR only (5 questions)
Reuse multispace_test Q06-Q10.

### Group 4: TASK routing with CONTENT (5 questions)

```
Q21: Is "is a dog an animal" a content task?
A21: Yes

Q22: Is "what is two plus three" a content task?
A22: No

Q23: What kind of task is "is the sun hot"?
A23: a content task

Q24: What kind of task is "is dog a noun"?
A24: a word task

Q25: What kind of task is "how many animals"?
A25: a number task
```

### Group 5: Cross-space with CONTENT (10 questions)

```
Q26: Is dog a noun?
A26: Yes
Spaces: CONTENT + GRAMMAR (dog in CONTENT, noun in GRAMMAR)

Q27: Is eat a verb?
A27: Yes
Spaces: CONTENT + GRAMMAR (eat in CONTENT, verb in GRAMMAR)

Q28: Is hot a property?
A28: Yes
Spaces: CONTENT + GRAMMAR

Q29: Is the sun a thing?
A29: Yes
Spaces: CONTENT (direct, within single space)

Q30: Is a dog an animal and a thing?
A30: Yes
Spaces: CONTENT (boolean, within single space)

Q31: Can an animal eat? Write the answer as a sentence.
A31: an animal can eat
Spaces: CONTENT + GRAMMAR

Q32: Is three a noun?
A32: Yes
Spaces: MATH + GRAMMAR

Q33: Is the sun big? Is five big?
A33: Yes and Yes
Spaces: CONTENT + MATH (big is bridge, applies differently per space)

Q34: What is a dog?
A34: an animal
Spaces: CONTENT

Q35: Is ball a noun or a verb?
A35: a noun
Spaces: CONTENT + GRAMMAR
```

### Group 6: Full pipeline (5 questions)

```
Q36: Two plus three. The answer is a number. Is it big?
A36: No
Spaces: TASK → MATH + CONTENT (five is not big in content terms)

Q37: What is a cat? Is cat a noun?
A37: an animal. Yes.
Spaces: CONTENT + GRAMMAR (compound, multi-answer)

Q38: Can a person make a sound? Write a sentence.
A38: a person can make a sound
Spaces: CONTENT + GRAMMAR

Q39: Is eat an action and a verb?
A39: Yes
Spaces: CONTENT + GRAMMAR (action in both, verb in GRAMMAR)

Q40: One plus one. Is the result equal to two? Answer with yes or no.
A40: Yes
Spaces: TASK → MATH
```

## IMPLEMENTATION PLAN

### Phase A: Update dict_task5.md (≈30 min)

1. Add `content`, `animal`, `hot`, `cold` entries
2. Verify closure
3. Re-run TASK space alone → must still pass Q11-Q15 from multispace_test

### Phase B: Four-space smoke test (≈1 hour)

1. Run four spaces with `--spaces` flag
2. Verify bridge detection finds CONTENT bridges
3. Run multispace_test.md → should score ≥ 22/25 (no regression)
4. Run dict5_test.md in CONTENT space alone → must be 20/20

### Phase C: Create unified_test.md (≈30 min)

Write the 40-question test file.

### Phase D: Routing fixes (≈1-2 days)

The CONTENT space shares most of its vocabulary with bridge terms. The routing algorithm may over-activate CONTENT because its words appear everywhere.

Potential fix: **priority routing**. If a query word is EXCLUSIVE to one space, that space is PRIMARY. Bridge terms don't trigger additional spaces unless the primary space can't resolve the query alone.

```
"Is a dog an animal?" 
  → "dog" exclusive to CONTENT → route to CONTENT only
  → CONTENT resolves: Yes → done

"Is dog a noun?"
  → "noun" exclusive to GRAMMAR → route to GRAMMAR primary
  → "dog" is in CONTENT → activate CONTENT as secondary
  → cross-space chain: dog(CONTENT) → thing(bridge) → noun(GRAMMAR)
```

### Phase E: Score and iterate (≈1 day)

Run unified_test.md. Target scores. Fix failures. Repeat.

## SUCCESS CRITERIA

| Test Group | Minimum | Target |
|------------|---------|--------|
| CONTENT only (Q01-Q10) | 9/10 | 10/10 |
| MATH only (Q11-Q15) | 4/5 | 5/5 |
| GRAMMAR only (Q16-Q20) | 3/5 | 4/5 |
| TASK routing (Q21-Q25) | 3/5 | 4/5 |
| Cross-space (Q26-Q35) | 5/10 | 7/10 |
| Full pipeline (Q36-Q40) | 2/5 | 3/5 |
| **Total** | **26/40** | **33/40** |

### Regression (hard requirements)

| Test | Required |
|------|----------|
| dict5 single-space | 20/20 |
| dict12 single-space | 14/20 |
| multispace_test Phase 16 | ≥ 22/25 |

## CODE CHANGES SCOPE

| File | Change |
|------|--------|
| `dictionaries/dict_task5.md` | Add content domain terms |
| `dictionaries/unified_test.md` | NEW: 40-question cross-domain test |
| `crates/dafhne-engine/src/multispace.rs` | Priority routing, CONTENT-aware composition |
| `crates/dafhne-eval/src/main.rs` | No change expected (--spaces already works) |

**No changes to**: dafhne-core, dafhne-parser, dafhne-evolve, resolver.rs, equilibrium engine

## KILL CRITERIA

- CONTENT single-space regresses below 18/20 → dict5 broken by integration
- TASK routing worse than 2/5 on CONTENT tasks → routing algorithm can't handle 4 spaces
- Phase 16 multispace_test regresses below 20/25 → four-space breaks three-space

## THE BIGGER PICTURE

Phase 17 proves that the multi-space architecture scales from 3 to 4 spaces without regression. More importantly, it proves that existing DAPHNE capabilities (14 phases of dict5 work) integrate cleanly into the new framework.

After Phase 17, DAPHNE is a system that:
- Knows about the physical world (CONTENT)
- Can count and compute (MATH)
- Understands its own language structure (GRAMMAR)
- Routes automatically between these capabilities (TASK)

What's missing: knowing what IT is. That's Phase 18.
