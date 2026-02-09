# DAFHNE Server

`dafhne-server` wraps the DAFHNE geometric comprehension engine in an HTTP server that speaks Ollama, OpenAI, and MCP protocols, and serves a web chat UI.

## Building

```bash
cargo build --release -p dafhne-server
```

## Running the Server

### Minimal (default parameters)

```bash
cargo run --release -p dafhne-server -- --data-dir ./dictionaries
```

This loads 3 models with default engine parameters:
- **dafhne-5** — 51-word single-space (dict5)
- **dafhne-12** — 1005-word single-space (dict12)
- **dafhne-50** — 5-space multispace (content + math + grammar + task + self)

### With evolved genome (recommended)

```bash
cargo run --release -p dafhne-server -- \
  --data-dir ./dictionaries \
  --multi-genome ./results_multi/gen_029/best_genome.json
```

The `--multi-genome` flag loads per-space evolved parameters for dafhne-50, which gives the best results.

### CLI reference

| Flag | Description | Default |
|------|-------------|---------|
| `--port` | HTTP port | `3000` |
| `--host` | Bind address | `0.0.0.0` |
| `--data-dir` | Directory containing dictionary `.md` files | `./dictionaries` |
| `--genome` | Single-space genome JSON (for dafhne-5, dafhne-12) | defaults |
| `--multi-genome` | Multi-space genome JSON (for dafhne-50) | defaults |
| `--mcp-stdio` | Run as MCP server on stdin/stdout (no HTTP) | off |
| `--log-level` | Log level (`trace`, `debug`, `info`, `warn`, `error`) | `info` |

Once running, the server exposes:

| Endpoint | Protocol | Description |
|----------|----------|-------------|
| `http://localhost:3000/chat` | HTTP | Web chat UI |
| `http://localhost:3000/api/*` | Ollama API | Chat completions, model listing |
| `http://localhost:3000/v1/*` | OpenAI API | Chat completions, model listing |
| `http://localhost:3000/mcp` | MCP (JSON-RPC) | Tool calls for LLM clients |

## Testing from the Command Line

### List models (Ollama)

```bash
curl -s http://localhost:3000/api/tags | python -m json.tool
```

### Ask a question (Ollama, non-streaming)

```bash
curl -s -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"model":"dafhne-50","messages":[{"role":"user","content":"Is a dog an animal?"}],"stream":false}'
```

### Ask a question (Ollama, streaming NDJSON)

```bash
curl -s -N -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"model":"dafhne-50","messages":[{"role":"user","content":"What is a cat?"}],"stream":true}'
```

### List models (OpenAI)

```bash
curl -s http://localhost:3000/v1/models | python -m json.tool
```

### Ask a question (OpenAI, non-streaming)

```bash
curl -s -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"dafhne-50","messages":[{"role":"user","content":"What is a dog?"}],"stream":false}'
```

### Ask a question (OpenAI, streaming SSE)

```bash
curl -s -N -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"dafhne-50","messages":[{"role":"user","content":"Is a cat a food?"}],"stream":true}'
```

### MCP: list tools

```bash
curl -s -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

### MCP: ask a question

```bash
curl -s -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"dafhne_ask","arguments":{"question":"Is a dog an animal?"}}}'
```

### MCP: describe a word

```bash
curl -s -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"dafhne_describe","arguments":{"word":"dog"}}}'
```

### MCP: list vocabulary

```bash
curl -s -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"dafhne_list_words","arguments":{"space":"content"}}}'
```

## Web Chat UI

Open `http://localhost:3000/chat` in a browser. The UI auto-detects available models from the server and connects immediately — no configuration needed.

## Connecting as an MCP Server

### Claude Code (stdio mode)

Run the server in MCP stdio mode — JSON-RPC on stdin/stdout, no HTTP:

```bash
dafhne-server --mcp-stdio --data-dir ./dictionaries --multi-genome ./results_multi/gen_029/best_genome.json
```

To register with Claude Code, add to `.claude/settings.json`:

```json
{
  "mcpServers": {
    "dafhne": {
      "command": "cargo",
      "args": ["run", "--release", "-p", "dafhne-server", "--", "--mcp-stdio", "--data-dir", "./dictionaries", "--multi-genome", "./results_multi/gen_029/best_genome.json"]
    }
  }
}
```

Or if you have the compiled binary:

```json
{
  "mcpServers": {
    "dafhne": {
      "command": "dafhne-server",
      "args": ["--mcp-stdio", "--data-dir", "/path/to/dictionaries", "--multi-genome", "/path/to/results_multi/gen_029/best_genome.json"]
    }
  }
}
```

This exposes 4 tools to Claude:

| Tool | Description |
|------|-------------|
| `dafhne_ask` | Ask DAFHNE a question (yes/no, what-is, why, arithmetic) |
| `dafhne_describe` | Get geometric description of a word |
| `dafhne_list_words` | List vocabulary, optionally filtered by space |
| `dafhne_which_space` | Ask which space handles a question |

### HTTP MCP (for other clients)

Any MCP client that supports HTTP transport can connect to `http://localhost:3000/mcp`:

- `POST /mcp` — send JSON-RPC requests
- `GET /mcp` — SSE stream for server-initiated messages

## Connecting Chat Clients

### Ollama-compatible clients

Any client that speaks the Ollama API (Open WebUI, Chatbox, etc.) can connect by pointing at `http://localhost:3000` as the Ollama server URL. Models will appear as `dafhne-5`, `dafhne-12`, `dafhne-50`.

### OpenAI-compatible clients

Any client that speaks the OpenAI API can connect by setting the base URL to `http://localhost:3000/v1`. The server accepts any `Authorization: Bearer` header without validation.

## Docker

### Build and run

```bash
docker build -t dafhne .
docker run -p 3000:3000 dafhne
```

### docker-compose

```bash
docker compose up
```

This mounts dictionaries and genome files as read-only volumes.
