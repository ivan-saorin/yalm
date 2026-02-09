# multispace_test — Cross-Space Query Tests

> Tests for Phase 16 multi-space architecture.
> Each query specifies expected activated spaces and answer.

---

## SINGLE-SPACE: MATH ONLY

**Q01**: What is two plus three?
**A**: five
Spaces: MATH

**Q02**: Is five a number?
**A**: Yes
Spaces: MATH

**Q03**: Is three more than one?
**A**: Yes
Spaces: MATH

**Q04**: What is five minus two?
**A**: three
Spaces: MATH

**Q05**: What comes after four?
**A**: five
Spaces: MATH

---

## SINGLE-SPACE: GRAMMAR ONLY

**Q06**: Is dog a noun?
**A**: Yes
Spaces: GRAMMAR

**Q07**: Is eat a verb?
**A**: Yes
Spaces: GRAMMAR

**Q08**: What is a sentence?
**A**: words in order that tell a thing
Spaces: GRAMMAR

**Q09**: Is big a property?
**A**: Yes
Spaces: GRAMMAR

**Q10**: What is a subject?
**A**: the thing in a sentence that does the action
Spaces: GRAMMAR

---

## SINGLE-SPACE: TASK ROUTING

**Q11**: Is "what is two plus three" a number task?
**A**: Yes
Spaces: TASK

**Q12**: Is "write a sentence" a word task?
**A**: Yes
Spaces: TASK

**Q13**: Is "what is a noun" a number task?
**A**: No
Spaces: TASK

**Q14**: What kind of task is count?
**A**: a number task
Spaces: TASK

**Q15**: What kind of task is write?
**A**: a word task
Spaces: TASK

---

## CROSS-SPACE: MATH + GRAMMAR

**Q16**: Is five a noun?
**A**: Yes
Spaces: MATH + GRAMMAR
Note: five is a number (MATH), number is a thing (MATH), noun is a name for a thing (GRAMMAR) → five is a noun

**Q17**: Is plus a verb?
**A**: Yes
Spaces: MATH + GRAMMAR
Note: plus is an action in MATH (put together), verb tells action (GRAMMAR)

**Q18**: Is the result of two plus three a number?
**A**: Yes
Spaces: MATH
Note: result and number both in MATH, but tests compound query

**Q19**: How many words are in "one plus two"?
**A**: three
Spaces: MATH + GRAMMAR
Note: count words (GRAMMAR) → get number (MATH). Hardest cross-space query.

**Q20**: Is three a noun or a verb?
**A**: a noun
Spaces: MATH + GRAMMAR
Note: three is a number (MATH), number is a thing name (bridge), noun is name for thing (GRAMMAR)

---

## CROSS-SPACE: TASK-ROUTED

**Q21**: Two plus three. Write the answer as a sentence.
**A**: two plus three is five
Spaces: TASK → MATH + GRAMMAR
Note: TASK routes to MATH (plus) and GRAMMAR (sentence). This is the key integration test.

**Q22**: Count to five. Is count a verb?
**A**: Yes
Spaces: TASK → MATH + GRAMMAR
Note: count is in MATH (say numbers in order) and GRAMMAR (verb tells action)

**Q23**: What is the subject in "the dog eats"?
**A**: dog
Spaces: TASK → GRAMMAR
Note: Pure grammar task, TASK routes correctly

**Q24**: Is minus the same as plus?
**A**: No
Spaces: TASK → MATH
Note: both in MATH space, TASK identifies number domain

**Q25**: What kind of task is "is dog a noun"?
**A**: a word task
Spaces: TASK
Note: TASK recognizes grammar/word domain from "noun"

---

## SCORING

Total: 25 questions
- Single-space MATH: 5 (baseline — must work, validates dict_math5)
- Single-space GRAMMAR: 5 (baseline — must work, validates dict_grammar5)
- Single-space TASK: 5 (baseline — validates routing)
- Cross-space: 5 (the real test)
- Task-routed cross-space: 5 (full architecture test)

### Success Criteria

| Test Group | Minimum | Target | Stretch |
|------------|---------|--------|---------|
| MATH only | 4/5 | 5/5 | 5/5 |
| GRAMMAR only | 4/5 | 5/5 | 5/5 |
| TASK routing | 3/5 | 4/5 | 5/5 |
| Cross-space | 2/5 | 3/5 | 5/5 |
| Task-routed | 1/5 | 2/5 | 4/5 |
| **Total** | **14/25** | **19/25** | **24/25** |
