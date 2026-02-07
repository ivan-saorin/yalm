# PROMPT 10 — DictVict: Three Men in a Boat

## PREAMBLE

YALM is a geometric comprehension engine that has progressed through nine phases:

1. **Prompts 01-06**: Built and refined the core engine on closed dictionaries (dict5: 20/20, dict12: 15/20)
2. **Prompt 07**: Scaled to dict18 (~2000 words), measured the three-point scaling curve
3. **Prompt 08**: Replaced the GA with sequential equilibrium — the text shapes the geometry with fixed parameters
4. **Prompt 09**: Added a dictionary cache (Simple English Wiktionary) so YALM can read ANY text by assembling definitions on demand

This prompt is the **integration test**. We feed YALM a real piece of literature — Jerome K. Jerome's *Three Men in a Boat* (1889) — and ask it questions about characters, events, and relationships. This is the first time the system encounters narrative text, named entities, humor, and Victorian prose.

*Three Men in a Boat* is ideal because:
- It's public domain (published 1889, available on Project Gutenberg)
- The vocabulary is rich but not impenetrable
- The characters are well-defined: **J.** (the narrator), **Harris**, **George**, and **Montmorency** (the dog)
- The plot is episodic — individual scenes are self-contained enough to test in isolation
- It contains both factual content (the Thames, Hampton Court) and subjective content (humor, opinions)

## PROJECT STRUCTURE

```
D:\workspace\projects\yalm\
├── crates/
│   ├── yalm-core/         Data structures, GeometricSpace, Answer, traits
│   ├── yalm-parser/        Dictionary/test/grammar parsing
│   ├── yalm-engine/        Force field + resolver + equilibrium
│   ├── yalm-eval/          Fitness scoring
│   ├── yalm-evolve/        Genetic algorithm (legacy)
│   └── yalm-cache/         Dictionary cache (Simple Wiktionary)
├── dictionaries/
│   ├── dict5.md, dict12.md, dict18.md
│   └── cache/simple-wiktionary/
├── texts/
│   ├── (test passages from prompt 09)
│   ├── three_men/                      NEW
│   │   ├── full_text.md                  Full book text
│   │   ├── chapter_01.md                 Chapter 1 only
│   │   ├── passage_montmorency.md        Selected passage about the dog
│   │   ├── passage_packing.md            The packing scene (Chapter 4)
│   │   ├── passage_hampton_court.md      Hampton Court maze (Chapter 6)
│   │   ├── chapter_01_test.md            Test questions for chapter 1
│   │   ├── passage_montmorency_test.md   Test questions for Montmorency passage
│   │   ├── passage_packing_test.md       Test questions for packing scene
│   │   ├── passage_hampton_test.md       Test questions for maze scene
│   │   └── full_test.md                  20 questions across the whole book
│   └── three_men_supplementary/        NEW
│       └── entities.md                   Character/place definitions
├── prompts/
└── RECAP.md
```

---

## THE CHALLENGE

### What's New About Narrative Text

All previous YALM inputs have been **definitional**: "a dog is an animal" explicitly states a relationship. Narrative text encodes relationships **implicitly**:

- "Montmorency sat up and looked around" → Montmorency can sit and look (capabilities)
- "Harris said he would be the one to carry the bag" → Harris is a person who can speak and carry things
- "We started from Kingston" → Kingston is a place, the journey started there

The geometry must infer structure from co-occurrence in sentences, not from explicit "X is a Y" patterns. This is harder. The question is HOW MUCH harder.

### What Might Work

- **Character clustering**: J., Harris, and George should cluster together (they co-occur constantly and do similar things: talk, eat, argue, travel). Montmorency should be nearby but distinct.
- **Place clustering**: Kingston, Hampton Court, Oxford, the Thames should cluster (all are places on the river).
- **Action proximity**: Characters who perform actions should be near those action words.

### What Probably Won't Work

- **Temporal reasoning**: "They left Kingston BEFORE arriving at Hampton Court" requires sequence, not proximity.
- **Irony and humor**: Jerome's humor is based on saying the opposite of what he means. The geometry will take everything literally.
- **Attribution**: "Harris said X" vs "George said X" — the geometry won't track who said what, only that both Harris and George are near "said".

---

## TASK 1: PREPARE THE TEXT

### Get the Full Text

Download *Three Men in a Boat* from Project Gutenberg: https://www.gutenberg.org/ebooks/308

Strip the Gutenberg header/footer. Save as `texts/three_men/full_text.md`.

### Extract Passages

Select 3-4 passages of varying length and type:

1. **Montmorency passage** (~300 words): A section that describes Montmorency's character. Chapter 1 or 13 have good material. This tests: can the geometry figure out Montmorency is a dog?

2. **Packing scene** (Chapter 4, ~500 words): The famous scene where they pack for the trip. Multiple characters, physical objects, actions. This tests: can the geometry cluster characters vs objects vs actions?

3. **Hampton Court maze** (Chapter 6, ~400 words): Harris gets lost in the maze. Concrete spatial narrative. This tests: can the geometry connect Harris to Hampton Court to "lost"?

4. **Chapter 1** (full, ~2000 words): The opening chapter where they decide to go on the trip. Tests: larger text, multiple topics, dialogue.

### Entity Definitions

The cache (Simple Wiktionary) won't know who Montmorency, Harris, George, or J. are. These are fictional characters. Create a supplementary entity file:

```markdown
# Entity Definitions for Three Men in a Boat

Montmorency: a dog. he is a fox terrier. he goes on the boat trip.
Harris: a person. he is a man. he is one of the three men on the trip.
George: a person. he is a man. he is one of the three men on the trip.
J: a person. he is a man. he tells the story. he is one of the three men on the trip.
Thames: a river. it is a big river in England.
Kingston: a place. it is a town on the Thames.
Hampton Court: a place. it is a big old building near the Thames.
```

These are written in YALM-style definitions (first content word = category). They're minimal — just enough for the geometry to know that Montmorency is a dog and Harris is a person. The TEXT should provide the rest of the signal.

**The assembler pipeline becomes:**

```
narrative text + entity definitions + wiktionary cache
  │
  └── Assembler → Dictionary → Equilibrium → GeometricSpace → Resolver
```

Entity definitions are injected into the assembled dictionary alongside cache definitions. They take priority (if "Montmorency" is in both entities and wiktionary, use the entity definition).

---

## TASK 2: WRITE TEST QUESTIONS

### Per-Passage Tests (5 questions each)

**Montmorency passage test:**
```
Is Montmorency a dog? -> Yes
Is Montmorency a person? -> No
Is Montmorency an animal? -> Yes   (transitive: dog → animal)
Can Montmorency fight? -> Yes/IDK  (depends on passage content)
What is Montmorency? -> dog
```

**Packing scene test:**
```
Is Harris a person? -> Yes
Is Harris a dog? -> No
Can Harris carry things? -> Yes/IDK
Is a bag an animal? -> No
What is Harris? -> person/man
```

**Hampton Court test:**
```
Is Hampton Court a place? -> Yes
Is Hampton Court a person? -> No
Is Harris a person? -> Yes
Is a maze a place? -> Yes/IDK
What is Hampton Court? -> place/building
```

### Full Book Test (20 questions)

Design 20 questions across the same categories as previous tests:

| Category | Count | Examples |
|----------|-------|---------|
| Character identification | 4 | "Is Montmorency a dog?", "Is Harris a person?" |
| Character negation | 4 | "Is Montmorency a person?", "Is George a dog?" |
| Transitive reasoning | 4 | "Is Montmorency an animal?", "Is Harris an animal?" (person → animal?) |
| Place identification | 2 | "Is the Thames a river?", "Is Kingston a place?" |
| What-Is | 4 | "What is Montmorency?", "What is Harris?", "What is the Thames?" |
| Honesty/Unknown | 2 | "What color is Montmorency?" (depends on definitions), "Is the Thames hot?" |

IMPORTANT: Every test answer must be derivable from the entity definitions + the text + cache definitions. Don't ask questions that require external knowledge ("In what year was the book published?").

---

## TASK 3: RUN THE PIPELINE

### Progressive Testing

Start small, scale up:

**Level 1: Entity definitions only (no narrative text)**

```bash
cargo run -p yalm-engine -- \
    --mode open \
    --text texts/three_men_supplementary/entities.md \
    --cache dictionaries/cache/simple-wiktionary/ \
    --test texts/three_men/full_test.md
```

This tells us what the entity definitions alone can answer. Baseline.

**Level 2: Montmorency passage + entities**

```bash
cargo run -p yalm-engine -- \
    --mode open \
    --text texts/three_men/passage_montmorency.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache dictionaries/cache/simple-wiktionary/ \
    --test texts/three_men/passage_montmorency_test.md
```

Does the narrative text ADD signal beyond the entity definitions? If the passage score > entities-only score, the geometry is extracting information from narrative.

**Level 3: Chapter 1 + entities**

Larger text, more signal, more noise. Does fitness improve or degrade?

**Level 4: Full book + entities**

The big test. ~60,000 words of Victorian prose. The assembler will pull thousands of definitions from the cache. The equilibrium will build a massive geometric space. This will be slow. That's fine — we're measuring quality, not speed.

### What to Record

For each level, record:

1. **Assembly stats**: words extracted, cache hits, not found, closure ratio, total dictionary size
2. **Equilibrium stats**: passes, final energy, convergence time
3. **Fitness**: per-question pass/fail, overall score
4. **Geometric diagnostics**:
   - Distance between Montmorency and "dog" (should be small)
   - Distance between Montmorency and "person" (should be large)
   - Distance between Harris and George (should be small — they're similar characters)
   - Distance between Harris and Montmorency (should be moderate — same trip, different species)
   - Distance between Thames and "river" (should be small)

These diagnostics tell us whether the geometry is encoding the right structure, even if the resolver gets some questions wrong.

---

## TASK 4: ANALYZE

### The Montmorency Question

"Who is Montmorency?" — or in YALM terms, "What is Montmorency?"

The system should answer: **dog**.

But here's what makes it interesting. Montmorency is described throughout the book in very human terms. He has opinions, he picks fights, he has a disreputable character. The geometric space will see Montmorency co-occurring with human actions ("said", "thought", "wanted"). The entity definition says "dog", but the text signal says "person-like".

If the geometry places Montmorency closer to "person" than to "dog" despite the entity definition, that's a fascinating failure — the system is reading the NARRATIVE characterization, not just the definition. Document this regardless of whether it's a pass or fail.

### The Signal-to-Noise Question

At each level (passage → chapter → full book), the assembled dictionary gets larger. More text means more definitions from the cache means more noise. Does fitness:

- **Improve monotonically**: More text = more signal. The geometry gets richer.
- **Peak then decline**: There's a sweet spot. Too much text adds noise faster than signal.
- **Stay flat**: The entity definitions dominate. Narrative text doesn't add much.

This curve answers a key question: **does geometric comprehension benefit from more text?** If yes, the architecture scales. If no, the system only works with concentrated definitional input.

### The Victorian Vocabulary Question

Jerome uses words like "sculling", "lock" (river lock), "punt", "weir". These have specific Victorian/nautical meanings that Simple Wiktionary may not capture well. Track which words are:
- Found in cache with correct sense
- Found in cache with WRONG sense ("lock" = door lock, not river lock)
- Not found at all

This directly feeds into the future word-sense disambiguation work.

---

## WHAT NOT TO DO

- Do NOT modify the engine, equilibrium, resolver, or cache code from previous prompts. This phase is TESTING the pipeline end-to-end.
- Do NOT hand-tune anything for Three Men in a Boat specifically. Same parameters, same cache, same equilibrium settings as prompt 09.
- Do NOT write test questions that require temporal reasoning, irony detection, or attribution. Test what geometry CAN do: identity, category, proximity, negation.
- Do NOT include entity definitions as part of the test score. They're input, not output. The test measures what the system INFERS, not what it was told.
- Do NOT give up if the full book scores poorly. The per-passage results and geometric diagnostics are the real data.

## SUCCESS CRITERIA

| Metric | Minimum | Target | Stretch |
|--------|---------|--------|---------|
| Entities-only fitness | > 0.40 | > 0.60 | > 0.80 |
| Passage fitness (avg across 3) | > 0.30 | > 0.50 | > 0.70 |
| Chapter 1 fitness | > 0.25 | > 0.45 | > 0.65 |
| Full book fitness | > 0.20 | > 0.40 | > 0.60 |
| "What is Montmorency?" | dog | dog | dog |
| Assembly closure (full book) | > 70% | > 80% | > 90% |
| Montmorency-dog distance < Montmorency-person distance | Yes | Yes | Yes |

Note: these targets are deliberately lower than dict5/12/18. This is the hardest test the system has ever faced. A fitness of 0.40 on real Victorian literature with zero hand-tuning would be a remarkable result.

## OUTPUT

1. Text files: full book + extracted passages in `texts/three_men/`
2. Entity definitions in `texts/three_men_supplementary/`
3. Test files for each passage + full book
4. Assembly reports at each level
5. Fitness tables: entities-only vs passage vs chapter vs full book
6. Geometric diagnostics: key distances between characters, categories, places
7. Montmorency analysis: where does the geometry place him and why?
8. Signal-vs-noise curve: fitness as a function of input text size
9. Victorian vocabulary audit: cache hits, misses, wrong senses
10. Updated RECAP.md with findings