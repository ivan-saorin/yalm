# PROMPT 01 — dict12 Closure Audit & Completion

## CONTEXT

You are working on a research project called DAFHNE (Definition-Anchored Force-field Heuristic Network Engine). The project explores whether a system can build internal geometric representations of knowledge from text alone — no predefined rules, no ontology, no parsing grammar.

The foundation is two closed dictionaries:
- **dict5**: 50 words at a 5-year-old comprehension level. Verified CLOSED.
- **dict12**: Same 50 core words upgraded to 12-year-old level, plus all additional words needed for closure. Currently NEAR-CLOSED (~400 entries, possibly incomplete).

## CLOSURE RULE

A dictionary is **closed** when every word appearing in any definition is itself defined in the dictionary. The only exceptions are:
- Basic inflections of defined words: -s, -es, -ed, -ing, -er, -est, -ly (e.g., "running" counts as "run")
- "an" is treated as an inflection of "a"
- "I" and "me" are treated as forms of "you" (speaker/listener symmetry)
- "she", "he", "her", "his", "him" are treated as pronoun forms of "person"
- "its" is treated as a form of "it"
- "their", "them", "they" are treated as plural forms of "it" or "person"
- Numbers written as words ("two", "three", "five", "ten", "sixty", "150", "365", "24") — treat basic numerals as primitives unless they appear prominently
- "myself", "yourself" — reflexive forms of defined pronouns

## YOUR TASK

### Phase 1: Extract all words from definitions

Read the attached `dict12.md` file. For EVERY entry, extract every unique word that appears in:
1. The definition text
2. The three usage examples

Build a complete set of all words used.

### Phase 2: Check against defined entries

For each word in the extracted set:
1. Check if it is a defined entry in the dictionary
2. Check if it is an allowed inflection of a defined entry
3. Check if it falls under the pronoun/number exceptions above
4. If NONE of the above → flag it as **LEAKED**

### Phase 3: Report

Produce a report with:
- Total unique words used across all definitions and examples
- Total defined entries
- Total leaked words (with the word and which entry's definition contains it)
- Leaked words grouped by likely category (nouns, verbs, adjectives, function words, etc.)

### Phase 4: Write definitions for leaked words

For each leaked word, write a definition following the dict12 format:
- Definition using ONLY words already in the dictionary (or other leaked words you're also defining)
- 3 usage examples
- Keep the 12-year-old comprehension level
- After writing all new definitions, RE-CHECK that no new leaked words were introduced
- If new leaks appear, define those too (iterate until closed)

### Phase 5: Produce the completed dict12

Output the complete, final `dict12_complete.md` with:
- All original entries (unchanged)
- All new entries added in a clearly marked section: `## CLOSURE ADDITIONS (Phase 5)`
- Updated closure status at the bottom with exact word counts
- The expansion ratio vs dict5 (final count / 50)

## FORMAT REQUIREMENTS

Each entry must follow this exact format:
```
**word** — definition text here.
- "usage example one"
- "usage example two"
- "usage example three"
```

## QUALITY CONSTRAINTS

- Do NOT simplify existing definitions to avoid closure issues. The point is to measure how many words a 12-year-old vocabulary ACTUALLY requires.
- Do NOT use circular definitions ("X is X").
- Every definition should be genuinely useful — a 12-year-old reading it should understand the word.
- Prefer shorter definitions when possible without losing meaning.
- If a leaked word is truly obscure or only appears in one example, consider whether the EXAMPLE should be rewritten instead of adding a new entry. Flag these cases.

## ATTACHED FILES

- `dict12.md` — the current near-closed dictionary
- `dict5.md` — the reference closed dictionary (for comparison)

## SUCCESS CRITERIA

- Zero leaked words
- Every definition comprehensible at 12-year-old level
- Final expansion ratio documented
- No circular definitions
