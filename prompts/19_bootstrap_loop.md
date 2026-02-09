# PROMPT 19 — Bootstrap Loop: Connector Enrichment from Generated Text

## GOAL

Implement the self-improvement loop: DAFHNE generates descriptions of its known concepts using `describe()`, feeds the generated text back through connector discovery, and uses the enriched connector set to produce a richer equilibrium. Grammar evolves without changing any dictionary.

This is the path from ELI5 sentence structure to more complex grammar, driven entirely by geometric self-reflection.

## PREREQUISITE

- Phase 15 complete (rich describe mode with property extraction from first sentences)
- Phase 18 complete: 45/50 on unified_test.md, 5 spaces stable
- `describe()` in resolver.rs produces multi-sentence output per concept
- `discover_connectors()` in connector_discovery.rs working
- Self-consistency from earlier phases: describe output can be re-read

## CORE INSIGHT

Connector discovery works by extracting sentence relations: topic_word → connector_pattern → topic_word. Currently, the only input text is dictionary definitions and examples. These are short, repetitive, ELI5 fragments.

Describe output is structurally different from dictionary text:
- Dictionary: "an animal. it can make sound. it can live with a person."
- Describe: "a dog is an animal. a dog can make sound. a dog is not a cat. a dog is not a ball."

The describe output contains explicit subject-predicate-object sentences that dictionary definitions don't. When fed back to connector discovery, these produce new relation patterns:

| Source | Sentence | Relation |
|--------|----------|----------|
| Dictionary | "it can make sound" | it → [can make] → sound |
| Describe | "a dog is an animal" | dog → [is] → animal |
| Describe | "a dog is not a cat" | dog → [is not] → cat |
| Describe | "the sun is big" | sun → [is] → big |
| Describe | "the sun is hot" | sun → [is] → hot |

More relations → higher connector frequencies → potentially new connectors passing the frequency threshold → richer geometry.

## WHAT EVOLVES, WHAT DOESN'T

| Component | Status | Reason |
|-----------|--------|--------|
| CONTENT dictionary | FIXED | dict5.md doesn't change |
| MATH dictionary | FIXED | dict_math5.md doesn't change |
| GRAMMAR dictionary | FIXED | dict_grammar5.md doesn't change |
| TASK dictionary | FIXED | dict_task5.md doesn't change |
| SELF dictionary | FIXED | dict_self5.md doesn't change |
| CONTENT connectors | **EVOLVES** | New patterns from describe output |
| GRAMMAR connectors | **EVOLVES** | New patterns from describe output |
| CONTENT equilibrium | **RE-RUNS** | Different connectors → different force field |
| GRAMMAR equilibrium | **RE-RUNS** | Different connectors → different force field |
| MATH/TASK/SELF | FIXED | No describe output targets these |

The key constraint: dictionaries are immutable. Only connectors and equilibrium change.

## THE LOOP

```
Level 0 (baseline):
  Each space loaded normally from dictionary
  Connector discovery on dictionary text only
  describe() produces ELI5 sentences
  Score: unified_test.md baseline (45/50)

Level 1:
  describe() all CONTENT concepts (dog, cat, sun, ball, water, food, ...)
  describe() all GRAMMAR concepts (noun, verb, sentence, property, ...)
  Feed generated text → extract_all_sentences()
  Re-run connector discovery with dictionary + generated sentences
  Compare connector sets: Level 0 vs Level 1
  Re-run equilibrium with enriched connectors
  Score: unified_test.md Level 1
  Self-consistency check: describe() output at Level 1

Level 2:
  describe() again with Level 1 geometry
  Feed back → connector discovery
  Compare: Level 1 vs Level 2 connectors
  If new connectors found: re-equilibrate, score, continue
  If no new connectors: convergence reached, stop

Level N:
  Converges when no new connectors emerge
  OR when regression detected
  OR when max_iterations reached
```

## IMPLEMENTATION DESIGN

### New module: `crates/dafhne-engine/src/bootstrap.rs`

Single new file, ~200-300 lines. Does NOT modify any existing module.

```rust
pub struct BootstrapConfig {
    pub max_iterations: usize,      // default: 5
    pub min_new_connectors: usize,  // stop if fewer than this many new (default: 1)
    pub regression_threshold: f64,  // stop if score drops by more than this (default: 0.05)
    pub describe_spaces: Vec<String>, // which spaces generate text (default: ["content", "grammar"])
}

pub struct BootstrapResult {
    pub levels: Vec<LevelResult>,
    pub converged_at: usize,
    pub final_connectors: HashMap<String, Vec<Connector>>,
}

pub struct LevelResult {
    pub level: usize,
    pub new_connectors: Vec<(String, Vec<String>)>, // (space_name, pattern)
    pub lost_connectors: Vec<(String, Vec<String>)>,
    pub generated_sentences: usize,
    pub score: Option<f64>,  // unified_test score if run
}

impl MultiSpace {
    pub fn bootstrap(
        &mut self,
        config: &BootstrapConfig,
        params: &EngineParams,
        strategy: &StrategyConfig,
    ) -> BootstrapResult;
}
```

### Algorithm: `bootstrap()`

```rust
fn bootstrap(&mut self, config, params, strategy) -> BootstrapResult {
    let mut results = Vec::new();
    
    for level in 0..config.max_iterations {
        // Step 1: Generate text from specified spaces
        let mut generated_sentences: Vec<String> = Vec::new();
        for space_name in &config.describe_spaces {
            let space = &self.spaces[space_name];
            // describe() every content word in this space
            for entry in &space.dictionary.entries {
                let content = &space.engine.content();
                if !content.contains(&entry.word) {
                    continue; // skip structural words
                }
                let desc = describe(
                    &entry.word, space.engine.space(),
                    &space.dictionary, space.engine.structural(),
                    content, params, strategy,
                );
                generated_sentences.extend(desc);
            }
        }
        
        // Step 2: For each evolving space, augment sentences and re-discover
        let mut level_result = LevelResult { level, ..default };
        
        for space_name in &config.describe_spaces {
            let space = &mut self.spaces[space_name];
            
            // Get original dictionary sentences
            let dict_sentences = extract_all_sentences(&space.dictionary);
            
            // Combine: dictionary + generated
            let all_sentences: Vec<String> = dict_sentences
                .into_iter()
                .chain(generated_sentences.iter().cloned())
                .collect();
            
            // Re-discover connectors from augmented corpus
            let old_connectors = space.engine.space().connectors.clone();
            let (new_connectors, _) = discover_connectors_from_sentences(
                &all_sentences, &space.dictionary, params, strategy,
            );
            
            // Diff: what's new, what's lost
            let new_patterns = diff_connectors(&old_connectors, &new_connectors);
            let lost_patterns = diff_connectors(&new_connectors, &old_connectors);
            
            level_result.new_connectors.extend(
                new_patterns.iter().map(|p| (space_name.clone(), p.clone()))
            );
            level_result.lost_connectors.extend(
                lost_patterns.iter().map(|p| (space_name.clone(), p.clone()))
            );
            
            // If new connectors found: re-run equilibrium
            if !new_patterns.is_empty() {
                space.engine.set_connectors(new_connectors);
                space.engine.re_equilibrate();
            }
        }
        
        level_result.generated_sentences = generated_sentences.len();
        
        // Step 3: Convergence check
        if level_result.new_connectors.is_empty() {
            results.push(level_result);
            return BootstrapResult {
                converged_at: level,
                levels: results,
                final_connectors: self.collect_connectors(),
            };
        }
        
        results.push(level_result);
    }
    
    BootstrapResult { converged_at: config.max_iterations, levels: results, .. }
}
```

### Key sub-functions

#### `discover_connectors_from_sentences()`

New function in `connector_discovery.rs`. Identical to `discover_connectors()` but takes pre-extracted sentences instead of extracting from dictionary. This avoids modifying the existing function signature.

```rust
pub fn discover_connectors_from_sentences(
    sentences: &[String],
    dictionary: &Dictionary,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> (Vec<Connector>, Vec<SentenceRelation>) {
    let (structural, content) = classify_word_roles(dictionary);
    let relations = extract_relations(sentences, dictionary, &structural, &content, params);
    // ... same frequency + uniformity pipeline as discover_connectors()
}
```

This is a small refactor: extract the shared pipeline into a helper, call it from both `discover_connectors()` and `discover_connectors_from_sentences()`.

#### `diff_connectors()`

Compare two connector sets, return patterns in `new` that aren't in `old`.

```rust
fn diff_connectors(old: &[Connector], new: &[Connector]) -> Vec<Vec<String>> {
    let old_patterns: HashSet<Vec<String>> = old.iter().map(|c| c.pattern.clone()).collect();
    new.iter()
        .filter(|c| !old_patterns.contains(&c.pattern))
        .map(|c| c.pattern.clone())
        .collect()
}
```

#### Engine methods needed

Small additions to `Engine` in `lib.rs`:

```rust
impl Engine {
    /// Replace the connector set and re-run equilibrium.
    pub fn set_connectors_and_retrain(&mut self, connectors: Vec<Connector>) {
        // Update space connectors
        // Re-run equilibrium with new force field
        // This reuses the existing train() pipeline
    }
}
```

## CLI INTERFACE

New flag: `--bootstrap <max_iterations>`

```bash
cargo run -p dafhne-eval -- \
  --spaces content:dictionaries/dict5.md,math:dictionaries/dict_math5.md,grammar:dictionaries/dict_grammar5.md,task:dictionaries/dict_task5.md,self:dictionaries/dict_self5.md \
  --test dictionaries/unified_test.md \
  --genome results_v11/best_genome.json \
  --bootstrap 3
```

Output per level:
```
=== Bootstrap Level 0 (baseline) ===
Content connectors: ["is", "is a", "can", "not"] (4)
Grammar connectors: ["is", "is a", "can"] (3)

=== Bootstrap Level 1 ===
Generated 127 sentences from describe()
Content: +1 new connector ["is not"] | -0 lost
Grammar: +0 new | -0 lost
Re-equilibrating CONTENT space...

=== Bootstrap Level 2 ===
Generated 134 sentences from describe()
Content: +0 new | -0 lost
Grammar: +0 new | -0 lost
Converged at level 2. No new connectors.

=== Final Score: 45/50 (no regression) ===
```

## TEST FILE

No new test questions needed. The bootstrap loop is validated by:

1. **No regression**: unified_test.md score at each level ≥ baseline (45/50)
2. **Connector diff**: at least 1 new connector discovered at Level 1
3. **Convergence**: loop terminates (doesn't diverge)
4. **Self-consistency**: describe output at Level N, when re-read as questions, produces same answers

### Self-consistency check

For each describe output sentence, verify it can be confirmed:
- "a dog is an animal" → "Is a dog an animal?" → Yes ✓
- "the sun is big" → "Is the sun big?" → Yes ✓
- "a dog is not a cat" → "Is a dog a cat?" → No ✓

If self-consistency drops below 90% at any level, the loop stops and reverts.

## EXPECTED OUTCOME

### What will likely happen

Based on how connector discovery works (frequency threshold + uniformity filter):

1. **Level 1 will find "is not" as a new connector** in CONTENT space. Dictionary text has "not big is small" patterns, but describe output has explicit "X is not Y" patterns (negation sentences). This increases the frequency of the `["is", "not"]` pattern above the minimum threshold.

2. **Level 1 may find "can make" or "can live"** as multi-word connectors, since describe output produces "a dog can make sound" and "a cat can live with a person" — patterns that repeat across multiple concepts.

3. **Level 2 will likely converge**. The describe output at Level 1 (with slightly different geometry) won't produce qualitatively different sentences. Same concepts, same definitions, same structure.

4. **Scores will be stable or slightly improve**. New connectors add force directions to the equilibrium, which may sharpen some distances. Unlikely to cause regression because the dictionary hasn't changed.

### What could go wrong

1. **No new connectors at all**: describe output is too similar to dictionary text. The same patterns appear, same frequencies. Loop is a no-op.
   - **Mitigation**: This is still informative — means ELI5 dictionaries are already saturated for connector discovery. Phase 15 (richer describe) is the key: property extraction adds sentences that dictionaries don't have.

2. **Spurious connectors**: describe output introduces high-frequency patterns that aren't real connectors (e.g., articles).
   - **Mitigation**: Uniformity filter already handles this. Spurious patterns are non-uniform.

3. **Regression from geometry change**: New connectors add force vectors that distort existing good distances.
   - **Mitigation**: Run unified_test at each level. Revert if score drops.

4. **Infinite loop / divergence**: Each level produces slightly different output, slightly different connectors, geometry oscillates.
   - **Mitigation**: max_iterations cap (default 5). Convergence = no new connectors.

## IMPLEMENTATION PLAN

### Phase A: Refactor connector discovery (~1 hour)

1. Extract `discover_connectors_from_sentences()` from `discover_connectors()`
2. Both functions call the same internal pipeline
3. Test: existing connector discovery produces identical results (regression check)

### Phase B: Engine re-equilibration method (~1 hour)

1. Add `set_connectors_and_retrain()` to Engine
2. This replaces the connector set in the GeometricSpace and re-runs the equilibrium loop
3. Test: re-equilibrating with same connectors produces same geometry

### Phase C: Bootstrap module (~2-3 hours)

1. Create `crates/dafhne-engine/src/bootstrap.rs`
2. Implement `bootstrap()` on MultiSpace
3. Add `--bootstrap` flag to dafhne-eval CLI
4. Test: run with `--bootstrap 1` and verify output format

### Phase D: Level 1 validation (~1-2 hours)

1. Run bootstrap Level 1
2. Log connector diffs
3. Run unified_test at Level 1
4. Compare scores: Level 0 vs Level 1
5. Run self-consistency check on describe output

### Phase E: Convergence testing (~1 hour)

1. Run bootstrap with max_iterations=5
2. Verify convergence (no new connectors after some level)
3. Log full connector evolution history
4. Verify no regression at any level

## SUCCESS CRITERIA

| Metric | Minimum | Target |
|--------|---------|--------|
| Loop runs without crash | Yes | Yes |
| At least 1 new connector at Level 1 | Yes | 2+ new |
| Convergence within 5 iterations | Yes | Within 3 |
| unified_test score at all levels | ≥ 43/50 | ≥ 45/50 |
| Self-consistency at all levels | ≥ 85% | ≥ 95% |
| No single-space regression | dict5 ≥ 18/20 | 20/20 |

### What counts as success

The loop runs, finds at least one new connector, doesn't regress, and converges. Even finding zero new connectors is informative (tells us ELI5 dictionaries are connector-saturated and we need richer input text).

## CODE CHANGES SCOPE

| File | Change |
|------|--------|
| `crates/dafhne-engine/src/bootstrap.rs` | **NEW**: Bootstrap loop module (~200-300 lines) |
| `crates/dafhne-engine/src/lib.rs` | Add `pub mod bootstrap;`, Engine::set_connectors_and_retrain() |
| `crates/dafhne-engine/src/connector_discovery.rs` | Extract `discover_connectors_from_sentences()` (refactor, not rewrite) |
| `crates/dafhne-eval/src/main.rs` | Add `--bootstrap` CLI flag |

**No changes to**: dafhne-core, dafhne-parser, dafhne-evolve, resolver.rs, multispace.rs, any dictionary, any test file

## KILL CRITERIA

- Bootstrap Level 1 causes unified_test regression below 40/50 → new connectors damage geometry
- Engine panics or hangs during re-equilibration → connector injection mechanism broken
- Self-consistency drops below 70% at any level → geometry is producing contradictory output
- Level 1 takes >10 minutes to complete → performance issue needs investigation first

## DESIGN DECISIONS LOG

### Decision 1: Connector enrichment, not dictionary growth
**Chosen**: Dictionaries are immutable. Only connectors evolve.
**Alternative**: Generate new dictionary entries from describe output.
**Reason**: Dictionary growth requires new definitions that satisfy closure. That's a generation problem (Phase 21). Connector enrichment requires no new definitions — it just finds more patterns in existing+generated text. Lower risk, cleaner separation.

### Decision 2: describe() as text generator, not a template engine
**Chosen**: Use existing describe() function as-is (with Phase 15 enhancements).
**Alternative**: Build a separate text generation module.
**Reason**: describe() already produces structured sentences from geometric knowledge. It IS the text generator. No need to build another one.

### Decision 3: Per-space connector evolution
**Chosen**: Each space's connectors evolve independently. CONTENT gets CONTENT-describe text, GRAMMAR gets GRAMMAR-describe text.
**Alternative**: Feed all describe text to all spaces.
**Reason**: Cross-contamination. GRAMMAR connector patterns from CONTENT sentences would be noise. Each space should learn from its own domain's generated text.

### Decision 4: New module, not inline in multispace.rs
**Chosen**: Separate `bootstrap.rs` module.
**Alternative**: Add bootstrap logic inside MultiSpace::resolve().
**Reason**: Bootstrap is an offline training step, not a query-time operation. It runs once before queries. Separate module keeps concerns clean.

### Decision 5: No MATH/TASK/SELF bootstrap
**Chosen**: Only CONTENT and GRAMMAR participate in the loop.
**Alternative**: Bootstrap all 5 spaces.
**Reason**: MATH describe output is trivial ("one is a number. one is not two."). TASK describes routing, not concepts. SELF describes capabilities. None of these produce interesting new connector patterns. CONTENT and GRAMMAR are where the linguistic action is.

## DEPENDENCY ON PHASE 15

Phase 15 (rich describe mode) is critical because it adds property extraction:

**Without Phase 15** (current describe):
```
a dog is an animal.
a dog can make sound.
a dog is not a cat.
```

**With Phase 15** (target describe):
```
a dog is an animal.
a dog can make sound.
a dog can live with a person.
a dog is not a cat.
a dog is not a ball.
the sun is big.
the sun is hot.
the sun is up.
the sun makes things hot.
```

Phase 15 output has more diverse sentence structures, more explicit property statements, and more topic-word pairs for connector discovery. Without Phase 15, the bootstrap loop may find zero new connectors because the describe output is too sparse.

## THE BIGGER PICTURE

Phase 19 tests the fundamental hypothesis: **can a geometric comprehension engine improve itself by reading its own output?**

The mechanism is deliberately simple: generate text → discover patterns → update geometry. No gradient descent, no loss function, no training signal beyond "find repeating patterns in what I wrote."

If this works (even marginally), it opens Phase 20: per-space parameter evolution, where the genetic algorithm tunes equilibrium parameters per-space using bootstrap quality as the fitness signal.

If it doesn't work, the failure mode tells us exactly where the bottleneck is: either describe() doesn't produce rich enough text (fix: richer dictionaries), or connector discovery is already saturated (fix: new discovery mechanisms), or the equilibrium is insensitive to connector changes (fix: force field architecture).

Every outcome is informative. That's the point.
