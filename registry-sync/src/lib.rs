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

        let handle = self.database.write().await.unwrap();

        info!("Updating crates from index");
        let index = self.index.clone().lock_owned().await;
        let (sender, receiver) = channel(1024);

        let reader = spawn_blocking(move || {
            for krate in index.crates() {
                sender.blocking_send(krate)?;
            }
            Ok(()) as Result<()>
        });

        let writer = async move {
            let handle_ref = &handle;
            ReceiverStream::new(receiver)
                .enumerate()
                .flat_map(move |(index, krate)| {
                    info!("Syncing crate #{index} {}", krate.name());
                    let name = krate.name().to_string();
                    let versions = krate.versions().to_vec();
                    #[allow(clippy::async_yields_async)]
                    let stream = once(async move {
                        Box::pin(async move {
                            handle_ref.crate_add(&name).await.unwrap();
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
                                .unwrap();
                            Ok(()) as Result<()>
                        }) as Pin<Box<dyn Future<Output = Result<()>>>>
                    }));

                    stream
                })
                .buffer_unordered(128)
                .try_collect::<()>()
                .await?;

            handle
                .tasks_create_all("metadata", "generic")
                .await
                .unwrap();

            Ok(handle) as Result<_>
        };

        let (reader, handle) = join(reader, writer).await;
        reader??;
        let handle: Box<dyn WriteHandle> = handle?;
        handle.commit().await.unwrap();

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
