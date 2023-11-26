//! # Registry Sync
//!
//! In order for builds.rs to do it's job, it needs to have an up-to-date list of crates at all
//! times. This crate is responsible for doing that, by implementing a one-way synchronization from
//! a Rust registry to the buildsrs database.
//!
//! Rust package registries (such as [crates.io](https://crates.io)) have several ways for getting
//! data out of them, including a HTTP API, nightly database dumps and the Git index. The latter
//! is a Git repository that contains all crate metadata. This index was chosen as the source of
//! data for synchronization purposes, because it is relatively straightforward to consume.
//!
//! This crate exports a [`Syncer`] type, which implements the synchronization between a given
//! Git index and a database connection.

use anyhow::Result;
use buildsrs_database::AnyMetadata;
use crates_index::GitIndex;
use log::*;
use std::time::Duration;
use tokio::time::{self, MissedTickBehavior};

/// Synchronize a package registry with the database.
pub struct Syncer {
    database: AnyMetadata,
    index: GitIndex,
}

impl Syncer {
    /// Create new instance, given a database connection and a [`GitIndex`].
    pub fn new(database: AnyMetadata, index: GitIndex) -> Self {
        Self { database, index }
    }

    /// Run a single synchronization.
    pub async fn sync(&mut self) -> Result<()> {
        info!("Updating crate index");
        self.index.update()?;

        let writer = self.database.write().await.unwrap();

        info!("Updating crates from index");
        for krate in self.index.crates() {
            if let Some(krate) = self.index.crate_(krate.name()) {
                writer.crate_add(krate.name()).await.unwrap();
                for version in krate.versions() {
                    writer
                        .crate_version_add(
                            krate.name(),
                            version.version(),
                            &hex::encode(version.checksum()),
                            version.is_yanked(),
                        )
                        .await
                        .unwrap();
                }
                writer
                    .tasks_create_all("metadata", "generic")
                    .await
                    .unwrap();
            }
        }

        writer.commit().await.unwrap();
        Ok(())
    }

    /// Launch a synchronization loop.
    pub async fn sync_loop(&mut self, interval: Duration) -> Result<()> {
        info!("Launching sync loop");
        let mut timer = time::interval(interval);
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            timer.tick().await;
            self.sync().await?;
        }
    }
}
