# PROMPT 21 — Scaling: Open Mode Multi-Space (STUB)

> **STATUS: STUB** — Long-term target.

## GOAL

Extend multi-space architecture to open mode (Ollama-generated dictionaries). Currently open mode works only in single-space. The target: read arbitrary text, auto-generate domain-specific dictionaries, and spin up spaces dynamically.

## CORE IDEA

```
Input: chapter of Three Men in a Boat
  │
  ├─ Ollama generates definitions (existing pipeline)
  ├─ Domain classifier assigns words to spaces:
  │   │  "river", "boat", "Thames" → GEOGRAPHY space
  │   │  "pack", "bag", "clothes" → OBJECTS space
  │   │  "Harris", "George", "Montmorency" → CHARACTERS space
  │   └─ Bridge terms identified automatically
  │
  ├─ Each space gets its own equilibrium
  ├─ TASK space updated with new domains
  └─ Full cross-space querying
```

## KEY CHALLENGES

1. **Automatic domain discovery**: How to cluster words into spaces without manual dictionaries?
2. **Dynamic TASK space**: TASK must learn new domains at runtime
3. **Bridge term quality**: Auto-detected bridges may be noisy
4. **Scaling**: N spaces × M words each — equilibrium cost

## PREREQUISITES

- Phase 19 complete (bootstrap loop working)
- Multi-space evolution (Phase 20) providing good per-space params
- Open mode + Ollama pipeline stable

## ESTIMATED EFFORT

- Design: 3-5 days
- Implementation: 1-2 weeks
- This is a major milestone: YALM becomes a general-purpose reading machine
