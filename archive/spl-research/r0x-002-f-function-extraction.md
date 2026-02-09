# r0x-002: f-Function Extraction — NN Decompiler

**Branch:** pure-research
**Prerequisite:** r0x-001 verdict = ALIVE (Spearman > 0.5)
**Goal:** Build f: NN → SPL-compatible geometric space with grounding.

## Context

r0x-001 proved that GPT-2 embedding distances correlate with YALM equilibrium distances for dict5. This experiment builds the actual transformation function f that extracts a usable geometric space from a neural network.

## Prior Art Search (Step 0 — Do This First)

Before writing any code, search for existing work. Use `paper-search2` MCP server.

### Required Searches

```
search_arxiv("Procrustes alignment embedding spaces semantic", max_results=10)
search_semantic("cross-lingual embedding alignment orthogonal mapping", year="2017-", max_results=10)
search_arxiv("neural network interpretability geometric probing", max_results=10)
search_semantic("knowledge extraction neural network embedding decompilation", max_results=10)
```

### Key Authors / Papers to Look For

- Conneau et al. — cross-lingual embedding alignment via Procrustes (MUSE)
- Li et al. — "Convergent Learning" (NN representations converge across architectures)
- Saphra & Lopez — understanding NN learning dynamics
- Any work on projecting transformer embeddings to interpretable spaces
- Linear representation hypothesis literature

### What We Need to Know

1. What Procrustes disparity values are typical for semantically aligned spaces?
2. Has anyone projected NN embeddings into non-NN semantic spaces and validated with QA?
3. What's the information loss from PCA dimensionality reduction of embeddings?
4. Are attention heads known to correspond to specific semantic relations?

### Output

Log findings in `research/r0x-002_prior_art.md` with same format as r0x-001.

Critical: if cross-lingual alignment literature gives us a proven f-function methodology, adopt it wholesale.

---

## The Grounding Problem

Raw embeddings are "conoscenza dello sciocco" — a perfect map without a legend. The embedding vector for token 2134 means nothing without the tokenizer mapping 2134 → "dog". But worse: GPT-2 uses BPE tokenization, so "animal" might be tokens [2134, 819] — the embedding is synthetic (mean of sub-tokens), not a real point in the model's space.

**Solution:** Use dict5 as the Rosetta Stone. YALM's geometry provides *meaning-grounded* distances. GPT-2 provides *statistically-rich* distances. Align the two via Procrustes analysis on the shared vocabulary.

## What To Build

Script: `research/r0x_002_f_function.py`

### Step 1: Procrustes Alignment

```python
from scipy.spatial import procrustes
import numpy as np

# yalm_points: 51 x N_yalm (from YALM equilibrium)
# gpt2_points: 51 x 768 (from GPT-2 wte, projected to same dimensionality)

# First: reduce GPT-2 to same dimensionality as YALM via PCA
from sklearn.decomposition import PCA
pca = PCA(n_components=N_yalm)
gpt2_reduced = pca.fit_transform(gpt2_points)

# Procrustes: find rotation + scaling that best aligns GPT-2 to YALM
mtx1, mtx2, disparity = procrustes(yalm_points, gpt2_reduced)
print(f"Procrustes disparity: {disparity:.4f}")
# disparity < 0.3 = strong structural match
```

### Step 2: Extract Extended Vocabulary

Once aligned, project GPT-2 words NOT in dict5 into YALM-compatible space:

```python
# Take GPT-2 embeddings for dict12 words (1005 words)
# Apply same PCA transform + Procrustes rotation
# These points are now in YALM-compatible space WITHOUT running equilibrium

dict12_words = [...]  # parse from dictionaries/dict12.md
dict12_gpt2 = []
for word in dict12_words:
    tokens = tokenizer.encode(word)
    emb = model.wte.weight[tokens].mean(dim=0).detach().numpy()
    dict12_gpt2.append(emb)

dict12_projected = pca.transform(np.array(dict12_gpt2))
# Apply Procrustes rotation (mtx2's transform)
```

### Step 3: Validate Against YALM dict12

Run YALM equilibrium on dict12. Compare distances:
- Between dict12 YALM distances and projected GPT-2 distances
- If correlation holds → f generalizes beyond the training set (dict5)
- If correlation drops → f is overfitting to dict5

### Step 4: The Decompiler Test

The real test: can the projected space ANSWER QUESTIONS without YALM equilibrium?

```python
# Take dict5_test.md questions (20 questions)
# For each yes/no question "Is X a Y?":
#   compute distance(X, Y) in projected GPT-2 space
#   apply YALM's threshold
#   compare with ground truth

# Score: X/20
# Baseline (YALM equilibrium): 20/20
# If projected space scores >= 15/20, f is usable
```

### Step 5: Connector Extraction from Attention

If Steps 1-4 pass, attempt to extract YALM-like connectors from GPT-2:

```python
# Attention heads as connector candidates
# Each head learns a specific "relation type"
# Hypothesis: some heads correspond to "is a", "can", "not"

# Extract attention patterns for dict5 word pairs
# Compare with YALM connector assignments
# This is exploratory — log everything, conclude nothing prematurely
```

## Output

```
research/
├── r0x_002_f_function.py
├── r0x_002_results.json
│   ├── procrustes_disparity
│   ├── dict12_correlation (generalization test)
│   ├── question_score (decompiler test)
│   └── attention_analysis (exploratory)
├── r0x_002_alignment.png      # dict5 points: YALM vs projected GPT-2
├── r0x_002_dict12_scatter.png  # Generalization scatter
└── r0x_002_verdict.md
```

## Success Criteria

| Metric | Dead | Alive |
|--------|------|-------|
| Procrustes disparity | > 0.5 | < 0.3 |
| dict12 Spearman | < 0.3 | > 0.4 |
| Question score | < 10/20 | >= 15/20 |

If ALIVE: f exists. We can "decompile" NN into inspectable geometric spaces.
If DEAD: NN and YALM capture fundamentally different structure. Proceed to r0x-003 (SPL native, no NN bridge).
