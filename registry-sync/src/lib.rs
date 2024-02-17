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

use anyhow::{anyhow, Result};
use buildsrs_database::{AnyMetadata, WriteHandle};
use crates_index::GitIndex;
use futures::{
    future::join,
    stream::{iter, once, StreamExt, TryStreamExt},
    Future,
};
use log::*;
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc::channel, Mutex},
    task::spawn_blocking,
    time::{self, MissedTickBehavior},
};
use tokio_stream::wrappers::ReceiverStream;

/// Synchronize a package registry with the database.
pub struct Syncer {
    database: AnyMetadata,
    index: Arc<Mutex<GitIndex>>,
}

/// Length of the crates queue.
const CRATES_QUEUE_LENGTH: usize = 1024;
/// How many database requests to keep in-flight in parallel at any given time.
const DATABASE_PIPELINED_REQUESTS: usize = 128;

impl Syncer {
    /// Create new instance, given a database connection and a [`GitIndex`].
    pub fn new(database: AnyMetadata, index: GitIndex) -> Self {
        Self {
            database,
            index: Arc::new(Mutex::new(index)),
        }
    }

    /// Updates crates index.
    ///
    /// This will cause a network access, because it will attempt to fetch the latest state from
    /// the remote crates index using git.
    pub async fn update(&self) -> Result<()> {
        let mut index = self.index.clone().lock_owned().await;
        spawn_blocking(move || index.update()).await??;
        Ok(())
    }

    /// Synchronize crate index with database.
    pub async fn sync(&self) -> Result<()> {
        let handle = self.database.write().await.map_err(|e| anyhow!(e))?;
        let index = self.index.clone().lock_owned().await;
        let (sender, receiver) = channel(CRATES_QUEUE_LENGTH);

        // launch a blocking reader which emits a stream of crates into a queue
        let reader = spawn_blocking(move || {
            for krate in index.crates() {
                sender.blocking_send(krate)?;
            }
            Ok(()) as Result<()>
        });

        // launch a writer, which turns the crates into a stream of database
        // writes. the database writes are pipelined.
        let writer = async move {
            let handle_ref = &handle;
            ReceiverStream::new(receiver)
                .enumerate()
                .flat_map(move |(index, krate)| {
                    debug!("Syncing crate #{index} {}", krate.name());
                    let name = krate.name().to_string();
                    let versions = krate.versions().to_vec();
                    #[allow(clippy::async_yields_async)]
                    let stream = once(async move {
                        Box::pin(async move {
                            handle_ref.crate_add(&name).await.map_err(|e| anyhow!(e))?;
                            Ok(()) as Result<()>
                        }) as Pin<Box<dyn Future<Output = Result<()>>>>
                    })
                    .chain(iter(versions.into_iter()).map(move |version| {
                        let name = krate.name().to_string();
                        Box::pin(async move {
                            handle_ref
                                .crate_version_add(
                                    &name,
                                    version.version(),
                                    &hex::encode(version.checksum()),
                                    version.is_yanked(),
                                )
                                .await
                                .map_err(|e| anyhow!(e))?;
                            Ok(()) as Result<()>
                        }) as Pin<Box<dyn Future<Output = Result<()>>>>
                    }));

                    stream
                })
                .buffer_unordered(DATABASE_PIPELINED_REQUESTS)
                .try_collect::<()>()
                .await?;

            debug!("Creating metadata tasks");
            handle
                .tasks_create_all("metadata", "generic")
                .await
                .map_err(|e| anyhow!(e))?;

            Ok(handle) as Result<_>
        };

        let (reader, handle) = join(reader, writer).await;
        reader??;
        let handle: Box<dyn WriteHandle> = handle?;

        info!("Committing changes");
        handle.commit().await.map_err(|e| anyhow!(e))?;
        info!("Done synchronizing");

        Ok(())
    }

    /// Launch a synchronization loop.
    pub async fn sync_loop(&mut self, interval: Duration) -> Result<()> {
        info!("Launching sync loop");
        let mut timer = time::interval(interval);
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            timer.tick().await;

            info!("Updating crate index");
            self.update().await?;

            info!("Synchronizing crate index");
            self.sync().await?;
        }
    }
}
