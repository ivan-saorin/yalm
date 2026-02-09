# r0x — Pure Research Experiments

**Branch:** pure-research
**Created:** 2025-02-08
**Status:** Active

## Dependency Graph

```
r0x-001 (f-function test)
  │
  ├─ ALIVE ──→ r0x-002 (f-function extraction / NN decompiler)
  │
  └─ DEAD ───→ skip r0x-002, document

r0x-003 (SPL equilibrium)  ←── independent, start in parallel
  │
  ├─ Phase D (set operations) ──→ r0x-004 (MoE geometric gating)
  │
  └─ SPL + r0x-004 Phase A ──→ r0x-005 (geometric generation)
```

## Prior Art Protocol

Every r0x experiment has a **Step 0: Prior Art Search** that MUST run before any code is written.

Uses `paper-search2` MCP server (configured in `.claude/settings.json`).

### Workflow

1. Run the search queries listed in the experiment's "Prior Art Search" section
2. Read abstracts of top 5 most relevant papers
3. For papers directly relevant: `read_arxiv_paper()` or `read_semantic_paper()` for full text
4. Log findings in `research/r0x-0XX_prior_art.md`
5. Decision:
   - **Already solved**: cite paper, use their numbers, skip implementation
   - **Partially explored**: adapt their methodology, credit
   - **Novel**: proceed with implementation, note differentiation
   - **Known to fail**: document why, close experiment early

### Prior Art File Format

```markdown
# r0x-0XX Prior Art

## Search Date: YYYY-MM-DD

### Paper 1: [Title]
- Authors: ...
- Year: ...
- Link: ...
- Key finding: ...
- Relevance: confirms_hypothesis / contradicts / extends / methodology_useful
- Impact on experiment: ...

### Paper 2: ...

## Summary Assessment
Novelty level: high / medium / low
Proceed: yes / adapt / skip
```

---

## Execution Order

| Priority | Experiment | Effort | Blocked By |
|----------|-----------|--------|------------|
| 1 | r0x-001 | 1 afternoon | Nothing |
| 1 | r0x-003 Phase A-B | 2-3 days | Nothing |
| 2 | r0x-002 | 1 day | r0x-001 = ALIVE |
| 2 | r0x-003 Phase C-D | 1-2 days | r0x-003 Phase B |
| 3 | r0x-004 | 2-3 days | r0x-003 Phase D |
| 4 | r0x-005 | 2-3 days | r0x-003 + r0x-004 |

**Start with r0x-001 and r0x-003 in parallel.** r0x-001 is quick (one afternoon) and its result determines if the NN bridge path is worth pursuing.

## Kill Criteria

**Prior art kill**: If Step 0 reveals the experiment is already solved or proven impossible, write verdict immediately. No code needed.

**Experimental kill**: Each experiment has explicit pass/fail thresholds.

Each experiment has explicit pass/fail thresholds. If FAIL:
1. Write verdict in `r0x-0XX_verdict.md`
2. Commit to pure-research branch
3. Do NOT merge to main
4. Move to next experiment (some paths are independent)

If ALL fail: the integration thesis is wrong. DAFHNE and SPL are better kept separate. Still valuable knowledge.

## Merge Criteria

An experiment merges to main ONLY if:
1. Pass criteria met
2. No regression on existing tests (dict5: 20/20, dict12: 14/20, full_test: 19/21)
3. Code is production-grade (no research hacks left in)

## Dependencies

### Python (for r0x-001, r0x-002)
```
pip install torch transformers scipy scikit-learn matplotlib numpy
```

### Rust (for r0x-003)
Extends existing DAFHNE crates or new `dafhne-spl` crate.

### Ollama (for r0x-004, r0x-005)
Already configured for DAFHNE open-mode. Uses qwen3 model.

## File Structure

```
research/
├── r0x-README.md                    # This file
├── r0x-001-f-function-test.md       # Prompt
├── r0x-002-f-function-extraction.md # Prompt
├── r0x-003-spl-equilibrium.md       # Prompt
├── r0x-004-moe-geometric-gating.md  # Prompt
├── r0x-005-geometric-generation.md  # Prompt
├── dict_science5.md                 # Test dictionary (r0x-003)
├── r0x_001_*.py / .json / .png      # Outputs
├── r0x_002_*.py / .json / .png
├── r0x_003_*.rs / .json / .png
├── r0x_004_*.py / .json / .png
├── r0x_005_*.py / .json / .png
└── r0x_0XX_verdict.md               # Per-experiment verdicts
```
