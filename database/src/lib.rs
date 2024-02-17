//! # Buildsrs Database
//!
//! The buildsrs project uses a database to store metadata about crates, crate versions, and
//! artifacts. The database that is used is the postgres database. This crate implements all
//! database interactions in the shape of methods that can be consumed elsewhere in the project.

#![allow(missing_docs)]

mod postgres;

use crate::entity::Builder;
use async_trait::async_trait;
use buildsrs_common::entities::*;
pub use postgres::*;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(feature = "options")]
mod options;

#[cfg(feature = "options")]
pub use options::DatabaseOptions;

/// Boxed generic error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Shared generic metadata instance.
///
/// This type is how downstream users should consume metadata instances.
pub type AnyMetadata = Arc<dyn Metadata>;

//#[derive(thiserror::Error, Debug)]
//pub enum ReadError {}

/// Metadata trait.
///
/// This trait represents a service that stores metadata in a consistent view. The semantics of
/// this store are transactional, meaning that in-progress writes should not be visible unless they
/// are committed. There are also some important constraints of preventing race conditions when
/// publishing packages.
///
/// There may be a limited number of connections to the metadata service, in which case the calls
/// used for creating new handles may be blocking unless other handles are released.
#[async_trait]
pub trait Metadata: Send + Sync + std::fmt::Debug {
    /// Get a read handle to use for reading.
    async fn read(&self) -> Result<Box<dyn ReadHandle>, BoxError>;

    /// Get a write handle to use for writing.
    async fn write(&self) -> Result<Box<dyn WriteHandle>, BoxError>;
}

#[async_trait]
pub trait ReadHandle: Send + Sync {
    async fn builder_lookup(&self, fingerprint: &str) -> Result<Uuid, Error>;
    async fn builder_get(&self, builder: Uuid) -> Result<Builder, Error>;
    async fn builder_list(&self) -> Result<Vec<Uuid>, Error>;

    async fn crate_list(&self, name: &str) -> Result<Vec<String>, Error>;
    async fn crate_info(&self, name: &str) -> Result<CrateInfo, Error>;
    async fn crate_versions(&self, name: &str) -> Result<Vec<String>, Error>;
    async fn crate_version_info(&self, name: &str, version: &str) -> Result<VersionInfo, Error>;

    async fn job_info(&self, job: Uuid) -> Result<JobInfo, Error>;
}

/// Handle used for writing to the metadata service.
///
/// This handle also implements the [`ReadHandle`] trait, and can thus be used to read written
/// data. However, the changes made using calls in this trait are not visible from other handles
/// unless they are committed, using the [`commit()`](WriteHandle::commit) call.
#[async_trait]
pub trait WriteHandle: ReadHandle + Send + Sync {
    async fn crate_add(&self, name: &str) -> Result<(), BoxError>;
    async fn crate_version_add(
        &self,
        name: &str,
        version: &str,
        checksum: &str,
        yanked: bool,
    ) -> Result<(), BoxError>;

    async fn tasks_create_all(&self, kind: &str, triple: &str) -> Result<(), BoxError>;
    async fn job_request(&self, builder: Uuid) -> Result<Uuid, BoxError>;
    async fn commit(self: Box<Self>) -> Result<(), BoxError>;
}
