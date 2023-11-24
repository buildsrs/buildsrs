use super::*;
use aws_sdk_s3::primitives::{ByteStream, SdkBody};
use std::error::Error;
use test_strategy::proptest;

#[proptest(async = "tokio", cases = 10)]
async fn can_write_artifact(version: ArtifactId, contents: Vec<u8>) {
    with(S3::new_temp, |storage| async move {
        // write artifact using trait
        storage.artifact_put(&version, &contents).await.unwrap();

        // verify manually that it is there.
        let response = storage
            .client()
            .get_object()
            .bucket(storage.bucket())
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
async fn can_write_artifact_existing(version: ArtifactId, previous: Vec<u8>, contents: Vec<u8>) {
    with(S3::new_temp, |storage| async move {
        // put an object into storage manually
        storage
            .client()
            .put_object()
            .bucket(storage.bucket())
            .key(version.file_name())
            .body(ByteStream::new(SdkBody::from(previous)))
            .send()
            .await
            .unwrap();

        // overwrite it using trait
        storage.artifact_put(&version, &contents).await.unwrap();

        // check that it was overwritten
        let response = storage
            .client()
            .get_object()
            .bucket(storage.bucket())
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
    with(S3::new_temp, |storage| async move {
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
    with(S3::new_temp, |storage| async move {
        // put an object into storage manually
        storage
            .client()
            .put_object()
            .bucket(storage.bucket())
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
