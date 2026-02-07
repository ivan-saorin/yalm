# PROMPT 02 — Geometric Comprehension Engine v0.1

## CONTEXT

You are building the first version of YALM (Yet Another Language Model), a system that reads a closed dictionary and builds geometric knowledge representations WITHOUT any predefined rules, ontology, or parsing grammar. Everything emerges from the text.

This is NOT a traditional NLP pipeline. There is no tokenizer, no POS tagger, no dependency parser, no predefined relation types. The system reads character streams, discovers structure, and builds a geometric space where proximity encodes meaning.

## THE CORE IDEA

Each word becomes a point in an N-dimensional space. Sentences MOVE points.

```
Initial state: all 50 words randomly positioned in some low-dim space

System reads: "Charlie is a dog"
  - It doesn't know what "is a" means
  - But it notices a PATTERN: [word1] [connector] [word2]
  - It moves Charlie TOWARD dog by some amount
  - The connector "is a" becomes a learned FORCE TYPE

System reads: "A dog is an animal"
  - Same connector pattern "is a"
  - Moves dog TOWARD animal with the SAME force type

After reading the whole dictionary:
  - Words that share connectors cluster
  - Words connected by the SAME connector pattern form chains
  - Charlie is near dog is near animal — NOT because IS-A was coded
    but because "is a" pushed them along the same axis
```

The critical difference from word2vec: this captures **directional force** — "Charlie is a dog" and "a dog is Charlie" would apply different forces because word order matters. Connector words become learned operators that define HOW to move points.

Questions become geometric queries:
- "Is X a Y?" → check proximity along relevant axis
- "What is X?" → find nearest neighbor along category axis
- "I don't know" → no proximity above threshold on any axis

Transitive reasoning is FREE (geometric proximity is inherently transitive). Honesty ("I don't know") is FREE (no proximity = no answer).

## ARCHITECTURE

The system has these components:

### 1. Dictionary Parser
Reads a closed dictionary in markdown format. Extracts:
- Entry words
- Definition text (as character stream)
- Usage examples (as character streams)

Input: `dict5.md` or `dict12.md` (attached)
Output: Vec<DictionaryEntry> where each entry has word, definition, examples

### 2. Word Boundary Detector
Finds word boundaries in text. For the MVP, this can be simple whitespace splitting + punctuation stripping. Later versions may use the existing character-level LM.

Input: raw text string
Output: Vec<String> of tokens

### 3. Connector Discovery
The system must discover which word sequences act as "connectors" between entities. It does NOT know in advance what connectors exist.

Approach: Statistical co-occurrence analysis.
- A **connector candidate** is a short sequence (1-3 words) that frequently appears BETWEEN dictionary entry words.
- High-frequency sequences between entries are likely structural connectors.
- Examples that should emerge: "is a", "is", "can", "has", "not", "in", "of", "with"

The system should discover these from the text, not be given them.

Input: all definition texts + examples
Output: Vec<Connector> with frequency, typical left/right word types

### 4. Force Field Builder (THE CORE)
This is where geometry happens.

- Each dictionary word gets a position vector in N-dimensional space (start random)
- Each discovered connector gets a **force operator** — a transformation that defines how to move the left word toward/away from the right word
- The system reads every sentence and applies forces:
  - For "dog is an animal": apply the "is" force from dog toward animal
  - For "not big is small": apply the "not" force + "is" force (negation inverts direction?)
  - For "a thing that lives": apply the "that" connector linking thing to lives

Force operators are NOT predefined. Each connector learns:
- A direction vector (which axis to push along)
- A magnitude (how far to push)
- Optionally: whether it's attractive or repulsive

After processing all dictionary text, the space should have:
- Entities clustered by category (dog near cat near animal)
- Properties aligned on axes (hot/cold on one axis, big/small on another)
- Actions grouped (move/eat/feel near each other if used similarly)

### 5. Question Resolver
Takes a question, decomposes it using the same connector discovery, and performs geometric lookup.

Question types (discovered, not coded):
- "Is X a Y?" → pattern [word] [connector] [word] → check distance along connector axis
- "What is X?" → pattern [what] [connector] [word] → find nearest word along connector axis
- "Can X do Y?" → pattern [word] [connector] [word] → check distance along "can" axis

Answer logic:
- Distance below threshold on relevant axis → "Yes" or the word
- Distance above threshold with negation axis signal → "No"
- No relevant axis signal at all → "I don't know"

Threshold is a tunable parameter (evolved later).

### 6. Fitness Evaluator
Runs the test questions from `dict5_test.md` and scores:

```
fitness = 0.5 * accuracy + 0.5 * honesty

accuracy = correct_yes_no_answers / total_yes_no_questions
honesty  = correct_i_dont_know / total_unknowable_questions
```

## IMPLEMENTATION LANGUAGE

Rust. The project already has a Cargo workspace at `D:\workspace\projects\yalm`.

## FILE STRUCTURE

```
yalm/
├── Cargo.toml          (workspace)
├── dictionaries/
│   ├── dict5.md
│   ├── dict5_test.md
│   └── dict12.md
├── crates/
│   ├── yalm-core/       (data structures, traits, geometry)
│   ├── yalm-parser/     (dictionary + question parsing)
│   ├── yalm-engine/     (force field builder + question resolver)
│   └── yalm-eval/       (fitness evaluation + test runner)
└── prompts/
```

## IMPLEMENTATION REQUIREMENTS

### yalm-core
```rust
// Key types
struct WordPoint {
    word: String,
    position: Vec<f64>,   // N-dimensional position
}

struct Connector {
    pattern: Vec<String>,  // e.g., ["is", "a"] or ["can"]
    force_direction: Vec<f64>,  // learned axis
    magnitude: f64,
    frequency: usize,
}

struct GeometricSpace {
    dimensions: usize,
    words: HashMap<String, WordPoint>,
    connectors: Vec<Connector>,
}

enum Answer {
    Yes,
    No,
    IDontKnow,
    Word(String),
}

trait Comprehend {
    fn train(&mut self, dictionary: &Dictionary);
    fn query(&self, question: &str) -> Answer;
    fn distance(&self, word_a: &str, word_b: &str, connector: &Connector) -> f64;
}
```

### yalm-parser
- Parse dict5.md format: extract entries with word, definition, examples
- Parse dict5_test.md: extract questions with expected answers
- Handle markdown bold (`**word**`), em-dash (—), bullet points
- Robust to minor format variations

### yalm-engine
This is the heart. Implement the Comprehend trait:

1. **train()**: 
   - Initialize all words at random positions
   - Scan all definitions + examples to discover connectors
   - For each sentence, identify [entity] [connector] [entity] patterns
   - Apply forces: move entities according to connector operators
   - Multiple passes over the dictionary (converge the space)
   - The number of passes is a tunable parameter

2. **query()**:
   - Parse the question using the same connector discovery
   - Identify the question pattern
   - Perform geometric lookup
   - Apply threshold to decide Yes/No/IDontKnow/Word

3. **Key design decisions to make:**
   - Dimensionality of the space (start with 8? 16?)
   - Force magnitude decay (constant? diminishing per pass?)
   - How negation works geometrically ("not hot is cold" — does "not" invert direction?)
   - Threshold for yes/no/unknown
   - How to handle multi-word connectors ("is a" vs "is")
   - How to handle sentences with more than one connector ("a thing that lives")

### yalm-eval
- Load test questions
- Run each through the engine
- Compute fitness score
- Print detailed results per question (expected vs actual, distance values)
- Summary statistics

## WHAT TO BUILD FIRST

1. Parser (dict5.md + dict5_test.md)
2. Core data structures
3. Connector discovery (run on dict5, print discovered connectors)
4. Force field builder (train on dict5, visualize/dump the space)
5. Question resolver (run dict5_test.md, report scores)
6. Fitness evaluator (compute fitness)

## INITIAL PARAMETERS (will be evolved later)

```
dimensions: 8
learning_passes: 10
force_magnitude: 0.1
force_decay: 0.95 per pass
connector_min_frequency: 3
yes_threshold: 0.3  (distance below this = yes)
no_threshold: 0.7   (distance above this = no)
// between thresholds = "I don't know"
```

## SUCCESS CRITERIA FOR v0.1

- Parses dict5.md correctly (all 50 entries)
- Discovers at least 5 meaningful connectors from text
- Builds a geometric space where related words are closer than unrelated words
- Answers at least 10/20 test questions correctly
- Says "I don't know" for at least 2/4 unknowable questions
- Fitness score > 0.4
- Code compiles and runs with `cargo run`
- Prints connector discovery results, space statistics, and per-question results

The bar is LOW for v0.1. The architecture must be correct and extensible. Performance will improve through evolution (Prompt 03).

## WHAT NOT TO DO

- Do NOT hardcode any relation types (is-a, has-property, etc.)
- Do NOT use any NLP library or pretrained model
- Do NOT parse sentence grammar — use statistical patterns only
- Do NOT optimize prematurely — clarity over performance
- Do NOT add features beyond the scope above
- Do NOT create mock implementations — everything must actually work

## ATTACHED FILES

- `dict5.md` — training dictionary
- `dict5_test.md` — test questions with expected answers
- `dict12.md` — larger dictionary (for later testing, not used in v0.1)

## OUTPUT

Produce all Rust source files, Cargo.toml files, and a README.md explaining:
- How to build and run
- What the output means
- Known limitations of v0.1
- What v0.2 should improve
