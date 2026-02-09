# PROMPT 25 — Chat-as-Config + Runtime Mutation

> **STATUS: Stub. Teach DAFHNE new words through conversation.**
>
> **Depends on**: Phase 23 (layered dictionary), Phase 24 (packages)
> **Enables**: Phase 26 (Flutter client — voice config)

## GOAL

Users configure DAFHNE by talking to it. Add words, name devices, define rooms, set preferences — all through natural language. Unknown words trigger a definition dialogue. The dictionary grows through use.

## COMMAND GRAMMAR

DAFHNE detects configuration commands by prefix pattern. These are NOT hardcoded intents — the COMMAND space (new) understands them geometrically.

```
dafhne add {word} as {definition}
dafhne add {word} to {place}
dafhne remove {word}
dafhne rename {word} to {new-word}
dafhne what do you know about {word}
dafhne list words [in {space}]
dafhne bind {action} on {entity} to {adapter}
dafhne forget {word}
```

Examples:
```
"dafhne add marco as a person in this home"
→ marco — a person. marco is in this home.

"dafhne add small-light as a light in the kitchen"  
→ small-light — a light. small-light is in the kitchen.

"dafhne add coffee-machine as a machine in the kitchen that makes coffee"
→ coffee-machine — a machine in the kitchen. coffee-machine makes coffee.

"dafhne rename small-light to kitchen-spot"
→ all references updated, geometry rebuilt

"dafhne remove marco"
→ check dependents, remove if safe, rebuild
```

## THE UNKNOWN WORD DIALOGUE

The key interaction. User says something DAFHNE doesn't understand:

```
User:   dafhne add a flux capacitor to the garage
DAFHNE: I don't know what "flux" means. Can you tell me?
User:   flux is a kind of energy flow
DAFHNE: I don't know what "flow" means. Can you tell me?
User:   flow is a way something moves from one place to another
DAFHNE: OK. I now know:
        → flow — a way something moves from one place to another
        → flux — a kind of energy flow
        I don't know what "capacitor" means. Can you tell me?
User:   a capacitor is a machine that holds energy
DAFHNE: OK. I now know:
        → capacitor — a machine that holds energy
        → flux-capacitor — a capacitor in the garage. it uses flux.
        Done. I added flux-capacitor to the garage.
```

### The Recursion

```rust
async fn add_word_interactive(
    service: &DafhneService,
    word: &str,
    definition: &str,
    session: &mut ChatSession,
) -> Result<AddResult> {
    // 1. Parse definition into content words
    let content_words = extract_content_words(definition);
    
    // 2. Check which words are unknown
    let unknown: Vec<&str> = content_words.iter()
        .filter(|w| !service.dictionary.read().knows(w))
        .collect();
    
    // 3. If all known → add directly
    if unknown.is_empty() {
        service.add_word(word, definition).await?;
        return Ok(AddResult::Done(word.to_string()));
    }
    
    // 4. If unknown words → ask user to define them first
    // Queue: define unknowns, then retry the original add
    Ok(AddResult::NeedDefinitions {
        pending_word: word.to_string(),
        pending_definition: definition.to_string(),
        undefined: unknown.into_iter().map(String::from).collect(),
    })
}
```

### LLM-Assisted Definition (optional fast path)

If an LLM is available (via adapter or API key), DAFHNE can generate definitions automatically:

```
User:   dafhne add a flux capacitor to the garage
DAFHNE: I don't know "flux" or "capacitor". Want me to figure them out? [Yes/No]
User:   yes
DAFHNE: [calls LLM: "Define 'flux' using only these words: {core vocabulary}. 
         Use simple language a child would understand."]
        I think:
        → flux — a kind of energy that moves and changes
        → capacitor — a thing that can hold energy inside it
        Does that sound right? [Yes/No/Fix]
User:   yes
DAFHNE: Done. Added flux-capacitor to the garage.
```

The LLM is a **definition assistant**, not a comprehension engine. It generates candidate definitions; DAFHNE validates closure; the user approves. The LLM never answers questions — only DAFHNE does.

## DEFINITION TEMPLATES

For common "add" patterns, generate structured definitions automatically:

| Pattern | Template |
|---------|----------|
| `add X as a Y` | `X — a Y` |
| `add X as a Y in Z` | `X — a Y. X is in Z.` |
| `add X to Z` | `X — a thing in Z.` (asks: "What kind of thing is X?") |
| `add X as a Y that can Z` | `X — a Y. X can Z.` |

## STATEFUL SESSIONS

Config commands need conversational state (the unknown-word dialogue spans multiple turns). But DAFHNE's engine is stateless. Solution: a thin session layer on the server.

```rust
struct ConfigSession {
    id: String,
    state: ConfigState,
    pending_additions: Vec<PendingWord>,
    started: Instant,
    timeout: Duration,  // auto-expire after 5 min inactivity
}

enum ConfigState {
    Idle,
    AwaitingDefinition { word: String, context: String },
    AwaitingConfirmation { additions: Vec<(String, String)> },
    AwaitingChoice { options: Vec<String> },
}
```

## PERMISSIONS

Not full RBAC, but two roles:

| Role | Can do |
|------|--------|
| **Admin** | Install/remove packages, configure adapters, modify core overrides |
| **User** | Add/remove Layer 3 words, ask questions, execute actions |

Enforcement: a simple pin code or local-network-only restriction. Not cryptographic auth — this is a home device.

## CONFLICT HANDLING

```
User:   dafhne add light as a thing that is not heavy
DAFHNE: "light" already exists (from core: "a thing that makes not-dark").
        Do you want to:
        1. Override it (your definition replaces core)
        2. Add as "light-weight" instead
        3. Cancel
```

For entities that shadow package words:
```
User:   dafhne add volume as the name of my cat
DAFHNE: "volume" exists in the music package (how loud a thing is).
        Your definition will override it for you. 
        Music commands will still use the music meaning.
        Add anyway? [Yes/No]
```

## ROLLBACK

Every mutation is logged:

```rust
struct MutationLog {
    timestamp: DateTime<Utc>,
    action: MutationAction,
    word: String,
    definition: Option<String>,
    previous: Option<String>,
}

enum MutationAction {
    Add,
    Remove,
    Override,
    Rename { from: String },
}
```

```
"dafhne undo"           → revert last mutation
"dafhne undo last 3"    → revert last 3 mutations
"dafhne history"        → show mutation log
```

## SERVER ENDPOINTS

```
POST /api/config/add         → { word, definition } or { word, as, in }
POST /api/config/remove      → { word }
POST /api/config/rename      → { from, to }
POST /api/config/undo        → { count: 1 }
GET  /api/config/history     → mutation log
GET  /api/config/session     → current config session state
POST /api/config/respond     → answer to DAFHNE's question (define unknown word, confirm, etc.)
```

## TESTING

- Add word with known vocabulary → immediate success, geometry rebuilt
- Add word with 1 unknown → dialogue asking for definition
- Add word with 3 unknowns → recursive dialogue, all resolved
- Override existing word → previous stored, override active
- Undo override → original resurfaces
- Remove word with dependents → error listing dependents
- LLM-assisted definition → generated, validated, user-approved
- Session timeout → pending additions discarded
- Concurrent config sessions → isolated per-connection

## WHAT NOT TO DO

- Do NOT build the LLM integration yet (stub the interface, manual definitions only first)
- Do NOT implement complex NLP for command parsing — pattern matching on prefixes is fine
- Do NOT allow modifying core or package dictionaries through chat (admin API only)
- Do NOT implement voice commands yet (Phase 26)
- Do NOT build a visual config UI (chat IS the config UI)
