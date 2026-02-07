# PROMPT 04 — Grammar Reinforcement & Self-Referential Learning

## CONTEXT

YALM is a geometric comprehension engine that reads closed dictionaries and builds an N-dimensional space where words are points, connectors are force operators, and questions are answered by geometric distance queries.

Current state (v7d):
- dict5 (50 words): 0.7812 fitness, 13/20 correct
- dict12 (400 words): 0.7812 fitness, zero overfitting gap
- Failing categories: negation (Q11-Q14), property queries (Q19-Q20), one direct lookup (Q05)
- Architecture: pure geometric — connector discovery, force field, threshold-based answers
- Evolution: 20 generations of parameter + strategy optimization, converged

The key insight: the system learns WHAT words mean from the dictionary, but it doesn't learn HOW WORDS RELATE TO EACH OTHER. Connectors like "is a" and "not" are statistical patterns, not understood relationships.

## THE NEW IDEA

Feed the system a second document — a grammar text — written in the same vocabulary as the dictionary. This text DESCRIBES the connectors and their meaning. The system processes it with the same force field, but the content teaches it about its own operations.

This is self-referential bootstrapping: learning about learning using the same learning mechanism.

## IMPLEMENTATION

### Training Pipeline Change

Current:
```
1. Parse dictionary → extract entries
2. Discover connectors → extract patterns
3. Build force field → position words
4. Answer questions
```

New:
```
1. Parse dictionary → extract entries
2. Discover connectors → extract patterns  
3. Build force field (PASS 1: dictionary) → initial word positions
4. Parse grammar text → extract sentences
5. Apply force field (PASS 2: grammar) → refined word positions
6. Optionally: re-discover connectors from combined text
7. Answer questions
```

The grammar text is processed by the SAME force field engine. No new code for parsing or force application. The only change is feeding a second document after the dictionary.

### What Changes in the Engine

1. **`train()` accepts multiple documents** with a processing order:
   ```rust
   pub fn train(&mut self, documents: &[Document]) {
       for doc in documents {
           self.discover_connectors(&doc.sentences);
           self.apply_forces(&doc.sentences);
       }
   }
   ```

2. **Document struct** distinguishes dictionary from grammar text:
   ```rust
   enum DocumentType {
       Dictionary,    // entries with definitions + examples
       GrammarText,   // prose sentences about language patterns  
   }
   
   struct Document {
       doc_type: DocumentType,
       sentences: Vec<String>,
   }
   ```
   
   For v0.2, both types are processed identically by the force field.
   The distinction exists for future use (grammar text might need different force weights).

3. **Connector re-discovery** after grammar text:
   Grammar text contains high-frequency connector usage WITH explicit descriptions.
   After processing grammar text, the connector list may expand or connector
   frequencies may shift. Re-running connector discovery on combined text
   should yield better connector quality.

### What Does NOT Change

- Force field algorithm (same geometric operations)
- Connector representation (same force operators)
- Question resolver (same threshold-based distance queries)
- Evolution genome (same parameters)
- Fitness function (same formula)

## VALIDATION PROTOCOL

### Experiment 1: Grammar Reinforcement Effect

**Setup:**
- Train engine on dict5 ONLY → run dict5_test.md → record baseline scores
- Train engine on dict5 + grammar5 → run dict5_test.md → record reinforced scores
- Train engine on dict5 + grammar5 → run grammar5_test.md → record grammar-specific scores

**Measure:**
- Per-question score change between baseline and reinforced
- Which categories improved most (expect: negation, unknown)
- Whether any categories DEGRADED (grammar text might add noise)
- Grammar-specific test total (baseline expectation: 13-17/20)

**Success criteria:**
- At least 2 previously-failing questions now pass after grammar reinforcement
- No more than 1 previously-passing question regresses
- grammar5_test score >= dict5_test score (grammar knowledge transfers)

### Experiment 2: Evolution Re-run

**Setup:**
- Re-run 20 generations of evolution with grammar5 included in training
- Compare convergence curve against v7d (no grammar)

**Measure:**
- Does best fitness improve beyond 0.7812?
- Do evolved parameters differ when grammar text is available?
- Does the parameter ceiling shift (plateau at higher fitness or same)?

**Success criteria:**
- Best fitness > 0.80 (grammar breaks the current ceiling)
- OR: same fitness but fewer generations to converge (grammar accelerates learning)

### Experiment 3: Grammar Text Quality

**Setup:**
- Analyze which grammar5 sentences produce the strongest force effects
- Measure word position shifts after grammar text processing
- Identify which sentences are redundant (no position change) vs impactful

**Measure:**
- Per-sentence force magnitude (how much did positions change?)
- Connector strength changes (did any connector get significantly stronger/weaker?)
- Word clustering changes (did category clusters tighten or loosen?)

**Output:**
- Ranked list of grammar sentences by impact
- Recommendations for grammar5 revision (remove low-impact, strengthen high-impact)

## ATTACHED FILES

- `dict5.md` — base dictionary
- `dict5_test.md` — base test questions (20)
- `grammar5.md` — grammar text in dict5 vocabulary
- `grammar5_test.md` — grammar-aware test questions (20)
- All existing engine source code

## IMPLEMENTATION STEPS

1. Modify `train()` to accept multiple documents (small change)
2. Add grammar text parser (extracts sentences from markdown prose)
3. Run Experiment 1 WITHOUT evolution (just the best v7d genome + grammar text)
4. Analyze results
5. If promising: run Experiment 2 (evolution with grammar)
6. Run Experiment 3 (grammar text quality analysis)
7. Update STATUS.md and SUGGESTIONS.md

## EXPECTED OUTCOMES

### Optimistic (grammar reinforcement works)
- Negation scores improve from 0/4 to 2-4/4
- Overall dict5 fitness exceeds 0.85
- Grammar5_test shows 15+/20
- Architecture validated for grammar12 + dict12 next

### Neutral (grammar text is just more training data)
- Marginal improvement (1-2 questions)
- Grammar text acts like additional definition examples
- The self-referential aspect doesn't provide special benefit
- Still useful but doesn't break the ceiling

### Pessimistic (grammar text adds noise)
- Some passing questions regress
- Grammar text's longer sentences confuse connector discovery  
- Force field can't distinguish "the sun is hot" (fact) from
  "when you see 'is' it tells you how a thing is" (meta-description)
- Architecture change needed: grammar text requires different processing

### If pessimistic: what to try next
- Weight grammar sentences differently (lower force magnitude)
- Process grammar text in a separate pass with learned weighting
- Extract only the pattern examples from grammar text, skip the meta-descriptions
- Create a "grammar space" separate from the "semantic space" (dual-space architecture)

## WHAT NOT TO DO

- Do NOT add special parsing rules for grammar text
- Do NOT modify the force field algorithm (same geometry for both documents)
- Do NOT hardcode grammar rules based on what grammar5.md teaches
- Do NOT skip the baseline comparison (Experiment 1 is critical)
- Do NOT run evolution before understanding the raw reinforcement effect
