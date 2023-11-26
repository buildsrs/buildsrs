use anyhow::Result;
use buildsrs_backend::Backend;
use buildsrs_database::DatabaseOptions;
use buildsrs_storage::StorageOptions;
use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(short, long, env = "BUILDSRS_LISTEN", default_value = "127.0.0.1:8000")]
    pub listen: SocketAddr,

    #[clap(flatten)]
    pub storage: StorageOptions,

    #[clap(flatten)]
    pub database: DatabaseOptions,
}

impl Options {
    pub async fn build(&self) -> Result<Backend> {
        let database = self.database.build().await.unwrap();
        let storage = self.storage.build().await.unwrap();
        Ok(Backend::new(database, storage))
    }
}
