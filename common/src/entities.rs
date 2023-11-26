//! # Entities

use bytes::Bytes;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use ssh_key::PublicKey;
use std::time::Duration;
use strum::EnumString;
#[cfg(feature = "proptest")]
use test_strategy::Arbitrary;
use url::Url;
use uuid::Uuid;

/// Target triple, such as `x86_64-unknown-linux-gnu`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TargetTriple(String);

impl TargetTriple {
    /// Architecture
    pub fn arch(&self) -> &str {
        self.0.split('-').nth(0).unwrap()
    }

    /// Vendor, might be `unknown`.
    pub fn vendor(&self) -> &str {
        self.0.split('-').nth(1).unwrap()
    }

    /// System (usually represents the kernel type)
    pub fn system(&self) -> &str {
        self.0.split('-').nth(2).unwrap()
    }

    /// ABI, if there are multiple.
    pub fn abi(&self) -> Option<&str> {
        self.0.split('-').nth(3)
    }
}

/// Represents a crate name
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CrateName(String);

/// Represents a crate version
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
//#[cfg_attr(feature = "proptest", derive(Arbitrary))]
//#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CrateVersion {
    krate: CrateName,
    version: semver::Version,
}

/// Kind of artifact.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, EnumString)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[strum(serialize_all = "snake_case")]
pub enum ArtifactKind {
    /// Manifest, with a `.json` extension.
    Metadata,
    /// Tarball, with a `.tar.gz` extension.
    Tarball,
    /// Debian package, with a `.deb` extension.
    Debian,
    /// Coverage report, with a `.deb` extension.
    Coverage,
}

impl ArtifactKind {
    /// Get extension for this artifact kind.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Metadata => "metadata.json",
            Self::Tarball => "tar.gz",
            Self::Debian => "deb",
            Self::Coverage => "coverage.json",
        }
    }
}

/// Artifact identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ArtifactId {
    /// Task that produced this artifact
    pub task: Task,
}

impl ArtifactId {
    /// Get the file name for this artifact identifier
    pub fn file_name(&self) -> String {
        self.task.file_name()
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

/// Builder that is registered in the database
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Builder {
    /// UUID of this builder
    pub uuid: Uuid,
    /// Public key this builder uses to sign messages and artifacts
    pub public_key: PublicKey,
    /// Comment
    pub comment: String,
    /// Enabled state
    pub enabled: bool,
}

/// Target that can be built
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TargetInfo {
    /// Name of this target
    pub name: String,
    /// Enabled status
    pub enabled: bool,
}

/// Crate
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CrateInfo {
    /// Name of this crate
    pub name: String,
    /// Is crate enabled
    pub enabled: bool,
}

/// Crate version
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VersionInfo {
    /// Name of this crate
    pub name: String,
    /// Version
    pub version: String,
    /// Checksum
    pub checksum: String,
    /// Yanked status
    pub yanked: bool,
}

/// Job
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct JobInfo {
    /// Job UUID
    pub uuid: Uuid,
    /// Builder processing this job
    pub builder: Uuid,
    /// Name of the crate being built
    pub name: String,
    /// Version of the crate being built
    pub version: String,
    /// Triple being built
    pub triple: String,
}

/// Task
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Task {
    /// Crate being built
    #[cfg_attr(feature = "proptest", strategy("[a-z]{20}"))]
    pub krate: String,
    /// Version being built
    #[cfg_attr(feature = "proptest", strategy("[a-z]{20}"))]
    pub version: String,
    /// Triple being built
    #[cfg_attr(feature = "proptest", strategy("[a-z]{20}"))]
    pub triple: String,
    /// Kind of artifact to build
    pub kind: ArtifactKind,
}

impl Task {
    /// Get the file name for this artifact identifier
    pub fn file_name(&self) -> String {
        let Self {
            krate,
            version,
            triple,
            kind,
        } = &self;
        let extension = kind.extension();
        format!("{krate}_{version}_{triple}.{extension}")
    }
}

#[cfg(all(test, feature = "proptest"))]
mod tests {
    use super::*;
    use test_strategy::proptest;

    #[proptest]
    fn task_path_different(left: Task, right: Task) {
        if left != right {
            assert!(left.file_name() != right.file_name());
        }
    }
}
