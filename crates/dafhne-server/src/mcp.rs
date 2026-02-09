use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use tokio_stream::StreamExt as _;

use crate::service::DafhneService;

// ─── JSON-RPC Types ──────────────────────────────────────────

#[derive(Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

// ─── MCP Protocol Constants ─────────────────────────────────

const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "dafhne";
const SERVER_VERSION: &str = "0.1.0";

// ─── Tool Definitions ───────────────────────────────────────

fn tool_definitions() -> Value {
    serde_json::json!({
        "tools": [
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
            },
            {
                "name": "dafhne_describe",
                "description": "Get DAFHNE's geometric description of a word — what it knows from definitions and spatial relationships.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "word": { "type": "string", "description": "The word to describe" },
                        "model": { "type": "string", "description": "Model ID (default: dafhne-50)", "default": "dafhne-50" }
                    },
                    "required": ["word"]
                }
            },
            {
                "name": "dafhne_list_words",
                "description": "List all words in DAFHNE's vocabulary, optionally filtered by space.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "model": { "type": "string", "description": "Model ID (default: dafhne-50)", "default": "dafhne-50" },
                        "space": { "type": "string", "description": "Filter by space name (content, math, grammar, task, self)" }
                    }
                }
            },
            {
                "name": "dafhne_which_space",
                "description": "Ask DAFHNE which geometric space would handle a given question (content, math, grammar, task, or self).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "question": { "type": "string", "description": "The question to route" }
                    },
                    "required": ["question"]
                }
            }
        ]
    })
}

// ─── Handle a single JSON-RPC request ───────────────────────

fn handle_request(req: &JsonRpcRequest, svc: &DafhneService) -> JsonRpcResponse {
    let id = req.id.clone().unwrap_or(Value::Null);

    match req.method.as_str() {
        "initialize" => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::json!({
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": SERVER_NAME,
                        "version": SERVER_VERSION
                    }
                })),
                error: None,
            }
        }
        "notifications/initialized" => {
            // Client acknowledgement — no response needed for notifications,
            // but since we got an id we respond
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(Value::Null),
                error: None,
            }
        }
        "tools/list" => {
            let defs = tool_definitions();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(defs),
                error: None,
            }
        }
        "tools/call" => {
            let params = req.params.as_ref();
            let tool_name = params
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = params
                .and_then(|p| p.get("arguments"))
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            let result = dispatch_tool(tool_name, &arguments, svc);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            }
        }
        _ => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", req.method),
                }),
            }
        }
    }
}

fn dispatch_tool(name: &str, args: &Value, svc: &DafhneService) -> Value {
    match name {
        "dafhne_ask" => {
            let question = args.get("question").and_then(|v| v.as_str()).unwrap_or("");
            let model_id = args.get("model").and_then(|v| v.as_str()).unwrap_or("dafhne-50");
            if let Some(model) = svc.get_model(model_id) {
                let (answer, dist, conn) = model.answer(question);
                serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("{}", answer)
                    }],
                    "isError": false,
                    "_debug": {
                        "distance": dist,
                        "connector": conn
                    }
                })
            } else {
                tool_error(&format!("Model '{}' not found", model_id))
            }
        }
        "dafhne_describe" => {
            let word = args.get("word").and_then(|v| v.as_str()).unwrap_or("");
            let model_id = args.get("model").and_then(|v| v.as_str()).unwrap_or("dafhne-50");
            if let Some(model) = svc.get_model(model_id) {
                let sentences = model.describe(word);
                if sentences.is_empty() {
                    serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("I don't know the word '{}'.", word)
                        }],
                        "isError": false
                    })
                } else {
                    serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": sentences.join("\n")
                        }],
                        "isError": false
                    })
                }
            } else {
                tool_error(&format!("Model '{}' not found", model_id))
            }
        }
        "dafhne_list_words" => {
            let model_id = args.get("model").and_then(|v| v.as_str()).unwrap_or("dafhne-50");
            let space = args.get("space").and_then(|v| v.as_str());
            if let Some(model) = svc.get_model(model_id) {
                let words = model.list_words(space);
                serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": words.join(", ")
                    }],
                    "isError": false,
                    "_count": words.len()
                })
            } else {
                tool_error(&format!("Model '{}' not found", model_id))
            }
        }
        "dafhne_which_space" => {
            let question = args.get("question").and_then(|v| v.as_str()).unwrap_or("");
            // Use dafhne-50 for space routing
            if let Some(model) = svc.get_model("dafhne-50") {
                let (answer, _dist, conn) = model.answer(question);
                // The connector or routing info tells us which space handled it
                let space_info = conn.unwrap_or_else(|| "unknown".to_string());
                serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Space: {} (answer: {})", space_info, answer)
                    }],
                    "isError": false
                })
            } else {
                tool_error("Model 'dafhne-50' not loaded")
            }
        }
        _ => {
            tool_error(&format!("Unknown tool: {}", name))
        }
    }
}

fn tool_error(msg: &str) -> Value {
    serde_json::json!({
        "content": [{
            "type": "text",
            "text": msg
        }],
        "isError": true
    })
}

// ─── HTTP Routes ─────────────────────────────────────────────

pub fn routes() -> Router<Arc<DafhneService>> {
    Router::new()
        .route("/mcp", post(mcp_post))
        .route("/mcp", get(mcp_sse))
}

async fn mcp_post(
    State(svc): State<Arc<DafhneService>>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let resp = handle_request(&req, &svc);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());
    (headers, serde_json::to_string(&resp).unwrap())
}

async fn mcp_sse(
    State(_svc): State<Arc<DafhneService>>,
) -> impl IntoResponse {
    // SSE endpoint for server-initiated messages. DAFHNE doesn't push events,
    // so this just keeps the connection open with a periodic heartbeat.
    let stream = tokio_stream::wrappers::IntervalStream::new(
        tokio::time::interval(std::time::Duration::from_secs(30)),
    )
    .map(|_| {
        Ok::<_, std::convert::Infallible>(
            axum::response::sse::Event::default().comment("heartbeat")
        )
    });

    axum::response::Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
}

// ─── Stdio Mode ──────────────────────────────────────────────

pub async fn run_stdio(svc: Arc<DafhneService>) {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(req) => {
                let resp = handle_request(&req, &svc);
                let mut out = serde_json::to_string(&resp).unwrap();
                out.push('\n');
                if stdout.write_all(out.as_bytes()).await.is_err() {
                    break;
                }
                if stdout.flush().await.is_err() {
                    break;
                }
            }
            Err(e) => {
                let err = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                let mut out = serde_json::to_string(&err).unwrap();
                out.push('\n');
                if stdout.write_all(out.as_bytes()).await.is_err() {
                    break;
                }
                if stdout.flush().await.is_err() {
                    break;
                }
            }
        }
    }
}
