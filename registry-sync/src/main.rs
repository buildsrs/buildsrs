use clap::Parser;
use std::path::PathBuf;
use crates_index::{Crate, GitIndex};
use url::Url;
use log::*;
use std::time::Duration;
use std::thread::sleep;
use std::collections::BTreeMap;

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
}

fn main() {
    env_logger::init();
    let options = Options::parse();

    info!("Setting up registry index");
    let mut index = GitIndex::with_path(&options.path, options.registry.as_str()).unwrap();
    info!("Ready for syncing");

    loop {
        info!("Syncing crates");
        let crates: BTreeMap<String, Crate> = index.crates().map(|c| (c.name().into(), c)).collect();

        info!("Sleeping until next iteration");
        sleep(options.interval);
        info!("Updating index");
        index.update().unwrap();
    }
}
