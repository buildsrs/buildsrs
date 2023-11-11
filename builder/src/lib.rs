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

#[async_trait]
pub trait Strategy {
    async fn builder_from_path(&self, path: &Path) -> Result<DynBuilder>;
    async fn builder_from_crate(&self, krate: &Path) -> Result<DynBuilder>;
    async fn builder_from_url(&self, url: &str, checksum: &str) -> Result<DynBuilder>;
}

#[async_trait]
pub trait Builder {
    async fn metadata(&self) -> Result<Metadata>;
}

pub type DynBuilder = Box<dyn Builder>;
pub type DynStrategy = Arc<dyn Strategy>;
