use std::{env, net::SocketAddr};

use anyhow::Context;
use axum::{
    Json, Router,
    extract::OriginalUri,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Serialize)]
struct HealthResponse<'a> {
    status: &'a str,
    service: &'a str,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mock_machine=info,tower_http=info".into()),
        )
        .init();

    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let addr: SocketAddr = bind_addr
        .parse()
        .with_context(|| format!("invalid BIND_ADDR: {bind_addr}"))?;

    let app = Router::new()
        .route("/mockadminapi/health", get(health))
        .fallback(mock_fallback)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    info!(%addr, "backend listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn health() -> Json<HealthResponse<'static>> {
    Json(HealthResponse {
        status: "ok",
        service: "mock-machine-backend",
    })
}

async fn mock_fallback(method: Method, OriginalUri(uri): OriginalUri) -> Response {
    info!(%method, path = %uri.path(), "mock route is not configured yet");

    (StatusCode::NOT_FOUND, "route is not configured").into_response()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
