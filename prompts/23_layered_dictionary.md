# PROMPT 23 — Layered Dictionary Engine

> **STATUS: Stub. Foundation for package system and runtime vocabulary growth.**
>
> **Depends on**: Phase 22 (server running)
> **Enables**: Phase 24 (packages), Phase 25 (chat-as-config)

## GOAL

Replace flat dictionary loading with a 3-layer merge system: Core → Packages → User. The engine trains from the merged result. Closure is validated across all layers. Server rebuilds geometry on any layer change and pushes to connected clients.

## THE THREE LAYERS

```
Layer 3: USER      — household customizations, grows via chat
Layer 2: PACKAGES  — installed feature sets (music, lights, etc.)
Layer 1: CORE      — immutable base vocabulary (~100 words)
```

Resolution: top-down. User overrides package overrides core.

## CORE DICTIONARY DESIGN

The core must be large enough that packages can define domain words using only core terms. Minimum viable core:

```
# Categories
thing, person, place, animal, food, machine, tool

# Properties  
big, small, hot, cold, fast, slow, loud, quiet, new, old, good, bad

# Spatial
in, on, at, up, down, near, far, inside, outside

# Temporal
now, before, after, when, always, never, sometimes

# Actions (abstract)
make, change, move, start, stop, give, take, find, show, hide

# States
on, off, open, closed, full, empty, ready

# Structure
a, an, the, is, are, not, and, or, of, to, for, with, from, by
it, this, that, one, all, some, no, every, each, more, less

# Relations
has, can, does, like, same, other, part, kind, way, about

# Numbers
zero through ten, number, count, how-many

# Colors
red, blue, green, yellow, white, black, color

# People/Communication
name, say, ask, answer, tell, know, want, need, help

# Home (borderline core vs package, but universally useful)
room, home, door, window, floor, wall, light, water, air
```

**Key decision**: Where to draw the line between core and first-party packages. The core should be domain-agnostic. "light" might belong in core (it's a basic concept) but "dimmer" belongs in a lighting package.

## IMPLEMENTATION

### Data Structures

```rust
/// A dictionary entry with provenance tracking
struct LayeredEntry {
    word: String,
    definition: String,
    source: EntrySource,
    original_definition: Option<String>,  // if overridden, what was underneath
}

enum EntrySource {
    Core,
    Package { name: String, version: String },
    User,
}

/// The merged dictionary state
struct LayeredDictionary {
    entries: BTreeMap<String, LayeredEntry>,
    core_words: HashSet<String>,
    package_words: HashMap<String, HashSet<String>>,  // pkg_name → words
    user_words: HashSet<String>,
}
```

### Merge Algorithm

```rust
impl LayeredDictionary {
    fn merge(
        core: &Dictionary,
        packages: &[Package],
        user: &Dictionary,
    ) -> Result<Self, ClosureError> {
        // 1. Insert core (lowest priority)
        // 2. Insert packages (dependency order — topological sort)
        // 3. Insert user (highest priority)
        // 4. Validate closure: every content word in every definition
        //    must exist as a key in the merged map
        // 5. Return merged or ClosureError listing violations
    }
    
    fn add_user_word(&mut self, word: &str, definition: &str) -> Result<(), ClosureError> {
        // Validate closure BEFORE inserting
        // If valid: insert, mark source=User, trigger rebuild signal
        // If invalid: return which words are undefined
    }
    
    fn remove_user_word(&mut self, word: &str) -> Result<(), DependencyError> {
        // Check if any OTHER user word depends on this one
        // If yes: return DependencyError listing dependents
        // If no: remove, trigger rebuild signal
    }
    
    fn to_flat_dictionary(&self) -> Dictionary {
        // Export merged entries as a flat dict for Engine::train()
    }
}
```

### Closure Validation

```rust
struct ClosureError {
    word: String,
    definition: String,
    undefined_words: Vec<String>,
    suggestions: Vec<(String, String)>,  // (word, "did you mean X?")
}
```

The validator must be fast — it runs on every user addition. For a 300-word merged dictionary, it's O(n) string lookups in a HashSet. Trivial.

### Server Integration

```rust
// In dafhne-server
struct DafhneService {
    dictionary: RwLock<LayeredDictionary>,
    engine: RwLock<MultiSpace>,  // rebuilt from merged dict
    rebuild_notify: broadcast::Sender<()>,  // notify clients
}

impl DafhneService {
    async fn add_word(&self, word: &str, def: &str) -> Result<()> {
        let mut dict = self.dictionary.write().await;
        dict.add_user_word(word, def)?;
        let flat = dict.to_flat_dictionary();
        drop(dict);
        
        let mut engine = self.engine.write().await;
        *engine = MultiSpace::train(&flat, &genome)?;
        
        self.rebuild_notify.send(())?;
        Ok(())
    }
}
```

### Client Sync Protocol

```
GET /api/dictionary/version → { "version": "sha256:abc123", "word_count": 287 }
GET /api/dictionary/space   → serialized GeometricSpace (pre-built, client skips training)
WS  /api/dictionary/watch   → push notification on change
```

Client startup:
1. Check local cache version vs server version
2. If stale: download pre-built space (not raw dict — skip training on client)
3. Subscribe to watch endpoint for live updates
4. On update: download new space, hot-swap engine

## FILESYSTEM LAYOUT

```
dafhne-server/
├── core/
│   └── core.dict.md           # ships with binary, immutable
├── packages/
│   ├── music/
│   │   ├── package.toml
│   │   └── dictionary.md
│   └── philips-hue/
│       ├── package.toml
│       └── dictionary.md
├── user/
│   └── dictionary.md          # grows via chat, persisted
└── cache/
    └── merged.space.bin        # serialized geometry cache
```

## TESTING

- Merge core + empty packages + empty user → identical to current dict5 behavior
- Add user word with valid closure → geometry rebuilds, word answerable
- Add user word with invalid closure → error listing undefined words
- Override package word → user definition takes priority
- Remove user override → package definition resurfaces
- Package dependency ordering → no forward references
- Client sync → version check, space download, hot-swap

## OPEN QUESTIONS

- Should the multi-space architecture be per-package? (music gets its own space vs. merging into content space)
- How to handle word collisions across packages? ("volume" in music vs "volume" in math)
- Should user overrides be per-space or global?
- Incremental re-equilibration vs full rebuild on mutation?

## WHAT NOT TO DO

- Do NOT build the package installer yet (Phase 24)
- Do NOT build chat-as-config yet (Phase 25)
- Do NOT modify existing crates — this is a new module in dafhne-server or a new crate
- Do NOT implement adapter/binding system yet (Phase 24)
