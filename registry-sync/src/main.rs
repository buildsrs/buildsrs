use buildsrs_database::Database;
use clap::Parser;
use crates_index::{Crate, GitIndex};
use log::*;
use std::{collections::BTreeMap, path::PathBuf, thread::sleep, time::Duration};
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
async fn main() {
    env_logger::init();
    let options = Options::parse();

    info!("Connecting to database");
    let mut database = Database::connect(&options.database).await.unwrap();

    info!("Setting up registry index");
    let mut index = GitIndex::with_path(&options.path, options.registry.as_str()).unwrap();
    info!("Ready for syncing");

    loop {
        info!("Syncing crates");
        let transaction = database.transaction().await.unwrap();
        for krate in index.crates() {
            if let Some(krate) = index.crate_(krate.name()) {
                database.crate_add(krate.name()).await.unwrap();
                for version in krate.versions() {
                    database
                        .crate_version_add(krate.name(), version.version(), "", version.is_yanked())
                        .await
                        .unwrap();
                }
            }
        }

        transaction.commit().await.unwrap();

        info!("Sleeping until next iteration");
        sleep(options.interval);
        info!("Updating index");
        index.update().unwrap();
    }
}
