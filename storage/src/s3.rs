use super::*;
use aws_sdk_s3::{
    error::SdkError,
    operation::get_object::GetObjectError,
    primitives::{ByteStream, SdkBody},
    Client,
};
use std::sync::Arc;

#[cfg(any(test, feature = "temp"))]
mod temp;

mod options;
pub(crate) use options::S3Options;

/// # S3-backed artifact storage.
///
/// This storage implementation keeps artifacts in an S3 bucket using the `aws_sdk` crate. The
/// artifacts are named similar to how they are named in the filesystem.
///
/// For example, a artifact named `myartifact` with version `0.1.5` would be stored as
/// `myartifact_0.1.5.tar.gz` in the bucket.
#[derive(Clone, Debug)]
pub struct S3 {
    client: Client,
    bucket: String,
}

impl S3 {
    /// Create new instance given an S3 [`Client`] and a bucket name.
    pub fn new(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }

    /// Get reference to the S3 client being used.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get reference to the name of the bucket that this instance writes to.
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[async_trait::async_trait]
impl Storage for S3 {
    async fn artifact_put(&self, version: &ArtifactId, data: &[u8]) -> Result<(), StorageError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(version.file_name())
            .body(ByteStream::new(SdkBody::from(data)))
            .send()
            .await
            .map(|_| ())
            .map_err(|error| StorageError::Other(Arc::new(error)))
    }

    async fn artifact_get(&self, version: &ArtifactId) -> Result<ArtifactData, StorageError> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(version.file_name())
            .send()
            .await;

        // determine if this is a no such key error and translate into artifact missing
        if let Err(SdkError::ServiceError(error)) = &response {
            if let GetObjectError::NoSuchKey(error) = error.err() {
                return Err(StorageError::NotFound(Arc::new(error.clone())));
            }
        }

        // return other errors as-is
        let response = match response {
            Ok(response) => response,
            Err(error) => return Err(StorageError::Other(Arc::new(error))),
        };

        // collect response
        let bytes = response
            .body
            .collect()
            .await
            .map_err(|error| StorageError::Other(Arc::new(error)))
            .map(|data| data.into_bytes())?;

        Ok(ArtifactData::Data { bytes })
    }
}
