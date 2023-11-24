#![warn(missing_docs)]

//! # Buildsrs Storage
//!
//! This crate defines the [`Storage`] trait, which is used by the backend to store artifacts.
//! It also defines some implementations for the storage trait, which can be enabled using the
//! appropriate features.

use bytes::Bytes;
use std::{error::Error, fmt::Debug, sync::Arc, time::Duration};
use test_strategy::Arbitrary;
use url::Url;

/// Shared generic error.
pub type SharedError = Arc<dyn Error + Send + Sync>;

#[cfg(any(test, feature = "temp"))]
mod temp;
#[cfg(any(test, feature = "temp"))]
pub use temp::*;

#[cfg(feature = "cache")]
mod cache;
#[cfg(feature = "filesystem")]
mod filesystem;
#[cfg(feature = "s3")]
mod s3;

#[cfg(feature = "cache")]
pub use cache::{Cache, CacheConfig};
#[cfg(feature = "filesystem")]
pub use filesystem::Filesystem;
#[cfg(feature = "s3")]
pub use s3::S3;

/// Error in storage operation.
#[derive(thiserror::Error, Debug, Clone)]
pub enum StorageError {
    /// Not found
    #[error("artifact not found")]
    NotFound(#[source] SharedError),

    /// Other error
    #[error(transparent)]
    Other(#[from] SharedError),
}

/// Kind of artifact.
#[derive(Clone, Debug, Arbitrary, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArtifactKind {
    /// Tarball, with a `.tar.gz` extension.
    Manifest,
    /// Manifest, with a `.json` extension.
    Tarball,
    /// Debian package, with a `.deb` extension.
    Debian,
}

impl ArtifactKind {
    /// Get extension for this artifact kind.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Manifest => "json",
            Self::Tarball => "tar.gz",
            Self::Debian => "deb",
        }
    }
}

/// Artifact identifier.
#[derive(Clone, Debug, Arbitrary, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactId {
    /// Name of crate
    #[strategy("[a-z]{20}")]
    pub krate: String,
    /// Name of crate version
    #[strategy("[a-z]{20}")]
    pub version: String,
    /// Target triple
    #[strategy("[a-z]{20}")]
    pub target: String,
    /// Kind of artifact
    pub kind: ArtifactKind,
}

impl ArtifactId {
    /// Get the file name for this artifact identifier
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

/// Data of artifact
#[derive(Clone, Debug)]
pub enum ArtifactData {
    /// Raw data
    Data {
        /// Bytes
        bytes: Bytes,
    },
    /// Redirect
    Redirect {
        /// How long this link is valid for
        validity: Duration,
        /// URL to redirect to
        url: Url,
    },
}

impl ArtifactData {
    /// Get raw bytes, if exists
    pub fn bytes(&self) -> Option<&Bytes> {
        match self {
            Self::Data { bytes } => Some(bytes),
            _ => None,
        }
    }
}

#[cfg(feature = "options")]
mod options {
    use super::*;
    use clap::{Parser, ValueEnum};
    use std::error::Error;

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
        /// Build storage instance
        pub async fn build(&self) -> Result<AnyStorage, Box<dyn Error + Send + Sync>> {
            let storage = match self.storage {
                #[cfg(feature = "filesystem")]
                StorageKind::Filesystem => Arc::new(self.filesystem.build().await) as AnyStorage,
                #[cfg(feature = "s3")]
                StorageKind::S3 => Arc::new(self.s3.build().await) as AnyStorage,
            };

            #[cfg(feature = "cache")]
            let storage = self.cache.maybe_cache(storage);

            Ok(storage)
        }
    }
}

/// Storage command-line options.
#[cfg(feature = "options")]
pub use options::StorageOptions;

/// Shared generic storage instance.
pub type AnyStorage = Arc<dyn Storage>;

/// Storage trait.
#[async_trait::async_trait]
pub trait Storage: Send + Sync + Debug {
    /// Put an artifact into storage.
    async fn artifact_put(&self, version: &ArtifactId, data: &[u8]) -> Result<(), StorageError>;

    /// Get an artifact from storage.
    async fn artifact_get(&self, version: &ArtifactId) -> Result<ArtifactData, StorageError>;
}
