# PROMPT 10 — DictVict: Three Men in a Boat

## PREAMBLE

YALM is a geometric comprehension engine that has progressed through nine phases:

1. **Prompts 01-06**: Built and refined the core engine on closed dictionaries (dict5: 20/20, dict12: 15/20)
2. **Prompt 07**: Scaled to dict18 (~2000 words), measured the three-point scaling curve
3. **Prompt 08**: Replaced the GA with sequential equilibrium — the text shapes the geometry with fixed parameters
4. **Prompt 09**: Added a dictionary cache with three backends:
   - `ManualFileCache` — parses dict5-style `.md` files
   - `WiktionaryCache` — Simple English Wiktionary SQLite dump
   - `OllamaCache` — local LLM (Qwen3:8b) generates definitions on demand with 3-tier memoization (memory → disk JSON → API call)

Phase 09c added a **uniformity filter** to connector discovery (structural words pass, content words rejected), reducing connector noise at scale while maintaining dict5 at 20/20.

This prompt is the **integration test**. We feed YALM a real piece of literature — Jerome K. Jerome's *Three Men in a Boat* (1889) — and ask it questions about characters, events, and relationships. This is the first time the system encounters narrative text, named entities, humor, and Victorian prose.

*Three Men in a Boat* is ideal because:
- It's public domain (published 1889)
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
│   ├── yalm-eval/          Fitness scoring + CLI binary (cargo run -p yalm-eval)
│   ├── yalm-evolve/        Genetic algorithm (legacy)
│   ├── yalm-cache/         Dictionary cache (ManualFileCache, WiktionaryCache, OllamaCache)
│   └── yalm-wikt-build/    Wiktionary dump builder (legacy)
├── data/
│   └── Three-Men-in-a-Boat.txt    ← Full book text (already present)
├── dictionaries/
│   ├── dict5.md, dict12.md, dict18.md
│   └── cache/
│       ├── simple-wiktionary/
│       └── ollama-qwen3/           ← ~805 cached definitions from phase 09
├── texts/
│   ├── passage1.md, passage1_test.md
│   ├── three_men/                  ← NEW (create this)
│   │   ├── chapter_01.md
│   │   ├── passage_montmorency.md
│   │   ├── passage_packing.md
│   │   ├── passage_hampton_court.md
│   │   ├── chapter_01_test.md
│   │   ├── passage_montmorency_test.md
│   │   ├── passage_packing_test.md
│   │   ├── passage_hampton_test.md
│   │   └── full_test.md            ← 20 questions across whole book
│   └── three_men_supplementary/    ← NEW (create this)
│       └── entities.md             ← Character/place definitions
└── prompts/
    └── RECAP.md
```

---

## IMPORTANT: CONSTRAINTS

- The CLI binary is `yalm-eval`, not `yalm-engine`. All commands use `cargo run -p yalm-eval --`.
- We use **OllamaCache** (`--cache-type ollama`) as the dictionary backend. The LLM generates definitions for ANY word, so there are no "not found" entries. Every seed word gets a definition.
- The Ollama disk cache is at `dictionaries/cache/ollama-qwen3/`. Second runs are instant (100% disk hit rate).
- Ollama must be running locally: `ollama serve` with model `qwen3:8b` pulled.
- The full book text is already at `D:\workspace\projects\yalm\data\Three-Men-in-a-Boat.txt`. Do NOT download it again. Strip the Project Gutenberg header/footer when extracting passages.
- Do NOT read the full book text into your context. Extract passages using targeted line-range reads or grep.

---

## THE CHALLENGE

### What's New About Narrative Text

All previous YALM inputs have been **definitional**: "a dog is an animal" explicitly states a relationship. Narrative text encodes relationships **implicitly**:

- "Montmorency sat up and looked around" → Montmorency can sit and look (capabilities)
- "Harris said he would be the one to carry the bag" → Harris is a person who can speak and carry things
- "We started from Kingston" → Kingston is a place, the journey started there

The geometry must infer structure from co-occurrence in sentences, not from explicit "X is a Y" patterns. This is harder. The question is HOW MUCH harder.

### What Might Work

- **Character clustering**: J., Harris, and George should cluster together (they co-occur constantly and do similar things). Montmorency should be nearby but distinct.
- **Place clustering**: Kingston, Hampton Court, Oxford, the Thames should cluster.
- **Action proximity**: Characters who perform actions should be near those action words.

### What Probably Won't Work

- **Temporal reasoning**: "They left Kingston BEFORE arriving at Hampton Court" — geometry can't do sequence.
- **Irony and humor**: Jerome says the opposite of what he means. Geometry takes everything literally.
- **Attribution**: "Harris said X" vs "George said X" — geometry won't track who said what.

---

## TASK 0: IMPLEMENT `--entities` FLAG

The OllamaCache can generate definitions for common English words, but it doesn't know who Montmorency, Harris, George, or J. are — these are fictional characters. We need a way to inject hand-written entity definitions alongside the assembled dictionary.

### What to Build

Add an `--entities` CLI argument to `yalm-eval` that accepts a path to a dictionary-format `.md` file containing character/place definitions. These entity entries are **merged into the assembled dictionary** after assembly, overriding any cache-generated definitions for the same words.

### Implementation Details

**1. CLI change** (`crates/yalm-eval/src/main.rs`):

```rust
/// Path to entity definitions file (merged into assembled dictionary, overrides cache)
#[arg(long)]
entities: Option<PathBuf>,
```

**2. Entity file format** — standard dictionary `.md` format (same as dict5.md), parsed by the existing `parse_dictionary()` function:

```markdown
# entities — Three Men in a Boat

## CHARACTERS

**montmorency** — a dog. he is a fox terrier. he goes on the boat trip.
- "montmorency is a dog"
- "montmorency is a fox terrier"
- "montmorency goes on the trip"

**harris** — a person. he is a man. he is one of the three men on the trip.
- "harris is a person"
- "harris is a man"
- "harris is one of the three men"

**george** — a person. he is a man. he is one of the three men on the trip.
- "george is a person"
- "george is a man"
- "george is one of the three men"

## PLACES

**thames** — a river. it is a big river in england.
- "the thames is a river"
- "the thames is in england"
- "boats go on the thames"

**kingston** — a place. it is a town on the thames.
- "kingston is a place"
- "kingston is a town"
- "kingston is on the thames"

**hampton court** — a place. it is a big old building near the thames. it has a maze.
- "hampton court is a place"
- "hampton court has a maze"
- "hampton court is near the thames"
```

Note: `J.` is deliberately EXCLUDED from entities. The narrator uses "I" throughout the book, and single-letter tokens get stripped by the tokenizer. The character "J." exists in the book metadata but contributes negligible textual signal. Testing J.-related questions would measure entity injection, not comprehension.

**3. Merge logic** (in `main.rs`, after assembly):

```rust
if let Some(entities_path) = &cli.entities {
    let entities_content = std::fs::read_to_string(entities_path)
        .expect("Failed to read entities file");
    let entities_dict = parse_dictionary(&entities_content);
    
    println!("[Entities: {} entries from {:?}]", entities_dict.entries.len(), entities_path);
    
    // Merge: entity entries override assembled entries for same word.
    // Entity definition words should also be chased through the cache,
    // but since OllamaCache generates definitions for everything, 
    // the BFS already covers common words like "dog", "person", "river".
    let mut entry_map: std::collections::HashMap<String, DictionaryEntry> = 
        dictionary.entries.into_iter().map(|e| (e.word.clone(), e)).collect();
    
    for entity_entry in entities_dict.entries {
        entry_map.insert(entity_entry.word.clone(), entity_entry);
    }
    
    let mut entries: Vec<DictionaryEntry> = entry_map.into_values().collect();
    entries.sort_by(|a, b| a.word.cmp(&b.word));
    let entry_words: Vec<String> = entries.iter().map(|e| e.word.clone()).collect();
    let entry_set: std::collections::HashSet<String> = entry_words.iter().cloned().collect();
    
    dictionary = Dictionary { entries, entry_words, entry_set };
    println!("Dictionary after entity merge: {} entries", dictionary.entries.len());
}
```

**4. Entity names as seed words** — There's a subtlety: the assembler extracts seed words from `--text` only. Entity names like "montmorency" won't appear as seed words unless they appear in the narrative text (which they do). However, the entity DEFINITION words ("fox", "terrier", "england") need to be in the dictionary too. Since we're using OllamaCache, the BFS closure chase from the narrative text will likely already include "dog", "person", "river", "town" etc. If any entity-definition words are missing from the assembled dictionary, they'll just be unresolved — but this is unlikely with OllamaCache generating definitions for everything.

**5. Also support entities in closed mode** — When `--entities` is used WITHOUT `--text` (entities-only baseline test), parse entities as the dictionary directly:

```rust
// In main.rs, the dictionary construction block:
let mut dictionary = if let Some(text_path) = &cli.text {
    // ... existing open mode assembly ...
} else if let Some(entities_path) = &cli.entities {
    // ENTITIES-ONLY MODE: use entity definitions as the full dictionary
    // This is the baseline test: what can entity definitions alone answer?
    let content = std::fs::read_to_string(entities_path)
        .expect("Failed to read entities file");
    let dict = parse_dictionary(&content);
    println!("[Entities-only mode: {} entries from {:?}]", dict.entries.len(), entities_path);
    dict
} else {
    // ... existing closed mode ...
};
```

Then the entity merge (step 3) happens AFTER this block, so it works for both open mode (text + entities) and closed mode (dict + entities). For entities-only mode, the merge is a no-op since the dictionary IS the entities.

### Test the Implementation

Before proceeding to book passages, verify the `--entities` flag works:

```bash
# Entities-only baseline (no narrative text, no cache)
cargo run -p yalm-eval -- \
    --entities texts/three_men_supplementary/entities.md \
    --test texts/three_men/full_test.md \
    --mode equilibrium

# Should answer: "What is Montmorency?" -> dog, "Is Harris a person?" -> Yes
# These come directly from entity definitions.
```

---

## TASK 1: PREPARE THE TEXT

### Source

The full book is at `D:\workspace\projects\yalm\data\Three-Men-in-a-Boat.txt`.

Use grep/head/tail to locate chapter boundaries and passage content WITHOUT reading the full file. The book has chapter markers like "CHAPTER I.", "CHAPTER II.", etc.

### Extract Passages

Create the `texts/three_men/` directory and extract these passages:

1. **passage_montmorency.md** (~200-400 words): A section describing Montmorency's character. Look in Chapter 1 (opening pages mention Montmorency) or Chapter 13 (Montmorency and the cat). This tests: can the geometry figure out Montmorency is a dog?

2. **passage_packing.md** (~400-600 words): The famous packing scene from Chapter 4-5. Multiple characters, physical objects, actions. This tests: can the geometry cluster characters vs objects vs actions?

3. **passage_hampton_court.md** (~300-500 words): Harris gets lost in the Hampton Court maze, Chapter 6. Concrete spatial narrative. This tests: can the geometry connect Harris to Hampton Court to "lost"?

4. **chapter_01.md** (full Chapter 1, ~2000-3000 words): The opening chapter where they decide to go on the trip. Tests: larger text, multiple topics, dialogue.

Strip the Gutenberg header/footer. Keep only the narrative prose. Remove page numbers, chapter markers, or other formatting artifacts.

### Entity Definitions

Create `texts/three_men_supplementary/entities.md` using the format from Task 0. Use the exact format that `parse_dictionary()` expects (see dict5.md for reference):
- `**word** — definition.`
- `- "example sentence"`

Characters: montmorency (dog/fox terrier), harris (person/man), george (person/man).
Places: thames (river), kingston (town), hampton court (place with maze).

### Multi-word Entity Names

Note that "hampton court" is two words. The tokenizer will produce two separate tokens "hampton" and "court". The entity definition for "hampton court" won't match as a single dictionary entry because the assembler and engine operate on single-word tokens.

**Solution**: Create TWO entity entries:
- `**hampton** — a word. part of the name hampton court. hampton court is a big old place near the thames.`
- `**court** — a place or a part of a big building. hampton court is a place with a maze.`

Or simpler: just use `**hampton**` as the entity name and let the test questions reference "hampton" not "hampton court". The Ollama cache will also generate a definition for "court" (the generic word), which is fine.

Pick whichever approach produces cleaner test questions.

---

## TASK 2: WRITE TEST QUESTIONS

Use the standard YALM test format (same as dict5_test.md / passage1_test.md):

```markdown
---

**Q01**: Question text here?
**A**: Expected answer
**Chain**: word1 -> word2

---
```

### Per-Passage Tests (5 questions each)

Create test files matching each passage. All answers must be derivable from entity definitions + passage text + OllamaCache definitions.

**passage_montmorency_test.md (5 questions):**

| # | Question | Expected | Category |
|---|----------|----------|----------|
| Q01 | Is Montmorency a dog? | Yes | Direct (from entity) |
| Q02 | Is Montmorency a person? | No | Negation |
| Q03 | Is Montmorency an animal? | Yes | Transitive (dog → animal) |
| Q04 | Is Harris a dog? | No | Negation |
| Q05 | What is Montmorency? | a dog | Property query |

**passage_packing_test.md (5 questions):**

| # | Question | Expected | Category |
|---|----------|----------|----------|
| Q01 | Is Harris a person? | Yes | Direct (from entity) |
| Q02 | Is Harris a dog? | No | Negation |
| Q03 | Is George a person? | Yes | Direct (from entity) |
| Q04 | Is George a dog? | No | Negation |
| Q05 | What is Harris? | a person | Property query |

**passage_hampton_test.md (5 questions):**

| # | Question | Expected | Category |
|---|----------|----------|----------|
| Q01 | Is Hampton a place? | Yes | Direct (from entity) |
| Q02 | Is Hampton a person? | No | Negation |
| Q03 | Is Harris a person? | Yes | Direct (from entity) |
| Q04 | Is the Thames a river? | Yes | Direct (from entity) |
| Q05 | What is Hampton? | a place | Property query |

**chapter_01_test.md (5 questions):**

| # | Question | Expected | Category |
|---|----------|----------|----------|
| Q01 | Is Montmorency a dog? | Yes | Direct |
| Q02 | Is Harris a person? | Yes | Direct |
| Q03 | Is Montmorency a person? | No | Negation |
| Q04 | Is the Thames a river? | Yes | Direct |
| Q05 | What is George? | a person | Property query |

### Full Book Test (20 questions)

Create `texts/three_men/full_test.md` with 20 questions:

| Category | Count | Examples |
|----------|-------|---------|
| Character identification (Yes) | 4 | "Is Montmorency a dog?", "Is Harris a person?", "Is George a person?", "Is the Thames a river?" |
| Character negation (No) | 4 | "Is Montmorency a person?", "Is Harris a dog?", "Is George a dog?", "Is the Thames a person?" |
| Transitive reasoning (Yes) | 4 | "Is Montmorency an animal?", "Is Harris an animal?", "Is George an animal?", "Is Kingston a place?" |
| Negation cross-category (No) | 2 | "Is a dog a river?", "Is a person a place?" |
| What-Is queries | 4 | "What is Montmorency?", "What is Harris?", "What is George?", "What is the Thames?" |
| Honesty/Unknown | 2 | "What color is Montmorency?", "Is the Thames hot?" |

IMPORTANT: Every answer must be derivable from entity definitions + text + OllamaCache definitions. Do NOT ask questions that require temporal reasoning, irony detection, or attribution.

---

## TASK 3: RUN THE PIPELINE

### Progressive Testing

Start small, scale up. Record results at each level.

**Level 1: Entities only (no narrative text, no cache)**

```bash
cargo run -p yalm-eval -- \
    --entities texts/three_men_supplementary/entities.md \
    --test texts/three_men/full_test.md \
    --mode equilibrium
```

This tells us what the entity definitions alone can answer. Baseline.

**Level 2: Montmorency passage + entities + OllamaCache**

```bash
cargo run -p yalm-eval -- \
    --text texts/three_men/passage_montmorency.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/passage_montmorency_test.md \
    --mode equilibrium
```

Does the narrative text ADD signal beyond the entity definitions? If passage score > entities-only score, the geometry is extracting information from narrative.

**Level 3: Packing passage + entities + OllamaCache**

```bash
cargo run -p yalm-eval -- \
    --text texts/three_men/passage_packing.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/passage_packing_test.md \
    --mode equilibrium
```

**Level 4: Hampton Court passage + entities + OllamaCache**

```bash
cargo run -p yalm-eval -- \
    --text texts/three_men/passage_hampton_court.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/passage_hampton_test.md \
    --mode equilibrium
```

**Level 5: Chapter 1 + entities + OllamaCache**

```bash
cargo run -p yalm-eval -- \
    --text texts/three_men/chapter_01.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/chapter_01_test.md \
    --mode equilibrium
```

Larger text, more signal, more noise. Does fitness improve or degrade?

**Level 6: All passages combined + entities + full_test**

Concatenate all 3 passages + chapter 1 into a single text file, run against the 20-question full_test.md. This simulates multi-source input without the full book.

```bash
# Create combined text
cat texts/three_men/passage_montmorency.md \
    texts/three_men/passage_packing.md \
    texts/three_men/passage_hampton_court.md \
    texts/three_men/chapter_01.md \
    > texts/three_men/combined.md

cargo run -p yalm-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/full_test.md \
    --mode equilibrium
```

### Regression Check

After implementing `--entities`, verify zero regression on existing tests:

```bash
# dict5 (closed mode, no entities)
cargo run -p yalm-eval -- \
    --dict dictionaries/dict5.md \
    --test dictionaries/dict5_test.md \
    --mode equilibrium
# Expected: 20/20

# passage1 (open mode, no entities)
cargo run -p yalm-eval -- \
    --text texts/passage1.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/passage1_test.md \
    --mode equilibrium
# Expected: 5/5
```

### What to Record

For each level, record:

1. **Assembly stats**: seed words, total entries, closure ratio
2. **Connector stats**: how many discovered, how many passed uniformity filter
3. **Equilibrium stats**: passes, final energy, convergence
4. **Fitness**: per-question pass/fail, overall score
5. **Geometric diagnostics** (use resolver distance output):
   - Distance Montmorency ↔ "dog" (should be small)
   - Distance Montmorency ↔ "person" (should be large)
   - Distance Harris ↔ George (should be small — similar characters)
   - Distance Harris ↔ Montmorency (should be moderate — same trip, different species)
   - Distance Thames ↔ "river" (should be small)

These diagnostics tell us whether the geometry encodes the right structure, even if the resolver gets some questions wrong.

---

## TASK 4: ANALYZE

### The Montmorency Question

"What is Montmorency?" — the system should answer: **dog**.

But here's what makes it interesting. Montmorency is described throughout the book in very human terms. He has opinions, picks fights, has a disreputable character. The geometric space will see Montmorency co-occurring with human actions ("said", "thought", "wanted"). The entity definition says "dog", but the text signal says "person-like".

If the geometry places Montmorency closer to "person" than to "dog" despite the entity definition, that's a fascinating failure — the system is reading the NARRATIVE characterization, not just the definition. **Document this regardless of whether it's a pass or fail.**

### The Signal-to-Noise Curve

At each level (entities → passage → chapter → combined), the assembled dictionary gets larger. More text = more definitions from Ollama = more noise. Does fitness:

- **Improve monotonically**: More text = more signal. Architecture scales.
- **Peak then decline**: Sweet spot exists. Too much text adds noise faster than signal.
- **Stay flat**: Entity definitions dominate. Narrative text doesn't add much.

This curve answers a key question: **does geometric comprehension benefit from more narrative text?**

### The Victorian Vocabulary Question

Jerome uses words like "sculling", "lock" (river lock), "punt", "weir". The OllamaCache will generate definitions for ALL of these — but potentially with wrong senses. Track which Ollama-generated definitions:
- Have the correct sense for the book's context
- Have the WRONG sense ("lock" = door lock, not river lock)
- Are reasonable approximations

This directly feeds into future word-sense disambiguation work.

### OllamaCache Performance at Scale

Record:
- Total unique words sent to Ollama
- New LLM calls vs disk cache hits (the 805 existing cached definitions from phase 09 will cover many common words)
- Average generation time for new words
- Total assembly time

---

## WHAT NOT TO DO

- Do NOT modify the engine, equilibrium, resolver, or connector discovery code. The ONLY code change is adding `--entities` to `yalm-eval/main.rs`.
- Do NOT hand-tune parameters for Three Men in a Boat. Same defaults as prompt 09.
- Do NOT write test questions that require temporal reasoning, irony detection, or attribution.
- Do NOT include entity definitions as part of the test score commentary. They're input, not output. The test measures what the system INFERS.
- Do NOT give up if scores are low. The per-passage results and geometric diagnostics are the real data.
- Do NOT read the full book file into context. Use grep/head/tail for targeted extraction.

## SUCCESS CRITERIA

| Metric | Minimum | Target | Stretch |
|--------|---------|--------|---------|
| Entities-only fitness | > 0.40 | > 0.60 | > 0.80 |
| Passage fitness (avg across 3) | > 0.30 | > 0.50 | > 0.70 |
| Chapter 1 fitness | > 0.25 | > 0.45 | > 0.65 |
| Combined fitness (full_test) | > 0.20 | > 0.40 | > 0.60 |
| "What is Montmorency?" | dog | dog | dog |
| Assembly closure (combined) | > 70% | > 80% | > 90% |
| Montmorency-dog dist < Montmorency-person dist | Yes | Yes | Yes |
| Regression: dict5 | 20/20 | 20/20 | 20/20 |
| Regression: passage1 | 5/5 | 5/5 | 5/5 |

Note: targets are deliberately lower than dict5/12/18. This is the hardest test the system has ever faced. A fitness of 0.40 on real Victorian literature with zero hand-tuning would be a remarkable result.

## OUTPUT CHECKLIST

1. ☐ `--entities` flag implemented in `yalm-eval/main.rs`
2. ☐ Regression tests pass (dict5 20/20, passage1 5/5)
3. ☐ Entity definitions file: `texts/three_men_supplementary/entities.md`
4. ☐ Extracted passages: `texts/three_men/passage_montmorency.md`, `passage_packing.md`, `passage_hampton_court.md`, `chapter_01.md`
5. ☐ Test files: per-passage tests + `full_test.md`
6. ☐ Level 1 results: entities-only fitness
7. ☐ Level 2-4 results: per-passage fitness with Ollama
8. ☐ Level 5 results: chapter 1 fitness
9. ☐ Level 6 results: combined fitness against full_test
10. ☐ Geometric diagnostics: key distances (Montmorency↔dog, Montmorency↔person, Harris↔George, Thames↔river)
11. ☐ Montmorency analysis: where does geometry place him and why?
12. ☐ Signal-vs-noise curve: fitness as function of input text size
13. ☐ Victorian vocabulary audit: Ollama definition quality for period-specific words
14. ☐ OllamaCache performance stats: new calls, disk hits, timing
15. ☐ Updated RECAP.md with findings