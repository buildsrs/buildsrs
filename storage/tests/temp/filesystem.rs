use super::*;
use std::error::Error;
use test_strategy::proptest;

#[proptest(async = "tokio")]
async fn can_write_artifact(version: ArtifactId, contents: Vec<u8>) {
    with(Filesystem::new_temp, |storage| async move {
        storage.artifact_put(&version, &contents).await.unwrap();

        let path = storage.path().join(version.file_name());
        let found = tokio::fs::read(&path).await.unwrap();
        assert_eq!(found, contents);
    })
    .await;
}

#[proptest(async = "tokio")]
async fn can_write_artifact_existing(version: ArtifactId, previous: Vec<u8>, contents: Vec<u8>) {
    with(Filesystem::new_temp, |storage| async move {
        let path = storage.path().join(version.file_name());
        tokio::fs::write(&path, &previous).await.unwrap();

        storage.artifact_put(&version, &contents).await.unwrap();

        let found = tokio::fs::read(&path).await.unwrap();
        assert_eq!(found, contents);
    })
    .await;
}

#[proptest(async = "tokio")]
async fn cannot_read_artifact_missing(version: ArtifactId) {
    with(Filesystem::new_temp, |storage| async move {
        let path = storage.path().join(version.file_name());

        let error = storage.artifact_get(&version).await.err().unwrap();

        assert!(matches!(error, StorageError::NotFound(_)));
        assert_eq!(error.to_string(), format!("artifact not found"));
        assert_eq!(
            error.source().unwrap().to_string(),
            format!("error writing to {path:?}")
        );
    })
    .await;
}

#[proptest(async = "tokio")]
async fn can_read_artifact(version: ArtifactId, contents: Vec<u8>) {
    with(Filesystem::new_temp, |storage| async move {
        let path = storage.path().join(version.file_name());
        tokio::fs::write(&path, &contents).await.unwrap();

        let found = storage.artifact_get(&version).await.unwrap();

        assert_eq!(&found.bytes().unwrap()[..], &contents);
    })
    .await;
}
