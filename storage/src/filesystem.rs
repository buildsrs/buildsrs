use super::*;
use std::{
    fmt::Debug,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

/// Filesystem-backed storage for artifacts.
#[derive(Clone, Debug)]
pub struct Filesystem<P: AsRef<Path> = PathBuf> {
    path: P,
}

/// Error interacting with the filesystem.
#[derive(thiserror::Error, Debug)]
#[error("error writing to {path:?}")]
pub struct FilesystemError {
    /// Path that was being written to or read from.
    path: PathBuf,
    /// Error that occured.
    #[source]
    error: std::io::Error,
}

impl<P: AsRef<Path>> Filesystem<P> {
    /// Create new Filesystem storage instance.
    pub fn new(path: P) -> Self {
        Self { path }
    }

    /// Get the base path of this filesystem storage instance.
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Get the full path where an artifact ID might be stored.
    pub fn artifact_path(&self, version: &ArtifactId) -> PathBuf {
        self.path().join(version.file_name())
    }

    async fn do_artifact_put(
        &self,
        version: &ArtifactId,
        data: &[u8],
    ) -> Result<(), FilesystemError> {
        let path = self.artifact_path(&version);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .await
            .map_err(|error| FilesystemError {
                path: path.clone(),
                error,
            })?;
        file.write_all(data)
            .await
            .map_err(|error| FilesystemError {
                path: path.clone(),
                error,
            })?;
        file.flush()
            .await
            .map_err(|error| FilesystemError { path, error })?;
        Ok(())
    }

    async fn do_artifact_get(&self, version: &ArtifactId) -> Result<Bytes, FilesystemError> {
        let path = self.artifact_path(version);
        tokio::fs::read(&path)
            .await
            .map(Into::into)
            .map_err(|error| FilesystemError { path, error })
    }
}

#[async_trait::async_trait]
impl<P: AsRef<Path> + Send + Sync + Debug> Storage for Filesystem<P> {
    async fn artifact_put(&self, version: &ArtifactId, data: &[u8]) -> Result<(), StorageError> {
        self.do_artifact_put(&version, data)
            .await
            .map_err(|error| StorageError::Other(Arc::new(error)))
    }

    async fn artifact_get(&self, version: &ArtifactId) -> Result<ArtifactData, StorageError> {
        let result = self.do_artifact_get(&version).await;
        match result {
            Ok(bytes) => Ok(ArtifactData::Data { bytes }),
            Err(error) if error.error.kind() == ErrorKind::NotFound => {
                Err(StorageError::NotFound(Arc::new(error)))
            }
            Err(error) => Err(StorageError::Other(Arc::new(error))),
        }
    }
}

#[cfg(any(feature = "options", test))]
mod options {
    use super::*;
    use clap::Parser;
    use std::path::PathBuf;

    #[derive(Parser, Clone, Debug)]
    pub struct FilesystemOptions {
        #[clap(long, env)]
        pub storage_filesystem_path: PathBuf,
    }

    impl FilesystemOptions {
        pub async fn build(&self) -> Filesystem {
            Filesystem::new(self.storage_filesystem_path.clone())
        }
    }
}

#[cfg(any(feature = "options", test))]
pub use options::FilesystemOptions;

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tests::*;
    use std::error::Error;
    use tempdir::TempDir;

    /// Create a temporary filesystem storage.
    pub async fn temp_filesystem() -> (Filesystem, Cleanup) {
        let dir = TempDir::new("storage").unwrap();
        let storage = Filesystem::new(dir.path().to_path_buf());
        let cleanup = async move {
            dir.close().unwrap();
        };
        (storage, Box::pin(cleanup))
    }

    #[proptest(async = "tokio")]
    async fn can_write_artifact(version: ArtifactId, contents: Vec<u8>) {
        with(temp_filesystem, |storage| async move {
            storage.artifact_put(&version, &contents).await.unwrap();

            let path = storage.path().join(version.file_name());
            let found = tokio::fs::read(&path).await.unwrap();
            assert_eq!(found, contents);
        })
        .await;
    }

    #[proptest(async = "tokio")]
    async fn can_write_artifact_existing(
        version: ArtifactId,
        previous: Vec<u8>,
        contents: Vec<u8>,
    ) {
        with(temp_filesystem, |storage| async move {
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
        with(temp_filesystem, |storage| async move {
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
        with(temp_filesystem, |storage| async move {
            let path = storage.path().join(version.file_name());
            tokio::fs::write(&path, &contents).await.unwrap();

            let found = storage.artifact_get(&version).await.unwrap();

            assert_eq!(&found.bytes().unwrap()[..], &contents);
        })
        .await;
    }
}
