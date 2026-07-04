use anyhow::Context;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client, config::Credentials, primitives::ByteStream};
use bytes::Bytes;

use crate::{
    domain::ObjectAsset,
    repository::{ObjectAssetRepository, RepositoryResult},
};

#[derive(Clone)]
pub struct MinioObjectAssetRepository {
    client: Client,
    bucket: String,
}

impl MinioObjectAssetRepository {
    pub async fn new(
        endpoint: String,
        region: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
    ) -> Self {
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(Region::new(region))
            .credentials_provider(Credentials::new(
                access_key_id,
                secret_access_key,
                None,
                None,
                "mock-machine-env",
            ))
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true)
            .build();

        Self {
            client: Client::from_conf(s3_config),
            bucket,
        }
    }
}

#[async_trait::async_trait]
impl ObjectAssetRepository for MinioObjectAssetRepository {
    async fn put(
        &self,
        object_key: &str,
        content_type: Option<&str>,
        body: Bytes,
    ) -> RepositoryResult<ObjectAsset> {
        let size_bytes = body.len() as i64;
        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(object_key)
            .body(ByteStream::from(body));

        if let Some(content_type) = content_type {
            request = request.content_type(content_type);
        }

        request
            .send()
            .await
            .with_context(|| format!("failed to put object in MinIO: {object_key}"))?;

        Ok(ObjectAsset {
            bucket: self.bucket.clone(),
            object_key: object_key.to_string(),
            content_type: content_type.map(ToOwned::to_owned),
            size_bytes,
        })
    }

    async fn get(&self, object_key: &str) -> RepositoryResult<Option<Bytes>> {
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object_key)
            .send()
            .await;

        let output = match result {
            Ok(output) => output,
            Err(error) if error.to_string().contains("NoSuchKey") => return Ok(None),
            Err(error) => {
                return Err(anyhow::anyhow!("failed to get object from MinIO: {error}").into());
            }
        };

        let body = output
            .body
            .collect()
            .await
            .with_context(|| format!("failed to read MinIO object body: {object_key}"))?
            .into_bytes();

        Ok(Some(body))
    }
}
