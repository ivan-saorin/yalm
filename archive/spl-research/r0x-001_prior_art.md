# r0x-001 Prior Art

## Search Date: 2026-02-09

### Searches Performed

1. `search_arxiv("word embedding geometric structure semantic distance correlation", 10)` — No relevant hits (recent papers off-topic)
2. `search_semantic("probing static embeddings semantic similarity structure", year="2018-", 10)` — Found Kozlowski 2025, Renner 2023, Nikolaev 2023
3. `search_arxiv("word2vec embedding space geometry isotropy", 10)` — No relevant hits
4. `search_google_scholar(Ethayarajh 2019, Mu & Viswanath, SimLex cosine correlation, Kozlowski 2025)` — Found all target papers

---

### Paper 1: Kozlowski, Dai & Boutyline (2025) — "Semantic Structure in Large Language Model Embeddings"
- Authors: Austin C. Kozlowski, Callin Dai, Andrei Boutyline
- Year: 2025
- Link: https://arxiv.org/abs/2508.10003
- Key finding: LLM embedding matrices encode semantic structure that correlates highly with human ratings. Word projections on antonym-pair directions (e.g., kind-cruel) match human judgments. Structure reduces to ~3 dimensions, closely resembling patterns from human survey data. Shifting tokens along one semantic direction causes off-target effects proportional to cosine similarity.
- Relevance: **confirms_hypothesis** — LLM static embeddings DO contain geometric semantic structure. The structure is low-dimensional (~3D), which is relevant since DAFHNE uses 8D.
- Impact on experiment: Strong evidence that the f-function should exist. However, their comparison is LLM-vs-human-ratings, not LLM-vs-geometric-space. r0x-001 compares LLM-vs-DAFHNE, which is novel.

### Paper 2: Ethayarajh (2019) — "How Contextual are Contextualized Word Representations?"
- Authors: Kawin Ethayarajh
- Year: 2019
- Link: https://arxiv.org/abs/1909.00512
- Key finding: Contextualized representations are NOT isotropic in any layer. Less than 5% of variance in contextualized representations is explained by a static embedding. Upper layers are increasingly context-specific. Same-word representations in different contexts have high cosine similarity, but much less so in upper layers.
- Relevance: **methodology_useful** — Critical warning: static (wte) embeddings exist in an anisotropic space. Cosine similarity may be biased. Using STATIC embeddings (as r0x-001 does) avoids the contextual variance problem but inherits the anisotropy issue.
- Impact on experiment: Must be aware that anisotropy can inflate cosine similarities between unrelated words. Consider applying centering (All-but-the-Top) as a robustness check.

### Paper 3: Mu, Bhat & Viswanath (2017) — "All-but-the-Top: Simple and Effective Postprocessing for Word Representations"
- Authors: Jiaqi Mu, Sanjeev Bhat, Pramod Viswanath
- Year: 2017
- Link: https://arxiv.org/abs/1702.01417
- Key finding: Word embeddings have a non-zero mean and a few dominating directions that don't carry semantic information. Removing the common mean vector and top dominating directions (PCA) consistently improves word similarity performance across all tested embeddings and benchmarks.
- Relevance: **methodology_useful** — The "All-but-the-Top" centering could improve our GPT-2 embedding distances. Should try correlation with and without this post-processing.
- Impact on experiment: Add as optional analysis: center GPT-2 embeddings (subtract mean, remove top 1-3 PCA components) and re-run correlation.

### Paper 4: Toshevska, Stojanovska & Kalajdjieski (2020) — "Comparative Analysis of Word Embeddings for Capturing Word Similarities"
- Authors: Marina Toshevska, Frosina Stojanovska, Jovan Kalajdjieski
- Year: 2020
- Link: https://arxiv.org/abs/2005.03812
- Key finding: Comprehensive comparison of word2vec, GloVe, fastText on SimLex-999 and WordSim-353 using cosine similarity. Typical Spearman correlations with human similarity judgments: 0.3-0.5 for raw static embeddings on SimLex-999.
- Relevance: **extends** — Provides baseline Spearman correlation ranges. If r0x-001 finds Spearman ~0.3-0.5 between DAFHNE and GPT-2, that's in the same range as GPT-2-vs-human, which would be a strong result.
- Impact on experiment: Our "ALIVE" threshold of 0.5 may be aggressive given that even embedding-vs-human correlations are often 0.3-0.5.

### Paper 5: Yang, Zhang, Han & Liu (2025) — "Semantic Enrichment of Neural Word Embeddings"
- Authors: Dongqiang Yang, Xinru Zhang, Tonghui Han, Yi Liu
- Year: 2025
- Link: https://doi.org/10.1017/nlp.2025.10005
- Key finding: Retrofitting static and contextualized embeddings with taxonomic similarity achieves SOTA: Spearman 0.78 on SimLex-999, 0.76 on SimVerb-3500. Raw embeddings are significantly lower.
- Relevance: **extends** — Shows that raw embeddings capture partial semantic structure, and structured knowledge injection (like DAFHNE's definition-based approach) can close the gap. The fact that retrofitting helps implies raw embeddings miss some semantic signal.
- Impact on experiment: DAFHNE is definition-based (structured knowledge), while GPT-2 wte are distributional. The gap between them may be the structured-vs-distributional gap, not a fundamental geometry mismatch.

### Paper 6: Renner, Denis, Gilleron & Brunelliere (2023) — "Exploring Category Structure with Contextual Language Models"
- Authors: Joseph Renner, Pascal Denis, Remi Gilleron, Angele Brunelliere
- Year: 2023
- Link: https://arxiv.org/abs/2302.06942
- Key finding: Static word embeddings fail at predicting typicality (category membership strength) using cosine similarity. BERT-based probes with disambiguation improve predictions. WordNet Information Content similarities match or beat BERT for typicality.
- Relevance: **extends** — DAFHNE's space encodes category structure geometrically (dog closer to animal). The finding that static embeddings fail at typicality while structured resources (WordNet) succeed suggests DAFHNE may capture different information than GPT-2 wte.
- Impact on experiment: May explain a low correlation — DAFHNE captures typicality/category structure, GPT-2 wte capture distributional co-occurrence. These are related but different.

---

## Summary Assessment

**Novelty level: medium**

The comparison of embedding cosine distances with human similarity judgments is well-studied. However, comparing a **non-NN geometric semantic space** (DAFHNE) with NN embeddings (GPT-2 wte) is novel. No paper in our search does exactly this.

**Key insights for r0x-001:**
1. Static embeddings DO encode semantic structure (Kozlowski 2025 confirms hypothesis)
2. Anisotropy is a real concern — consider centering (Mu & Viswanath 2017)
3. Baseline Spearman for embedding-vs-human is 0.3-0.5 (Toshevska 2020), so our 0.5 ALIVE threshold may be aggressive
4. Category/typicality structure may differ between distributional and definition-based spaces (Renner 2023)
5. The 8D DAFHNE space vs 768D GPT-2 space dimensionality mismatch is not a problem per se — Kozlowski shows semantic structure is ~3D in LLM embeddings

**Proceed: yes** — No paper already answers our specific question. The experiment is worth running.
