use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::service::DafhneService;

// ─── Request / Response types ────────────────────────────────

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct ChatCompletionRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(default)]
    stream: Option<bool>,
    #[serde(default)]
    temperature: Option<f64>,
    #[serde(default)]
    max_tokens: Option<u64>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct OpenAIMessage {
    role: String,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Serialize)]
struct ModelsResponse {
    object: String,
    data: Vec<ModelObject>,
}

#[derive(Serialize)]
struct ModelObject {
    id: String,
    object: String,
    created: i64,
    owned_by: String,
}

#[derive(Serialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Serialize)]
struct Choice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: String,
}

#[derive(Serialize)]
struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

#[derive(Serialize)]
struct ChatCompletionChunk {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<ChunkChoice>,
}

#[derive(Serialize)]
struct ChunkChoice {
    index: u32,
    delta: OpenAIMessage,
    finish_reason: Option<String>,
}

// ─── Routes ──────────────────────────────────────────────────

pub fn routes() -> Router<Arc<DafhneService>> {
    Router::new()
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
}

// ─── Handlers ────────────────────────────────────────────────

async fn list_models(State(svc): State<Arc<DafhneService>>) -> Json<ModelsResponse> {
    let created = chrono::Utc::now().timestamp();
    let data: Vec<ModelObject> = svc.model_order.iter()
        .filter_map(|id| svc.models.get(id))
        .map(|m| ModelObject {
            id: m.id.clone(),
            object: "model".to_string(),
            created,
            owned_by: "dafhne".to_string(),
        })
        .collect();

    Json(ModelsResponse {
        object: "list".to_string(),
        data,
    })
}

async fn chat_completions(
    State(svc): State<Arc<DafhneService>>,
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    let stream = req.stream.unwrap_or(false);
    let completion_id = format!("chatcmpl-dafhne-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("x"));
    let created = chrono::Utc::now().timestamp();

    // Find model
    let model = match svc.get_model(&req.model) {
        Some(m) => m,
        None => {
            let err = serde_json::json!({
                "error": {
                    "message": format!("Model '{}' not found", req.model),
                    "type": "invalid_request_error",
                    "code": "model_not_found"
                }
            });
            return axum::response::Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(err.to_string()))
                .unwrap();
        }
    };

    // Extract last user message
    let question = req.messages.iter()
        .rev()
        .find(|m| m.role == "user")
        .and_then(|m| m.content.clone())
        .unwrap_or_default();

    let (answer, _dist, _conn) = model.answer(&question);
    let content = answer.to_string();

    // Count tokens (approximate: split on whitespace)
    let prompt_tokens: u64 = req.messages.iter()
        .filter_map(|m| m.content.as_ref())
        .map(|c| c.split_whitespace().count() as u64)
        .sum();
    let completion_tokens = content.split_whitespace().count() as u64;

    if stream {
        // SSE streaming response
        let chunk1 = serde_json::to_string(&ChatCompletionChunk {
            id: completion_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: req.model.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(content),
                },
                finish_reason: None,
            }],
        }).unwrap();

        let chunk2 = serde_json::to_string(&ChatCompletionChunk {
            id: completion_id,
            object: "chat.completion.chunk".to_string(),
            created,
            model: req.model.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        }).unwrap();

        let body = format!("data: {}\n\ndata: {}\n\ndata: [DONE]\n\n", chunk1, chunk2);
        axum::response::Response::builder()
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .body(axum::body::Body::from(body))
            .unwrap()
    } else {
        let resp = ChatCompletionResponse {
            id: completion_id,
            object: "chat.completion".to_string(),
            created,
            model: req.model.clone(),
            choices: vec![Choice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(content),
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
        };
        axum::response::Response::builder()
            .header("content-type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&resp).unwrap()))
            .unwrap()
    }
}
