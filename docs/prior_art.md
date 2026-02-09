# YALM Prior Art Analysis

> An honest assessment of novelty vs. reinvention.
> "Did we reinvent the wheel? Has no one thought to teach a machine like a kid before?"

---

## Section 1: The Landscape

Machine language understanding has been approached from at least four directions:

| Approach | Era | Key Idea | Scale |
|----------|-----|----------|-------|
| **Symbolic AI / Knowledge Engineering** | 1970s-1990s | Hand-crafted rules and ontologies | CYC: millions of assertions |
| **Statistical NLP** | 1990s-2010s | Co-occurrence statistics from corpora | Billions of tokens |
| **Neural Embeddings** | 2013-present | Learned vector spaces from data | Billions of parameters |
| **Geometric / Conceptual Spaces** | 2000-present | Quality dimensions, convex regions | Theoretical / small-scale |

YALM occupies a fifth position: **dictionary-driven geometric comprehension** — learning from definitions using physical forces, with no neural networks and no corpus. This position is sparsely populated.

---

## Section 2: Closest Relatives

### 2.1 Conceptual Spaces (Gardenfors, 2000, 2014)

**What**: Peter Gardenfors proposed that concepts live in geometric spaces organized by "quality dimensions" (color, size, temperature, etc.). Concepts are convex regions. Category membership is betweenness. Similarity is distance.

- Gardenfors, P. (2000). *Conceptual Spaces: The Geometry of Thought*. MIT Press.
- Gardenfors, P. (2014). *The Geometry of Meaning: Semantics Based on Conceptual Spaces*. MIT Press.

**Shared with YALM**: Words as points in geometric space. Distance as similarity. Geometry as the representation of meaning.

**Different from YALM**: Gardenfors assumes quality dimensions are given (color has hue/saturation/brightness). YALM discovers its dimensions from text — the 8 dimensions of the space have no predefined meaning. Gardenfors' spaces are hand-designed; YALM's emerge from force-field equilibrium. Gardenfors does not specify a learning algorithm; YALM has a complete pipeline from text to geometry.

**Assessment**: YALM could be viewed as an **implementation of Gardenfors' theory** with a crucial addition: the dimensions and positions are discovered from text, not assumed. Gardenfors provides the philosophical framework; YALM provides a construction algorithm.

### 2.2 Word2Vec, GloVe, FastText (Mikolov et al., 2013; Pennington et al., 2014)

**What**: Neural (Word2Vec) and matrix-factorization (GloVe) methods that place words in vector spaces based on co-occurrence in massive corpora.

- Mikolov, T. et al. (2013). "Efficient Estimation of Word Representations in Vector Space." *arXiv:1301.3781*.
- Pennington, J. et al. (2014). "GloVe: Global Vectors for Word Representation." *EMNLP 2014*.

**Shared with YALM**: Words as vectors. Similar words are close. The output is a geometric space where distance encodes meaning.

**Different from YALM**:
- **Input**: Word2Vec/GloVe learn from billions of tokens of raw text. YALM learns from 51-2008 definitions.
- **Method**: Word2Vec uses a shallow neural network. GloVe uses co-occurrence matrix factorization. YALM uses physical forces with typed connectors.
- **Relations**: Word2Vec/GloVe encode a single notion of similarity. YALM's connectors encode TYPED relations — "is a" pushes differently than "can" or "not". This is closer to knowledge graph embeddings (see 2.3).
- **Scale**: Word2Vec needs millions of contexts. YALM works from 51 words.
- **Interpretability**: YALM's forces are traceable — you can see which connector pushed which words together. Word2Vec's dimensions are opaque.

**Assessment**: YALM produces a similar output (word vectors) but through a fundamentally different process. The key distinction is **typed forces from definitions** vs. **untyped co-occurrence from corpora**. This is NOT reinvention — it's a different path to a related destination.

### 2.3 Knowledge Graph Embeddings: TransE, TransR, RotatE (Bordes et al., 2013)

**What**: TransE embeds knowledge graph triples (head, relation, tail) such that `head + relation ≈ tail` in vector space. Relations are translation vectors.

- Bordes, A. et al. (2013). "Translating Embeddings for Modeling Multi-relational Data." *NeurIPS 2013*.
- Lin, Y. et al. (2015). "Learning Entity and Relation Embeddings for Knowledge Graph Completion." *AAAI 2015*. (TransR)
- Sun, Z. et al. (2019). "RotatE: Knowledge Graph Embedding by Relational Rotation in Complex Space." *ICLR 2019*.

**Shared with YALM**: Remarkably similar. YALM's connectors ARE typed relation vectors. The force `dog -[is a]→ animal` pushes dog toward animal along the "is a" direction. This is the same geometric intuition as TransE's `dog + is_a ≈ animal`.

**Different from YALM**:
- **Input**: TransE is GIVEN the (h, r, t) triples. YALM DISCOVERS them from text via connector discovery and relation extraction. This is the critical difference.
- **Learning**: TransE uses stochastic gradient descent on a margin-based loss. YALM uses physical force-field equilibrium.
- **Scale**: TransE works on Freebase (millions of triples). YALM works on 51-2008 words.

**Assessment**: YALM's connector forces are an independent rediscovery of the TransE intuition (relations as translations in vector space), but with a fundamentally different input pipeline. TransE assumes the knowledge graph exists. YALM constructs it from definitions. **This is the strongest "reinvention" signal** — the geometric operation is the same, but the construction path is novel.

### 2.4 Dictionary-Based Embeddings

**What**: Several projects have specifically learned word vectors from dictionary definitions.

- Tissier, J. et al. (2017). "Dict2Vec: Learning Word Embeddings using Lexical Dictionaries." *EMNLP 2017*.
- Bahdanau, D. et al. (2017). "Learning to Compute Word Embeddings On the Fly." *arXiv:1706.00286*.
- Hill, F. et al. (2016). "Learning to Understand Phrases by Embedding the Dictionary." *TACL 2016*.

**Shared with YALM**: Learning from dictionary definitions specifically (not raw corpus). The idea that definitions encode semantic structure.

**Different from YALM**:
- **Method**: Dict2Vec uses definitions as additional training signal alongside standard Word2Vec. Hill et al. use an LSTM to encode definitions. Both use neural networks.
- **Closure**: These systems don't require a closed dictionary. They use definitions as supplementary data, not as the sole input.
- **Architecture**: YALM uses force-directed layout, not neural networks. No backpropagation, no gradient descent.

**Assessment**: Dictionary-based learning exists as a research thread, but it typically augments neural methods rather than replacing them. YALM's innovation is going **all-in on definitions** as the SOLE input, without neural networks, and with the closed-dictionary constraint that creates a self-contained universe.

### 2.5 The CYC Project and Knowledge-Based AI (Lenat, 1984-present)

**What**: Doug Lenat's CYC project attempted to encode common-sense knowledge as millions of hand-crafted logical assertions. The goal: if you give a machine enough facts, understanding emerges.

- Lenat, D. (1995). "CYC: A Large-Scale Investment in Knowledge Infrastructure." *CACM 38(11)*.

**Shared with YALM**: The belief that structured knowledge produces understanding. Both start from hand-crafted representations (CYC's assertions, YALM's definitions).

**Different from YALM**:
- **Representation**: CYC uses formal logic (predicates, rules, inference). YALM uses geometry (positions, distances, forces).
- **Scale**: CYC has millions of assertions. YALM has 51-2008 definitions.
- **Learning**: CYC's assertions are manually authored by knowledge engineers. YALM's geometry EMERGES from definitions through force-field equilibrium.
- **Failure mode**: CYC suffers from brittleness — new knowledge doesn't compose with old knowledge reliably. YALM's geometric space composes naturally (adding a word adjusts the local geometry without breaking global structure).

**Assessment**: YALM and CYC share the "knowledge produces understanding" philosophy but differ completely in method. CYC is logic; YALM is physics. CYC was abandoned (for practical purposes) when statistical methods won in the 2000s. YALM sidesteps CYC's brittleness by using continuous geometry instead of discrete logic.

### 2.6 Force-Directed Graph Layouts (Fruchterman & Reingold, 1991)

**What**: Force-directed algorithms position graph nodes by simulating physical forces: connected nodes attract, non-connected nodes repel. Used in graph visualization.

- Fruchterman, T. & Reingold, E. (1991). "Graph Drawing by Force-Directed Placement." *Software: Practice and Experience 21(11)*.
- Kamada, T. & Kawai, S. (1989). "An algorithm for drawing general undirected graphs." *Information Processing Letters 31(1)*.

**Shared with YALM**: YALM's equilibrium IS a force-directed layout. Related words attract, the space reaches a minimum-energy configuration.

**Different from YALM**:
- **Semantics**: Fruchterman-Reingold forces are untyped — connected nodes attract equally. YALM's forces are TYPED — different connectors produce different force directions.
- **Purpose**: FR is for visualization (2D/3D layout). YALM is for comprehension (8D+ space).
- **Dimensions**: FR operates in 2-3 dimensions. YALM in 8+.
- **Input**: FR takes a graph as input. YALM discovers the graph from text.

**Assessment**: YALM's force-field engine is a typed, higher-dimensional extension of force-directed layout. The individual technique is well-established (1989-1991). What's new is applying typed forces from linguistic connectors in a high-dimensional semantic space.

### 2.7 Bootstrap Learning and Self-Play

**What**: Systems that learn from their own output. AlphaGo Zero learns by playing against itself. Self-distillation trains a student network from a teacher network's soft labels.

- Silver, D. et al. (2017). "Mastering the game of Go without human knowledge." *Nature 550*.
- Hinton, G. et al. (2015). "Distilling the Knowledge in a Neural Network." *NeurIPS Workshop 2015*.

**Shared with YALM**: Phase 19's bootstrap loop reads its own describe() output, discovers new patterns, and rebuilds the space. This is self-improvement from generated data.

**Different from YALM**:
- **Domain**: AlphaGo generates game positions. YALM generates sentences about words.
- **Scale**: AlphaGo runs millions of self-play games. YALM's bootstrap converges in 2 iterations.
- **What improves**: AlphaGo improves its policy (strategy). YALM improves its connector set (grammar), not its parameters.

**Assessment**: Bootstrap learning is a known technique. YALM's specific application — using describe() to surface implicit grammar that enriches connector discovery — is a novel application of the general principle.

### 2.8 Semantic Bootstrapping in Child Language Acquisition

**What**: Children use semantic knowledge to bootstrap syntactic learning — they learn grammar by noticing patterns in meaningful speech.

- Pinker, S. (1984). *Language Learnability and Language Development*. Harvard UP.
- Tomasello, M. (2003). *Constructing a Language: A Usage-Based Theory of Language Acquisition*. Harvard UP.
- Gleitman, L. (1990). "The Structural Sources of Verb Meanings." *Language Acquisition 1(1)*.

**Shared with YALM**: The "teach like a kid" principle. ELI5 definitions mimic the simple input a child receives. Connector discovery from text mirrors how children extract grammatical patterns from speech. The bootstrap loop mirrors how children use known words to learn new grammar.

**Different from YALM**: Children have embodied experience (vision, touch, proprioception). YALM has only text. Children learn incrementally over years. YALM builds a space in milliseconds.

**Assessment**: YALM's approach has intuitive parallels to usage-based language acquisition (Tomasello's framework). The ELI5 constraint maps to the simplified input children receive. But YALM has no embodied grounding — its "understanding" is purely textual geometry.

### 2.9 Symbol Grounding Problem (Harnad, 1990)

**What**: Stevan Harnad argued that symbols in a formal system are meaningless without grounding in sensory experience — the "Chinese Room" problem applied to AI.

- Harnad, S. (1990). "The Symbol Grounding Problem." *Physica D 42*.
- Searle, J. (1980). "Minds, Brains, and Programs." *Behavioral and Brain Sciences 3(3)*.

**Relevance to YALM**: YALM's closed dictionary is exactly a system of ungrounded symbols — definitions defined in terms of other definitions, with no external reference. "Dog" is defined as "an animal" and "animal" is defined as "a thing that can move and eat" — but nowhere does the system see a dog or hear a bark.

**Assessment**: YALM operates in Searle's Chinese Room. It manipulates symbols (definitions) according to rules (forces, chains) and produces correct answers, but there is no sensory grounding. Whether this constitutes "understanding" is a philosophical question YALM cannot answer from within. The system is honest about this: it knows what its definitions tell it, nothing more.

---

## Section 3: What's Actually Novel

The individual components of YALM are established techniques:

| Component | Established Since | YALM's Version |
|-----------|------------------|----------------|
| Word vectors | 2013 (Word2Vec) | Connector-typed force field |
| Force-directed layout | 1991 (Fruchterman-Reingold) | N-dimensional with typed forces |
| Genetic algorithms | 1975 (Holland) | Strategy + parameter co-evolution |
| Dictionary-based learning | 2016+ (Dict2Vec, Hill et al.) | Closed dictionary as sole input |
| Knowledge graph embeddings | 2013 (TransE) | Discovered (not given) relations |
| Bootstrap/self-play | 2017 (AlphaGo Zero) | describe-then-rediscover loop |

**What appears to be novel is the specific combination**:

1. **The full pipeline**: Closed dictionary → connector discovery → typed force field → equilibrium → geometric QA → bootstrap. No neural networks at any stage. We are not aware of prior work that implements this complete chain.

2. **The ELI5 closure constraint**: Requiring every definition word to be itself defined, at a 5-year-old level, creating a self-contained universe. Dict2Vec uses definitions as supplementary signal; YALM uses them as the SOLE signal, and requires closure.

3. **Connector discovery → typed forces**: The automatic extraction of relation types from text statistics, used as DIFFERENT force types with DIFFERENT directions. TransE has typed relations but doesn't discover them. Word2Vec discovers patterns but doesn't type them.

4. **Multi-space domain separation from ELI5 dictionaries**: Independent geometric spaces for different knowledge domains, connected only through bridge terms. Each space has its own equilibrium. This compositional architecture applied to dictionary-based learning appears novel.

5. **Bootstrap loop using describe-then-rediscover**: Generating sentences from the geometric space, feeding them back through connector discovery, and enriching the grammar without changing the dictionary. This specific self-improvement mechanism applied to geometric word spaces appears novel.

---

## Section 4: What's NOT Novel (and That's Fine)

- **Word vectors**: Placing words in geometric space has been standard since 2013. YALM's output (a space where similar words are close) is the same as Word2Vec/GloVe.
- **Force-directed layouts**: Well-established since 1989. YALM's equilibrium is a higher-dimensional, typed version of the same physics.
- **Genetic algorithms**: Well-established since 1975. YALM's evolution is standard tournament selection with Gaussian mutation.
- **QA from knowledge bases**: Decades of work in KBQA. YALM's question answering is simpler than most.
- **The definition-chain gate**: Symbolic chain traversal over a graph is a basic graph algorithm, not an innovation.
- **Relations as translations**: TransE (2013) had this exact insight. YALM rediscovered it independently.

The individual pieces are known. The combination may be new. This is common in engineering — the Wright brothers didn't invent wings, engines, or propellers. They combined them in a way that flew.

---

## Section 5: The Hard Question

> "If everything is as it seems, we basically rewrote ML from scratch.
> No one thought to teach a machine like a kid before? How could it be?"

The honest answer has five parts:

### 1. People DID try, and it didn't scale

Knowledge-based AI (CYC, 1984-present) tried to teach machines through hand-crafted knowledge. It worked for narrow domains but couldn't scale — every new concept required manual encoding. Statistical methods (Word2Vec, 2013; transformers, 2017) won because they scale with data. YALM at 2008 words is impressive for its approach but is still in the territory where knowledge-based methods worked fine. The real test is 100K+ words.

### 2. The ELI5 closure trick IS genuinely underexplored

We could not find prior work that specifically requires a CLOSED dictionary of ELI5-level definitions as the sole input to a geometric comprehension engine. Dict2Vec and related work use definitions as supplementary training signal, not as the sole input. The closure constraint (every word defined in terms of other defined words) creates a self-consistent universe that standard dictionary-based methods don't require. This specific constraint appears to be YALM's most distinctive contribution.

### 3. It's a rediscovery with a different path

YALM independently arrived at TransE-like geometry (relations as translations) and Gardenfors-like conceptual spaces (meaning as geometry) through a novel construction path (definitions → forces → equilibrium). The destination is familiar; the journey is new.

### 4. The hybrid nature IS the finding

The original vision was "geometry IS the knowledge." The actual system is geometry + definition-chain traversal. This isn't a failure — it's a genuine empirical finding: **metric spaces encode similarity, but not identity**. You need both geometry (for association) and directed symbolic operations (for discrimination). This finding is consistent with the broader field's experience: knowledge graph embeddings (TransE) work for similarity but struggle with negation and asymmetric relations.

### 5. Scale is the open question

At 51-2008 words, YALM demonstrates that the architecture works. At 100K words, the architecture might collapse (thresholds break, equilibrium doesn't converge, connector discovery drowns in noise) or it might scale sublinearly (as the 1005→2008 curve suggests). Until the 10K+ test is done, claims about "rewriting ML" are premature. What YALM proves at current scale: **definitions + physics + evolution = comprehension**. Whether this generalizes beyond toy-to-medium scale is unknown.

---

## The One-Sentence Summary

**YALM is a geometric comprehension engine that combines established techniques (force-directed layout, typed relation embeddings, genetic evolution) in a novel configuration (closed ELI5 dictionary → automatic connector discovery → typed force-field equilibrium → multi-space architecture → bootstrap self-improvement), differing from existing approaches primarily in its construction path (from definitions to geometry without neural networks) and its closure constraint (every symbol defined in terms of other symbols). The main limitation is unproven scalability beyond 2000 words and the irreducible need for symbolic chain traversal alongside geometry.**

---

## References

1. Gardenfors, P. (2000). *Conceptual Spaces: The Geometry of Thought*. MIT Press.
2. Gardenfors, P. (2014). *The Geometry of Meaning: Semantics Based on Conceptual Spaces*. MIT Press.
3. Mikolov, T. et al. (2013). "Efficient Estimation of Word Representations in Vector Space." *arXiv:1301.3781*.
4. Pennington, J., Socher, R., Manning, C. (2014). "GloVe: Global Vectors for Word Representation." *EMNLP 2014*.
5. Bordes, A. et al. (2013). "Translating Embeddings for Modeling Multi-relational Data." *NeurIPS 2013*.
6. Lin, Y. et al. (2015). "Learning Entity and Relation Embeddings for Knowledge Graph Completion." *AAAI 2015*.
7. Sun, Z. et al. (2019). "RotatE: Knowledge Graph Embedding by Relational Rotation in Complex Space." *ICLR 2019*.
8. Tissier, J., Gravier, C., Habrard, A. (2017). "Dict2Vec: Learning Word Embeddings using Lexical Dictionaries." *EMNLP 2017*.
9. Hill, F., Cho, K., Korhonen, A. (2016). "Learning to Understand Phrases by Embedding the Dictionary." *TACL 2016*.
10. Bahdanau, D. et al. (2017). "Learning to Compute Word Embeddings On the Fly." *arXiv:1706.00286*.
11. Lenat, D. (1995). "CYC: A Large-Scale Investment in Knowledge Infrastructure." *CACM 38(11)*.
12. Fruchterman, T. & Reingold, E. (1991). "Graph Drawing by Force-Directed Placement." *Software: Practice and Experience 21(11)*.
13. Kamada, T. & Kawai, S. (1989). "An algorithm for drawing general undirected graphs." *Information Processing Letters 31(1)*.
14. Silver, D. et al. (2017). "Mastering the game of Go without human knowledge." *Nature 550*.
15. Hinton, G., Vinyals, O., Dean, J. (2015). "Distilling the Knowledge in a Neural Network." *NeurIPS Workshop 2015*.
16. Pinker, S. (1984). *Language Learnability and Language Development*. Harvard UP.
17. Tomasello, M. (2003). *Constructing a Language: A Usage-Based Theory of Language Acquisition*. Harvard UP.
18. Gleitman, L. (1990). "The Structural Sources of Verb Meanings." *Language Acquisition 1(1)*.
19. Harnad, S. (1990). "The Symbol Grounding Problem." *Physica D 42*.
20. Searle, J. (1980). "Minds, Brains, and Programs." *Behavioral and Brain Sciences 3(3)*.
