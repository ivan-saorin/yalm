# PROMPT 18 — SELF Space: Identity and Capabilities (STUB)

> **STATUS: STUB** — To be expanded after Phase 17 is complete.

## GOAL

Create the SELF space: a dictionary that teaches YALM what it is, what it can do, and what it cannot do. This is the system prompt equivalent in geometric form.

## CORE IDEA

`dict_self5.md` defines YALM's identity in ELI5:

```
yalm: a thing that reads words and learns. it is not a person. it is not an animal.
yalm can: read, learn, count, answer, tell.
yalm cannot: see, feel, move, eat. it has no eyes, no body.
learn: to read a thing and know it after.
know: to have a thing in you. yalm knows words. yalm knows numbers.
answer: to give words to a question. yalm answers questions.
mistake: an answer that is not good. yalm can make mistakes.
certain: you know a thing is yes. not certain is you do not know.
```

The SELF space enables:
- "What are you?" → "a thing that reads words and learns"
- "Can you see?" → "No" (with geometric evidence: yalm is far from see)
- "Do you know what a dog is?" → Yes (checks CONTENT space, SELF knows it has knowledge)
- "Are you certain?" → depends on distance confidence in the answering space

## KEY CHALLENGES

1. **Meta-knowledge**: SELF must know about the OTHER spaces. "I can count" requires knowing MATH space exists. This is a cross-space dependency that doesn't exist in Phase 16/17.
2. **Calibrated uncertainty**: "I don't know" is already honest in YALM. SELF formalizes this.
3. **Capability boundaries**: SELF must accurately model what YALM can/cannot do, which changes as spaces are added.

## PREREQUISITES

- Phase 17 complete (four spaces integrated)
- Clear understanding of which queries each space handles

## ESTIMATED EFFORT

- Dictionary: 1 day (small, ~20-30 terms)
- Integration: 1-2 days (TASK routing update, meta-queries)
- Testing: 1 day

## SUCCESS CRITERIA (tentative)

- Identity questions: 5/5 ("What are you?", "Can you see?", etc.)
- Capability questions: 4/5 ("Can you count?", "Can you read?", etc.)
- Uncertainty: 3/5 ("Are you certain?", "Do you know X?")
- No regression on Phase 17 scores
