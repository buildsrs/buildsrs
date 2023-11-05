use super::*;
use aws_sdk_s3::{
    error::SdkError,
    operation::get_object::GetObjectError,
    primitives::{ByteStream, SdkBody},
    Client,
};
use bytes::Bytes;
use std::sync::Arc;

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
        match &response {
            Err(SdkError::ServiceError(error)) => match error.err() {
                GetObjectError::NoSuchKey(error) => {
                    return Err(StorageError::NotFound(Arc::new(error.clone())));
                }
                _ => {}
            },
            _ => {}
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

#[cfg(any(feature = "options", test))]
mod options {
    use super::*;
    use aws_config::SdkConfig;
    use aws_credential_types::Credentials;
    use aws_sdk_s3::{types::*, Config};
    use aws_types::region::Region;
    use clap::Parser;
    use url::Url;

    #[derive(Parser, Clone, Debug)]
    pub struct S3Options {
        #[clap(long, env)]
        pub storage_s3_endpoint: Url,

        #[clap(long, env)]
        pub storage_s3_access_key_id: String,

        #[clap(long, env)]
        pub storage_s3_secret_access_key: String,

        #[clap(long, env)]
        pub storage_s3_region: String,

        #[clap(long, env)]
        pub storage_s3_path_style: bool,

        #[clap(long, env, default_value = "buildsrs")]
        pub storage_s3_bucket: String,
    }

    impl S3Options {
        fn credentials(&self) -> Credentials {
            Credentials::new(
                &self.storage_s3_access_key_id,
                &self.storage_s3_secret_access_key,
                None,
                None,
                "S3Options",
            )
        }

        async fn config(&self) -> SdkConfig {
            aws_config::from_env()
                .endpoint_url(self.storage_s3_endpoint.as_str())
                .region(Region::new(self.storage_s3_region.clone()))
                .credentials_provider(self.credentials())
                .load()
                .await
        }

        pub async fn build(&self) -> S3 {
            let config = Config::from(&self.config().await)
                .to_builder()
                .force_path_style(self.storage_s3_path_style)
                .build();
            let client = Client::from_conf(config);
            S3::new(client, self.storage_s3_bucket.clone())
        }
    }
}

#[cfg(any(feature = "options", test))]
pub use options::S3Options;

#[cfg(test)]
pub mod tests {
    //! Unit tests for [`S3`].
    //!
    //! These test verify that the S3 storage layer is implemented correctly. Every single test
    //! uses a new temporary bucket created by [`temp_s3`] to ensure that tests do not interfere
    //! with each other. Every single test performs some setup using manual bucket interactions,
    //! run at most one method under test, and verify the outputs and the bucket side effects.

    use super::*;
    use crate::tests::*;
    use aws_credential_types::Credentials;
    use aws_sdk_s3::types::*;
    use clap::Parser;
    use rand::{thread_rng, Rng};
    use std::error::Error;

    /// Generate random name for a bucket.
    fn random_bucket() -> String {
        let mut rng = thread_rng();
        (0..10).map(|_| rng.gen_range('a'..'z')).collect()
    }

    /// Delete bucket.
    async fn delete_bucket(client: Client, bucket: String) {
        let objects = client
            .list_objects_v2()
            .bucket(&bucket)
            .send()
            .await
            .unwrap();

        let mut delete_objects: Vec<ObjectIdentifier> = vec![];
        for obj in objects.contents().iter() {
            let obj_id = ObjectIdentifier::builder()
                .set_key(Some(obj.key().unwrap().to_string()))
                .build()
                .unwrap();
            delete_objects.push(obj_id);
        }

        if !delete_objects.is_empty() {
            client
                .delete_objects()
                .bucket(&bucket)
                .delete(
                    Delete::builder()
                        .set_objects(Some(delete_objects))
                        .build()
                        .unwrap(),
                )
                .send()
                .await
                .unwrap();
        }

        client.delete_bucket().bucket(bucket).send().await.unwrap();
    }

    /// Create test client for S3.
    pub async fn temp_s3() -> (S3, Cleanup) {
        let mut options = S3Options::parse();
        options.storage_s3_bucket = random_bucket();
        let s3 = options.build().await;
        println!("Using client {s3:?}");
        s3.client()
            .create_bucket()
            .bucket(s3.bucket())
            .send()
            .await
            .unwrap();
        let cleanup = Box::pin(delete_bucket(s3.client().clone(), s3.bucket().into()));
        (s3, cleanup)
    }

    #[proptest(async = "tokio", cases = 10)]
    async fn can_write_artifact(version: ArtifactId, contents: Vec<u8>) {
        with(temp_s3, |storage| async move {
            // write artifact using trait
            storage.artifact_put(&version, &contents).await.unwrap();

            // verify manually that it is there.
            let response = storage
                .client
                .get_object()
                .bucket(&storage.bucket)
                .key(version.file_name())
                .send()
                .await
                .unwrap();
            let data = response.body.collect().await.unwrap().into_bytes();
            assert_eq!(contents, data);
        })
        .await;
    }

    #[proptest(async = "tokio", cases = 10)]
    async fn can_write_artifact_existing(
        version: ArtifactId,
        previous: Vec<u8>,
        contents: Vec<u8>,
    ) {
        with(temp_s3, |storage| async move {
            // put an object into storage manually
            storage
                .client
                .put_object()
                .bucket(&storage.bucket)
                .key(version.file_name())
                .body(ByteStream::new(SdkBody::from(previous)))
                .send()
                .await
                .unwrap();

            // overwrite it using trait
            storage.artifact_put(&version, &contents).await.unwrap();

            // check that it was overwritten
            let response = storage
                .client
                .get_object()
                .bucket(&storage.bucket)
                .key(version.file_name())
                .send()
                .await
                .unwrap();
            let data = response.body.collect().await.unwrap().into_bytes();
            assert_eq!(contents, data);
        })
        .await;
    }

    #[proptest(async = "tokio", cases = 10)]
    async fn cannot_read_artifact_missing(version: ArtifactId) {
        with(temp_s3, |storage| async move {
            // read a non-existing artifact
            let error = storage.artifact_get(&version).await.err().unwrap();

            // ensure we get the right error and a cause
            assert!(matches!(error, StorageError::NotFound(_)));
            let error = error.source().unwrap();
            assert_eq!(
                error.to_string(),
                "NoSuchKey: The specified key does not exist."
            );
        })
        .await;
    }

    #[proptest(async = "tokio", cases = 10)]
    async fn can_read_artifact(version: ArtifactId, contents: Vec<u8>) {
        with(temp_s3, |storage| async move {
            // put an object into storage manually
            storage
                .client
                .put_object()
                .bucket(&storage.bucket)
                .key(version.file_name())
                .body(ByteStream::new(SdkBody::from(&contents[..])))
                .send()
                .await
                .unwrap();

            // read a artifact using trait
            let found = storage.artifact_get(&version).await.unwrap();

            // verify it was what we had written
            assert_eq!(&found.bytes().unwrap()[..], &contents);
        })
        .await;
    }
}
