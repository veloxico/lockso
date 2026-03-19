use anyhow::Result;
use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, Credentials, Region},
    primitives::ByteStream,
};

/// Unified S3-compatible file storage client.
///
/// All files are encrypted **before** being passed to this layer.
/// This struct only handles transport to/from S3/MinIO.
#[derive(Clone)]
pub struct FileStorage {
    client: Client,
    bucket: String,
}

impl FileStorage {
    pub async fn new(
        endpoint: &str,
        region: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
    ) -> Result<Self> {
        let creds = Credentials::new(access_key, secret_key, None, None, "lockso");
        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(region.to_string()))
            .endpoint_url(endpoint)
            .credentials_provider(creds)
            .force_path_style(true) // Required for MinIO
            .build();

        let client = Client::from_conf(config);

        // Ensure bucket exists
        let buckets = client.list_buckets().send().await?;
        let exists = buckets
            .buckets()
            .iter()
            .any(|b| b.name().is_some_and(|n| n == bucket));

        if !exists {
            client
                .create_bucket()
                .bucket(bucket)
                .send()
                .await?;
            tracing::info!(bucket, "Created S3 bucket");
        }

        Ok(Self {
            client,
            bucket: bucket.to_string(),
        })
    }

    /// Upload encrypted data to S3.
    pub async fn put(&self, key: &str, encrypted_data: &[u8]) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(encrypted_data.to_vec()))
            .content_type("application/octet-stream")
            .send()
            .await?;
        Ok(())
    }

    /// Download encrypted data from S3.
    pub async fn get(&self, key: &str) -> Result<Vec<u8>> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        let bytes = resp.body.collect().await?.to_vec();
        Ok(bytes)
    }

    /// Delete a file from S3.
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }

    /// Check if a file exists in S3.
    pub async fn exists(&self, key: &str) -> Result<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Health check — verify S3 connectivity by listing bucket.
    pub async fn health_check(&self) -> Result<()> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await?;
        Ok(())
    }
}
