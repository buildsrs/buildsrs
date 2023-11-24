//! # Filesystem Storage
//!
//! This storage implementation uses the filesystem to store artifacts.

use super::*;
use std::{
    fmt::Debug,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[cfg(any(test, feature = "temp"))]
mod temp;

#[cfg(feature = "options")]
mod options;

#[cfg(feature = "options")]
pub(crate) use options::FilesystemOptions;

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
        let path = self.artifact_path(version);
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
        self.do_artifact_put(version, data)
            .await
            .map_err(|error| StorageError::Other(Arc::new(error)))
    }

    async fn artifact_get(&self, version: &ArtifactId) -> Result<ArtifactData, StorageError> {
        let result = self.do_artifact_get(version).await;
        match result {
            Ok(bytes) => Ok(ArtifactData::Data { bytes }),
            Err(error) if error.error.kind() == ErrorKind::NotFound => {
                Err(StorageError::NotFound(Arc::new(error)))
            }
            Err(error) => Err(StorageError::Other(Arc::new(error))),
        }
    }
}
