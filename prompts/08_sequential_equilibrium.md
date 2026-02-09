# PROMPT 08 — Sequential Equilibrium: From Optimization to Plasticity

## PREAMBLE

DAFHNE is a geometric comprehension engine that reads closed dictionaries and builds N-dimensional spaces where words are points. Until now, the geometry has been shaped by a **genetic algorithm (GA)** that tunes ~15 parameters over 50 generations × 50 population. The GA finds good configurations, but the system is not **plastic** — it doesn't adapt its geometry as it reads. The parameters are optimized externally, not discovered from the text itself.

This is the single biggest architectural limitation. DAFHNE currently demonstrates that a geometric configuration EXISTS that encodes comprehension. It does not demonstrate that text PRODUCES comprehension through geometry. That's a fundamentally different — and far stronger — claim.

### What We Have (from prompts 01-07)

| Dict | Words | Best Fitness | Overfitting Gap |
|------|-------|--------------|-----------------|
| dict5 | 50 | 1.0000 | 0.00 |
| dict12 | ~400 | 0.7500 | 0.00 |
| dict18 | ~2000 | (from prompt 07) | (from prompt 07) |

Three data points on the scaling curve, all produced by GA optimization. The grammar-as-regularizer finding hints at passive plasticity: more text = better geometry without parameter changes. This prompt makes that active.

## PROJECT STRUCTURE

```
D:\workspace\projects\dafhne\
├── crates/
│   ├── dafhne-core/         Data structures, GeometricSpace, Answer, traits
│   ├── dafhne-parser/        Dictionary/test/grammar parsing
│   ├── dafhne-engine/        Force field + resolver
│   ├── dafhne-eval/          Fitness scoring
│   └── dafhne-evolve/        Genetic algorithm (THE THING WE'RE REPLACING)
├── dictionaries/
│   ├── dict5.md, dict5_test.md
│   ├── dict12.md, dict12_test.md
│   ├── dict18.md, dict18_test.md       (from prompt 07)
│   ├── grammar5.md, grammar5_test.md
│   └── grammar18.md                    (from prompt 07)
├── prompts/
├── results_v11/, results_v12/
└── RECAP.md
```

**Key files for this phase:**
- `crates/dafhne-engine/src/force_field.rs` — where word positions are computed
- `crates/dafhne-engine/src/engine.rs` — orchestration
- NEW: `crates/dafhne-engine/src/equilibrium.rs` — the sequential learning algorithm

---

## THE CORE IDEA

Replace the GA's batch optimization with a **sequential equilibrium process**:

```
For each definition D in the dictionary (read in order):
    1. Parse D into (word, definition_words, connectors_found)
    2. Place `word` in the geometric space (if not already placed)
    3. Apply forces FROM this definition:
       - Push `word` toward words in its definition (semantic proximity)
       - Push `word` along connector axes ("is a" → taxonomic axis)
       - Push `word` away from words connected via "not" (antonym repulsion)
    4. Let ALL currently-placed words settle (local relaxation, N steps)
    5. Measure local energy (sum of force magnitudes on all words)
    6. If energy < threshold: equilibrium reached, read next definition
       If energy > threshold after max steps: log instability, continue anyway
```

This is analogous to **simulated annealing** or **elastic network relaxation**: each new piece of information perturbs the system, the system relaxes to a new equilibrium, and the geometry incrementally encodes more comprehension.

### Why This Might Work

The grammar-as-regularizer result is the key evidence. When grammar text was added alongside definitions, the geometry improved WITHOUT parameter changes. The grammar provided additional constraints — more force vectors — that prevented the space from degenerating. That's exactly what sequential equilibrium does: each definition adds constraints, and the space must satisfy all of them simultaneously.

The GA found that the best strategies (Gravitational, MutualInformation, Spherical) all share a property: they compute forces from word co-occurrence. Sequential equilibrium doesn't change WHAT the forces are. It changes WHEN they're applied — incrementally instead of all-at-once.

### Why This Might Fail

The GA optimizes globally. Sequential processing is path-dependent: the ORDER of definitions affects the final geometry. "Dog" defined before "animal" might settle differently than "animal" defined before "dog". If the final geometry depends heavily on reading order, the system is fragile.

Mitigation: test with multiple random orderings. If results are consistent (±0.05 fitness), order-dependence is negligible. If results vary wildly, add a second pass (re-read all definitions with the geometry as starting point).

---

## ARCHITECTURE

### New Module: `equilibrium.rs`

```rust
pub struct SequentialEquilibrium {
    /// The geometric space being built incrementally
    space: GeometricSpace,
    
    /// Force parameters — these are FIXED, not evolved
    /// Use the best values found by GA across dict5/12/18 as starting point
    params: EquilibriumParams,
    
    /// Energy history for convergence tracking
    energy_log: Vec<f64>,
    
    /// Order in which definitions were processed
    processing_order: Vec<String>,
}

pub struct EquilibriumParams {
    /// How much a new definition perturbs existing positions
    pub perturbation_strength: f64,      // start: 0.1
    
    /// How quickly forces decay during relaxation
    pub damping: f64,                     // start: 0.95
    
    /// Maximum relaxation steps per definition
    pub max_relax_steps: usize,           // start: 50
    
    /// Energy threshold for equilibrium
    pub energy_threshold: f64,            // start: 0.001
    
    /// Learning rate: how much word positions move per force application
    pub learning_rate: f64,               // start: 0.01
    
    /// Number of full re-read passes
    pub passes: usize,                    // start: 3
    
    /// Whether to shuffle order between passes
    pub shuffle_between_passes: bool,     // start: true
}
```

### The Algorithm in Detail

```rust
impl SequentialEquilibrium {
    pub fn build_from_dictionary(
        &mut self,
        dictionary: &Dictionary,
        grammar: Option<&GrammarText>,
        connectors: &[Connector],
    ) -> GeometricSpace {
        // Discover connectors first (same as current pipeline)
        // This is a batch operation and that's fine — connectors are 
        // structural, not content
        
        for pass in 0..self.params.passes {
            let mut entries: Vec<_> = dictionary.entries().collect();
            
            if self.params.shuffle_between_passes && pass > 0 {
                entries.shuffle(&mut thread_rng());
            }
            
            let lr = self.params.learning_rate 
                     * (1.0 / (1.0 + pass as f64 * 0.5));  // decay LR per pass
            
            for entry in &entries {
                self.process_definition(entry, connectors, lr);
            }
            
            // After all definitions: process grammar text
            if let Some(grammar) = grammar {
                for sentence in grammar.sentences() {
                    self.process_sentence(sentence, connectors, lr);
                }
            }
            
            self.energy_log.push(self.compute_total_energy());
        }
        
        self.space.clone()
    }
    
    fn process_definition(
        &mut self,
        entry: &DictEntry,
        connectors: &[Connector],
        learning_rate: f64,
    ) {
        let word = &entry.word;
        
        // Initialize position if new word
        if !self.space.has_word(word) {
            self.initialize_word(word, entry, connectors);
        }
        
        // Compute forces from this definition
        let forces = self.compute_definition_forces(word, entry, connectors);
        
        // Apply forces to the word and its neighbors
        self.apply_forces(&forces, learning_rate);
        
        // Local relaxation: let nearby words settle
        self.relax(self.params.max_relax_steps);
    }
    
    fn initialize_word(
        &mut self,
        word: &str,
        entry: &DictEntry,
        connectors: &[Connector],
    ) {
        // Smart initialization: place near centroid of already-placed 
        // definition words (if any exist in space). This gives a better
        // starting position than random.
        let def_words = tokenize(&entry.definition);
        let placed: Vec<_> = def_words.iter()
            .filter(|w| self.space.has_word(w))
            .collect();
        
        if placed.is_empty() {
            // No context yet — random initialization
            self.space.place_random(word);
        } else {
            // Place at centroid of known definition words + small noise
            let centroid = self.space.centroid(&placed);
            self.space.place_at(word, &add_noise(&centroid, 0.1));
        }
    }
    
    fn compute_definition_forces(
        &self,
        word: &str,
        entry: &DictEntry,
        connectors: &[Connector],
    ) -> Vec<Force> {
        let mut forces = Vec::new();
        let def_words = tokenize(&entry.definition);
        
        for def_word in &def_words {
            if !self.space.has_word(def_word) { continue; }
            
            // Base attraction: word is related to words in its definition
            forces.push(Force::attract(word, def_word, self.params.perturbation_strength));
            
            // Connector-specific forces
            for conn in connectors {
                if definition_contains_pattern(&def_words, &conn.pattern, word, def_word) {
                    // This definition links word to def_word via this connector
                    // Apply directional force along connector axis
                    forces.push(Force::along_axis(
                        word, def_word, &conn.force_direction,
                        self.params.perturbation_strength * conn.weight,
                    ));
                }
            }
            
            // Negation: "not X" in definition → repel from X
            if preceded_by_not(&def_words, def_word) {
                forces.push(Force::repel(word, def_word, self.params.perturbation_strength * 2.0));
            }
        }
        
        forces
    }
    
    fn relax(&mut self, max_steps: usize) {
        for step in 0..max_steps {
            let energy = self.apply_all_current_forces();
            if energy < self.params.energy_threshold {
                break;
            }
            // Damp forces each step
            self.dampen_forces(self.params.damping);
        }
    }
}
```

### Key Design Decisions

1. **Connector discovery stays batch.** Connectors are structural patterns ("is a", "can", "not"). They don't change as you read more definitions. Discover them once from the full corpus, then use them during sequential processing. This is analogous to knowing the grammar before reading sentences.

2. **Multiple passes with decaying learning rate.** First pass: rough placement. Second pass: refinement. Third pass: fine-tuning. Like annealing — big moves first, small adjustments later.

3. **Smart initialization.** New words placed near the centroid of their definition words. This exploits the CURRENT state of the space and gives the relaxation a much better starting point than random.

4. **Grammar processed AFTER definitions each pass.** Grammar acts as regularizer — it applies additional constraints that prevent drift. Same role as before, but now applied incrementally.

---

## FIXED PARAMETERS VS GA PARAMETERS

The whole point of this phase is to REDUCE dependence on external optimization. The EquilibriumParams should be:

- **Derived from physics** where possible (damping, learning rate, energy threshold)
- **Stable across dict sizes** — same params for dict5, dict12, dict18
- **Few** — ideally 5-7, not the 15+ the GA optimizes

Start with sensible defaults. If the system works on dict5 with defaults, test on dict12 and dict18 WITHOUT tuning. If it degrades, the sequential process isn't truly plastic.

The ultimate test: **can you set the parameters ONCE and have them work on any closed dictionary?** If yes, the system is plastic. If no, you still need the GA, just with fewer parameters.

---

## TESTING PROTOCOL

### Step 1: Dict5 Baseline

Run sequential equilibrium on dict5 with grammar5. Compare against GA-optimized result (20/20, fitness 1.0).

```bash
cargo run -p dafhne-engine -- \
    --mode equilibrium \
    --dict dictionaries/dict5.md \
    --grammar dictionaries/grammar5.md \
    --test dictionaries/dict5_test.md
```

Acceptable: ≥ 17/20. The sequential process may not match the GA's perfection, but it should be close.

### Step 2: Dict12 WITHOUT Parameter Changes

Same params as dict5. Run on dict12.

Acceptable: ≥ 13/20. If this matches or beats the GA cross-validation result (15/20), that's remarkable — it means the sequential process found a geometry at LEAST as good as the one the GA spent 50×50 evaluations finding.

### Step 3: Dict18 WITHOUT Parameter Changes

Same params. Run on dict18.

This is the real test. If the sequential process produces comparable fitness to the GA on a vocabulary it was never tuned for, the plasticity claim is validated.

### Step 4: Order Sensitivity

Run dict5 with 10 different random orderings of definitions. Record fitness for each.

- If variance < 0.05: order-independent. Strong result.
- If variance 0.05-0.15: mildly order-dependent. Acceptable with multiple passes.
- If variance > 0.15: strongly order-dependent. Need to investigate which orderings fail and why.

### Step 5: Convergence Visualization

Plot energy_log across passes for each dictionary. Expected shape:
- Pass 1: high energy, large drops
- Pass 2: moderate energy, gradual decline
- Pass 3: low energy, minimal change

If energy doesn't decline monotonically, the relaxation is unstable. Check damping and learning rate.

---

## INTEGRATION WITH EXISTING CODE

The sequential equilibrium module must produce a `GeometricSpace` identical in type to what the force field currently produces. The resolver doesn't care HOW the space was built — it only queries distances and positions.

```
Current pipeline:
  Parser → Connectors → ForceField(params from GA) → GeometricSpace → Resolver

New pipeline:
  Parser → Connectors → SequentialEquilibrium(fixed params) → GeometricSpace → Resolver
```

Both pipelines must be available. Add a `--mode` flag to the engine binary:
- `--mode forcefield` (default, existing behavior)
- `--mode equilibrium` (new)

Do NOT remove the force field code. It's the baseline.

---

## WHAT NOT TO DO

- Do NOT evolve the EquilibriumParams with the GA. That defeats the purpose. If you need to tune them, do it by hand on dict5 only, then FREEZE.
- Do NOT change the resolver. This phase is about HOW the space is built, not how it's queried.
- Do NOT skip the order-sensitivity test. It's the most important diagnostic.
- Do NOT add word2vec or any external embedding. The positions must come from the dictionary text alone.
- Do NOT over-engineer the relaxation. Start with simple gradient descent on forces. Add sophistication only if it's unstable.

## SUCCESS CRITERIA

| Metric | Minimum | Target | Stretch |
|--------|---------|--------|---------|
| Dict5 fitness (equilibrium) | > 0.80 | > 0.90 | 1.00 |
| Dict12 fitness (same params) | > 0.60 | > 0.70 | > 0.75 |
| Dict18 fitness (same params) | > 0.45 | > 0.60 | > 0.70 |
| Order variance (dict5, 10 runs) | < 0.15 | < 0.05 | < 0.02 |
| Parameter count | ≤ 10 | ≤ 7 | ≤ 5 |
| Params tuned per dict | 0 | 0 | 0 |

The CRITICAL metric is "params tuned per dict = 0". If the same parameters work across all three dictionaries, the system is plastic. Everything else is secondary.

## OUTPUT

1. `crates/dafhne-engine/src/equilibrium.rs` — the sequential learning module
2. Updated `engine.rs` with `--mode` flag
3. Fitness table: equilibrium vs GA for all three dicts
4. Order-sensitivity report (10 runs on dict5)
5. Convergence plots (energy vs pass for each dict)
6. Analysis: where sequential equilibrium beats/loses to GA, and why