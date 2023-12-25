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
use buildsrs_database::{WriteHandle, AnyMetadata};
use crates_index::GitIndex;
use log::*;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    task::spawn_blocking,
    time::{self, MissedTickBehavior},
};
use rayon::iter::ParallelIterator;

/// Synchronize a package registry with the database.
pub struct Syncer {
    database: AnyMetadata,
    index: Arc<Mutex<GitIndex>>,
}

impl Syncer {
    /// Create new instance, given a database connection and a [`GitIndex`].
    pub fn new(database: AnyMetadata, index: GitIndex) -> Self {
        Self {
            database,
            index: Arc::new(Mutex::new(index)),
        }
    }

    /// Run a single synchronization.
    pub async fn sync(&mut self) -> Result<()> {
        info!("Updating crate index");

        let mut index = self.index.clone().lock_owned().await;
        spawn_blocking(move || index.update()).await??;

        let transaction = self.database.write().await.unwrap();

        info!("Updating crates from index");
        let index = self.index.clone().lock_owned().await;
        let (sender, mut receiver) = channel(128);

        let reader = spawn_blocking(move || {
            index.crates_parallel().try_for_each(|krate| Ok(sender.blocking_send(krate?)?) as Result<_>)
        });

        let writer = async move {
            while let Some(krate) = receiver.recv().await {
                transaction.crate_add(krate.name()).await.unwrap();
                for version in krate.versions() {
                    transaction
                        .crate_version_add(
                            krate.name(),
                            version.version(),
                            &hex::encode(version.checksum()),
                            version.is_yanked(),
                        )
                        .await
                        .unwrap();
                }
            }

            transaction
                .tasks_create_all("metadata", "generic")
                .await
                .unwrap();

            Ok(transaction) as Result<_>
        };

        let (reader, writer) = futures::future::join(reader, writer).await;
        reader??;
        let transaction: Box<dyn WriteHandle> = writer?;

        transaction.commit().await.unwrap();

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
