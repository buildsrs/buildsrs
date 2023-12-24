//! # Buildsrs Storage
//!
//! This crate defines the [`Storage`] trait, which is used by the backend to store artifacts.
//! It also defines some implementations for the storage trait, which can be enabled using the
//! appropriate features.

pub use buildsrs_common::entities::{ArtifactData, ArtifactId, ArtifactKind};
use bytes::Bytes;
use std::{error::Error, fmt::Debug, sync::Arc};

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

#[cfg(feature = "options")]
mod options {
    use super::*;
    use clap::{Parser, ValueEnum};
    use std::error::Error;

    /// Kind of storage to use.
    #[derive(ValueEnum, Clone, Debug, PartialEq)]
    pub enum StorageKind {
        #[cfg(feature = "filesystem")]
        Filesystem,
        #[cfg(feature = "s3")]
        S3,
    }

    /// Options for storage.
    #[derive(Parser, Clone, Debug, PartialEq)]
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
