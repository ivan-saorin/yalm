mod service;
mod ollama;
mod openai;
mod mcp;
mod chat;

use std::sync::Arc;
use std::path::PathBuf;

use axum::Router;
use clap::Parser;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use service::DafhneService;

#[derive(Parser)]
#[command(name = "dafhne-server", about = "DAFHNE geometric comprehension engine — API server")]
struct Cli {
    /// HTTP port
    #[arg(long, default_value = "3000")]
    port: u16,
    /// Bind address
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    /// Dictionary directory
    #[arg(long, default_value = "./dictionaries")]
    data_dir: PathBuf,
    /// Genome file (single-space, for dafhne-5 / dafhne-12)
    #[arg(long)]
    genome: Option<PathBuf>,
    /// Multi-space genome file (for dafhne-50)
    #[arg(long)]
    multi_genome: Option<PathBuf>,
    /// Run as MCP server on stdio (no HTTP)
    #[arg(long)]
    mcp_stdio: bool,
    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&cli.log_level)),
        )
        .init();

    // MCP stdio mode — run JSON-RPC on stdin/stdout, no HTTP
    if cli.mcp_stdio {
        tracing::info!("Loading models for MCP stdio mode...");
        let svc = Arc::new(DafhneService::load(&cli.data_dir, cli.genome.as_deref(), cli.multi_genome.as_deref()));
        tracing::info!("Models loaded. Starting MCP stdio...");
        mcp::run_stdio(svc).await;
        return;
    }

    tracing::info!("Loading models from {:?} ...", cli.data_dir);
    let svc = Arc::new(DafhneService::load(&cli.data_dir, cli.genome.as_deref(), cli.multi_genome.as_deref()));
    tracing::info!("Loaded {} model(s)", svc.model_count());

    let app = Router::new()
        // Ollama-compatible API
        .merge(ollama::routes())
        // OpenAI-compatible API
        .merge(openai::routes())
        // MCP endpoint
        .merge(mcp::routes())
        // Chat UI
        .merge(chat::routes())
        // Root redirect
        .route("/", axum::routing::get(|| async {
            axum::response::Redirect::temporary("/chat")
        }))
        .layer(CorsLayer::permissive())
        .with_state(svc);

    let addr = format!("{}:{}", cli.host, cli.port);
    tracing::info!("DAFHNE server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
