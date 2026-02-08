# PROMPT 12 — Boolean Operators: AND, OR, NOT

> **STATUS: PLACEHOLDER** — To be expanded after Prompt 11 (3W) is complete.

## GOAL

Extend the resolver to handle compound queries with boolean composition:

- **AND**: "Is a dog an animal AND a pet?" → decompose into two sub-queries, both must be Yes
- **OR**: "Is a cat a dog OR an animal?" → decompose into two sub-queries, either can be Yes
- **NOT** (explicit query negation): "Is a dog NOT an animal?" → invert the base query result

## PREREQUISITES

- Prompt 11 complete (3W handlers working, all identity questions route correctly)
- The resolver handles What/Who/Where and Yes/No reliably

## KEY DESIGN QUESTIONS

1. **Query-level vs geometric-level composition?**
   - Query-level: parse into sub-queries, resolve each independently, apply boolean logic to answers. Simpler, predictable.
   - Geometric-level: compute intersection/union of proximity regions. More elegant, harder to implement, may not add value.
   - Recommendation: query-level composition first.

2. **How to detect AND/OR/NOT in token stream?**
   - Tokenizer already produces "and", "or", "not" as tokens
   - Challenge: "not" already has a role in negation detection (`preceded_by_not`, `negated` flag in YesNo)
   - Need to distinguish "Is a dog NOT a cat?" (query negation) from "Is a dog not-big?" (property negation)

3. **New QuestionType variant or wrapper?**
   - Option A: `QuestionType::Compound { operator: BoolOp, parts: Vec<QuestionType> }`
   - Option B: Detect boolean at the top level, split into sub-queries, resolve each with existing types
   - Recommendation: Option B (less invasive)

4. **Truth table for IDK combinations:**
   - AND: Yes+Yes=Yes, Yes+No=No, Yes+IDK=IDK, No+anything=No, IDK+IDK=IDK
   - OR: Yes+anything=Yes, No+No=No, No+IDK=IDK, IDK+IDK=IDK
   - NOT: Yes→No, No→Yes, IDK→IDK

## SCOPE

- Detect AND/OR/NOT tokens in Yes/No questions
- Split compound questions into sub-queries
- Resolve each sub-query independently
- Combine results with boolean truth table
- Test with dict5 + dict12 + Three Men in a Boat questions
- No engine/equilibrium changes

## TEST EXAMPLES

```
Is a dog an animal and a pet? -> Yes (if both chains succeed) or IDK
Is a dog a cat or an animal? -> Yes (second sub-query succeeds)
Is a dog not an animal? -> No (base query is Yes, NOT inverts)
Is a dog an animal and a person? -> No (second sub-query fails)
Can a dog eat and move? -> Yes (both capabilities in definition)
```

## ESTIMATED EFFORT

- Resolver changes: ~100 lines (detection + decomposition + boolean combiner)
- Test file: ~15 questions
- No engine changes