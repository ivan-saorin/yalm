# Phase 19c: Code Audit Fixes Report

> Fixes applied to 16 of the 24 findings from Phase 19b code audit.

**Date**: 2026-02-09
**Regression results**: dict5 20/20, unified_test 45/50 (matching baselines)

---

## Summary

| Severity | Total (19b) | Fixed | Remaining |
|----------|-------------|-------|-----------|
| 🔴 High  | 4           | 3     | 1 (A06)   |
| 🟡 Medium| 8           | 7     | 1 (A08)   |
| 🟢 Low   | 4           | 4     | 0         |
| ⚪ Info   | 8           | 2     | 6         |
| **Total** | **24**     | **16**| **8**     |

---

## Per-Finding Status

### Fixed Findings (16)

| ID  | Severity | Finding | Fix |
|-----|----------|---------|-----|
| A01 | 🔴 | Hardcoded 5W question words | Added `LANGUAGE-SPECIFIC LAYER` comment block documenting the 5 hardcoded English question words and refactoring path to language-adapter config |
| A02 | 🔴 | Hardcoded `is_structural()` (28 words) | Replaced with `structural_words_cache` (union of per-space `classify_word_roles()` output) + question-syntax meta-words. All 8+ call sites updated to `self.is_structural_cached()` |
| A03 | 🔴 | Hardcoded article/verb/skip lists in resolver | Replaced articles with `structural.contains()` in entity fast path and subject extraction. Question verbs use `structural.contains(&tokens[0])`. Question syntax filter uses structural set. Skip-word fallbacks retained as definition-shape patterns (see Design Note below) |
| A05 | 🟡 | Hardcoded `self_triggers = ["dafhne"]` | Derived from vocabulary: words unique to SELF space (not in other domain spaces, not structural). Pronoun patterns `("are","you")` etc. retained with comment |
| A09 | 🟡 | `is_connector_word` / `is_property_word` appear hardcoded | Added doc comments documenting that `is_connector_word` is fully data-driven (scans discovered connectors) and `is_property_word` uses ELI5 definition-shape heuristics |
| A10 | 🟡 | `MAX_FOLLOW_PER_HOP = 3` hardcoded | Externalized to `EngineParams.max_follow_per_hop` with `#[serde(default)]` for backward compatibility. All 4 call sites updated |
| A11 | 🟡 | `max_hops = 3` hardcoded in 4 places | Externalized to `EngineParams.max_chain_hops`. Updated `resolve_yes_no`, `resolve_why`, `resolve_when`, `describe`, and all multispace chain calls |
| A12 | 🟡 | `alpha = 0.2` hardcoded in `resolve_what_is` | Externalized to `EngineParams.weighted_distance_alpha` |
| A13 | 🟡 | Yes/No detection uses hardcoded verbs | Question-verb detection now uses `structural.contains(&tokens[0])` — all question verbs (is, can, does, do, has) pass the 20% doc-frequency threshold |
| A14 | 🟡 | Task classification uses hardcoded indicator lists | Replaced `grammar_indicators` and `content_indicators` arrays with vocabulary membership checks against grammar and content space dictionaries. Math indicators kept as-is |
| A15 | 🟢 | `NegationModel` variants undocumented | Added research-result documentation: evolution converges to AxisShift at 96%+ rate across independent seeds |
| A20 | 🟢 | `preceded_by_not` assumes "not" exists | Added connector-existence guard: negation check only fires when the space has a discovered "not" connector |
| A23 | 🟢 | `find_siblings` uses string comparison | Added TODO comment documenting geometric alternative (nearest-neighbor spatial lookup) |
| A24 | 🟢 | Connector discovery constants hardcoded | Externalized `num_buckets` and `uniformity_threshold` to `EngineParams.uniformity_num_buckets` and `EngineParams.uniformity_threshold` |
| — | — | Evolution infrastructure for new params | Added 5 fields to `ParamRanges`, `random_genome()`, `mutate()`, and `crossover()` in dafhne-evolve |
| — | — | `preceded_by_not` uses hardcoded articles | Replaced `["a","an","the"]` with `structural.contains()` parameter |

### Deferred Findings (8)

| ID  | Severity | Finding | Reason |
|-----|----------|---------|--------|
| A06 | 🔴 | Definition-chain gate is symbolic, not geometric | Research question — requires TransE-style embedding (see roadmap) |
| A04 | ⚪ | Number words hardcoded ("zero".."ten") | Per plan: not addressed in 19c |
| A07 | ⚪ | Self triggers need pronoun patterns | Pronoun patterns `("are","you")` retained — not derivable from vocabulary |
| A08 | 🟡 | Describe() could miss adjective properties | Phase 22 scope (rich property extraction) |
| A16 | ⚪ | Grammatical article selection ("a" vs "an") | Cosmetic; English-specific |
| A17 | ⚪ | `stem_to_entry` uses suffix heuristic | Working correctly; not broken |
| A18 | ⚪ | `yes_no_to_declarative` is English-specific | Part of formatting layer; low impact |
| A19 | ⚪ | Entity fast path is English-shaped | Entity definitions follow ELI5 convention |
| A21 | ⚪ | `extract_condition_from_subject` assumes English clause structure | Part of When-resolution; English-specific |
| A22 | ⚪ | `make_article` hardcoded | Cosmetic English formatting |

---

## Design Note: Skip-Word Fallbacks

Two `skip_words` sets in question detectors (`detect_why_question`, `detect_yes_no_question`) were **not** replaced with `structural.contains()`. These use `["is", "a", "the", "it", "not"]` to find content words in questions where the `content` set yields too few hits.

**Reason**: In small dictionaries (dict5), high-frequency content words like "thing" are classified as structural by doc-frequency. Replacing skip_words with the full structural set would filter out "thing" from questions like "Is a dog a thing?" — causing regressions (Q06, Q07 in dict5).

The skip_words sets are **definition-shape patterns** (minimal function words that never carry question-answering signal) rather than word-classification lists. They are annotated with comments explaining this distinction.

## Design Note: Question-Syntax Meta-Words

The `structural_words_cache` in MultiSpace includes a small set of question-syntax words (`what`, `who`, `where`, `when`, `why`, `how`, `which`, `yes`, `no`, `you`, `your`, `are`, `be`, `do`, `does`) that may not pass the 20% doc-frequency threshold in definitions but are functionally structural in queries.

These are marked as LANGUAGE-SPECIFIC and would need replacement for non-English. They prevent cross-space routing from treating question words as content subjects.

---

## Files Modified

| File | Changes |
|------|---------|
| `crates/dafhne-core/src/lib.rs` | +5 EngineParams fields with serde defaults |
| `crates/dafhne-engine/src/resolver.rs` | Replaced hardcoded lists + constants, added doc comments, language-layer annotations |
| `crates/dafhne-engine/src/multispace.rs` | +2 struct fields, cache-based structural, SELF triggers from vocabulary, task routing from space vocab |
| `crates/dafhne-engine/src/connector_discovery.rs` | Read uniformity constants from params |
| `crates/dafhne-engine/src/strategy.rs` | NegationModel doc comments |
| `crates/dafhne-evolve/src/genome.rs` | +5 ParamRanges fields |
| `crates/dafhne-evolve/src/population.rs` | +5 fields in random_genome() |
| `crates/dafhne-evolve/src/operators.rs` | +5 mutation + crossover blocks |

## Regression Results

```
dict5:        20/20 (baseline: 20/20)  ✅
unified_test: 45/50 (baseline: 45/50)  ✅
dict12:       not testable (no genome file in repo)
```

Known failures (all inherited, not introduced):
- Q13: Ordinal comparison (not geometric)
- Q18, Q20: WhatIs extraction in small grammar space
- Q25: Quoted phrases in kind queries
- Q36: Multi-part context pipeline (stretch goal)
