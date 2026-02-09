# r0x-001: The f-Function Test

**Branch:** pure-research
**Goal:** Determine if neural network embeddings contain the same geometric structure as YALM equilibrium.
**Verdict:** Binary. Correlation > 0.5 = f exists. Correlation < 0.3 = different knowledge. Between = inconclusive.

## Hypothesis

If YALM's geometric space and LLM embedding spaces capture the same semantic structure, then pairwise distances between dict5 words in GPT-2 embedding space should correlate with pairwise distances in YALM equilibrium space.

## Prior Art Search (Step 0 — Do This First)

Before writing any code, search for existing work. Use `paper-search2` MCP server.

### Required Searches

```
search_arxiv("word embedding geometric structure semantic distance correlation", max_results=10)
search_semantic("probing static embeddings semantic similarity structure", year="2018-", max_results=10)
search_arxiv("word2vec embedding space geometry isotropy", max_results=10)
```

### Key Authors / Papers to Look For

- Mikolov et al. — word2vec arithmetic properties (seminal)
- Ethayarajh 2019 — "How Contextual are Contextualized Word Representations?" (embedding geometry)
- Cai et al. — isotropy in embedding spaces
- Mu & Viswanath — "All-but-the-Top" (embedding space structure)
- Any work comparing embedding distances with human semantic judgments (SimLex-999, WordSim-353)

### What We Need to Know

1. Has anyone already correlated static embeddings with non-NN semantic spaces?
2. What's the expected Spearman range for embedding-vs-semantic correlations?
3. Are there known pitfalls (anisotropy, frequency bias) we should control for?
4. Does someone already have a "decompiler" or geometry extractor from NNs?

### Output

Log findings in `research/r0x-001_prior_art.md`:
- Paper title, authors, year, link
- Key finding relevant to r0x-001
- Status: `confirms_hypothesis` / `contradicts` / `extends` / `methodology_useful`

If a paper already does exactly what r0x-001 proposes: USE THEIR NUMBERS. Don't re-run the experiment. Cite and move on.

---

## What To Build

A standalone Python script: `research/r0x_001_f_test.py`

### Step 1: Extract YALM distances

Run YALM on dict5 with the v11 parameters (from results_v11/) and dump ALL pairwise distances between the 51 words to a JSON file.

```bash
# The project already has yalm-eval. Add a --dump-distances flag
# or parse from existing debug output.
# If easier: add a minimal Rust binary in research/ that loads
# the equilibrium and prints the distance matrix as CSV.
```

Alternative: read the equilibrium output directly. Check if `yalm-eval` already prints distances (it does for test queries). If not, add a `--dump-matrix` flag to yalm-eval that outputs:

```
word_a,word_b,distance
dog,cat,0.45
dog,animal,0.32
...
```

For ALL 51*50/2 = 1275 pairs.

### Step 2: Extract GPT-2 embeddings

```python
# Use transformers library
# Model: gpt2 (small, 768 dimensions)
# Extract the STATIC embedding (wte - word token embedding)
# NOT contextual (no forward pass needed)

from transformers import GPT2Tokenizer, GPT2Model
import torch
import json

model = GPT2Model.from_pretrained('gpt2')
tokenizer = GPT2Tokenizer.from_pretrained('gpt2')

# Load dict5 words
dict5_words = [...]  # parse from dictionaries/dict5.md

embeddings = {}
for word in dict5_words:
    tokens = tokenizer.encode(word)
    # Some words may be multi-token. Use MEAN of sub-token embeddings.
    token_embeddings = model.wte.weight[tokens]
    embeddings[word] = token_embeddings.mean(dim=0).detach().numpy()
```

### Step 3: Compute GPT-2 pairwise distances

```python
from scipy.spatial.distance import cosine, euclidean
import numpy as np

gpt2_distances = {}
for w1 in dict5_words:
    for w2 in dict5_words:
        if w1 < w2:
            d_cos = cosine(embeddings[w1], embeddings[w2])
            d_euc = euclidean(embeddings[w1], embeddings[w2])
            gpt2_distances[(w1, w2)] = {'cosine': d_cos, 'euclidean': d_euc}
```

### Step 4: Correlate

```python
from scipy.stats import spearmanr, pearsonr

# Align the pairs
pairs = sorted(yalm_distances.keys())
yalm_dists = [yalm_distances[p] for p in pairs]
gpt2_cos = [gpt2_distances[p]['cosine'] for p in pairs]
gpt2_euc = [gpt2_distances[p]['euclidean'] for p in pairs]

spearman_cos, p_cos = spearmanr(yalm_dists, gpt2_cos)
spearman_euc, p_euc = spearmanr(yalm_dists, gpt2_euc)
pearson_cos, _ = pearsonr(yalm_dists, gpt2_cos)
pearson_euc, _ = pearsonr(yalm_dists, gpt2_euc)

print(f"Spearman (cosine):    {spearman_cos:.4f}  p={p_cos:.2e}")
print(f"Spearman (euclidean): {spearman_euc:.4f}  p={p_euc:.2e}")
print(f"Pearson (cosine):     {pearson_cos:.4f}")
print(f"Pearson (euclidean):  {pearson_euc:.4f}")
```

### Step 5: Visualize

Generate a scatter plot: X = YALM distance, Y = GPT-2 distance. One plot per metric (cosine, euclidean). Save as PNG.

Also generate a 2D projection (t-SNE or PCA) of both spaces side by side. Label the 51 words. Visual inspection: do the clusters match?

## Output

```
research/
├── r0x_001_f_test.py          # Main script
├── r0x_001_dump_distances.rs  # If needed: Rust helper for YALM distances
├── r0x_001_results.json       # Raw numbers
├── r0x_001_scatter.png        # Correlation scatter
├── r0x_001_projection.png     # Side-by-side t-SNE/PCA
└── r0x_001_verdict.md         # One paragraph: pass/fail/inconclusive + numbers
```

## Critical Notes

- Use STATIC embeddings (wte), not contextual. We're comparing vocabulary geometry, not sentence processing.
- dict5 words are ELI5 level. GPT-2's tokenizer should handle them as single tokens mostly. Log any multi-token words — they're noise sources.
- Spearman over Pearson: we care about rank correlation (relative ordering), not linear relationship.
- The 51 words include connectors ("is", "a", "not", "can"). These may behave differently — compute correlation with and without function words.

## Success Criteria

| Metric | Dead | Inconclusive | Alive |
|--------|------|-------------|-------|
| Spearman (cosine) | < 0.2 | 0.2 - 0.5 | > 0.5 |
| Spearman (euclidean) | < 0.2 | 0.2 - 0.5 | > 0.5 |
| Visual cluster match | No overlap | Partial | Clear correspondence |

If ALIVE: proceed to r0x-002 (full f-function extraction).
If DEAD: document and move to r0x-003 (SPL integration without NN bridge).
If INCONCLUSIVE: try with dict12 (1005 words) for more statistical power.
