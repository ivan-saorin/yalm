# PROMPT 09 — Dictionary Cache: From Closed Dictionaries to Open Text

## PREAMBLE

YALM is a geometric comprehension engine. Through prompts 01-08 it has demonstrated:

1. **Geometric comprehension works** on closed dictionaries at three scales (50, ~400, ~2000 words)
2. **The scaling curve** tells us how fitness degrades with vocabulary size (from prompt 07)
3. **Sequential equilibrium** replaces the GA with a self-organizing process — the text shapes the geometry directly, with fixed parameters (from prompt 08)

The remaining bottleneck is **closure**. Every dictionary so far was hand-crafted to ensure every word in every definition is itself defined. This constraint made the science clean but the system impractical. Nobody will hand-close 10,000 words.

This prompt adds a **dictionary cache** — an external lookup that provides definitions on demand. YALM stops reading a specific constructed dictionary and starts reading ANY text, pulling definitions as needed. The dictionary becomes infrastructure, not input.

## PROJECT STRUCTURE

```
D:\workspace\projects\yalm\
├── crates/
│   ├── yalm-core/         Data structures, GeometricSpace, Answer, traits
│   ├── yalm-parser/        Dictionary/test/grammar parsing + NEW: cache parser
│   ├── yalm-engine/        Force field + resolver + equilibrium
│   ├── yalm-eval/          Fitness scoring
│   ├── yalm-evolve/        Genetic algorithm (legacy)
│   └── yalm-cache/         NEW: dictionary cache crate
├── dictionaries/
│   ├── dict5.md, dict12.md, dict18.md  (existing)
│   └── cache/
│       └── simple-wiktionary/             NEW: cached definitions
├── texts/                                  NEW: free text inputs
│   └── (test passages)
├── prompts/
└── RECAP.md
```

---

## THE ARCHITECTURE

### Current Flow (Closed Dictionary)

```
dict18.md (hand-crafted, closed)
  │
  └── Parser → Connectors → Equilibrium → GeometricSpace → Resolver
```

### New Flow (Open Text + Cache)

```
free_text.md (any text: story, article, passage)
  │
  ├─ 1. Extract unique words from text
  │
  ├─ 2. For each word → lookup in DictionaryCache
  │      │
  │      └─ Cache returns: definition, examples (or "not found")
  │
  ├─ 3. For each word in each returned definition:
  │      │
  │      ├─ Already in our working set? → skip
  │      └─ Not yet? → lookup in cache → add to working set
  │
  ├─ 4. Repeat step 3 until:
  │      ├─ Closure reached (all words defined), OR
  │      ├─ Max depth reached (configurable, e.g., 3 hops), OR
  │      └─ Word not found in cache (mark as opaque)
  │
  ├─ 5. Assemble into a Dictionary struct (same type as dict5/12/18)
  │
  └─ 6. Feed into Equilibrium → GeometricSpace → Resolver
       (identical pipeline from prompt 08)
```

The key insight: **steps 1-5 produce an object identical to what dict18.md provides**. The rest of the pipeline doesn't change at all. The cache is a DICTIONARY CONSTRUCTOR, not a new comprehension mechanism.

---

## TASK 1: DICTIONARY CACHE CRATE

### New Crate: `yalm-cache`

This crate provides a `DictionaryCache` trait and implementations.

```rust
pub trait DictionaryCache {
    /// Look up a word. Returns definition text or None.
    fn lookup(&self, word: &str) -> Option<CacheEntry>;
    
    /// Check if word exists without fetching full definition.
    fn contains(&self, word: &str) -> bool;
}

pub struct CacheEntry {
    pub word: String,
    pub definitions: Vec<String>,   // multiple senses
    pub examples: Vec<String>,      // usage examples (if available)
    pub simple: bool,               // is this from Simple English source?
}
```

### Implementation 1: Simple English Wiktionary (Offline)

Simple English Wiktionary has ~70,000 entries with definitions written in basic vocabulary. This is the ideal first cache because:

- Definitions use limited vocabulary (closer to dict12/18 level than full English)
- Approximately closed at the corpus level
- Free, downloadable as XML dump
- No API rate limits

**Setup:**

1. Download Simple English Wiktionary dump from https://dumps.wikimedia.org/simplewiktionary/latest/
2. Parse XML into a flat lookup structure (word → definitions)
3. Store as a simple format: one file per letter, or a single JSON/bincode file
4. The cache is READ-ONLY at runtime. No network calls.

**Parsing the dump:**

Wiktionary XML contains wiki markup. The parser needs to:
- Extract `<title>` as the word
- Extract definition lines (lines starting with `#` in wikitext)
- Strip wiki markup: `[[links]]`, `{{templates}}`, `'''bold'''`, etc.
- Filter: skip entries that are inflected forms ("dogs" → skip, keep "dog")
- Filter: skip non-English entries (Simple Wiktionary is mostly English but has some translations)

This parsing is messy but bounded. Don't over-engineer. A 90% clean parse is fine — the geometry is robust to noise (demonstrated by grammar regularization surviving noisy force fields).

### Implementation 2: Offline File Cache (Fallback)

For testing without Wiktionary:

```
dictionaries/cache/manual/
  ├── a.txt
  ├── b.txt
  ...
```

Each file contains entries in the same format as dict5.md. This lets us test the cache pipeline with hand-crafted entries before plugging in the noisy Wiktionary data.

---

## TASK 2: TEXT-TO-DICTIONARY ASSEMBLER

### The Closure Chase Algorithm

This is the core new logic. Given free text and a cache:

```rust
pub struct DictionaryAssembler {
    cache: Box<dyn DictionaryCache>,
    max_depth: usize,           // max hops from original text words (default: 3)
    max_words: usize,           // safety limit (default: 5000)
    stop_words: HashSet<String>, // connector/function words to skip
}

impl DictionaryAssembler {
    pub fn assemble_from_text(&self, text: &str) -> AssemblyResult {
        let mut dictionary = Dictionary::new();
        let mut frontier: VecDeque<(String, usize)> = VecDeque::new(); // (word, depth)
        let mut visited: HashSet<String> = HashSet::new();
        let mut not_found: Vec<String> = Vec::new();
        
        // Seed: all unique words from the input text
        let text_words = tokenize(text);
        for word in &text_words {
            if !self.stop_words.contains(word) && !visited.contains(word) {
                frontier.push_back((word.clone(), 0));
                visited.insert(word.clone());
            }
        }
        
        // Chase closure
        while let Some((word, depth)) = frontier.pop_front() {
            if dictionary.len() >= self.max_words {
                break; // safety limit
            }
            
            match self.cache.lookup(&word) {
                Some(entry) => {
                    // Pick the simplest/shortest definition (sense 1)
                    let best_def = self.select_definition(&entry);
                    dictionary.add(word.clone(), best_def.clone());
                    
                    // Chase: add definition words to frontier
                    if depth < self.max_depth {
                        let def_words = tokenize(&best_def);
                        for dw in def_words {
                            if !self.stop_words.contains(&dw) 
                               && !visited.contains(&dw) {
                                frontier.push_back((dw.clone(), depth + 1));
                                visited.insert(dw);
                            }
                        }
                    }
                }
                None => {
                    not_found.push(word);
                }
            }
        }
        
        AssemblyResult {
            dictionary,
            not_found,
            depth_reached: /* max depth actually used */,
            closure_ratio: /* defined / (defined + not_found) */,
        }
    }
    
    fn select_definition(&self, entry: &CacheEntry) -> String {
        // Prefer: shortest definition that uses mostly known words.
        // For now: just pick the first (primary sense).
        // Future: rank by vocabulary overlap with already-assembled dict.
        entry.definitions.first()
            .cloned()
            .unwrap_or_default()
    }
}

pub struct AssemblyResult {
    pub dictionary: Dictionary,
    pub not_found: Vec<String>,
    pub depth_reached: usize,
    pub closure_ratio: f64,
}
```

### Sense Selection (Polysemy)

Real dictionaries have multiple senses. "bank" means river edge AND financial institution. Which definition does the assembler pick?

For now: **always pick sense 1 (primary sense).** This is wrong for some words but simple. The geometry will place "bank" somewhere between its senses, which is noisy but not catastrophic — it's similar to how dict12 handles words with broad definitions.

Future (NOT this prompt): context-aware sense selection using the surrounding text. If the input text talks about rivers, pick the river sense of "bank". This is a word-sense disambiguation problem and it's explicitly deferred.

### Stop Words

Function words ("the", "a", "is", "are", "can", "not", etc.) should NOT be chased through the cache. They're connectors, not content. The stop word list should come from the connector discovery on the assembled dictionary (bootstrap: run connector discovery on the text, use the found patterns as stop words, then assemble, then re-discover).

Simpler alternative: use a minimal hardcoded stop word list for the first implementation. This is a compromise on the "no linguistic knowledge" principle, but pragmatic. The list should be small (20-30 words) and limited to words that are universally function words in English.

---

## TASK 3: INTEGRATION

### New CLI Mode

```bash
# Old: closed dictionary
cargo run -p yalm-engine -- \
    --mode equilibrium \
    --dict dictionaries/dict5.md \
    --test dictionaries/dict5_test.md

# New: free text with cache
cargo run -p yalm-engine -- \
    --mode open \
    --text texts/passage1.md \
    --cache dictionaries/cache/simple-wiktionary/ \
    --test texts/passage1_test.md \
    --max-depth 3 \
    --max-words 3000
```

The `--mode open` flag triggers:
1. Parse text from `--text`
2. Assemble dictionary using cache from `--cache`
3. Report assembly statistics (words found, not found, closure ratio)
4. Run equilibrium on assembled dictionary
5. Run resolver on test questions

### Assembly Report

Before running the engine, print the assembly statistics:

```
=== Dictionary Assembly ===
Input text words:    347 unique
Depth 0 (from text): 312 found, 35 not found
Depth 1 (from defs): 891 found, 67 not found  
Depth 2 (from defs): 423 found, 31 not found
Total assembled:     1626 words
Closure ratio:       92.4%
Not found (sample):  "Thames", "Montmorency", "sculling", ...
```

This report is critical for diagnosing failures. If closure ratio is below 80%, the geometry will be noisy.

---

## TASK 4: TEST PASSAGES

### Test 1: Dict5 Reconstruction

The acid test: take dict5's definitions, feed them as "free text" to the assembler with Simple Wiktionary cache. The assembled dictionary will use WIKTIONARY definitions instead of the hand-crafted dict5 ones. Run dict5_test.

If this scores ≥ 15/20, the cache definitions are good enough. If it scores < 10/20, the noise from real definitions is too much and we need to investigate which definitions break the geometry.

### Test 2: Simple Passage

A short paragraph using common English words:

```markdown
A dog sat in the garden. The sun was hot. The dog was not a cat. 
The dog was an animal. It could run and make sound.
```

Test questions:
```
Is a dog an animal? -> Yes
Is the sun hot? -> Yes  
Is a dog a cat? -> No
Can a dog run? -> Yes
What is a dog? -> animal
```

This passage overlaps heavily with dict5 vocabulary. It should work if the cache does.

### Test 3: Simple Wikipedia Passage

Take one paragraph from Simple English Wikipedia on a concrete topic (e.g., "Dogs", "Sun", "Water"). Write 5 test questions. This tests whether the cache-assembled dictionary captures enough structure for basic comprehension of real (simple) text.

### Test 4: Scaling Measurement

Run the assembler on texts of increasing length:
- 50 words of input text
- 200 words of input text  
- 1000 words of input text

Measure: assembly time, assembled dictionary size, closure ratio, fitness. This gives a scaling profile for the cache pipeline.

---

## WHAT NOT TO DO

- Do NOT add network calls to the runtime. The cache must be fully offline. Download and parse the Wiktionary dump as a separate build step.
- Do NOT try to achieve 100% closure. Some words (proper nouns, archaic terms, slang) won't be in Simple Wiktionary. Mark them as opaque and move on. The geometry handles unknown words by ignoring them.
- Do NOT implement word-sense disambiguation in this prompt. Always use sense 1. WSD is a future enhancement.
- Do NOT change the equilibrium algorithm. The cache produces a Dictionary; the equilibrium consumes it. Clean separation.
- Do NOT skip the dict5 reconstruction test. It's the bridge between the controlled and open worlds.

## SUCCESS CRITERIA

| Metric | Minimum | Target | Stretch |
|--------|---------|--------|---------|
| Wiktionary parse coverage | > 50k entries | > 60k entries | > 70k entries |
| Assembly closure ratio (simple text) | > 75% | > 85% | > 95% |
| Dict5 reconstruction fitness | > 0.50 | > 0.70 | > 0.85 |
| Simple passage fitness | > 0.60 | > 0.80 | 1.00 |
| Assembly time (1000-word text) | < 30s | < 10s | < 2s |
| Wikipedia passage fitness | > 0.30 | > 0.50 | > 0.70 |

The CRITICAL metric is dict5 reconstruction. If hand-crafted definitions score 20/20 and cache definitions score < 10/20, the noise gap is too large and we need cleaner definition selection before proceeding.

## OUTPUT

1. `crates/yalm-cache/` — new crate with DictionaryCache trait + implementations
2. Dictionary assembler module (in yalm-cache or yalm-parser)
3. Simple English Wiktionary parser + build script
4. Updated engine binary with `--mode open` support
5. Test passages + test questions in `texts/`
6. Assembly reports for all test passages
7. Fitness comparison: closed dict vs cache-assembled for dict5 vocabulary
8. Scaling profile: assembly size and fitness vs input text length