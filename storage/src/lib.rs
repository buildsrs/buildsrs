use bytes::Bytes;
use std::{error::Error, fmt::Debug, sync::Arc, time::Duration};
use test_strategy::Arbitrary;
use url::Url;

/// Shared generic error.
pub type SharedError = Arc<dyn Error + Send + Sync>;

#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "filesystem")]
pub mod filesystem;
#[cfg(feature = "s3")]
pub mod s3;
#[cfg(test)]
pub mod tests;

/// Error in storage operation.
#[derive(thiserror::Error, Debug, Clone)]
pub enum StorageError {
    #[error("artifact not found")]
    NotFound(#[source] SharedError),

    #[error(transparent)]
    Other(#[from] SharedError),
}

/// Kind of artifact.
#[derive(Clone, Debug, Arbitrary, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArtifactKind {
    Manifest,
    Tarball,
    Debian,
}

impl ArtifactKind {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Manifest => "json",
            Self::Tarball => "tar.gz",
            Self::Debian => "deb",
        }
    }
}

#[derive(Clone, Debug, Arbitrary, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub fn file_name(&self) -> String {
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

#[cfg(feature = "options")]
mod options {
    use super::*;
    use clap::{Parser, ValueEnum};
    use std::error::Error;

    const DEFAULT_STORAGE: &'static str = "filesystem";

    /// Kind of storage to use.
    #[derive(ValueEnum, Clone, Debug)]
    pub enum StorageKind {
        #[cfg(feature = "filesystem")]
        Filesystem,
        #[cfg(feature = "s3")]
        S3,
    }

    /// Options for storage.
    #[derive(Parser, Clone, Debug)]
    pub struct StorageOptions {
        /// Which storage backend to use.
        #[clap(long)]
        storage: StorageKind,

        #[clap(flatten)]
        #[cfg(feature = "filesystem")]
        filesystem: filesystem::FilesystemOptions,

        #[clap(flatten)]
        #[cfg(feature = "s3")]
        s3: s3::S3Options,

        #[clap(flatten)]
        #[cfg(feature = "cache")]
        cache: cache::CacheOptions,
    }

    impl StorageOptions {
        pub async fn build(&self) -> Result<AnyStorage, Box<dyn Error + Send + Sync>> {
            let storage = match self.storage {
                #[cfg(feature = "filesystem")]
                StorageKind::Filesystem => Arc::new(self.filesystem.build().await) as AnyStorage,
                #[cfg(feature = "s3")]
                StorageKind::S3 => Arc::new(self.s3.build().await) as AnyStorage,
            };

            #[cfg(feature = "storage-cache")]
            let storage = self.cache.maybe_cache(storage);

            Ok(storage)
        }
    }
}

#[cfg(feature = "options")]
pub use options::StorageOptions;

pub type AnyStorage = Arc<dyn Storage>;

#[async_trait::async_trait]
pub trait Storage: Send + Sync + Debug {
    async fn artifact_put(&self, version: &ArtifactId, data: &[u8]) -> Result<(), StorageError>;
    async fn artifact_get(&self, version: &ArtifactId) -> Result<ArtifactData, StorageError>;
}
