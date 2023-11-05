use bytes::Bytes;
use std::{error::Error, fmt::Debug, sync::Arc, time::Duration};
use test_strategy::Arbitrary;
use url::Url;

pub type SharedError = Arc<dyn Error + Send + Sync>;

#[cfg(feature = "filesystem")]
pub mod filesystem;
#[cfg(test)]
pub mod tests;

#[derive(thiserror::Error, Debug, Clone)]
pub enum StorageError {
    #[error("artifact not found")]
    NotFound(#[source] SharedError),

    #[error(transparent)]
    Other(#[from] SharedError),
}

#[derive(Clone, Debug, Arbitrary)]
pub enum ArtifactKind {
    Manifest,
    Tarball,
    Debian,
}

impl ArtifactKind {
    fn extension(&self) -> &'static str {
        match self {
            Self::Manifest => "json",
            Self::Tarball => "tar.gz",
            Self::Debian => "deb",
        }
    }
}

#[derive(Clone, Debug, Arbitrary)]
pub struct ArtifactId {
    #[strategy("[a-z]{20}")]
    pub krate: String,
    #[strategy("[a-z]{20}")]
    pub version: String,
    #[strategy("[a-z]{20}")]
    pub target: String,
    pub kind: ArtifactKind,
}

impl ArtifactId {
    fn file_name(&self) -> String {
        let Self {
            krate,
            version,
            target,
            kind,
        } = &self;
        let extension = kind.extension();
        format!("{krate}_{version}_{target}.{extension}")
    }
}

#[derive(Clone, Debug)]
pub enum ArtifactData {
    Data { bytes: Bytes },
    Redirect { validity: Duration, url: Url },
}

impl ArtifactData {
    fn bytes(&self) -> Option<&Bytes> {
        match self {
            Self::Data { bytes } => Some(&bytes),
            _ => None,
        }
    }
}

pub type AnyStorage = Arc<dyn Storage>;

#[async_trait::async_trait]
pub trait Storage: Send + Sync + Debug {
    async fn artifact_put(&self, version: &ArtifactId, data: &[u8]) -> Result<(), StorageError>;
    async fn artifact_get(&self, version: &ArtifactId) -> Result<ArtifactData, StorageError>;
}
