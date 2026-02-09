use std::sync::Arc;

use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;

use crate::service::DafhneService;

const CHAT_HTML: &str = include_str!("../static/chat.html");

pub fn routes() -> Router<Arc<DafhneService>> {
    Router::new()
        .route("/chat", get(serve_chat))
}

async fn serve_chat() -> impl IntoResponse {
    Html(CHAT_HTML)
}
