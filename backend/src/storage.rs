use bytes::Bytes;
use std::{error::Error, fmt::Debug, sync::Arc, time::Duration};
use url::Url;

pub type SharedError = Arc<dyn Error + Send + Sync>;

#[derive(thiserror::Error, Debug, Clone)]
pub enum StorageError {
    #[error("package missing")]
    NotFound(#[source] SharedError),

    #[error(transparent)]
    Other(#[from] SharedError),
}

#[derive(Clone, Debug)]
pub enum ArtifactKind {
    Manifest,
    Tarball,
    Debian,
}

#[derive(Clone, Debug)]
pub struct ArtifactId {
    pub krate: String,
    pub version: String,
    pub target: String,
    pub kind: ArtifactKind,
}

#[derive(Clone, Debug)]
pub enum Artifact {
    Data { bytes: Bytes },
    Redirect { validity: Duration, url: Url },
}

pub type AnyStorage = Arc<dyn Storage>;

#[async_trait::async_trait]
pub trait Storage: Send + Sync + Debug {
    async fn artifact_put(&self, version: &ArtifactId, data: &[u8]) -> Result<(), StorageError>;
    async fn artifact_get(&self, version: &ArtifactId) -> Result<Artifact, StorageError>;
}
