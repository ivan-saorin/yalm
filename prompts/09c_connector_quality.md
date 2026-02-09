# PROMPT 09c — Connector Quality: Uniformity Filter

## PREAMBLE

DAPHNE is a geometric comprehension engine. Prompt 09b added an LLM cache (Qwen3:8b via Ollama) that generates dict5-style definitions for any word on demand. The definitions are excellent — 10/10 quality, proper "is a" / "can" / "not" patterns.

Prompt 09c-scaling fixed the connector discovery threshold to scale logarithmically with dictionary size. Result: connectors found went from 1 → 16 at 783 entries. But the discovered connectors are WRONG:

| Dict5 connectors (correct) | Ollama connectors (wrong) |
|---|---|
| "is", "a", "is a", "can", "not", "it", "the", "and", "to", "in", "of" | "thing", "things", "part", "people", ... |

The scaling fix lowered the frequency threshold, which let more patterns through — but it can't distinguish **structural words** (appear uniformly across all topics) from **frequent content words** (appear often but cluster in specific domains).

This prompt adds a **uniformity filter** as a second pass in connector discovery. The frequency threshold selects CANDIDATES. The uniformity filter selects CONNECTORS.

## PROJECT STRUCTURE

```
D:\workspace\projects\dafhne\
├── crates/
│   ├── dafhne-core/         Data structures, GeometricSpace, Answer, traits
│   ├── dafhne-parser/        Dictionary/test/grammar parsing
│   ├── dafhne-engine/        Force field + resolver + equilibrium
│   │   └── src/
│   │       └── connector_discovery.rs   ← THE FILE TO CHANGE
│   ├── dafhne-eval/          Fitness scoring, CLI
│   ├── dafhne-evolve/        Genetic algorithm (legacy)
│   ├── dafhne-cache/         DictionaryCache trait + Manual + Wiktionary + Ollama
│   └── dafhne-wikt-build/    Wiktionary XML parser
├── dictionaries/
│   ├── dict5.md, dict12.md, dict18.md
│   └── cache/
│       ├── simple-wiktionary/
│       └── ollama-qwen3/       805 cached definitions
├── texts/
│   ├── passage1.md, passage1_test.md
│   └── dict5_defs.md
├── prompts/
└── RECAP.md
```

**Key file:** `crates/dafhne-engine/src/connector_discovery.rs`

---

## THE PROBLEM IN DETAIL

Connector discovery currently works in one pass:

```
For each word/bigram pattern:
    count = how many definitions contain this pattern
    if count > topic_threshold:
        → it's a connector
```

This finds frequent patterns. But frequency alone conflates two categories:

**True connectors** — structural words that glue definitions together:
- "is a" appears in definitions of dogs, emotions, places, colors, chemicals, tools, ...
- "can" appears in definitions of animals, people, machines, wind, ...
- "not" appears in definitions of cold, dark, empty, slow, ...
- Distribution: UNIFORM across the dictionary. Every topic uses them.

**Frequent content words** — common nouns/adjectives that cluster by topic:
- "thing" appears in definitions of objects, concepts, and abstract nouns — but NOT in definitions of emotions, actions, or people
- "people" appears in definitions of social/human concepts — but NOT in definitions of rocks, water, or colors
- "part" appears in definitions of components and anatomy — but NOT in definitions of feelings or weather
- Distribution: CLUSTERED. Specific topics use them heavily, others don't.

The mathematical signal is **distribution uniformity**, not frequency.

---

## THE UNIFORMITY FILTER

### Concept

After the frequency threshold identifies candidate patterns, measure how uniformly each candidate is distributed across the dictionary. True connectors spread evenly; content words cluster.

### Algorithm

```
For each candidate pattern P (already passed frequency threshold):
    1. Divide the dictionary into K buckets (e.g., K = 10)
       - Buckets are formed by sorting entries alphabetically and splitting evenly
       - This is a crude proxy for "different topics" without requiring topic modeling
    
    2. For each bucket B_i:
       - count_i = number of entries in B_i that contain pattern P
       - ratio_i = count_i / size_of(B_i)
    
    3. Compute uniformity score:
       - mean_ratio = average of all ratio_i
       - variance = average of (ratio_i - mean_ratio)²
       - uniformity = 1.0 - (variance / (mean_ratio² + epsilon))
       - This is essentially 1 - coefficient_of_variation²
    
    4. If uniformity > uniformity_threshold:
       → P is a connector (structurally uniform)
       If uniformity <= uniformity_threshold:
       → P is a frequent content word (topically clustered) → reject
```

### Why Alphabetical Buckets Work

Ideal bucketing would be by topic (animals, emotions, places, ...). But we don't have topic labels — that's what we're trying to discover. Alphabetical bucketing is a rough but effective proxy because:

- English vocabulary is NOT alphabetically organized by topic ("anger" and "atom" and "animal" are in the same bucket but are different domains)
- Random assignment would also work, but alphabetical is deterministic and reproducible
- The key property: any bucketing that DOESN'T correlate with topics will distinguish uniform from clustered distributions

If alphabetical proves too crude, an alternative is **random bucketing with averaging**: assign entries to K random buckets, compute uniformity, repeat M times, average the scores. More robust but slower. Try alphabetical first.

### Worked Example

Dictionary with 100 entries, K = 5 buckets (20 entries each).

**Pattern "is a"** — appears in 60/100 entries:
- Bucket 1: 12/20 = 0.60
- Bucket 2: 11/20 = 0.55
- Bucket 3: 13/20 = 0.65
- Bucket 4: 12/20 = 0.60
- Bucket 5: 12/20 = 0.60
- mean = 0.60, variance = 0.001, uniformity = 1.0 - (0.001 / 0.36) = **0.997** → CONNECTOR ✓

**Pattern "people"** — appears in 40/100 entries:
- Bucket 1: 15/20 = 0.75 (many human-related words in a-e)
- Bucket 2: 2/20 = 0.10
- Bucket 3: 12/20 = 0.60
- Bucket 4: 1/20 = 0.05
- Bucket 5: 10/20 = 0.50
- mean = 0.40, variance = 0.076, uniformity = 1.0 - (0.076 / 0.16) = **0.525** → CONTENT WORD ✗

**Pattern "thing"** — appears in 50/100 entries:
- Bucket 1: 18/20 = 0.90 (objects, abstracts)
- Bucket 2: 3/20 = 0.15 (emotions, actions)
- Bucket 3: 14/20 = 0.70
- Bucket 4: 5/20 = 0.25
- Bucket 5: 10/20 = 0.50
- mean = 0.50, variance = 0.073, uniformity = 1.0 - (0.073 / 0.25) = **0.708** → borderline, depends on threshold

### Parameters

```rust
pub struct UniformityParams {
    /// Number of buckets to divide the dictionary into
    pub num_buckets: usize,          // default: 10, range: 5-20
    
    /// Minimum uniformity score to qualify as connector
    pub uniformity_threshold: f64,   // default: 0.75, range: 0.5-0.95
    
    /// Epsilon to avoid division by zero
    pub epsilon: f64,                // default: 1e-10
}
```

`num_buckets` should scale mildly with dictionary size: 5 for <100 entries, 10 for 100-1000, 15 for 1000-5000. But start with fixed 10 and see if it works across scales.

`uniformity_threshold` is the key tuning parameter. Too high (0.95): rejects valid connectors like "not" that are genuinely less uniform than "is a" (negation is less common than categorization). Too low (0.5): lets through content words like "thing". Start at 0.75.

---

## IMPLEMENTATION

Modify `connector_discovery.rs`. The change is a SECOND pass after the existing frequency filter.

```rust
pub fn discover_connectors(
    entries: &[DictEntry],
    params: &ConnectorParams,
) -> Vec<Connector> {
    // === PASS 1: Frequency filter (EXISTING, unchanged) ===
    let candidates = frequency_filter(entries, params);
    
    // === PASS 2: Uniformity filter (NEW) ===
    let connectors = uniformity_filter(entries, &candidates, &params.uniformity);
    
    connectors
}

fn uniformity_filter(
    entries: &[DictEntry],
    candidates: &[CandidatePattern],
    params: &UniformityParams,
) -> Vec<Connector> {
    let n = entries.len();
    let num_buckets = params.num_buckets.min(n); // can't have more buckets than entries
    let bucket_size = n / num_buckets;
    
    // Build buckets (entries sorted alphabetically, split evenly)
    let mut sorted_entries: Vec<_> = entries.iter().collect();
    sorted_entries.sort_by(|a, b| a.word.cmp(&b.word));
    
    let buckets: Vec<Vec<&DictEntry>> = (0..num_buckets)
        .map(|i| {
            let start = i * bucket_size;
            let end = if i == num_buckets - 1 { n } else { (i + 1) * bucket_size };
            sorted_entries[start..end].to_vec()
        })
        .collect();
    
    let mut connectors = Vec::new();
    
    for candidate in candidates {
        // Compute per-bucket ratio
        let ratios: Vec<f64> = buckets.iter()
            .map(|bucket| {
                let hits = bucket.iter()
                    .filter(|e| entry_contains_pattern(e, &candidate.pattern))
                    .count();
                hits as f64 / bucket.len() as f64
            })
            .collect();
        
        let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
        let variance = ratios.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / ratios.len() as f64;
        let uniformity = 1.0 - (variance / (mean * mean + params.epsilon));
        
        if uniformity > params.uniformity_threshold {
            connectors.push(Connector {
                pattern: candidate.pattern.clone(),
                frequency: candidate.frequency,
                uniformity,  // store for diagnostics
                // ... other fields
            });
        }
    }
    
    connectors
}
```

### Add Uniformity to Connector Struct

Store the uniformity score for diagnostics:

```rust
pub struct Connector {
    pub pattern: Vec<String>,
    pub frequency: usize,
    pub uniformity: f64,       // NEW: how uniformly distributed (0.0-1.0)
    pub force_direction: Vec<f64>,
    pub weight: f64,
}
```

### Diagnostic Output

When running connector discovery, print the filter results:

```
=== Connector Discovery ===
Entries: 783 | Frequency candidates: 47 | Buckets: 10

Pattern        Freq    Uniformity  Status
"a"            623     0.992       ✓ CONNECTOR
"is"           412     0.987       ✓ CONNECTOR
"is a"         389     0.985       ✓ CONNECTOR
"can"          201     0.962       ✓ CONNECTOR
"not"          156     0.891       ✓ CONNECTOR
"to"           534     0.979       ✓ CONNECTOR
"it"           298     0.971       ✓ CONNECTOR
"the"          345     0.968       ✓ CONNECTOR
"and"          402     0.983       ✓ CONNECTOR
"in"           267     0.954       ✓ CONNECTOR
"of"           312     0.961       ✓ CONNECTOR
"has"          178     0.902       ✓ CONNECTOR
"thing"        289     0.712       ✗ content (clustered)
"things"       134     0.645       ✗ content (clustered)
"people"       98      0.523       ✗ content (clustered)
"part"         112     0.601       ✗ content (clustered)
"person"       87      0.498       ✗ content (clustered)
...

Accepted: 12 connectors | Rejected: 35 content words
```

This table is essential for debugging. If valid connectors are rejected (uniformity too low), lower the threshold. If content words leak through (uniformity too high), raise it.

---

## TESTING PROTOCOL

### Test 1: Dict5 Regression (CRITICAL)

Run dict5 with the uniformity filter. Must produce the SAME 11 connectors and 20/20 score.

```bash
cargo run -p dafhne-eval -- \
    --dict dictionaries/dict5.md \
    --grammar dictionaries/grammar5.md \
    --test dictionaries/dict5_test.md
```

If ANY dict5 connector is rejected by the uniformity filter, the threshold is too aggressive. Dict5 has only 51 entries — uniformity measurements are noisy at this scale. You may need to SKIP the uniformity filter for dictionaries below a size threshold (e.g., < 100 entries). The filter is designed for medium-to-large dictionaries where content word frequency becomes a problem.

```rust
if entries.len() < 100 {
    // Small dictionary: frequency filter is sufficient, skip uniformity
    return candidates.into_iter().map(|c| c.into_connector()).collect();
}
```

### Test 2: Dict12

```bash
cargo run -p dafhne-eval -- \
    --dict dictionaries/dict12.md \
    --grammar dictionaries/grammar5.md \
    --test dictionaries/dict12_test.md
```

Previous result: 14/20 with 117 connectors (too many). The uniformity filter should cut this dramatically — most of those 117 are content words. Target: 15-25 connectors, score ≥ 14/20.

### Test 3: Dict18

```bash
cargo run -p dafhne-eval -- \
    --dict dictionaries/dict18.md \
    --test dictionaries/dict18_test.md
```

Previous result: 14/20 with 297 connectors. Target: 20-40 connectors, score ≥ 14/20.

### Test 4: Ollama Dict5 Reconstruction (THE KEY TEST)

```bash
cargo run -p dafhne-eval -- \
    --mode open \
    --text texts/dict5_defs.md \
    --cache dictionaries/cache/ollama-qwen3/ \
    --cache-type ollama \
    --max-depth 3 \
    --max-words 2000 \
    --test dictionaries/dict5_test.md
```

Previous: 13/20 with 16 connectors (mostly content words).
Target: ≥ 14/20 with 8-15 connectors (mostly structural words).

This is the test that tells us if the uniformity filter fixes the open-mode pipeline.

### Test 5: Passage1

```bash
cargo run -p dafhne-eval -- \
    --mode open \
    --text texts/passage1.md \
    --cache dictionaries/cache/ollama-qwen3/ \
    --cache-type ollama \
    --test texts/passage1_test.md
```

Previous: 5/5. Must not regress.

### Test 6: Connector Quality Audit

For EACH test above, capture the diagnostic table (pattern / freq / uniformity / status). Compare connectors found across all scales:

| Pattern | dict5 | dict12 | dict18 | Ollama 783 |
|---------|-------|--------|--------|------------|
| "is"    | ✓     | ?      | ?      | ?          |
| "a"     | ✓     | ?      | ?      | ?          |
| "is a"  | ✓     | ?      | ?      | ?          |
| "can"   | ✓     | ?      | ?      | ?          |
| "not"   | ✓     | ?      | ?      | ?          |
| "thing" | ✓(?)  | ?      | ?      | ?          |
| ...     |       |        |        |            |

This table reveals whether the SAME connectors are discovered at every scale. If "is a" and "can" and "not" appear consistently across all four, the discovery algorithm is robust.

---

## EDGE CASES

### "thing" Is Borderline

In dict5, "thing" is effectively a connector — it appears in almost every definition as the root category. In larger dictionaries, it becomes more of a content word (definitions of emotions, actions, and relationships don't use "thing"). The uniformity filter may correctly reject it at scale while keeping it at dict5 scale. This is fine — "thing" serves a different role at different vocabulary sizes.

### "has" and "part of"

These are legitimate connector patterns added in grammar18 (prompt 07). They're less frequent than "is a" but should be uniformly distributed. Watch whether the uniformity threshold lets them through. If they're rejected, consider lowering the threshold slightly or adding a minimum-frequency exemption for known useful patterns. But prefer discovering them naturally over whitelisting.

### Single-Word vs Bigram Connectors

The uniformity filter works on both. "is" (single word) and "is a" (bigram) should both pass uniformity. But longer patterns (trigrams like "is a kind") will have lower frequency AND lower uniformity simply because they're rare. The frequency threshold will catch most of these before uniformity even runs. No special handling needed.

---

## WHAT NOT TO DO

- Do NOT change the frequency threshold formula from prompt 09c-scaling. The uniformity filter is a SECOND pass, not a replacement.
- Do NOT hardcode a whitelist of "known good" connectors. The whole point is discovery from statistics.
- Do NOT change the resolver, force field, equilibrium, or cache code. This is connector discovery only.
- Do NOT use topic modeling, LDA, or any NLP technique for bucketing. Alphabetical split is sufficient and keeps the system NLP-free.
- Do NOT remove the uniformity score from the Connector struct after debugging. It's useful metadata for future analysis.
- Do NOT over-tune the uniformity threshold. Set it once (0.75), test across all scales, adjust ONCE if needed, freeze.

## SUCCESS CRITERIA

| Metric | Minimum | Target | Stretch |
|--------|---------|--------|---------|
| Dict5 regression | 20/20 | 20/20 | 20/20 |
| Dict5 connectors | 11 | 11 | 11 |
| Dict12 score | ≥ 14/20 | ≥ 15/20 | > 15/20 |
| Dict12 connectors | 10-30 | 12-20 | ~15 |
| Dict18 score | ≥ 14/20 | ≥ 15/20 | > 15/20 |
| Dict18 connectors | 10-40 | 15-25 | ~20 |
| Ollama dict5 recon score | ≥ 13/20 | ≥ 15/20 | ≥ 17/20 |
| Ollama connectors quality | "is a" + "can" + "not" found | + "has" + "to" | matches dict5 set |
| Passage1 score | 5/5 | 5/5 | 5/5 |
| Connector overlap across scales | 5+ shared | 8+ shared | core set identical |

The CRITICAL metric is **connector overlap**: do the same structural patterns emerge at 51, 400, 783, and 2008 entries? If yes, connector discovery is scale-invariant and the pipeline is ready for Three Men in a Boat.

## OUTPUT

1. Modified `connector_discovery.rs` — uniformity filter added as second pass
2. Updated `Connector` struct with uniformity field
3. Diagnostic output: pattern / freq / uniformity / status table
4. Results table: all five tests (dict5, dict12, dict18, Ollama recon, passage1)
5. Connector comparison table across all scales
6. Analysis: which connectors are universal vs scale-dependent