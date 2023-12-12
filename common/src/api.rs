//! Types for the API of buildsrs
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Response for crate API
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CratesQuery {
    /// Crate name
    pub name: String,
}

/// Response for crate API
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CratesResponse {
    /// Crates that matched
    pub crates: Vec<String>,
}

/// Response for crate API
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CrateResponse {
    /// Crate name
    pub name: String,
    /// Crate versions
    pub versions: BTreeSet<String>,
}

/// Crate version response
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CrateVersionResponse {
    /// Crate name
    pub name: String,
    /// Version name
    pub version: String,
    /// Yanked status of this crate
    pub yanked: bool,
    /// Digest of this crate
    pub checksum: String,
    /// Artifacts
    pub artifacts: BTreeSet<String>,
}

/// Crate artifact response
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ArtifactResponse {
    /// Name of the crate
    pub name: String,
    /// Version of the crate
    pub version: String,
    /// Size in bytes
    pub size: usize,
}
