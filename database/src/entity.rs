//! # Entities
//!
//! This module defines entites that are stored in the database.

use ssh_key::PublicKey;
use uuid::Uuid;

/// Builder that is registered in the database
#[derive(Clone, Debug)]
pub struct Builder {
    pub uuid: Uuid,
    pub public_key: PublicKey,
    pub comment: String,
    pub enabled: bool,
}

/// Target that can be built
#[derive(Clone, Debug)]
pub struct TargetInfo {
    pub name: String,
    pub enabled: bool,
}

/// Crate
#[derive(Clone, Debug)]
pub struct CrateInfo {
    pub name: String,
    pub enabled: bool,
}

/// Crate version
#[derive(Clone, Debug)]
pub struct VersionInfo {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub yanked: bool,
}

/// Job
#[derive(Clone, Debug)]
pub struct JobInfo {
    pub uuid: Uuid,
    pub builder: Uuid,
    pub name: String,
    pub version: String,
    pub target: String,
}
