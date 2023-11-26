//! # Entities

use url::Url;
use bytes::Bytes;
use std::time::Duration;
#[cfg(feature = "proptest")]
use test_strategy::Arbitrary;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Crate info.
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CrateVersion {
    /// Version
    pub version: String,
    /// Yanked
    pub yanked: bool,
    /// Checksum
    pub checksum: String,
}

/// Kind of artifact.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ArtifactId {
    /// Name of crate
    #[cfg_attr(feature = "proptest", strategy("[a-z]{20}"))]
    pub krate: String,
    /// Name of crate version
    #[cfg_attr(feature = "proptest", strategy("[a-z]{20}"))]
    pub version: String,
    /// Target triple
    #[cfg_attr(feature = "proptest", strategy("[a-z]{20}"))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
            Self::Redirect { .. } => None,
        }
    }
}
