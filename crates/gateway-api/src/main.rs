use anyhow::Result;
use axum::routing::{get, post};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use gateway_api::handlers;
use gateway_api::AppState;

#[derive(Parser)]
#[command(name = "gateway-api", about = "Rust AI Gateway API Server")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/default.yaml")]
    config: String,
}

pub fn build_router(state: Arc<AppState>) -> axum::Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    axum::Router::new()
        .route("/health", get(handlers::health::health_check))
        .route(
            "/v1/chat/completions",
            post(handlers::chat::chat_completion),
        )
        .route("/v1/models", get(handlers::models::list_models))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::Extension(state))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,gateway_api=debug,gateway_core=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    tracing::info!("Loading configuration from: {}", cli.config);

    let config = gateway_config::validation::load_config_with_env(&cli.config)
        .expect("Failed to load configuration");

    tracing::info!(
        "Starting gateway on {}:{}",
        config.server.host,
        config.server.port
    );

    // Create router from config
    let router = gateway_core::router::Router::new(&config).expect("Failed to create router");
    let state = Arc::new(AppState {
        router: Arc::new(router),
    });

    let app = build_router(state);
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid address");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}
