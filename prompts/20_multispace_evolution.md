# PROMPT 20 — Multi-Space Evolution: Per-Space Parameter Tuning (STUB)

> **STATUS: STUB** — To be expanded when Phase 17+ reveals parameter sensitivity.

## GOAL

Extend the genetic algorithm (yalm-evolve) to tune parameters per-space. Currently all spaces use the same v11 params, but optimal geometry likely differs: MATH may want tighter clustering, GRAMMAR may want more separation between categories.

## CORE IDEA

```
Current: one genome → all spaces
Target:  one genome per space + one genome for routing thresholds

Genome_MATH:    {yes_threshold: 0.8, force_magnitude: 0.3, ...}
Genome_GRAMMAR: {yes_threshold: 1.2, force_magnitude: 0.5, ...}
Genome_CONTENT: {yes_threshold: 1.0, force_magnitude: 0.4, ...}  → may equal v11
Genome_TASK:    {routing_threshold: 0.6, domain_bias: ...}
```

## KEY QUESTIONS

1. Can yalm-evolve run on individual spaces in isolation?
2. Or must evolution consider cross-space performance (requires multi-space eval in fitness function)?
3. Is there a simpler approach: tune only the 2-3 most sensitive params per space?

## PREREQUISITES

- Phase 17+ complete
- Enough test data per space to drive evolution (at least 15-20 questions each)
- Understanding of which params matter most per domain

## ESTIMATED EFFORT

- Design: 1 day
- Implementation: 2-3 days
- Evolution runs: 1-2 days compute
