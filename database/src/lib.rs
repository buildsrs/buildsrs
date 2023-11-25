//! # Buildsrs Database
//!
//! The buildsrs project uses a database to store metadata about crates, crate versions, and
//! artifacts. The database that is used is the postgres database. This crate implements all
//! database interactions in the shape of methods that can be consumed elsewhere in the project.

#![allow(missing_docs)]

mod postgres;

use async_trait::async_trait;
pub use postgres::*;
use std::sync::Arc;

/// Shared generic error type.
pub type SharedError = Arc<dyn std::error::Error + Send + Sync>;

/// Boxed generic error type.
pub type BoxError = Arc<dyn std::error::Error + Send + Sync>;

/// Shared generic metadata instance.
///
/// This type is how downstream users should consume metadata instances.
pub type AnyMetadata = Arc<dyn Metadata>;

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
    async fn read(&self) -> Result<Box<dyn ReadHandle>, SharedError>;

    /// Get a write handle to use for writing.
    async fn write(&self) -> Result<Box<dyn WriteHandle>, SharedError>;
}

#[async_trait]
pub trait ReadHandle: Send + Sync {}

/// Handle used for writing to the metadata service.
///
/// This handle also implements the [`ReadHandle`] trait, and can thus be used to read written
/// data. However, the changes made using calls in this trait are not visible from other handles
/// unless they are committed, using the [`commit()`](WriteHandle::commit) call.
#[async_trait]
pub trait WriteHandle: ReadHandle + Send + Sync {}
