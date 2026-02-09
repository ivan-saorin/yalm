# PROMPT 22 — DAFHNE Server

> **STATUS: New crate. Build an API server exposing DAFHNE as a chat service.**
>
> **Target: Claude Code execution**

## CONTEXT

DAFHNE (Definition-Anchored Force-field Heuristic Network Engine) is a geometric comprehension engine that learns from ELI5 dictionaries. It currently scores 50/50 on a unified test across 5 spaces (content, math, grammar, task, self) and 20/20 on single-space.

This phase wraps the engine in an HTTP server that speaks both Ollama and OpenAI protocols, serves a web chat UI, and exposes an MCP interface — so any LLM client (including Claude itself) can talk to DAFHNE.

Project location: `D:\workspace\projects\dafhne`

## WHAT TO BUILD

### New crate: `dafhne-server`

Add to workspace in root `Cargo.toml`:
```toml
[workspace]
members = [
    "crates/dafhne-core",
    "crates/dafhne-engine",
    "crates/dafhne-eval",
    "crates/dafhne-evolve",
    "crates/dafhne-server",  # NEW
]
```

**NOTE**: Crate names may still be `yalm-*` if the rename hasn't been completed in Cargo.toml. Check the actual workspace members and use whatever names are current. The prompt uses `dafhne-*` throughout — adjust as needed.

### Dependencies

```toml
[dependencies]
dafhne-core = { path = "../dafhne-core" }
dafhne-engine = { path = "../dafhne-engine" }
axum = { version = "0.8", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower-http = { version = "0.6", features = ["cors", "fs"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4", features = ["derive"] }
rust-embed = "8"  # for embedding static assets
mime_guess = "2"
```

---

## ARCHITECTURE

```
┌──────────────────────────────────────────────────┐
│                  dafhne-server                     │
│                                                    │
│  ┌─────────┐  ┌───────────┐  ┌─────────────────┐ │
│  │ /chat    │  │ /api/*    │  │ /v1/*           │ │
│  │ Web UI   │  │ Ollama API│  │ OpenAI API      │ │
│  └────┬─────┘  └─────┬─────┘  └───────┬─────────┘ │
│       │              │                │            │
│       └──────────────┼────────────────┘            │
│                      │                             │
│              ┌───────▼────────┐                    │
│              │  DafhneService  │                    │
│              │                │                    │
│              │  models: Map   │                    │
│              │    "dafhne-5"  ├──► Engine(dict5)   │
│              │    "dafhne-50" ├──► MultiSpace(5sp) │
│              │    "dafhne-12" ├──► Engine(dict12)  │
│              └────────────────┘                    │
│                                                    │
│  ┌──────────────────────────────────────────────┐ │
│  │ MCP Server (SSE transport on /mcp)           │ │
│  │  Tools: ask, describe, list_words, which_space│ │
│  └──────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

---

## PART 1: Model Registry

A "model" in DAFHNE = a preconfigured combination of dictionaries + genome. Define models in a TOML/JSON config file or hardcode the initial set.

### Model Definitions

```rust
struct DafhneModel {
    id: String,           // e.g. "dafhne-5"
    name: String,         // e.g. "DAFHNE 5-Word Spaces"
    description: String,
    // Single-space or multi-space
    kind: ModelKind,
    // Pre-trained engine(s)
    engine: ModelEngine,
}

enum ModelKind {
    SingleSpace,
    MultiSpace,
}

enum ModelEngine {
    Single(Engine),
    Multi(MultiSpace),
}
```

### Initial Models

Load at startup from the `dictionaries/` and `results_v11/` directories:

| Model ID | Type | Dictionaries | Genome | Description |
|----------|------|-------------|--------|-------------|
| `dafhne-5` | single | dict5.md | best_genome.json | Core 20-word vocabulary, 20/20 |
| `dafhne-12` | single | dict12.md + grammar18.md | best_genome.json | Extended 50-word vocabulary, 14/20 |
| `dafhne-50` | multi | dict5 + dict_math5 + dict_grammar5 + dict_task5 + dict_self5 | best_genome.json | Full 5-space, 50/50 |

The server should also support a `models/` directory for user-provided model configs (future extensibility).

### Loading Models

At startup:
1. Parse CLI args for `--data-dir` (default: `./dictionaries`) and `--genome` (default: `./results_v11/best_genome.json`)
2. Load each model by training the engine from its dictionary files
3. Store in an `Arc<DafhneService>` shared across all request handlers
4. Training happens once at startup — models are immutable after that

**IMPORTANT**: Engine training reads dictionary markdown, discovers connectors, builds geometric space, loads genome parameters. This takes ~100ms per space. Do it once, serve many.

---

## PART 2: Ollama API (`/api/*`)

Implement the subset of the Ollama API that chat clients expect:

### `GET /api/tags` — List models

```json
{
  "models": [
    {
      "name": "dafhne-50",
      "model": "dafhne-50",
      "modified_at": "2026-02-09T00:00:00Z",
      "size": 0,
      "digest": "dafhne50",
      "details": {
        "parent_model": "",
        "format": "geometric",
        "family": "dafhne",
        "families": ["dafhne"],
        "parameter_size": "51 words",
        "quantization_level": "none"
      }
    }
  ]
}
```

### `POST /api/chat` — Chat completion

**Request**:
```json
{
  "model": "dafhne-50",
  "messages": [
    {"role": "system", "content": "..."},
    {"role": "user", "content": "Is a dog an animal?"}
  ],
  "stream": true,
  "options": {}
}
```

**Processing**:
1. Extract the last user message as the question
2. Route to the appropriate model's engine
3. Call `engine.answer(&question)` (single-space) or `multispace.answer(&question)` (multi-space)
4. Return the answer

**Response (streaming = true)**: NDJSON, one object per line:
```json
{"model":"dafhne-50","created_at":"2026-02-09T12:00:00Z","message":{"role":"assistant","content":"Yes"},"done":false}
{"model":"dafhne-50","created_at":"2026-02-09T12:00:00Z","message":{"role":"assistant","content":""},"done":true,"total_duration":1234567,"eval_count":1}
```

For DAFHNE, the answer is instant (no token-by-token generation), so emit the full answer in one chunk then the `done` message. But respect the streaming protocol for client compatibility.

**Response (streaming = false)**:
```json
{
  "model": "dafhne-50",
  "created_at": "2026-02-09T12:00:00Z",
  "message": {"role": "assistant", "content": "Yes"},
  "done": true,
  "total_duration": 1234567,
  "eval_count": 1
}
```

### `POST /api/show` — Model info (optional, nice to have)

Return model metadata including dictionary word count, space count, test scores.

---

## PART 3: OpenAI API (`/v1/*`)

### `GET /v1/models` — List models

```json
{
  "object": "list",
  "data": [
    {
      "id": "dafhne-50",
      "object": "model",
      "created": 1707436800,
      "owned_by": "dafhne"
    }
  ]
}
```

### `POST /v1/chat/completions` — Chat completion

**Request**:
```json
{
  "model": "dafhne-50",
  "messages": [
    {"role": "user", "content": "Is a dog an animal?"}
  ],
  "stream": false,
  "temperature": 0.7,
  "max_tokens": 100
}
```

**Response (non-streaming)**:
```json
{
  "id": "chatcmpl-dafhne-xxxxx",
  "object": "chat.completion",
  "created": 1707436800,
  "model": "dafhne-50",
  "choices": [{
    "index": 0,
    "message": {"role": "assistant", "content": "Yes"},
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 6,
    "completion_tokens": 1,
    "total_tokens": 7
  }
}
```

**Response (streaming)**: SSE format:
```
data: {"id":"chatcmpl-dafhne-xxxxx","object":"chat.completion.chunk","created":1707436800,"model":"dafhne-50","choices":[{"index":0,"delta":{"role":"assistant","content":"Yes"},"finish_reason":null}]}

data: {"id":"chatcmpl-dafhne-xxxxx","object":"chat.completion.chunk","created":1707436800,"model":"dafhne-50","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

### Authentication

Accept `Authorization: Bearer <key>` header but don't validate it (DAFHNE has no auth). This lets OpenAI clients connect without errors.

---

## PART 4: Web Chat UI (`/chat`)

Embed the chat HTML (from the file I already provided, or a new version) as a static asset using `rust-embed` or serve from a `static/` directory.

**Key modifications to the chat HTML**:
- Default Ollama URL should be `window.location.origin` (the server itself, not localhost:11434)
- Auto-detect provider: if the page is served from DAFHNE server, default to Ollama provider pointing at self
- On page load, auto-fetch models from the server's own `/api/tags`
- Add a "DAFHNE" provider tab alongside Ollama/OpenAI (or just make Ollama the default pointing at self)

The chat UI should work out of the box when you open `http://localhost:3000/chat` — no configuration needed.

### Static file serving

```rust
// Serve static files from embedded or filesystem
Router::new()
    .route("/chat", get(serve_chat_html))
    .route("/chat/*path", get(serve_static))
```

Or simpler: just serve a single `index.html` at `/chat`.

---

## PART 5: MCP Server (`/mcp`)

Implement an MCP (Model Context Protocol) server so Claude and other MCP clients can use DAFHNE as a tool.

### Transport: SSE (Streamable HTTP)

Use the [MCP Streamable HTTP transport](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports#streamable-http):

```
POST /mcp  — send JSON-RPC messages
GET  /mcp  — SSE stream for server-initiated messages
```

### Tools to expose

#### `dafhne_ask`
Ask DAFHNE a question.
```json
{
  "name": "dafhne_ask",
  "description": "Ask the DAFHNE geometric comprehension engine a question. DAFHNE understands questions about a small ELI5 vocabulary (~51 words across 5 spaces: content, math, grammar, task, self). It can answer yes/no questions, what-is questions, why questions, and perform basic arithmetic.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "question": { "type": "string", "description": "The question to ask" },
      "model": { "type": "string", "description": "Model ID (default: dafhne-50)", "default": "dafhne-50" }
    },
    "required": ["question"]
  }
}
```

#### `dafhne_describe`
Get DAFHNE's description of a word (what it knows about it).
```json
{
  "name": "dafhne_describe",
  "description": "Get DAFHNE's geometric description of a word — what it knows from definitions and spatial relationships.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "word": { "type": "string" },
      "model": { "type": "string", "default": "dafhne-50" }
    },
    "required": ["word"]
  }
}
```

#### `dafhne_list_words`
List all words DAFHNE knows.
```json
{
  "name": "dafhne_list_words",
  "description": "List all words in DAFHNE's vocabulary, optionally filtered by space.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "model": { "type": "string", "default": "dafhne-50" },
      "space": { "type": "string", "description": "Filter by space name (content, math, grammar, task, self)" }
    }
  }
}
```

#### `dafhne_which_space`
Ask which space would handle a question.
```json
{
  "name": "dafhne_which_space",
  "description": "Ask DAFHNE which geometric space would handle a given question (content, math, grammar, task, or self).",
  "inputSchema": {
    "type": "object",
    "properties": {
      "question": { "type": "string" }
    },
    "required": ["question"]
  }
}
```

### MCP Implementation

You can implement MCP from scratch (it's just JSON-RPC 2.0 over SSE) or use a Rust MCP SDK if one exists. The protocol is simple:

1. Client sends `initialize` → server responds with capabilities and tool list
2. Client sends `tools/list` → server responds with tool definitions
3. Client sends `tools/call` with tool name and arguments → server executes and responds

Key JSON-RPC methods to handle:
- `initialize` → return server info + capabilities
- `tools/list` → return tool definitions
- `tools/call` → dispatch to DAFHNE engine, return result

### MCP stdio mode (bonus)

Also support `dafhne-server --mcp-stdio` which runs the MCP server on stdin/stdout instead of HTTP. This lets Claude Code use it directly:

```json
// In .claude/mcp_servers.json or similar
{
  "dafhne": {
    "command": "dafhne-server",
    "args": ["--mcp-stdio"]
  }
}
```

---

## PART 6: CLI

```
dafhne-server [OPTIONS]

Options:
  --port <PORT>           HTTP port [default: 3000]
  --host <HOST>           Bind address [default: 0.0.0.0]
  --data-dir <DIR>        Dictionary directory [default: ./dictionaries]
  --genome <FILE>         Genome file [default: ./results_v11/best_genome.json]
  --models-dir <DIR>      Additional model configs [default: ./models]
  --mcp-stdio             Run as MCP server on stdio (no HTTP)
  --log-level <LEVEL>     Log level [default: info]
```

---

## PART 7: Docker

### Dockerfile

```dockerfile
# Build stage
FROM rust:1.82-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p dafhne-server

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/dafhne-server /usr/local/bin/
COPY dictionaries/ /data/dictionaries/
COPY results_v11/ /data/results_v11/
COPY static/ /data/static/

WORKDIR /data
EXPOSE 3000

ENTRYPOINT ["dafhne-server"]
CMD ["--data-dir", "/data/dictionaries", "--genome", "/data/results_v11/best_genome.json", "--port", "3000"]
```

### docker-compose.yml

```yaml
version: "3.8"
services:
  dafhne:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./dictionaries:/data/dictionaries:ro
      - ./results_v11:/data/results_v11:ro
    environment:
      - RUST_LOG=info
    restart: unless-stopped
```

### .dockerignore

```
target/
.git/
*.md
reports/
prompts/
archive/
texts/
```

---

## PART 8: Answer Processing

The tricky part: DAFHNE answers questions, it doesn't generate free-form text. The chat interface sends arbitrary messages, but DAFHNE only understands questions in its vocabulary.

### Strategy

1. **Direct questions**: Route directly to engine. "Is a dog an animal?" → "Yes"
2. **Non-questions / unknown words**: Return an honest "I don't know" style response
3. **Conversation context**: DAFHNE is stateless per-question. Previous messages are ignored (the engine has no concept of conversation history). The system prompt is informational only.
4. **Formatting**: Wrap DAFHNE's terse answers in natural language:
   - Yes/No → "Yes." or "No."  (keep it clean)
   - What-is → "A dog is an animal." (the full sentence)
   - Why → "Because a dog is an animal, and an animal is a thing." (chain)
   - I don't know → "I don't know." (honest geometric absence)
   - Describe → Multi-line description output

### Answer Enhancement (optional but recommended)

Add a `verbose` mode that includes DAFHNE's reasoning:
```json
{
  "content": "Yes.\n\n[DAFHNE] Space: content | Distance: 0.12 | Chain: dog → animal ✓ | Confidence: high"
}
```

This helps users understand what DAFHNE is doing geometrically. Enable via a system prompt keyword like "verbose" or "debug", or a request parameter.

---

## IMPLEMENTATION ORDER

1. **Scaffold** crate structure, Cargo.toml, main.rs with CLI parsing
2. **Model loading** — DafhneService that loads engines at startup
3. **Ollama API** — `/api/tags` + `/api/chat` (streaming + non-streaming)
4. **OpenAI API** — `/v1/models` + `/v1/chat/completions` (streaming + non-streaming)
5. **Chat UI** — embed and serve at `/chat`, auto-configure to point at self
6. **MCP Server** — SSE transport on `/mcp` + optional stdio mode
7. **Docker** — Dockerfile + compose + .dockerignore
8. **Test** — verify with curl, the chat UI, and an MCP client

## PREREQUISITE READING

Before coding, read:
1. `crates/dafhne-engine/src/lib.rs` — `Engine` public API: `train()`, `answer()`, `describe()`
2. `crates/dafhne-engine/src/multispace.rs` — `MultiSpace` public API: `new()`, `answer()`
3. `crates/dafhne-core/src/lib.rs` — `EngineParams`, `GeometricSpace`, `Genome` types
4. `crates/dafhne-eval/src/main.rs` — how eval loads dictionaries, genomes, and builds engines/multispace. **Copy this pattern for the server's model loading.**
5. `Cargo.toml` (root) — workspace structure, actual crate names

**CRITICAL**: Look at how `dafhne-eval` (or `yalm-eval`) builds the engine and multispace. The server's model loading should follow the exact same code path. Don't reinvent dictionary parsing — use the existing `Engine::train()` and `MultiSpace::new()` constructors.

## WHAT NOT TO DO

- Do NOT modify any existing crate (dafhne-core, dafhne-engine, dafhne-eval, dafhne-evolve)
- Do NOT change any dictionary files
- Do NOT implement authentication (accept all requests)
- Do NOT implement conversation memory (DAFHNE is stateless per-question)
- Do NOT try to make DAFHNE generate long text — it answers questions, period
- Do NOT add heavy dependencies (no database, no Redis, no message queue)
- Do NOT implement model hot-reloading (restart to reload)

## SUCCESS CRITERIA

| Criterion | Test |
|-----------|------|
| Ollama list models | `curl http://localhost:3000/api/tags` returns model list |
| Ollama chat | `curl -X POST http://localhost:3000/api/chat -d '{"model":"dafhne-50","messages":[{"role":"user","content":"Is a dog an animal?"}]}'` → Yes |
| Ollama streaming | Same with `"stream":true` returns NDJSON |
| OpenAI list models | `curl http://localhost:3000/v1/models` returns model list |
| OpenAI chat | `curl -X POST http://localhost:3000/v1/chat/completions -d '{"model":"dafhne-50","messages":[{"role":"user","content":"What is a dog?"}]}'` → an animal |
| OpenAI streaming | Same with `"stream":true` returns SSE |
| Web chat | Open `http://localhost:3000/chat` → UI loads, auto-connects, models listed |
| MCP tools/list | MCP client can discover 4 tools |
| MCP ask | MCP client can call dafhne_ask and get answer |
| Docker build | `docker build -t dafhne .` succeeds |
| Docker run | `docker run -p 3000:3000 dafhne` starts and serves |
| 50/50 via API | All 50 unified_test questions answered correctly through the API |

## OUTPUT

When complete:
- New crate at `crates/dafhne-server/`
- `Dockerfile` at project root
- `docker-compose.yml` at project root
- `.dockerignore` at project root
- `static/chat.html` (or embedded in binary)
- Update `RECAP.md` with Phase 22 entry
- Update `STATUS.md` with server info

### Quick verification script

Create `scripts/test_server.sh`:
```bash
#!/bin/bash
BASE="http://localhost:3000"

echo "=== Ollama: List models ==="
curl -s "$BASE/api/tags" | jq .

echo -e "\n=== Ollama: Chat ==="
curl -s "$BASE/api/chat" -d '{"model":"dafhne-50","messages":[{"role":"user","content":"Is a dog an animal?"}],"stream":false}' | jq .

echo -e "\n=== OpenAI: List models ==="
curl -s "$BASE/v1/models" | jq .

echo -e "\n=== OpenAI: Chat ==="
curl -s "$BASE/v1/chat/completions" -d '{"model":"dafhne-50","messages":[{"role":"user","content":"What is a dog?"}],"stream":false}' | jq .

echo -e "\n=== OpenAI: Streaming ==="
curl -s -N "$BASE/v1/chat/completions" -d '{"model":"dafhne-50","messages":[{"role":"user","content":"Is a cat a food?"}],"stream":true}'

echo -e "\n\n=== All 50 questions ==="
# Read unified_test.md, extract questions, hit API, compare
```
