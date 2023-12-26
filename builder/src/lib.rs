//! # Buildsrs Builder
//!
//! This crate implements the functionality of building crate artifacts. It receives jobs from the
//! backend, telling it which crates to build and which artifacts. It builds the crates using
//! whichever strategy it is configured to use, by default it will use Docker. It streams the
//! progress while the build is in progress, and finally signs and uploads the artifacts.

use anyhow::Result;
use async_trait::async_trait;
use cargo_metadata::Metadata;
use std::{path::Path, sync::Arc};

#[cfg(feature = "docker")]
mod docker;
#[cfg(feature = "docker")]
pub use docker::{DockerBuilder, DockerStrategy};

#[cfg(feature = "options")]
mod options;
#[cfg(feature = "options")]
pub use options::StrategyOptions;

#[cfg(feature = "websocket")]
mod websocket;
#[cfg(feature = "websocket")]
pub use websocket::Connection;

/// Represents a strategy for building artifacts.
///
/// A strategy is able to create a builder instance for a given crate. For example, building
/// binaries in Docker might be one strategy, while building binaries in a QEMU VM might be another
/// one.
#[async_trait]
pub trait Strategy {
    /// Create a builder instance from an extraced crate at the given path.
    async fn builder_from_path(&self, path: &Path) -> Result<DynBuilder>;

    /// Create a builder instance from a `.crate` file.
    async fn builder_from_crate(&self, krate: &Path) -> Result<DynBuilder>;

    /// Create a builder instance from a crate by download URL.
    async fn builder_from_url(&self, url: &str, checksum: &str) -> Result<DynBuilder>;
}

/// Represents an instance of a builder with a single crate.
///
/// The builder is able to produce artifacts for the crate.
#[async_trait]
pub trait Builder {
    /// Build crate metadata.
    async fn metadata(&self) -> Result<Metadata>;
}

/// Dynamic builder.
pub type DynBuilder = Box<dyn Builder>;

/// Dynamic strategy.
pub type DynStrategy = Arc<dyn Strategy>;
