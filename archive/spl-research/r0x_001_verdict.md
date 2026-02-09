# r0x-001 Verdict: **DEAD**

## Results

| Metric | All words (1275 pairs) |
|--------|----------------------|
| Spearman (cosine) | +0.0139 |
| Spearman (euclidean) | -0.1726 |
| Pearson (cosine) | +0.0478 |
| Pearson (euclidean) | -0.0415 |
| p-value (cosine) | 6.21e-01 |
| p-value (euclidean) | 5.55e-10 |

| Metric | Content words only (496 pairs) |
|--------|-------------------------------|
| Spearman (cosine) | +0.1202 |
| Spearman (euclidean) | -0.0305 |

## Thresholds
| Spearman | Verdict |
|----------|---------|
| > 0.5 | ALIVE |
| 0.2 - 0.5 | INCONCLUSIVE |
| < 0.2 | DEAD |

## Next Steps
Skip r0x-002. Proceed to **r0x-003** (SPL integration without NN bridge).