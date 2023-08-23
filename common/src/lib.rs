use serde::{Deserialize, Serialize};
use url::Url;

/// Crate info.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct CrateInfo {
    pub name: String,
    pub homepage: Option<Url>,
    pub repository: Option<Url>,
    pub documentation: Option<Url>,
    pub license: Option<String>,
}

/// Crate version info.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct CrateVersion {
    pub version: String,
    pub yanked: bool,
    pub checksum: String,
}
