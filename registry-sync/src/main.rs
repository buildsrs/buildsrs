#![allow(missing_docs)]
use anyhow::Result;
use buildsrs_database::Database;
use buildsrs_registry_sync::Syncer;
use clap::Parser;
use crates_index::GitIndex;
use log::*;
use std::{path::PathBuf, time::Duration};
use url::Url;

#[derive(Parser, PartialEq, Clone, Debug)]
pub struct Options {
    /// Path to keep index at.
    #[clap(long, short, env = "REGISTRY_PATH")]
    path: PathBuf,

    /// Registry to clone.
    #[clap(short, long, env = "REGISTRY_URL", default_value = crates_index::git::URL)]
    registry: Url,

    /// Interval to sync registry at.
    #[clap(short, long, env = "SYNC_INTERVAL", value_parser = humantime::parse_duration, default_value = "1h")]
    interval: Duration,

    /// Database to connect to
    #[clap(short, long, env = "DATABASE")]
    database: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();
    let options = Options::parse();

    info!("Connecting to database");
    let database = Database::connect(&options.database).await.unwrap();

    info!("Setting up registry index");
    let index = GitIndex::with_path(&options.path, options.registry.as_str()).unwrap();

    let mut context = Syncer::new(database, index);

    context.sync_loop(options.interval).await
}
