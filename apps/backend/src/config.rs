use std::env;

use anyhow::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageMode {
    Postgres,
    InMemory,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: String,
    pub storage_mode: StorageMode,
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_region: String,
    pub s3_bucket: Option<String>,
    pub s3_access_key_id: Option<String>,
    pub s3_secret_access_key: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let storage_mode = match env::var("APP_STORAGE")
            .unwrap_or_else(|_| "postgres".to_string())
            .as_str()
        {
            "postgres" => StorageMode::Postgres,
            "in_memory" => StorageMode::InMemory,
            other => anyhow::bail!("unsupported APP_STORAGE value: {other}"),
        };

        let database_url = env::var("DATABASE_URL").ok();
        if storage_mode == StorageMode::Postgres {
            database_url
                .as_ref()
                .context("DATABASE_URL must be set when APP_STORAGE=postgres")?;
        }
        let redis_url = env::var("REDIS_URL").ok();
        if storage_mode == StorageMode::InMemory {
            redis_url
                .as_ref()
                .context("REDIS_URL must be set when APP_STORAGE=in_memory")?;
        }

        Ok(Self {
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            storage_mode,
            database_url,
            redis_url,
            s3_endpoint: env::var("S3_ENDPOINT").ok(),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            s3_bucket: env::var("S3_BUCKET").ok(),
            s3_access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
            s3_secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
        })
    }
}
