//! Thin wrapper over the AWS S3 SDK. All report files are stored here (only);
//! Postgres holds just the metadata pointer.

use std::time::Duration;

use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

use crate::error::{AppError, AppResult};

/// A presigned download URL and how long it stays valid.
#[derive(Debug)]
pub struct PresignedUrl {
    pub url: String,
    pub expires_in_secs: u64,
}

pub struct S3Service {
    client: Client,
    bucket: String,
    presign_expiry: Duration,
}

impl S3Service {
    pub fn new(client: Client, bucket: String, presign_expiry: Duration) -> Self {
        Self {
            client,
            bucket,
            presign_expiry,
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// Upload bytes to `key` with the given content type.
    pub async fn put_object(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> AppResult<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(bytes))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::Aws(format!("s3 put_object failed: {e}")))?;
        Ok(())
    }

    /// Generate a short-lived presigned GET URL so clients can download a
    /// report directly from S3 without the file passing through this service.
    pub async fn presign_get(&self, key: &str) -> AppResult<PresignedUrl> {
        let config = PresigningConfig::expires_in(self.presign_expiry)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid presign config: {e}")))?;

        let request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(config)
            .await
            .map_err(|e| AppError::Aws(format!("s3 presign failed: {e}")))?;

        Ok(PresignedUrl {
            url: request.uri().to_string(),
            expires_in_secs: self.presign_expiry.as_secs(),
        })
    }

    /// Delete an object (used when a report record is removed).
    pub async fn delete_object(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Aws(format!("s3 delete_object failed: {e}")))?;
        Ok(())
    }
}
