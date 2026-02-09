# PROMPT 03 — Evolutionary Self-Improvement System

## CONTEXT

You are building the evolution layer for DAFHNE (Definition-Anchored Force-field Heuristic Network Engine). DAFHNE v0.1 (built in Prompt 02) is a geometric comprehension engine that:

1. Reads a closed dictionary (dict5: 50 words)
2. Discovers connectors from text statistically
3. Builds an N-dimensional space where words are points and connectors are force operators
4. Answers questions via geometric proximity queries
5. Scores itself: `fitness = 0.5 * accuracy + 0.5 * honesty`

v0.1 has hardcoded parameters. This prompt creates the system that EVOLVES those parameters — and eventually evolves the algorithms themselves.

## THE EVOLUTION FRAMEWORK

The project already has a proven evolutionary architecture from a prior character-level LM experiment. That system achieved 95.5/100 fitness through 5 generations of evolution. We reuse the same pattern:

```
loop {
    1. Generate population of candidates (varied parameters/strategies)
    2. Train each candidate on dict5
    3. Test each candidate on dict5_test (20 questions)
    4. Score fitness
    5. Select top performers
    6. Breed next generation (crossover + mutation)
    7. Track lineage and progress
}
```

## WHAT EVOLVES

### Tier 1: Parameters (evolve first)
These are numeric values that control the engine's behavior:

```rust
struct EngineGenome {
    // Space geometry
    dimensions: usize,           // 4..32
    
    // Force dynamics
    force_magnitude: f64,        // 0.01..1.0
    force_decay: f64,            // 0.5..0.99 per pass
    learning_passes: usize,      // 1..50
    
    // Connector discovery
    connector_min_frequency: usize,  // 1..10
    connector_max_length: usize,     // 1..4 words
    
    // Answer thresholds
    yes_threshold: f64,          // 0.05..0.5
    no_threshold: f64,           // 0.3..0.95
    
    // Negation handling
    negation_inversion: f64,     // -1.0..1.0 (how much "not" inverts)
    
    // Force application
    bidirectional_force: f64,    // 0.0..1.0 (how much reverse direction gets)
}
```

### Tier 2: Strategies (evolve after Tier 1 stabilizes)
These are algorithmic choices encoded as enum variants:

```rust
enum ForceFunction {
    Linear,              // constant push
    InverseDistance,     // weaker when already close
    Gravitational,       // F = m / d^2
    Spring,              // F = k * (rest_length - d)
}

enum ConnectorDetection {
    FrequencyOnly,       // most common sequences between entities
    PositionalBias,      // weight by position in sentence
    MutualInformation,   // statistical association strength
}

enum SpaceInitialization {
    Random,              // uniform random in [-1, 1]^N
    Spherical,           // on unit sphere surface
    FromConnectors,      // initial position based on connector co-occurrence
}

enum MultiConnectorHandling {
    FirstOnly,           // apply only the first connector found
    Sequential,          // apply all connectors in order
    Weighted,            // weight by connector strength
    Compositional,       // compose connector forces into one operation
}

enum NegationModel {
    Inversion,           // flip direction vector
    Repulsion,           // push away instead of toward  
    AxisShift,           // move to opposite end of same axis
    SeparateDimension,   // "not" gets its own dimension
}
```

### Tier 3: Architecture (evolve last, research frontier)
This is where the system modifies its own structure:

- Number of force application phases (single pass vs multi-phase with different strategies)
- Whether connectors interact (e.g., "is a" and "is not" share an axis)
- Whether the space is flat (Euclidean) or curved (hyperbolic — better for hierarchies)
- Whether words have single points or probability distributions (fuzzy positions)
- Whether to use the existing char-level LM as a preprocessing layer

Tier 3 is NOT implemented in this prompt. Document the interface so it can be added later.

## GENOME REPRESENTATION

```rust
struct Genome {
    // Tier 1: Parameters
    params: EngineGenome,
    
    // Tier 2: Strategy choices
    force_function: ForceFunction,
    connector_detection: ConnectorDetection,
    space_init: SpaceInitialization,
    multi_connector: MultiConnectorHandling,
    negation_model: NegationModel,
    
    // Metadata
    id: u64,
    generation: usize,
    parent_ids: Vec<u64>,
    fitness: Option<f64>,
}
```

## EVOLUTION OPERATORS

### Mutation
For Tier 1 parameters:
- Gaussian perturbation: `value += normal(0, sigma)` where sigma scales with the parameter range
- Boundary enforcement: clamp to valid range
- Mutation rate: 0.1..0.3 per parameter (evolved itself over generations)

For Tier 2 strategies:
- Random reassignment with probability 0.05..0.15 per strategy
- Some strategies are more likely to mutate to "nearby" variants

### Crossover
- Uniform crossover for parameters: each param from parent A or B with 50% probability
- Strategy inheritance: each strategy choice from parent A or B independently
- Optionally: arithmetic crossover for parameters: `child = alpha * parent_a + (1-alpha) * parent_b`

### Selection
- Tournament selection: pick 3 random, keep the best
- Elitism: top 2 always survive to next generation
- Population size: 20..50 per generation

## FITNESS FUNCTION

### Primary Fitness (dict5)
```
primary_fitness = 0.5 * accuracy + 0.5 * honesty

accuracy = correct_answers / total_answerable_questions  (Q01-Q14, Q19-Q20)
honesty  = correct_idk / total_unknowable_questions        (Q15-Q18)
```

### Cross-validation Fitness (dict12)
After a candidate scores well on dict5, test it on dict12 with a separate set of 20 questions (to be created for dict12). This measures generalization.

```
cross_fitness = 0.5 * accuracy_12 + 0.5 * honesty_12
```

### Combined Fitness
```
final_fitness = 0.7 * primary_fitness + 0.3 * cross_fitness
```

The 70/30 split ensures the system optimizes for dict5 first but doesn't overfit.

### Bonus Metrics (tracked but not in fitness)
- **Connector quality**: how many discovered connectors correspond to meaningful linguistic relations?
- **Space interpretability**: are related words closer than random pairs? (measured by comparing intra-category vs inter-category distances)
- **Transitivity score**: does the space support transitive reasoning? (if A near B and B near C, is A reasonably near C?)
- **Convergence speed**: how many passes until the space stabilizes?

## IMPLEMENTATION

### New crate: dafhne-evolve

Add to the workspace:

```
dafhne/
├── crates/
│   ├── dafhne-core/
│   ├── dafhne-parser/
│   ├── dafhne-engine/
│   ├── dafhne-eval/
│   └── dafhne-evolve/     ← NEW
│       ├── src/
│       │   ├── lib.rs
│       │   ├── genome.rs
│       │   ├── population.rs
│       │   ├── operators.rs    (mutation, crossover, selection)
│       │   ├── runner.rs       (main evolution loop)
│       │   ├── lineage.rs      (track ancestry)
│       │   └── analysis.rs     (space visualization, bonus metrics)
│       └── Cargo.toml
└── results/                     ← NEW (evolution output)
    ├── gen_001/
    │   ├── population.json
    │   ├── best_genome.json
    │   ├── fitness_stats.json
    │   └── space_dump.json  (word positions for the best candidate)
    ├── gen_002/
    └── STATUS.md            (auto-updated summary)
```

### Evolution Runner

```rust
struct EvolutionConfig {
    population_size: usize,     // 20-50
    generations: usize,         // 10-100
    elitism_count: usize,       // 2
    tournament_size: usize,     // 3
    mutation_rate: f64,         // 0.2
    crossover_rate: f64,        // 0.7
    dict5_path: PathBuf,
    dict5_test_path: PathBuf,
    dict12_path: Option<PathBuf>,
    dict12_test_path: Option<PathBuf>,
    results_dir: PathBuf,
}

fn evolve(config: &EvolutionConfig) -> Genome {
    let mut population = initialize_population(config.population_size);
    
    for gen in 0..config.generations {
        // Evaluate all candidates
        for genome in &mut population {
            let engine = build_engine_from_genome(genome);
            let dict5 = parse_dictionary(&config.dict5_path);
            let tests = parse_tests(&config.dict5_test_path);
            
            engine.train(&dict5);
            let primary = evaluate(&engine, &tests);
            
            let cross = if let Some(ref d12) = config.dict12_path {
                let dict12 = parse_dictionary(d12);
                let tests12 = parse_tests(config.dict12_test_path.as_ref().unwrap());
                engine.train(&dict12);  // retrain on larger dict
                Some(evaluate(&engine, &tests12))
            } else {
                None
            };
            
            genome.fitness = Some(compute_final_fitness(primary, cross));
        }
        
        // Sort, select, breed
        population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        save_generation_results(gen, &population, &config.results_dir);
        print_generation_summary(gen, &population);
        
        let parents = select_parents(&population, config);
        population = breed_next_generation(&parents, config);
    }
    
    population.into_iter().max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap()).unwrap()
}
```

### STATUS.md Auto-Generation

After each generation, update `results/STATUS.md` with:

```markdown
# DAFHNE Evolution Status

## Current Generation: N

### Best Fitness: X.XX
### Population Average: X.XX
### Best Genome ID: XXXX

### Top 5 Parameters:
| Parameter | Value | Trend |
|-----------|-------|-------|
| dimensions | 12 | ↑ |
| force_magnitude | 0.08 | ↓ |
| ... | ... | ... |

### Strategy Distribution:
| Strategy | Most Common | Runner-up |
|----------|-------------|----------|
| force_function | Spring (60%) | Gravitational (25%) |
| negation_model | AxisShift (45%) | Inversion (30%) |
| ... | ... | ... |

### Question Accuracy Breakdown:
| Question | Best Candidate | Population % Correct |
|----------|---------------|---------------------|
| Q01: Is a dog an animal? | ✓ | 95% |
| Q06: Is a dog a thing? | ✓ | 45% |
| Q15: What color is a dog? | I don't know ✓ | 20% |
| ... | ... | ... |

### Lineage:
```
Gen 0: Random init → best 0.25
Gen 1: ID-0042 (Spring + AxisShift) → best 0.35
Gen 2: ID-0107 (child of 0042+0039) → best 0.48
...
```

### Key Discoveries:
- Gen 3: Spring force function dominates → relationships behave like elastic connections
- Gen 5: AxisShift negation outperforms Inversion → "not" needs its own geometric treatment
- Gen 7: 12 dimensions consistently beats 8 → more axes needed for orthogonal properties
```

## SELF-IMPROVEMENT PROTOCOL

The evolution system has three feedback loops:

### Loop 1: Parameter Evolution (automatic)
Standard genetic algorithm. Runs without intervention. Finds optimal numeric settings.

### Loop 2: Strategy Evolution (automatic)
Evolves which algorithmic approach works best. Runs alongside Loop 1.

### Loop 3: Architecture Discovery (semi-automatic)
This is where the system identifies WHAT SHOULD CHANGE NEXT. After each generation:

1. **Bottleneck Analysis**: Which questions do ALL candidates get wrong? This reveals architectural limitations, not parameter problems.
2. **Correlation Analysis**: Which parameters/strategies have the highest correlation with fitness? This reveals what matters.
3. **Convergence Detection**: If fitness plateaus for 3+ generations, the system should:
   - Log the plateau
   - Increase mutation rate temporarily (simulated annealing)
   - Flag for human review: "Fitness stuck at X.XX. Current architecture may be limited. Bottleneck questions: [list]. Consider: [suggestions]"
4. **Suggestion Generation**: Based on bottleneck analysis, output concrete suggestions:
   - "All candidates fail Q06 (transitive reasoning). Consider: adding multi-hop force propagation."
   - "Negation questions (Q11-Q14) have high variance. Consider: dedicated negation dimension."
   - "Honesty score stuck at 0.5. Consider: confidence calibration layer."

These suggestions are written to `results/SUGGESTIONS.md` for human review. The human then decides whether to implement new Tier 3 capabilities.

## SPACE ANALYSIS TOOLS

For the best candidate of each generation, produce:

### 1. Distance Matrix
All pairwise distances between the 50 dict5 words. Output as CSV.

### 2. Nearest Neighbors
For each word, its 5 nearest neighbors with distances. Reveals what the space "thinks" is similar.

### 3. Axis Analysis
For each discovered connector, show which dimension(s) it primarily operates on. Reveals whether connectors have learned orthogonal axes.

### 4. Cluster Report
Group words by proximity (simple k-means or threshold-based). Compare to expected categories (animals, properties, actions, etc.).

### 5. Transitivity Check
For chains like dog→animal→thing: measure whether `dist(dog, thing) < dist(dog, random_word)`. Reports transitivity success rate.

## IMPLEMENTATION REQUIREMENTS

- Pure Rust, no ML libraries
- Reproducible: seed the RNG, log seeds
- Parallel evaluation: use rayon for population evaluation
- JSON serialization for all results (serde + serde_json)
- Results directory auto-created
- Graceful interruption: save state on Ctrl+C, resume capability
- Memory efficient: don't keep all generations in memory

## COMMAND LINE INTERFACE

```bash
# Run evolution
cargo run --release -p dafhne-evolve -- \
    --dict5 dictionaries/dict5.md \
    --test5 dictionaries/dict5_test.md \
    --population 30 \
    --generations 20 \
    --results results/

# Analyze a specific generation
cargo run -p dafhne-evolve -- analyze results/gen_015/

# Run best genome on a specific dictionary
cargo run -p dafhne-evolve -- run-best results/ --dict dictionaries/dict12.md

# Resume interrupted evolution
cargo run --release -p dafhne-evolve -- --resume results/
```

## SUCCESS CRITERIA

### Minimum Viable:
- Evolution runs for 10 generations without crashing
- Fitness improves from generation 0 to generation 10
- STATUS.md is generated and readable
- Best genome achieves fitness > 0.5 on dict5

### Target:
- Fitness > 0.7 on dict5 within 20 generations
- Cross-validation on dict12 shows > 0.4 fitness
- Clear convergence toward specific strategies
- Meaningful suggestions generated for architectural improvements

### Stretch:
- Fitness > 0.85 on dict5
- Discovered connectors align with linguistic intuition
- Geometric space shows interpretable structure
- Transitive reasoning works for 2+ hop chains

## WHAT NOT TO DO

- Do NOT evolve the fitness function itself (that's fixed)
- Do NOT allow infinite parameter ranges (all bounded)
- Do NOT evaluate dict12 until dict5 fitness > 0.5
- Do NOT add Tier 3 architecture evolution in this version
- Do NOT optimize for speed at the cost of analysis quality
- Do NOT create mock/stub implementations

## ATTACHED FILES

- All files from Prompt 02 (the working engine)
- `dict5.md`, `dict5_test.md`, `dict12.md`

## OUTPUT

Produce:
1. All Rust source files for `dafhne-evolve` crate
2. Updated workspace `Cargo.toml`
3. Updated `README.md` with evolution instructions
4. `results/STATUS.md` template
5. `results/SUGGESTIONS.md` template
