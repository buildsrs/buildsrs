//! # Buildsrs Common
//!
//! Common types shared by the buildsrs project.
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use url::Url;

/// Crate info.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct CrateInfo {
    /// Crate name
    pub name: String,
    /// Link to crate homepage
    pub homepage: Option<Url>,
    /// Link to source code repository
    pub repository: Option<Url>,
    /// Link to documentation
    pub documentation: Option<Url>,
    /// Crate license specifier
    pub license: Option<String>,
}

/// Crate version info.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct CrateVersion {
    /// Version
    pub version: String,
    /// Yanked
    pub yanked: bool,
    /// Checksum
    pub checksum: String,
}
