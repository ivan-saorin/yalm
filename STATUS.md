# dafhne → DAFHNE — Current Status

> Last updated: 2026-02-09 (Phase 21)
>
> **Name change decided**: dafhne → **DAFHNE** (Definition-Anchored Force-field Heuristic Network Engine)
> *Dafhne = laurel = victory in Greek. Rename phase pending.*

## Current Score

| Test Suite | Score | Notes |
|------------|-------|-------|
| dict5 (single-space) | 20/20 | Perfect |
| dict12 (single-space) | 14/20 | Stable since v11 |
| unified_test (5-space) | **50/50** | CONTENT+MATH+GRAMMAR+TASK+SELF — PERFECT |
| Bootstrap | 4 new connectors, converges Level 2 | Self-improvement loop works |

## Architecture

5 geometric spaces (CONTENT, MATH, GRAMMAR, TASK, SELF), each an independent DAFHNE instance with per-space evolved parameters, connected via bridge terms. Bootstrap loop reads own describe() output → discovers new connectors → re-equilibrates.

## Recent Phases

| Phase | Status | Summary |
|-------|--------|---------|
| 21 | ✅ Complete | Hard questions: 5 surgical fixes, 45/50 → 50/50 |
| 20 | ✅ Complete | Per-space parameter evolution, per-space genome tuning |
| 19c | ✅ Complete | Code audit fixes: 16/24 findings fixed, 0 regression |
| 19b | ✅ Complete | Code audit + README overhaul + prior art analysis |
| 19 | ✅ Complete | Bootstrap loop: connector enrichment from self-generated text |
| 18 | ✅ Complete | SELF space: identity and capabilities as geometry |
| 17 | ✅ Complete | CONTENT space integration into multi-space |
| 16 | ✅ Complete | Multi-space architecture (MATH+GRAMMAR+TASK) |

## Key Research Findings

### AxisShift Negation — The Quiet Winner

Evolution selects `NegationModel::AxisShift` at 96% across all runs. While the definition-chain gate handles all practical negation (geometry alone never solved negation in 100+ generations), AxisShift is the geometric strategy that *least interferes* with the chain gate. It shifts negated words along a perpendicular axis rather than inverting or repelling them, which preserves the distance relationships the chain gate relies on.

This is worth revisiting: AxisShift doesn't solve negation, but it creates a geometric configuration that's *compatible* with symbolic negation. The other three models (Inversion, Repulsion, SeparateDimension) actively fight the chain gate by distorting distances. AxisShift cooperates.

**Implication**: If a future phase attempts geometric negation (without chain gate), AxisShift's perpendicular-shift approach is the starting point — it's the only model that preserves positive-relationship geometry while encoding negation signal.

### The Hybrid Finding

Geometry encodes similarity. Definitions encode identity. You need both. ~35% of answers rely primarily on geometry, ~40% on symbolic chain operations, ~25% on geometric absence (honesty). This is the central research result.

### ELI5 Closure as Innovation

The novel contribution isn't geometry or forces or evolution — it's the constraint that definitions use only defined words, creating a self-consistent universe. This is what makes 51 words sufficient for comprehension. See `docs/prior_art.md`.

### Per-Space Parameter Divergence (Phase 20)

Evolution confirms that different knowledge domains need different geometric configurations: MATH needs high dimensions (29), TASK needs very few (5). CONTENT favors Gravitational force, GRAMMAR favors Spring, MATH and TASK favor Linear. Shared parameters leave 10% performance on the table.

## What's Broken / Known Issues

- dict12 Q04 (Can a cat climb?) — chain needs 3+ hops through richer definitions
- dict12 Q09 (Does a plant need water?) — "need" is causal, not taxonomic
- Phase 15 (property extraction) never implemented — bootstrap works without it but sub-optimally
- Math space will be reworked (hardcoded number-to-word mapping, math routing indicators)

## Pending Decisions

- **Rename DAFHNE → DAFHNE**: Requires dedicated phase (crate names, Cargo.toml, imports, README, all docs, git history note)
- **Phase 22**: Open mode multi-space (LLM-assembled dictionaries per domain)
- **Phase 23**: Property extraction for richer bootstrap signal

## Code Health (post-21)

| Severity | Count | Was (19c) |
|----------|-------|-----------|
| 🔴 VIOLATION | 1 | 1 |
| 🟡 PRAGMATIC | 6 | 5 |
| 🟢 ALIGNED | 4 | 4 |
| ⚪ TECH DEBT | 4 | 4 |

Remaining 🔴: A04 (hardcoded number-to-word mapping) — deferred to math rework.
New 🟡: Phase 21 adds hardcoded domain indicator lists in quoted-phrase routing (Fix 3) — consistent with existing math routing approach, to be addressed in math rework.
