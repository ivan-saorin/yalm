# PROMPT 10b — Granularity Probe: What Does the Geometry Actually See?

## PREAMBLE

Phase 10 proved DAFHNE can comprehend Victorian literature at 0.87 fitness (16/21). But the 21-question test mostly asks Level 2-3 questions ("Is Montmorency a dog?", "Is Harris a person?"). We know the system gets entity-type classification right. We DON'T know where it breaks down.

This prompt expands the test suite from 21 to ~50 questions organized along a **coarse-to-fine gradient**. The goal is NOT to improve the score — it's to map the comprehension boundary. At what level of detail does geometric comprehension degrade from "knows" to "guesses" to "can't"?

## THE GRANULARITY LADDER

Questions are organized into 6 levels, from broadest categories to finest distinctions:

### Level 1 — Ontological ("Is X a thing?")
The most basic existence/category boundaries. Can the geometry distinguish something from nothing, alive from not-alive?

```
Is Montmorency a thing?        -> Yes  (dog -> animal -> thing)
Is Harris a thing?              -> Yes  (person -> thing)
Is the Thames a thing?          -> Yes  (river -> thing)
Is Kingston a thing?            -> Yes  (place -> thing)
Is Montmorency alive?           -> Yes  (dog -> animal -> alive)
Is the Thames alive?            -> No   (river is not alive)
Is Harris alive?                -> Yes  (person -> animal -> alive)
Is a dog a thing?               -> Yes  (direct from definition)
```

These require 2-3 hop transitive chains. If Level 1 fails, the geometry has no deep taxonomy.

### Level 2 — Kingdom ("Is X a person/animal/place/object?")
Entity-type classification. Phase 10 already tests this — we're filling gaps.

```
Is Montmorency a place?         -> No
Is the Thames an animal?        -> No
Is Kingston a person?           -> No
Is Harris a place?              -> No
Is Hampton a person?            -> No
Is George an animal?            -> Yes  (person -> animal — 2-hop, known failure)
```

These are cross-category negations. The geometry should place entities far from wrong categories.

### Level 3 — Species/Type ("Is X a dog/man/river/town?")
Direct identity from entity definitions. Phase 10 tests this well.

```
Is Montmorency a fox terrier?   -> Yes  (entity def: "a small fox terrier")
Is Harris a man?                -> Yes  (entity def: "he is a man")
Is George a man?                -> Yes  (entity def: "he is a man")
Is Kingston a town?             -> Yes  (entity def: "a town on the thames")
Is the Thames a big river?      -> Yes  (entity def: "a big river")
```

These test whether the entity definitions propagate beyond the first content word.

### Level 4 — Properties & Capabilities ("Can X do Y?", "Is X [adjective]?")
Attributes that come from definitions AND from narrative co-occurrence. This is where it gets interesting — do the Ollama-generated definitions for common words carry enough signal?

```
Can a dog eat?                  -> Yes  (ollama def of dog should mention eating)
Can a dog move?                 -> Yes  (ollama def of dog should mention movement)
Can a person eat?               -> Yes  (ollama def of person)
Can a river move?               -> Yes  (rivers flow / move)
Is a dog big?                   -> I don't know  (dogs vary in size)
Is the Thames big?              -> Yes  (entity def: "a big river")
Is a town a place?              -> Yes  (ollama def of town)
Is a river a place?             -> I don't know  (rivers are things, debatably places)
```

These test the Ollama definition quality. The ELI5 definitions should include basic capabilities ("can eat", "can move") but may not include all properties.

### Level 5 — Relational ("Is X near/part of/with Y?")
Relationships between specific entities. These require reading BOTH entity definitions and finding connections.

```
Is Kingston on the Thames?      -> Yes  (entity def: "a town on the thames")
Is Hampton near the Thames?     -> Yes  (entity def: "near the thames")
Is Harris on the boat?          -> Yes  (entity def: "on the boat trip")
Is George on the boat?          -> Yes  (entity def: "on the boat trip")
Is Montmorency on the boat?     -> Yes  (entity def: "on the boat trip")
Is Kingston near Hampton?       -> I don't know  (not stated in definitions)
```

The resolver currently doesn't handle relational queries ("on", "near", "with"). These will likely all return IDK or get misrouted to Yes/No category checks. That's the EXPECTED result — documenting the boundary.

### Level 6 — Narrative Characterization ("Is Montmorency friendly?")
Fine-grained attributes that only exist in the narrative text, not in entity definitions. This tests whether the geometry extracts characterization from Victorian prose.

```
Is Montmorency small?           -> Yes  (entity def: "a small fox terrier")
Is Harris a friend?             -> I don't know  (narrative implies it but not in defs)
Is the Thames long?             -> I don't know  (not stated)
Can Montmorency fight?          -> I don't know  (narrative yes, but depends on def)
Is a maze a place?              -> Yes  (ollama def should classify maze as place)
Is Hampton old?                 -> Yes  (entity def: "a big old building")
```

These probe whether narrative co-occurrence creates geometric proximity for attributes not in definitions.

---

## TASK 1: CREATE THE 50-QUESTION TEST FILE

Create `texts/three_men/granularity_test.md` with ~50 questions organized by level.

Use the standard DAFHNE test format:

```markdown
# granularity_test — Coarse-to-Fine Comprehension Probe

## LEVEL 1 — ONTOLOGICAL

---

**Q01**: Is Montmorency a thing?
**A**: Yes
**Chain**: montmorency -> dog -> animal -> thing (3-hop transitive)

---

(etc.)

## LEVEL 2 — KINGDOM

(etc.)
```

The questions listed above in The Granularity Ladder are a starting point. Expand to fill ~50 total, distributed roughly as:

| Level | Questions | Purpose |
|-------|-----------|---------|
| 1 Ontological | 8 | Deepest transitive chains |
| 2 Kingdom | 6 | Cross-category negation (filling gaps from full_test) |
| 3 Species/Type | 6 | Entity sub-type identification |
| 4 Properties | 10 | Ollama definition quality + capability chains |
| 5 Relational | 10 | Entity-to-entity relationships |
| 6 Narrative | 10 | Fine-grained characterization from text |

### Question Design Rules

1. Every answer must be derivable from entity definitions + Ollama definitions + passage text. No external knowledge.
2. Mark expected answers conservatively. If uncertain whether the chain reaches, use `I don't know`.
3. For Level 4-6, check Ollama's cached definitions first. If the Ollama definition of "dog" says "an animal that can bark", then "Can a dog bark?" is valid. If it says "an animal. it can move fast.", then "bark" is NOT in scope.
4. Level 5-6 questions should include questions you EXPECT to fail. The purpose is mapping the boundary, not maximizing score.
5. Each question should have a clear level assignment. Don't mix levels.

### Checking Ollama Definitions

Before writing Level 4-6 questions, inspect the cached definitions for key words:

```bash
# Check what Ollama says about "dog", "person", "river", "town", "maze", etc.
# The cache is in dictionaries/cache/ollama-qwen3/*.json (per-letter files)
# Use jq or grep to extract specific definitions:
cd D:\workspace\projects\dafhne
cat dictionaries/cache/ollama-qwen3/d.json | python -c "import sys,json; d=json.load(sys.stdin); print(d.get('dog',{}).get('definitions',['not found']))"
cat dictionaries/cache/ollama-qwen3/p.json | python -c "import sys,json; d=json.load(sys.stdin); print(d.get('person',{}).get('definitions',['not found']))"
cat dictionaries/cache/ollama-qwen3/r.json | python -c "import sys,json; d=json.load(sys.stdin); print(d.get('river',{}).get('definitions',['not found']))"
cat dictionaries/cache/ollama-qwen3/t.json | python -c "import sys,json; d=json.load(sys.stdin); print(d.get('town',{}).get('definitions',['not found']))"
cat dictionaries/cache/ollama-qwen3/m.json | python -c "import sys,json; d=json.load(sys.stdin); print(d.get('maze',{}).get('definitions',['not found']))"
cat dictionaries/cache/ollama-qwen3/a.json | python -c "import sys,json; d=json.load(sys.stdin); print(d.get('alive',{}).get('definitions',['not found']))"
cat dictionaries/cache/ollama-qwen3/t.json | python -c "import sys,json; d=json.load(sys.stdin); print(d.get('thing',{}).get('definitions',['not found']))"
```

Use the actual cached definitions to write accurate Level 4+ questions. If "dog" is defined as "an animal. it can move fast." then "Can a dog move?" → Yes, but "Can a dog bark?" → IDK (not in definition).

Some words may not be cached yet (they'll be generated on first run). For these, write the question but mark the expected answer as `I don't know` to be safe. After the first run, check the generated definitions and update expected answers if needed.

---

## TASK 2: RUN THE PROBE

### Combined text + entities (same as Phase 10 Level 6)

```bash
cargo run -p dafhne-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/granularity_test.md \
    --mode equilibrium
```

### Record per-level results

Group the results by level and compute per-level fitness:

| Level | Questions | Correct | Fitness | Interpretation |
|-------|-----------|---------|---------|----------------|
| 1 Ontological | 8 | ?/8 | ? | Deep taxonomy |
| 2 Kingdom | 6 | ?/6 | ? | Entity-type |
| 3 Species/Type | 6 | ?/6 | ? | Sub-type |
| 4 Properties | 10 | ?/10 | ? | Capabilities |
| 5 Relational | 10 | ?/10 | ? | Entity-entity |
| 6 Narrative | 10 | ?/10 | ? | Characterization |
| **Total** | **~50** | **?/50** | **?** | |

This table IS the deliverable. It maps the comprehension gradient.

---

## TASK 3: ANALYZE THE GRADIENT

### Expected Pattern

The most likely outcome is a **monotonic decline** from Level 1 to Level 6:

```
Level 1 (Ontological):  ~50-75%  — depends on chain depth reaching "thing"/"alive"
Level 2 (Kingdom):      ~80-90%  — already proven in Phase 10
Level 3 (Species/Type): ~80-90%  — direct entity def extraction
Level 4 (Properties):   ~40-60%  — depends on Ollama definition richness
Level 5 (Relational):   ~10-30%  — resolver doesn't handle relational queries
Level 6 (Narrative):    ~20-40%  — some co-occurrence signal, mostly IDK
```

But the ACTUAL pattern might surprise us:
- Level 1 might be LOWER than Level 2 if transitive chains don't reach "thing"
- Level 4 might be HIGHER than expected if Ollama definitions are rich
- Level 6 might show unexpected passes if narrative co-occurrence creates real signal

### What Each Level Tells Us

**Level 1 failure** = the geometry has no deep taxonomy. Chain traversal stops at 1-2 hops. The system knows what things ARE but not what categories things BELONG TO at the ontological level.

**Level 2-3 success** = entity injection works. The system correctly classifies entities into their defined types. This is a given from Phase 10.

**Level 4 breakpoint** = this reveals whether ELI5 definitions carry enough signal for property/capability reasoning. If "Can a dog eat?" passes, the Ollama definitions include capabilities. If it fails, the definitions are too sparse.

**Level 5 failure** = relational queries are outside the resolver's current capability. Expected and informative — this is the NEXT architectural gap after 3W.

**Level 6 pattern** = reveals whether the equilibrium encodes narrative signal. If "Is Montmorency small?" passes (entity def says "small") but "Is Harris a friend?" fails (narrative only), then definitions dominate over text co-occurrence. If some narrative-only properties pass, the geometry is extracting characterization.

### The Gradient Shape

Plot fitness vs level. The SHAPE of the curve matters more than the absolute values:

- **Cliff at Level 1**: Deep taxonomy doesn't work. Chain hops are too shallow.
- **Cliff at Level 4**: Properties/capabilities not encoded. Definitions too sparse.
- **Cliff at Level 5**: Relational reasoning absent. Expected gap.
- **Gradual decline**: The geometry degrades smoothly. Architecture has potential at all levels.
- **Non-monotonic** (Level 1 < Level 3 > Level 4): Interesting — some intermediate levels work better than extremes.

---

## TASK 4: SECOND RUN — ENTITIES ONLY

Run the granularity test WITHOUT narrative text, using only entity definitions:

```bash
cargo run -p dafhne-eval -- \
    --entities texts/three_men_supplementary/entities.md \
    --test texts/three_men/granularity_test.md \
    --mode equilibrium
```

Compare per-level fitness with vs without narrative text:

| Level | With Text | Entities Only | Delta |
|-------|-----------|---------------|-------|
| 1 | ? | ? | ? |
| 2 | ? | ? | ? |
| 3 | ? | ? | ? |
| 4 | ? | ? | ? |
| 5 | ? | ? | ? |
| 6 | ? | ? | ? |

The delta column shows exactly which levels benefit from narrative text. If Level 4-6 show positive delta, the text adds real signal at those granularity levels.

---

## TASK 5: EXAMINE FAILING QUESTIONS

For every question that fails, classify the failure mode:

| Failure Mode | Description | Example |
|-------------|-------------|----------|
| **Chain too short** | Transitive chain doesn't reach target in max_hops | montmorency→dog→animal→thing (3 hops, limit is 2) |
| **Wrong definition** | Ollama definition doesn't contain expected word | "person" defined as "human being" not "animal" |
| **Resolver routing** | Question type detection sends to wrong resolver | "Can X do Y?" parsed as YesNo instead of capability |
| **No connector** | Question pattern doesn't match any discovered connector | "Is X near Y?" — "near" not a connector |
| **IDK zone** | Distance falls between yes_threshold and no_threshold | Geometric proximity is ambiguous |
| **False positive** | System says Yes but answer should be No or IDK | Spurious proximity through shared definition words |

Create a failure table grouped by mode. The distribution of failure modes tells us what to fix:
- Mostly "chain too short" → increase max_hops or enrich definitions
- Mostly "wrong definition" → improve Ollama style prompt
- Mostly "resolver routing" → extend resolver (Prompt 11+)
- Mostly "no connector" → relational connectors needed (new connector types)

---

## WHAT NOT TO DO

- Do NOT modify any engine code. This is a measurement prompt.
- Do NOT tune parameters to improve scores. Same defaults as Phase 10.
- Do NOT add new resolver capabilities (that's Prompt 11+).
- Do NOT fix failing questions. Document them.
- Do NOT read the full book text into context. Use existing `combined.md` and `entities.md`.

## SUCCESS CRITERIA

This prompt has no fitness target. Success is a complete granularity map:

| Deliverable | Required |
|------------|----------|
| `granularity_test.md` with ~50 questions | Yes |
| Per-level fitness table (with text) | Yes |
| Per-level fitness table (entities only) | Yes |
| Delta table (text contribution per level) | Yes |
| Failure mode classification table | Yes |
| Gradient shape analysis | Yes |
| Regression: full_test.md still 16/21 | Yes |

## OUTPUT CHECKLIST

1. ☐ `texts/three_men/granularity_test.md` created (~50 questions, 6 levels)
2. ☐ Ollama definitions inspected for Level 4-6 question design
3. ☐ Combined run: per-level fitness table
4. ☐ Entities-only run: per-level fitness table
5. ☐ Delta analysis: which levels benefit from narrative text
6. ☐ Failure mode classification for every wrong answer
7. ☐ Gradient shape analysis (where does comprehension break down?)
8. ☐ Regression check: full_test.md unchanged at 16/21
9. ☐ RECAP.md updated with granularity findings