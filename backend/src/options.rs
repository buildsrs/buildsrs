use anyhow::Result;
use buildsrs_backend::Backend;
use buildsrs_database::Database;
use buildsrs_storage::StorageOptions;
use clap::Parser;
use std::{net::SocketAddr, sync::Arc};

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(short, long, env = "BUILDSRS_DATABASE")]
    pub database: String,

    #[clap(short, long, env = "BUILDSRS_LISTEN", default_value = "127.0.0.1:8000")]
    pub listen: SocketAddr,

    #[clap(flatten)]
    pub storage: StorageOptions,
}

impl Options {
    pub async fn build(&self) -> Result<Backend> {
        let database = Arc::new(Database::connect(&self.database).await?);
        let storage = self.storage.build().await.unwrap();
        Ok(Backend::new(database, storage))
    }
}
