//! Common types

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use url::Url;
use uuid::Uuid;

/// Request a job from server.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobRequest {
    /// Target for this job.
    pub target: String,
}

/// Job information.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    /// UUID of job.
    pub uuid: Uuid,
    /// Name of crate
    pub name: String,
    /// Version of crate
    pub version: String,
    /// URL to download crate from.
    pub source: Url,
    /// Kind of job
    pub kind: JobKind,
}

/// Named variants
#[derive(
    Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum NamedVariant {
    /// Default variant
    #[default]
    Default,
}

/// Variant of build.
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
#[serde(untagged)]
pub enum Variant {
    /// Named variants
    Named(NamedVariant),
    /// Custom variant
    Custom(String),
}

impl Default for Variant {
    fn default() -> Self {
        Variant::Named(NamedVariant::default())
    }
}

/// Build environment
///
/// This struct contains all inputs needed for the build.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildEnv {
    /// Name of this variant
    pub variant: Variant,
    /// Target triple to build for
    pub target: String,
    /// Crate features to activate
    pub features: BTreeSet<String>,
    /// Enable default features
    pub default_features: bool,
    /// Environment variables to set
    pub environment: BTreeMap<String, String>,
    /// Additional dependencies to install
    pub dependencies: BTreeMap<String, String>,
}

/// Kind of job to build.
///
/// Each variant of this enumeration corresponds to one Cargo command.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum JobKind {
    /// Build metadata
    Metadata,
    /// Build binaries
    Binary(BuildEnv),
    /// Build debian package
    Debian(BuildEnv),
    /// Build coverage
    Coverage(BuildEnv),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn default_variant() {
        assert_eq!(Variant::default(), Variant::Named(NamedVariant::Default));
    }

    #[test]
    fn variant_serialize_default() {
        assert_tokens(
            &Variant::Named(NamedVariant::Default),
            &[Token::UnitVariant {
                name: "NamedVariant",
                variant: "default",
            }],
        );
    }

    #[test]
    fn variant_serialize_custom() {
        assert_tokens(&Variant::Custom("custom".into()), &[Token::Str("custom")]);
    }
}
