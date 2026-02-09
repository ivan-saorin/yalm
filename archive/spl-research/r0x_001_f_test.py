#!/usr/bin/env python3
"""r0x-001: f-Function Test

Determines if GPT-2 static embeddings contain the same geometric
structure as DAPHNE equilibrium space by correlating pairwise distances.

Usage:
    # First, generate the DAPHNE space dump:
    #   cargo run --bin dafhne-eval -- --dict dictionaries/dict5.md \
    #       --test dictionaries/dict5_test.md \
    #       --dump-space research/r0x_001_dafhne_space.json
    #
    # Then run this script:
    #   python research/r0x_001_f_test.py

Verdict thresholds (Spearman):
    > 0.5  = ALIVE  (proceed to r0x-002)
    0.2-0.5 = INCONCLUSIVE (try dict12)
    < 0.2  = DEAD   (skip r0x-002, go to r0x-003)
"""

import json
import sys
from pathlib import Path

import numpy as np
from scipy.spatial.distance import cosine, euclidean
from scipy.stats import spearmanr, pearsonr
from sklearn.decomposition import PCA
from sklearn.manifold import TSNE
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

# ── Paths ────────────────────────────────────────────────────────
SCRIPT_DIR = Path(__file__).parent
SPACE_JSON = SCRIPT_DIR / "r0x_001_dafhne_space.json"
RESULTS_JSON = SCRIPT_DIR / "r0x_001_results.json"
SCATTER_PNG = SCRIPT_DIR / "r0x_001_scatter.png"
PROJECTION_PNG = SCRIPT_DIR / "r0x_001_projection.png"
VERDICT_MD = SCRIPT_DIR / "r0x_001_verdict.md"

# Structural / function words in dict5 (connectors + grammar)
STRUCTURAL_WORDS = {
    "a", "and", "can", "has", "in", "is", "it", "not", "of",
    "on", "or", "the", "this", "to", "with",
    "what", "yes", "no", "you",
}


# ── Step 1: Load DAPHNE space ─────────────────────────────────────

def load_dafhne_space(path: Path):
    """Load DAPHNE geometric space and compute pairwise Euclidean distances."""
    with open(path) as f:
        space = json.load(f)

    words_data = space["words"]  # dict: word -> {word, position}
    words = sorted(words_data.keys())
    positions = {w: np.array(words_data[w]["position"]) for w in words}

    # Pairwise Euclidean distances
    distances = {}
    for i, w1 in enumerate(words):
        for j in range(i + 1, len(words)):
            w2 = words[j]
            distances[(w1, w2)] = np.linalg.norm(positions[w1] - positions[w2])

    print(f"[DAPHNE] {len(words)} words, {len(distances)} pairs, "
          f"{len(positions[words[0]])} dimensions")
    return words, positions, distances


# ── Step 2: Extract GPT-2 static embeddings ─────────────────────

def extract_gpt2_embeddings(words):
    """Extract static (wte) embeddings from GPT-2 for each word."""
    from transformers import GPT2Tokenizer, GPT2Model
    import torch

    print("[GPT-2] Loading model...")
    model = GPT2Model.from_pretrained("gpt2")
    tokenizer = GPT2Tokenizer.from_pretrained("gpt2")

    embeddings = {}
    multi_token = []

    for word in words:
        tokens = tokenizer.encode(word, add_special_tokens=False)
        token_embs = model.wte.weight[tokens]
        embeddings[word] = token_embs.mean(dim=0).detach().numpy()

        if len(tokens) > 1:
            decoded = tokenizer.convert_ids_to_tokens(tokens)
            multi_token.append((word, len(tokens), decoded))

    if multi_token:
        print(f"[GPT-2] WARNING: {len(multi_token)} multi-token words:")
        for w, n, toks in multi_token:
            print(f"  {w} -> {n} tokens: {toks}")
    else:
        print("[GPT-2] All words are single-token.")

    dim = len(embeddings[words[0]])
    print(f"[GPT-2] {len(embeddings)} embeddings, {dim} dimensions")
    return embeddings, multi_token


# ── Step 3: Compute GPT-2 pairwise distances ────────────────────

def compute_gpt2_distances(words, embeddings):
    """Compute pairwise cosine and Euclidean distances."""
    distances = {}
    for i, w1 in enumerate(words):
        for j in range(i + 1, len(words)):
            w2 = words[j]
            distances[(w1, w2)] = {
                "cosine": float(cosine(embeddings[w1], embeddings[w2])),
                "euclidean": float(euclidean(embeddings[w1], embeddings[w2])),
            }
    return distances


# ── Step 4: Correlate ────────────────────────────────────────────

def correlate(dafhne_dists, gpt2_dists, label="all"):
    """Compute Spearman and Pearson correlations between DAPHNE and GPT-2 distances."""
    pairs = sorted(set(dafhne_dists.keys()) & set(gpt2_dists.keys()))
    if len(pairs) < 10:
        print(f"  [{label}] Only {len(pairs)} pairs — skipping.")
        return None

    y = [dafhne_dists[p] for p in pairs]
    g_cos = [gpt2_dists[p]["cosine"] for p in pairs]
    g_euc = [gpt2_dists[p]["euclidean"] for p in pairs]

    sp_cos, p_cos = spearmanr(y, g_cos)
    sp_euc, p_euc = spearmanr(y, g_euc)
    pe_cos, _ = pearsonr(y, g_cos)
    pe_euc, _ = pearsonr(y, g_euc)

    result = {
        "label": label,
        "n_pairs": len(pairs),
        "spearman_cosine": float(sp_cos),
        "p_cosine": float(p_cos),
        "spearman_euclidean": float(sp_euc),
        "p_euclidean": float(p_euc),
        "pearson_cosine": float(pe_cos),
        "pearson_euclidean": float(pe_euc),
    }

    print(f"\n  [{label}] {len(pairs)} pairs")
    print(f"    Spearman (cosine):    {sp_cos:+.4f}  p={p_cos:.2e}")
    print(f"    Spearman (euclidean): {sp_euc:+.4f}  p={p_euc:.2e}")
    print(f"    Pearson  (cosine):    {pe_cos:+.4f}")
    print(f"    Pearson  (euclidean): {pe_euc:+.4f}")
    return result


def filter_pairs(distances, exclude_words):
    """Remove pairs where either word is in exclude_words."""
    return {k: v for k, v in distances.items()
            if k[0] not in exclude_words and k[1] not in exclude_words}


# ── Step 5: Visualize ───────────────────────────────────────────

def plot_scatter(dafhne_dists, gpt2_dists, results, output_path):
    """Scatter plot: DAPHNE dist vs GPT-2 dist (cosine and euclidean)."""
    pairs = sorted(set(dafhne_dists.keys()) & set(gpt2_dists.keys()))
    y = [dafhne_dists[p] for p in pairs]
    g_cos = [gpt2_dists[p]["cosine"] for p in pairs]
    g_euc = [gpt2_dists[p]["euclidean"] for p in pairs]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

    ax1.scatter(y, g_cos, alpha=0.25, s=6, c="steelblue")
    ax1.set_xlabel("DAPHNE Euclidean Distance")
    ax1.set_ylabel("GPT-2 Cosine Distance")
    rho = results["spearman_cosine"]
    ax1.set_title(f"Cosine — Spearman ρ = {rho:+.4f}")

    ax2.scatter(y, g_euc, alpha=0.25, s=6, c="indianred")
    ax2.set_xlabel("DAPHNE Euclidean Distance")
    ax2.set_ylabel("GPT-2 Euclidean Distance")
    rho = results["spearman_euclidean"]
    ax2.set_title(f"Euclidean — Spearman ρ = {rho:+.4f}")

    fig.suptitle("r0x-001: DAPHNE vs GPT-2 Pairwise Distance Correlation", y=1.02)
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"[Plot] Saved scatter to {output_path}")


def plot_projections(dafhne_positions, gpt2_embeddings, words, output_path):
    """Side-by-side PCA and t-SNE projections of both spaces."""
    dafhne_mat = np.array([dafhne_positions[w] for w in words])
    gpt2_mat = np.array([gpt2_embeddings[w] for w in words])

    fig, axes = plt.subplots(2, 2, figsize=(18, 16))

    # PCA
    dafhne_pca = PCA(n_components=2).fit_transform(dafhne_mat)
    gpt2_pca = PCA(n_components=2).fit_transform(gpt2_mat)

    # t-SNE (perplexity must be < n_samples)
    perp = min(15, len(words) - 1)
    dafhne_tsne = TSNE(n_components=2, perplexity=perp, random_state=42).fit_transform(dafhne_mat)
    gpt2_tsne = TSNE(n_components=2, perplexity=perp, random_state=42).fit_transform(gpt2_mat)

    datasets = [
        (axes[0, 0], dafhne_pca,  "DAPHNE (PCA)"),
        (axes[0, 1], gpt2_pca,  "GPT-2 wte (PCA)"),
        (axes[1, 0], dafhne_tsne, "DAPHNE (t-SNE)"),
        (axes[1, 1], gpt2_tsne, "GPT-2 wte (t-SNE)"),
    ]

    for ax, data, title in datasets:
        # Color by category
        colors = []
        for w in words:
            if w in STRUCTURAL_WORDS:
                colors.append("gray")
            elif w in {"dog", "cat", "sun", "ball"}:
                colors.append("red")
            elif w in {"animal", "person", "food", "water", "color",
                       "place", "sound", "part", "name", "one", "all"}:
                colors.append("green")
            elif w in {"big", "small", "good", "bad", "hot", "cold", "up", "down"}:
                colors.append("orange")
            elif w in {"see", "feel", "move", "make", "eat", "give", "live", "do"}:
                colors.append("purple")
            else:
                colors.append("steelblue")

        ax.scatter(data[:, 0], data[:, 1], s=30, c=colors, alpha=0.7)
        for i, w in enumerate(words):
            ax.annotate(w, (data[i, 0], data[i, 1]),
                       fontsize=6, alpha=0.8,
                       xytext=(3, 3), textcoords="offset points")
        ax.set_title(title, fontsize=12)
        ax.set_xticks([])
        ax.set_yticks([])

    fig.suptitle("r0x-001: Word Space Projections — DAPHNE vs GPT-2", fontsize=14)
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"[Plot] Saved projections to {output_path}")


# ── Step 6: Verdict ──────────────────────────────────────────────

def determine_verdict(results_all):
    """Apply success criteria from r0x-001 spec."""
    sp_cos = results_all["spearman_cosine"]
    sp_euc = results_all["spearman_euclidean"]

    if sp_cos > 0.5 or sp_euc > 0.5:
        return "ALIVE"
    elif sp_cos < 0.2 and sp_euc < 0.2:
        return "DEAD"
    else:
        return "INCONCLUSIVE"


def write_verdict(results_all, results_content, verdict, multi_token, path):
    """Write verdict markdown file."""
    sp_cos = results_all["spearman_cosine"]
    sp_euc = results_all["spearman_euclidean"]
    n = results_all["n_pairs"]

    lines = [
        f"# r0x-001 Verdict: **{verdict}**\n",
        f"## Results\n",
        f"| Metric | All words ({n} pairs) |",
        f"|--------|----------------------|",
        f"| Spearman (cosine) | {sp_cos:+.4f} |",
        f"| Spearman (euclidean) | {sp_euc:+.4f} |",
        f"| Pearson (cosine) | {results_all['pearson_cosine']:+.4f} |",
        f"| Pearson (euclidean) | {results_all['pearson_euclidean']:+.4f} |",
        f"| p-value (cosine) | {results_all['p_cosine']:.2e} |",
        f"| p-value (euclidean) | {results_all['p_euclidean']:.2e} |",
        "",
    ]

    if results_content:
        nc = results_content["n_pairs"]
        lines += [
            f"| Metric | Content words only ({nc} pairs) |",
            f"|--------|-------------------------------|",
            f"| Spearman (cosine) | {results_content['spearman_cosine']:+.4f} |",
            f"| Spearman (euclidean) | {results_content['spearman_euclidean']:+.4f} |",
            "",
        ]

    if multi_token:
        lines.append("## Multi-token words (noise sources)")
        for w, n_tok, toks in multi_token:
            lines.append(f"- **{w}** -> {n_tok} tokens: {toks}")
        lines.append("")

    lines += [
        "## Thresholds",
        "| Spearman | Verdict |",
        "|----------|---------|",
        "| > 0.5 | ALIVE |",
        "| 0.2 - 0.5 | INCONCLUSIVE |",
        "| < 0.2 | DEAD |",
        "",
        "## Next Steps",
    ]

    if verdict == "ALIVE":
        lines.append("Proceed to **r0x-002** (f-function extraction / NN decompiler).")
    elif verdict == "DEAD":
        lines.append("Skip r0x-002. Proceed to **r0x-003** (SPL integration without NN bridge).")
    else:
        lines.append("Try with **dict12** (1005 words) for more statistical power, or proceed to r0x-003.")

    path.write_text("\n".join(lines), encoding="utf-8")
    print(f"[Verdict] {verdict} — written to {path}")


# ── Main ─────────────────────────────────────────────────────────

def main():
    print("=" * 60)
    print("r0x-001: f-Function Test")
    print("=" * 60)

    # Step 1: Load DAPHNE space
    if not SPACE_JSON.exists():
        print(f"\nERROR: {SPACE_JSON} not found.")
        print("Run DAPHNE first:")
        print("  cargo run --bin dafhne-eval -- \\")
        print("    --dict dictionaries/dict5.md \\")
        print("    --test dictionaries/dict5_test.md \\")
        print("    --dump-space research/r0x_001_dafhne_space.json")
        sys.exit(1)

    words, dafhne_positions, dafhne_dists = load_dafhne_space(SPACE_JSON)

    # Step 2: Extract GPT-2 embeddings
    gpt2_embeddings, multi_token = extract_gpt2_embeddings(words)

    # Step 3: Compute GPT-2 distances
    print("\n[Computing GPT-2 pairwise distances...]")
    gpt2_dists = compute_gpt2_distances(words, gpt2_embeddings)

    # Step 4: Correlate
    print("\n=== Correlation Results ===")

    results_all = correlate(dafhne_dists, gpt2_dists, label="all_words")

    # Content words only (exclude structural)
    dafhne_content = filter_pairs(dafhne_dists, STRUCTURAL_WORDS)
    gpt2_content = filter_pairs(gpt2_dists, STRUCTURAL_WORDS)
    results_content = correlate(dafhne_content, gpt2_content, label="content_only")

    # Without multi-token words (if any)
    results_single = None
    if multi_token:
        multi_words = {w for w, _, _ in multi_token}
        dafhne_single = filter_pairs(dafhne_dists, multi_words)
        gpt2_single = filter_pairs(gpt2_dists, multi_words)
        results_single = correlate(dafhne_single, gpt2_single, label="single_token_only")

    # Step 5: Visualize
    print("\n=== Generating Plots ===")
    plot_scatter(dafhne_dists, gpt2_dists, results_all, SCATTER_PNG)
    plot_projections(dafhne_positions, gpt2_embeddings, words, PROJECTION_PNG)

    # Step 6: Verdict
    verdict = determine_verdict(results_all)

    # Save results JSON
    output = {
        "experiment": "r0x-001",
        "n_words": len(words),
        "words": words,
        "multi_token_words": [
            {"word": w, "n_tokens": n, "tokens": t}
            for w, n, t in multi_token
        ],
        "results_all_words": results_all,
        "results_content_only": results_content,
        "results_single_token_only": results_single,
        "verdict": verdict,
    }
    RESULTS_JSON.write_text(json.dumps(output, indent=2), encoding="utf-8")
    print(f"\n[Results] Saved to {RESULTS_JSON}")

    # Write verdict
    print("\n" + "=" * 60)
    write_verdict(results_all, results_content, verdict, multi_token, VERDICT_MD)
    print("=" * 60)


if __name__ == "__main__":
    main()
