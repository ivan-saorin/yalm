#!/bin/bash
# Test script for dafhne-server
# Start the server first: cargo run -p dafhne-server -- --data-dir ./dictionaries
BASE="${1:-http://localhost:3000}"
PASS=0
FAIL=0

check() {
  local desc="$1"
  local result="$2"
  if [ -n "$result" ] && [ "$result" != "null" ]; then
    echo "[PASS] $desc"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] $desc — got: $result"
    FAIL=$((FAIL + 1))
  fi
}

echo "=== Ollama: List models ==="
TAGS=$(curl -s "$BASE/api/tags")
echo "$TAGS" | python3 -m json.tool 2>/dev/null || echo "$TAGS"
check "Ollama /api/tags returns models" "$(echo "$TAGS" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(len(d.get("models",[])))' 2>/dev/null)"

echo ""
echo "=== Ollama: Chat (non-streaming) ==="
CHAT=$(curl -s "$BASE/api/chat" -H 'Content-Type: application/json' \
  -d '{"model":"dafhne-50","messages":[{"role":"user","content":"Is a dog an animal?"}],"stream":false}')
echo "$CHAT" | python3 -m json.tool 2>/dev/null || echo "$CHAT"
ANS=$(echo "$CHAT" | python3 -c 'import sys,json; print(json.load(sys.stdin)["message"]["content"])' 2>/dev/null)
check "Ollama chat: 'Is a dog an animal?' = Yes" "$(echo "$ANS" | grep -i yes)"

echo ""
echo "=== Ollama: Chat (streaming) ==="
STREAM=$(curl -s "$BASE/api/chat" -H 'Content-Type: application/json' \
  -d '{"model":"dafhne-50","messages":[{"role":"user","content":"Is a cat a food?"}],"stream":true}')
echo "$STREAM"
check "Ollama streaming returns NDJSON" "$(echo "$STREAM" | head -1 | python3 -c 'import sys,json; json.load(sys.stdin); print("ok")' 2>/dev/null)"

echo ""
echo "=== OpenAI: List models ==="
MODELS=$(curl -s "$BASE/v1/models")
echo "$MODELS" | python3 -m json.tool 2>/dev/null || echo "$MODELS"
check "OpenAI /v1/models returns list" "$(echo "$MODELS" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(len(d.get("data",[])))' 2>/dev/null)"

echo ""
echo "=== OpenAI: Chat (non-streaming) ==="
OAI=$(curl -s "$BASE/v1/chat/completions" -H 'Content-Type: application/json' \
  -d '{"model":"dafhne-50","messages":[{"role":"user","content":"What is a dog?"}],"stream":false}')
echo "$OAI" | python3 -m json.tool 2>/dev/null || echo "$OAI"
OAI_ANS=$(echo "$OAI" | python3 -c 'import sys,json; print(json.load(sys.stdin)["choices"][0]["message"]["content"])' 2>/dev/null)
check "OpenAI chat: 'What is a dog?' returns answer" "$OAI_ANS"

echo ""
echo "=== OpenAI: Chat (streaming) ==="
SSE=$(curl -s -N "$BASE/v1/chat/completions" -H 'Content-Type: application/json' \
  -d '{"model":"dafhne-50","messages":[{"role":"user","content":"Is a cat a food?"}],"stream":true}')
echo "$SSE"
check "OpenAI streaming returns SSE" "$(echo "$SSE" | grep 'data:')"

echo ""
echo "=== Chat UI ==="
CHAT_HTML=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/chat")
check "Chat UI returns 200" "$([ "$CHAT_HTML" = "200" ] && echo "ok")"

echo ""
echo "=== MCP: tools/list ==="
MCP=$(curl -s "$BASE/mcp" -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}')
echo "$MCP" | python3 -m json.tool 2>/dev/null || echo "$MCP"
TOOL_COUNT=$(echo "$MCP" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(len(d.get("result",{}).get("tools",[])))' 2>/dev/null)
check "MCP tools/list returns 4 tools" "$([ "$TOOL_COUNT" = "4" ] && echo "4")"

echo ""
echo "=== MCP: dafhne_ask ==="
ASK=$(curl -s "$BASE/mcp" -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"dafhne_ask","arguments":{"question":"Is a dog an animal?"}}}')
echo "$ASK" | python3 -m json.tool 2>/dev/null || echo "$ASK"
ASK_ANS=$(echo "$ASK" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d["result"]["content"][0]["text"])' 2>/dev/null)
check "MCP dafhne_ask: 'Is a dog an animal?' = Yes" "$(echo "$ASK_ANS" | grep -i yes)"

echo ""
echo "================================"
echo "Results: $PASS passed, $FAIL failed"
echo "================================"
