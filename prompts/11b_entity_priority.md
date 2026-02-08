# PROMPT 11b — Entity Priority in Definition Category Extraction

## PREAMBLE

Phase 11 added who/where routing and max_hops=3, but 3w_test scored only 3/10. All 7 failures trace to one root cause: `definition_category()` fails to extract the correct category for proper nouns that are also common English words ("harris", "george", "hampton").

The entity merge in `main.rs` works correctly — it replaces Ollama-generated entries with entity definitions. But `definition_category()` still fails to return "person" for "harris" because its filter pipeline (structural check → connector-word check → property-word check → noun check) rejects category words that have high frequency in the 2429-entry dictionary.

This is a surgical fix: tag entity entries, bypass filters for entity definitions.

## ROOT CAUSE ANALYSIS

### What happens now

```
Question: "What is Harris?"
  → detect_what_question → subject = "harris"
  → definition_category("harris", ...)
  → finds entry: "a person. he is a man. he is one of the three men on the boat trip."
  → first sentence: "a person"
  → tokenize: ["a", "person"]
  → "a" → structural → skip
  → "person" → [filter pipeline rejects it] → skip
  → falls through → returns None
  → resolver falls back to geometric nearest neighbor
  → returns wrong answer ("a melt", "a thoroughly", etc.)
```

### Why "person" gets rejected

`definition_category()` applies four filters in sequence. In a 2429-entry dictionary built from Victorian text, common category words like "person" can be caught by one of:

1. **`structural.contains()`** — if "person" appears in enough definitions with enough uniformity, connector discovery may classify it as structural
2. **`is_connector_word()`** — if "person" appears in any connector pattern (unlikely but possible with aggressive discovery)
3. **`is_property_word()`** — if "person"'s own definition triggers the property heuristic
4. **noun check fails** — if the definition of "person" doesn't start with an article

The exact mechanism may vary by dictionary content, but the effect is consistent: the filter pipeline designed for messy Ollama definitions is too aggressive for clean entity definitions.

### Why Montmorency works but Harris doesn't

- "montmorency" is NOT a common English word → no Ollama entry conflicts
- "dog" (the category word) is a concrete, low-frequency noun → passes all filters
- "harris" and "george" ARE common English words → Ollama generates definitions for them
- "person" (the category word) is high-frequency → may be filtered

## THE FIX

### Step 1: Add `is_entity` flag to DictionaryEntry

In `crates/yalm-core/src/lib.rs`:

```rust
#[derive(Debug, Clone)]
pub struct DictionaryEntry {
    pub word: String,
    pub definition: String,
    pub examples: Vec<String>,
    pub section: String,
    /// True if this entry comes from an entity definition file.
    /// Entity definitions are hand-crafted and should bypass
    /// filter heuristics in definition_category().
    #[serde(default)]
    pub is_entity: bool,
}
```

**Important**: Every existing place that constructs a `DictionaryEntry` needs updating to include `is_entity: false`. This includes:
- `parse_dictionary()` in yalm-parser
- Any assembler code that creates entries
- The entity merge code (where `is_entity: true` is set)

Search for all struct literal constructions of `DictionaryEntry` across the codebase.

### Step 2: Set the flag during entity merge

In `crates/yalm-eval/src/main.rs`, in the entity merge section:

```rust
for mut entity_entry in entities_dict.entries {
    entity_entry.is_entity = true;  // tag as entity
    entry_map.insert(entity_entry.word.clone(), entity_entry);
}
```

### Step 3: Bypass filters for entity definitions in definition_category()

In `crates/yalm-engine/src/resolver.rs`, modify `definition_category()`:

```rust
fn definition_category(
    subject: &str,
    dictionary: &Dictionary,
    space: &GeometricSpace,
    structural: &HashSet<String>,
) -> Option<String> {
    let entry = dictionary.entries.iter().find(|e| e.word == subject)?;
    let first_sentence = entry.definition.split('.').next().unwrap_or(&entry.definition);
    let words = tokenize(first_sentence);

    for word in &words {
        let stemmed = match stem_to_entry(word, &dictionary.entry_set) {
            Some(s) => s,
            None => continue,
        };
        // Skip the subject itself
        if stemmed == subject {
            continue;
        }

        // ENTITY FAST PATH: entity definitions are hand-crafted in
        // ELI5 format ("a person", "a dog", "a river"). The first
        // non-subject, non-article content word IS the category.
        // Skip all heuristic filters that were designed for messy
        // auto-generated definitions.
        if entry.is_entity {
            // Only skip articles (a, an, the) and the subject itself
            let articles: HashSet<&str> = ["a", "an", "the"].iter().copied().collect();
            if articles.contains(stemmed.as_str()) {
                continue;
            }
            // First non-article word is the category
            if dictionary.entry_set.contains(&stemmed) {
                return Some(stemmed);
            }
            continue;
        }

        // STANDARD PATH (unchanged): apply all filters for auto-generated definitions
        if structural.contains(&stemmed) {
            continue;
        }
        if is_connector_word(&stemmed, space) {
            continue;
        }
        if is_property_word(&stemmed, dictionary) {
            continue;
        }
        if dictionary.entry_set.contains(&stemmed) {
            let is_noun = dictionary.entries.iter()
                .find(|e| e.word == stemmed)
                .map_or(false, |e| {
                    let fw = tokenize(&e.definition).into_iter().next().unwrap_or_default();
                    matches!(fw.as_str(), "a" | "an" | "the" | "one" | "any" | "something")
                });
            if is_noun {
                return Some(stemmed);
            }
        }
    }
    None
}
```

The entity fast path is simple and correct because:
- Entity definitions are written in dict5-style ELI5 format: "a [category]. details..."
- The first content word after the article IS the category by construction
- No heuristic filters needed — the definition is already clean

### Step 4: Also apply entity priority in resolve_what_is()

`resolve_what_is()` calls `definition_category()` first (which now handles entities), then falls back to geometric nearest neighbor. After the entity fast path, this should work automatically. But verify: if `definition_category()` returns `Some("person")` for harris, the resolver should produce `Answer::Word("a person")` with distance 0.0.

## DIAGNOSTIC STEP

Before implementing the fix, add a temporary diagnostic print in `definition_category()` to confirm the root cause:

```rust
// TEMPORARY: print what's happening for entity words
if entry.is_entity || subject == "harris" || subject == "george" || subject == "hampton" {
    eprintln!("[DEBUG definition_category] subject={}, def_first_sentence='{}'", subject, first_sentence);
    for word in &words {
        if let Some(stemmed) = stem_to_entry(word, &dictionary.entry_set) {
            eprintln!("  word='{}' stemmed='{}' structural={} connector={} property={} in_dict={}",
                word, stemmed,
                structural.contains(&stemmed),
                is_connector_word(&stemmed, space),
                is_property_word(&stemmed, dictionary),
                dictionary.entry_set.contains(&stemmed));
        }
    }
}
```

Run once with this diagnostic to capture exactly which filter catches "person". Record the output in the RECAP. Then remove the diagnostic and apply the real fix.

---

## TESTING

### Run order

```bash
# 1. Diagnostic run (with debug prints, BEFORE fix)
cargo run -p yalm-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/3w_test.md \
    --mode equilibrium 2>&1 | grep -A5 "DEBUG definition_category"
# → Record which filter blocks "person" for harris/george

# 2. Apply fix, then re-run 3w_test
cargo run -p yalm-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/3w_test.md \
    --mode equilibrium
# Expected: ≥7/10 (was 3/10)

# 3. full_test
cargo run -p yalm-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/full_test.md \
    --mode equilibrium
# Expected: ≥19/21 (was 17/21, +Q17 What is Harris, +Q18 What is George)

# 4. granularity_test
cargo run -p yalm-eval -- \
    --text texts/three_men/combined.md \
    --entities texts/three_men_supplementary/entities.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/three_men/granularity_test.md \
    --mode equilibrium
# Expected: ≥36/50 (no regression, possible improvement at Levels 2-3)

# 5. dict5 regression
cargo run -p yalm-eval -- \
    --dict dictionaries/dict5.md \
    --test dictionaries/dict5_test.md \
    --mode equilibrium
# Expected: 20/20 (no entities involved, is_entity always false)

# 6. dict12 regression
cargo run -p yalm-eval -- \
    --dict dictionaries/dict12.md \
    --test dictionaries/dict12_test.md \
    --mode equilibrium
# Expected: 14/20 (no entities involved)

# 7. passage1 regression
cargo run -p yalm-eval -- \
    --text texts/passage1.md \
    --cache-type ollama \
    --cache dictionaries/cache/ollama-qwen3 \
    --test texts/passage1_test.md \
    --mode equilibrium
# Expected: 5/5 (no entities involved)
```

---

## EXPECTED IMPACT

### 3w_test: 3/10 → 8-9/10

| Q | Question | Before | After | Reason |
|---|----------|--------|-------|--------|
| Q01 | What is Montmorency? | a dog ✅ | a dog ✅ | Already works |
| Q02 | What is the Thames? | a river ✅ | a river ✅ | Already works |
| Q03 | What is Kingston? | a place ✅ | a place ✅ | Already works |
| Q04 | Who is Montmorency? | a dog ✅ | a dog ✅ | P11 fixed |
| Q05 | Who is Harris? | ❌ | a person ✅ | Entity fast path |
| Q06 | Who is George? | ❌ | a person ✅ | Entity fast path |
| Q07 | Where is Kingston? | ❌ | a place ✅ | Entity fast path |
| Q08 | Where is Hampton? | ❌ | a place ✅ | Entity fast path |
| Q09 | What is Harris? | ❌ | a person ✅ | Entity fast path |
| Q10 | What is George? | ❌ | a person ✅ | Entity fast path |

Q07/Q08 depend on whether kingston/hampton entity definitions extract correctly. "a place" should work since the entity def starts with "a place."

### full_test: 17/21 → 19/21

Q17 (What is Harris?) and Q18 (What is George?) should now return "a person" instead of "a melt"/"a thoroughly".

Remaining 2 failures:
- Q10: Is Harris an animal? (person→animal 2-hop chain, definition content issue)
- Q11: Is George an animal? (same)

### Closed-dict tests: unaffected

dict5, dict12, passage1 don't use `--entities`, so `is_entity` is always false. The standard filter path is unchanged.

---

## WHAT NOT TO DO

- Do NOT change the standard (non-entity) path in `definition_category()`. That path handles the 2400+ Ollama-generated entries and works correctly.
- Do NOT modify engine, equilibrium, force field, or connector discovery.
- Do NOT modify entity definitions. The definitions are correct; the extraction is broken.
- Do NOT remove any existing filters from the standard path. They prevent false positives on Ollama definitions.

## SUCCESS CRITERIA

| Metric | Expected |
|--------|----------|
| 3w_test | ≥8/10 (was 3/10) |
| full_test | ≥19/21 (was 17/21) |
| granularity_test | ≥36/50 (no regression) |
| dict5 | 20/20 |
| dict12 | 14/20 |
| passage1 | 5/5 |
| Code changes | yalm-core (DictionaryEntry + is_entity field), yalm-parser (default false), yalm-eval/main.rs (set true on merge), yalm-engine/resolver.rs (entity fast path) |

## OUTPUT CHECKLIST

1. ☐ Diagnostic run: identified which filter blocks "person" for harris/george
2. ☐ `DictionaryEntry.is_entity` field added to yalm-core
3. ☐ All `DictionaryEntry` constructors updated with `is_entity: false`
4. ☐ Entity merge sets `is_entity: true`
5. ☐ `definition_category()` entity fast path implemented
6. ☐ Diagnostic prints removed
7. ☐ 3w_test results (expect ≥8/10)
8. ☐ full_test results (expect ≥19/21)
9. ☐ granularity_test results (expect ≥36/50)
10. ☐ Regression: dict5 20/20, dict12 14/20, passage1 5/5
11. ☐ RECAP.md updated with root cause finding + Phase 11b results