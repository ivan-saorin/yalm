use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::service::DafhneService;

// ─── Request / Response types ────────────────────────────────

#[derive(Deserialize)]
pub struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(default)]
    stream: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct TagsResponse {
    models: Vec<ModelTag>,
}

#[derive(Serialize)]
struct ModelTag {
    name: String,
    model: String,
    modified_at: String,
    size: u64,
    digest: String,
    details: ModelDetails,
}

#[derive(Serialize)]
struct ModelDetails {
    parent_model: String,
    format: String,
    family: String,
    families: Vec<String>,
    parameter_size: String,
    quantization_level: String,
}

#[derive(Serialize)]
struct ChatResponse {
    model: String,
    created_at: String,
    message: ChatMessage,
    done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_count: Option<u64>,
}

#[derive(Serialize)]
struct ShowResponse {
    modelfile: String,
    parameters: String,
    template: String,
    details: ModelDetails,
}

// ─── Routes ──────────────────────────────────────────────────

pub fn routes() -> Router<Arc<DafhneService>> {
    Router::new()
        .route("/api/tags", get(list_tags))
        .route("/api/chat", post(chat))
        .route("/api/show", post(show))
}

// ─── Handlers ────────────────────────────────────────────────

async fn list_tags(State(svc): State<Arc<DafhneService>>) -> Json<TagsResponse> {
    let models: Vec<ModelTag> = svc.model_order.iter()
        .filter_map(|id| svc.models.get(id))
        .map(|m| ModelTag {
            name: m.id.clone(),
            model: m.id.clone(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            size: 0,
            digest: m.id.replace('-', ""),
            details: ModelDetails {
                parent_model: String::new(),
                format: "geometric".to_string(),
                family: "dafhne".to_string(),
                families: vec!["dafhne".to_string()],
                parameter_size: format!("{} words", m.word_count),
                quantization_level: "none".to_string(),
            },
        })
        .collect();

    Json(TagsResponse { models })
}

async fn chat(
    State(svc): State<Arc<DafhneService>>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    let stream = req.stream.unwrap_or(false);

    // Find model
    let model = match svc.get_model(&req.model) {
        Some(m) => m,
        None => {
            let err = ChatResponse {
                model: req.model.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: format!("Error: model '{}' not found", req.model),
                },
                done: true,
                total_duration: Some(0),
                eval_count: Some(0),
            };
            return axum::response::Response::builder()
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&err).unwrap()))
                .unwrap();
        }
    };

    // Extract last user message
    let question = req.messages.iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let start = std::time::Instant::now();
    let (answer, _dist, _conn) = model.answer(&question);
    let duration_ns = start.elapsed().as_nanos() as u64;
    let content = answer.to_string();
    let now = chrono::Utc::now().to_rfc3339();

    if stream {
        // Streaming: NDJSON — content chunk then done chunk
        let chunk1 = serde_json::to_string(&ChatResponse {
            model: req.model.clone(),
            created_at: now.clone(),
            message: ChatMessage {
                role: "assistant".to_string(),
                content: content.clone(),
            },
            done: false,
            total_duration: None,
            eval_count: None,
        }).unwrap();

        let chunk2 = serde_json::to_string(&ChatResponse {
            model: req.model.clone(),
            created_at: now,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: String::new(),
            },
            done: true,
            total_duration: Some(duration_ns),
            eval_count: Some(1),
        }).unwrap();

        let body = format!("{}\n{}\n", chunk1, chunk2);
        axum::response::Response::builder()
            .header("content-type", "application/x-ndjson")
            .body(axum::body::Body::from(body))
            .unwrap()
    } else {
        let resp = ChatResponse {
            model: req.model.clone(),
            created_at: now,
            message: ChatMessage {
                role: "assistant".to_string(),
                content,
            },
            done: true,
            total_duration: Some(duration_ns),
            eval_count: Some(1),
        };
        axum::response::Response::builder()
            .header("content-type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&resp).unwrap()))
            .unwrap()
    }
}

async fn show(
    State(svc): State<Arc<DafhneService>>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let model_name = req.get("name")
        .or_else(|| req.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("dafhne-50");

    if let Some(model) = svc.get_model(model_name) {
        let resp = ShowResponse {
            modelfile: format!("# {}\n{}", model.name, model.description),
            parameters: format!("words: {}, spaces: {}", model.word_count, model.space_count),
            template: "{{ .Prompt }}".to_string(),
            details: ModelDetails {
                parent_model: String::new(),
                format: "geometric".to_string(),
                family: "dafhne".to_string(),
                families: vec!["dafhne".to_string()],
                parameter_size: format!("{} words", model.word_count),
                quantization_level: "none".to_string(),
            },
        };
        Json(serde_json::to_value(resp).unwrap()).into_response()
    } else {
        axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from(
                serde_json::json!({"error": format!("model '{}' not found", model_name)}).to_string()
            ))
            .unwrap()
            .into_response()
    }
}
