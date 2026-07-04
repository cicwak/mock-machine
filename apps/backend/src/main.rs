use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use sea_orm::Database;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

mod config;
mod domain;
mod entities;
mod http;
mod migrations;
mod repository;
mod state;

use config::{AppConfig, StorageMode};
use repository::{
    cached::CachedRouteRepository, in_memory::InMemoryRepository,
    minio::MinioObjectAssetRepository, postgres::PostgresRepository, redis::RedisRepository,
};
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mock_machine=info,tower_http=info".into()),
        )
        .init();

    let config = AppConfig::from_env()?;
    let addr: SocketAddr = config
        .bind_addr
        .parse()
        .with_context(|| format!("invalid BIND_ADDR: {}", config.bind_addr))?;

    let app_state = build_state(&config).await?;
    let app = http::router(app_state)
        .layer(CorsLayer::permissive())
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

async fn build_state(config: &AppConfig) -> anyhow::Result<AppState> {
    match config.storage_mode {
        StorageMode::Postgres => {
            let database_url = config
                .database_url
                .as_ref()
                .context("DATABASE_URL must be set")?;
            let db = Database::connect(database_url)
                .await
                .context("failed to connect to PostgreSQL")?;
            migrations::run(&db)
                .await
                .context("failed to run database migrations")?;
            let postgres = Arc::new(PostgresRepository::new(db));
            let routes: Arc<dyn repository::MockRouteRepository> =
                if let Some(redis_url) = config.redis_url.as_ref() {
                    Arc::new(
                        CachedRouteRepository::new((*postgres).clone(), redis_url, 300)
                            .await
                            .context("failed to build route cache")?,
                    )
                } else {
                    postgres.clone()
                };
            let assets = build_asset_repository(config).await;

            Ok(AppState {
                unknown_requests: postgres.clone(),
                routes,
                assets,
                storage: if config.redis_url.is_some() {
                    "postgres+redis-cache"
                } else {
                    "postgres"
                },
            })
        }
        StorageMode::InMemory => {
            let redis_url = config.redis_url.as_ref().context("REDIS_URL must be set")?;
            let memory = Arc::new(
                RedisRepository::new(redis_url)
                    .await
                    .context("failed to connect to Redis")?,
            );
            Ok(AppState {
                unknown_requests: memory.clone(),
                routes: memory.clone(),
                assets: memory,
                storage: "in_memory",
            })
        }
    }
}

async fn build_asset_repository(config: &AppConfig) -> Arc<dyn repository::ObjectAssetRepository> {
    match (
        &config.s3_endpoint,
        &config.s3_bucket,
        &config.s3_access_key_id,
        &config.s3_secret_access_key,
    ) {
        (Some(endpoint), Some(bucket), Some(access_key_id), Some(secret_access_key)) => Arc::new(
            MinioObjectAssetRepository::new(
                endpoint.clone(),
                config.s3_region.clone(),
                bucket.clone(),
                access_key_id.clone(),
                secret_access_key.clone(),
            )
            .await,
        ),
        _ => Arc::new(InMemoryRepository::default()),
    }
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
