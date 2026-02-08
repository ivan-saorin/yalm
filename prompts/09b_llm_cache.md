# PROMPT 09b — LLM Dictionary Cache: Qwen3:8b via Ollama

## PREAMBLE

YALM is a geometric comprehension engine. Prompt 09 added a dictionary cache pipeline: free text → BFS closure chase → assembled Dictionary → Equilibrium → Resolver. Two backends were built: ManualFileCache and WiktionaryCache.

Results showed the architecture works, but **Wiktionary definitions are too noisy for connector discovery** — the engine found only 1 connector ("a") vs 11 for hand-crafted dict5 definitions. Fitness dropped from 0.8438 (manual cache) to 0.6875 (Wiktionary).

The root cause isn't the pipeline — it's the SHAPE of real dictionary definitions. "A domestic mammal of the family Canidae" doesn't contain the "is a", "can", "not" patterns the connector discovery algorithm was built for.

This prompt adds a third backend: **OllamaCache**, which calls a local Qwen3:8b model to generate definitions in dict5 style. A style prompt constrains the LLM output to produce exactly the sentence patterns YALM's connector discovery expects. The LLM becomes a definition TRANSLATOR — it knows what "dog" means and expresses it in YALM-compatible language.

Verified: Qwen3:8b with `/nothink` generates clean output in ~2-5 seconds per word.

Example: `soul: a part of a person. it is a thing that can feel love and care. it has a link to life and spirit.`

## PROJECT STRUCTURE

```
D:\workspace\projects\yalm\
├── crates/
│   ├── yalm-core/         Data structures, GeometricSpace, Answer, traits
│   ├── yalm-parser/        Dictionary/test/grammar parsing
│   ├── yalm-engine/        Force field + resolver + equilibrium
│   ├── yalm-eval/          Fitness scoring, CLI (--mode open)
│   ├── yalm-evolve/        Genetic algorithm (legacy)
│   ├── yalm-cache/         DictionaryCache trait + ManualFileCache + WiktionaryCache
│   └── yalm-wikt-build/    Wiktionary XML parser
├── dictionaries/
│   ├── dict5.md, dict12.md, dict18.md
│   └── cache/
│       ├── simple-wiktionary/     Parsed Wiktionary JSON
│       └── ollama-qwen3/          NEW: LLM-generated definitions (disk memo)
├── texts/
│   ├── passage1.md, passage1_test.md
│   └── dict5_defs.md
├── prompts/
└── RECAP.md
```

**Key file to modify:** `crates/yalm-cache/src/lib.rs` (or new file `ollama.rs`)
**Key file to modify:** `crates/yalm-eval/src/main.rs` (add `--cache-type ollama` option)

---

## THE STYLE PROMPT

This is the most important piece. It turns Qwen3:8b into a dict5-style definition factory.

```text
You are a simple dictionary writer. Define the given word using ONLY basic English.

Rules:
- First sentence states the category: "a [category]." or "to [verb]." or "not [opposite]."
- Use patterns: "is a", "can", "not", "has", "part of", "makes"
- Maximum 3 short sentences
- Use the simplest words you know
- No examples, no etymology, no "such as", no parentheses
- Output ONLY the definition, nothing else

Examples:
dog: an animal. it can make sound and move fast.
sun: a big hot thing in the sky. it makes light.
cold: not hot. when something has little heat.
run: to move fast using legs.
water: a thing that is not hard. it can move and it is clear.
happy: a good feeling. when a person is not sad.
king: a person. a man that has power over a big place.

Define: {word}
```

Critical details:
- The examples cover nouns, adjectives, verbs, and abstract concepts
- Every example uses at least one connector pattern
- Category word is ALWAYS first — this is what `resolve_what_is()` extracts
- "nothing else" prevents the model from adding explanations or caveats

### Ollama API Call

Endpoint: `POST http://localhost:11434/api/generate`

```json
{
    "model": "qwen3:8b",
    "prompt": "<style prompt with {word} replaced>",
    "stream": false,
    "options": {
        "temperature": 0.3,
        "num_predict": 100,
        "top_p": 0.9
    }
}
```

Low temperature (0.3) for consistent, predictable definitions. `num_predict: 100` caps output at ~100 tokens — more than enough for 3 sentences, prevents runaway generation.

**Disabling thinking mode:** Qwen3 has a thinking mode that adds ~70 seconds of overhead. To disable it, prepend `/nothink` to the prompt OR use the `chat` endpoint with a system message. Test which approach your Ollama version supports. The goal is <5 seconds per definition.

If `/nothink` doesn't work via the `generate` endpoint, use the `chat` endpoint instead:

```json
{
    "model": "qwen3:8b",
    "messages": [
        {"role": "system", "content": "<style prompt without the Define: line>"},
        {"role": "user", "content": "{word}"}
    ],
    "stream": false,
    "options": {
        "temperature": 0.3,
        "num_predict": 100
    }
}
```

---

## IMPLEMENTATION

### OllamaCache

```rust
pub struct OllamaCache {
    /// Ollama API base URL
    base_url: String,           // default: "http://localhost:11434"
    
    /// Model name
    model: String,              // "qwen3:8b"
    
    /// The style prompt template (contains {word} placeholder)
    style_prompt: String,
    
    /// In-memory cache (populated from disk + new lookups)
    memory: HashMap<String, CacheEntry>,
    
    /// Disk cache directory for persistence
    disk_cache_dir: PathBuf,    // dictionaries/cache/ollama-qwen3/
}

impl DictionaryCache for OllamaCache {
    fn lookup(&self, word: &str) -> Option<CacheEntry> {
        let normalized = word.to_lowercase().trim().to_string();
        
        // 1. Check in-memory cache
        if let Some(entry) = self.memory.get(&normalized) {
            return Some(entry.clone());
        }
        
        // 2. Check disk cache
        if let Some(entry) = self.load_from_disk(&normalized) {
            self.memory.insert(normalized.clone(), entry.clone());
            return Some(entry);
        }
        
        // 3. Call Ollama
        match self.call_ollama(&normalized) {
            Ok(entry) => {
                self.save_to_disk(&normalized, &entry);
                self.memory.insert(normalized, entry.clone());
                Some(entry)
            }
            Err(e) => {
                eprintln!("[OllamaCache] Failed for '{}': {}", word, e);
                None
            }
        }
    }
    
    fn contains(&self, word: &str) -> bool {
        // For LLM cache, assume any word CAN be defined.
        // Return true always — let lookup handle failures.
        true
    }
    
    fn name(&self) -> &str { "ollama-qwen3" }
    
    fn len(&self) -> usize { self.memory.len() }
}
```

### Disk Memoization

Every LLM call costs 2-5 seconds. For 3000 unique words that's up to 4 hours. With disk memo, second run is instant.

Storage format — one JSON file per first letter:

```
dictionaries/cache/ollama-qwen3/
├── a.json    {"abandon": {"word": "abandon", "definitions": ["..."]}, ...}
├── b.json
├── ...
└── z.json
```

Same structure as the Wiktionary cache. The assembler doesn't care which backend produced it.

```rust
impl OllamaCache {
    fn disk_path(&self, word: &str) -> PathBuf {
        let letter = word.chars().next().unwrap_or('_').to_lowercase().next().unwrap();
        self.disk_cache_dir.join(format!("{}.json", letter))
    }
    
    fn load_from_disk(&self, word: &str) -> Option<CacheEntry> {
        let path = self.disk_path(word);
        if !path.exists() { return None; }
        let data: HashMap<String, CacheEntry> = 
            serde_json::from_reader(File::open(&path).ok()?).ok()?;
        data.get(word).cloned()
    }
    
    fn save_to_disk(&self, word: &str, entry: &CacheEntry) {
        let path = self.disk_path(word);
        let mut data: HashMap<String, CacheEntry> = if path.exists() {
            serde_json::from_reader(File::open(&path).unwrap()).unwrap_or_default()
        } else {
            HashMap::new()
        };
        data.insert(word.to_string(), entry.clone());
        let file = File::create(&path).unwrap();
        serde_json::to_writer_pretty(file, &data).unwrap();
    }
}
```

### Response Parsing

The LLM response needs light cleanup:

```rust
fn parse_response(&self, word: &str, raw: &str) -> CacheEntry {
    let text = raw.trim();
    
    // Strip the word prefix if the model echoes it: "dog: an animal..." → "an animal..."
    let definition = if let Some(rest) = text.strip_prefix(&format!("{}:", word)) {
        rest.trim().to_string()
    } else {
        text.to_string()
    };
    
    // Strip any markdown, quotes, or thinking artifacts
    let definition = definition
        .replace("```", "")
        .replace("\"", "")
        .trim()
        .to_string();
    
    CacheEntry {
        word: word.to_string(),
        definitions: vec![definition],
        examples: vec![],
        simple: true,
    }
}
```

### Progress Reporting

The BFS assembler will make many sequential LLM calls. Print progress:

```
[OllamaCache] Generating: dog (1/347) ... 2.3s
[OllamaCache] Generating: animal (2/347) ... 1.8s
[OllamaCache] Cached hit: thing (disk)
...
[OllamaCache] Complete: 312 generated, 35 from disk cache, 0 failed
```

---

## CLI INTEGRATION

Add `--cache-type ollama` to yalm-eval:

```bash
cargo run -p yalm-eval -- \
    --mode open \
    --text texts/passage1.md \
    --cache dictionaries/cache/ollama-qwen3/ \
    --cache-type ollama \
    --ollama-url http://localhost:11434 \
    --ollama-model qwen3:8b \
    --max-depth 3 \
    --max-words 2000 \
    --test texts/passage1_test.md
```

If `--cache-type` is omitted, auto-detect from cache directory contents (JSON with Wiktionary structure vs Ollama structure).

---

## TESTING PROTOCOL

### Prerequisites

Verify Ollama is running and Qwen3:8b responds:

```bash
curl http://localhost:11434/api/generate -d '{
  "model": "qwen3:8b",
  "prompt": "/nothink\nDefine in simple English (max 3 sentences): dog",
  "stream": false,
  "options": {"temperature": 0.3, "num_predict": 100}
}'
```

Expected: response in <5 seconds, clean definition with "is a" / "can" patterns.

### Test 1: Definition Quality Spot-Check

Before running the full pipeline, generate definitions for 10 known dict5 words and inspect manually:

```
dog, cat, sun, water, person, animal, big, small, hot, cold
```

Check:
- Does every definition start with a category word?
- Are "is a", "can", "not", "has" patterns present?
- Are definitions ≤ 3 sentences?
- Are words simple enough that their OWN definitions won't chain too deep?

If any definition is bad (verbose, uses jargon, missing patterns), adjust the style prompt. This is cheaper than debugging the whole pipeline.

### Test 2: Dict5 Reconstruction (The Acid Test)

```bash
cargo run -p yalm-eval -- \
    --mode open \
    --text texts/dict5_defs.md \
    --cache dictionaries/cache/ollama-qwen3/ \
    --cache-type ollama \
    --max-depth 3 \
    --max-words 2000 \
    --test dictionaries/dict5_test.md
```

Compare against all three backends:

| Backend | Score | Fitness | Connectors Found |
|---------|-------|---------|------------------|
| Closed dict5 | 20/20 | 1.0000 | 11 |
| Manual cache | 15/20 | 0.8438 | (from 09) |
| Wiktionary | 10/20 | 0.6875 | 1 |
| **Ollama Qwen3:8b** | **?/20** | **?** | **?** |

Target: ≥ 15/20, ≥ 8 connectors discovered. If connector count is close to dict5's 11, the style prompt is working.

### Test 3: Passage1

Same passage1.md test from prompt 09:

| Backend | Score | Fitness |
|---------|-------|---------|
| Wiktionary | 2/5 | 0.2000 |
| **Ollama** | **?/5** | **?** |

Target: ≥ 3/5.

### Test 4: Abstract Vocabulary Chain Depth

Generate a definition for "soul" and trace the BFS closure chase:

```
Depth 0: soul → "a part of a person. it is a thing that can feel love and care. it has a link to life and spirit."
Depth 1: spirit → ? , love → ? , care → ? , link → ? , life → ?
Depth 2: (words from depth 1 definitions)
Depth 3: (stop)
```

Record: total words assembled, closure ratio, any circular chains. Abstract words are the stress test for `max_depth`.

### Test 5: Timing Profile

Generate definitions for 100 words. Record:
- Average time per LLM call
- Total wall-clock time
- Projected time for 3000 words (Three Men in a Boat estimate)

If average > 5s/word, the full book will take >4 hours for first run. Acceptable if disk memo makes subsequent runs instant.

---

## WHAT NOT TO DO

- Do NOT change the DictionaryCache trait. OllamaCache implements the EXISTING trait.
- Do NOT change the assembler, equilibrium, or resolver. Only the cache backend is new.
- Do NOT let the LLM generate definitions longer than 3 sentences. Enforce via `num_predict` cap AND post-processing truncation.
- Do NOT call the LLM for stop words. Check the stop word list BEFORE calling `lookup()`.
- Do NOT skip disk memoization. Every LLM call costs seconds. Memo is mandatory.
- Do NOT use `--cache-type ollama` without checking that Ollama is actually running. Fail fast with a clear error message.
- Do NOT attempt to batch LLM calls. Ollama's generate endpoint is sequential. The BFS naturally processes one word at a time.

## SUCCESS CRITERIA

| Metric | Minimum | Target | Stretch |
|--------|---------|--------|---------|
| Dict5 reconstruction fitness | > 0.70 | > 0.85 | > 0.95 |
| Connectors found (dict5 recon) | > 5 | > 8 | ≥ 11 |
| Passage1 fitness | > 0.30 | > 0.50 | > 0.70 |
| Avg time per definition (no-think) | < 10s | < 5s | < 2s |
| Disk memo hit rate (second run) | 100% | 100% | 100% |
| Definition quality (spot check 10) | 8/10 clean | 10/10 clean | — |

The CRITICAL metric is **connectors found**. If Ollama definitions produce ≥ 8 connectors on dict5 reconstruction, the style prompt works and the Wiktionary noise problem is solved. Everything else follows.

## OUTPUT

1. `crates/yalm-cache/src/ollama.rs` — OllamaCache implementation
2. Updated `yalm-cache/src/lib.rs` — export OllamaCache
3. Updated `yalm-eval/src/main.rs` — `--cache-type ollama` + `--ollama-url` + `--ollama-model` flags
4. Style prompt stored as a constant or config file
5. Spot-check results for 10 dict5 words
6. Comparison table: all four backends on dict5 reconstruction
7. Timing profile for 100 words
8. Disk cache populated in `dictionaries/cache/ollama-qwen3/`