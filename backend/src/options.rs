use anyhow::Result;
use buildsrs_backend::Backend;
use buildsrs_database::DatabaseOptions;
use buildsrs_storage::StorageOptions;
use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug, PartialEq)]
pub struct Options {
    #[clap(short, long, env = "BUILDSRS_LISTEN", default_value = "0.0.0.0:8000")]
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
        let backend = Backend::new(database, storage);

        #[cfg(feature = "frontend-vendor")]
        let backend = backend.with_frontend(buildsrs_backend::frontend().into());

        Ok(backend)
    }
}

#[test]
fn test_default_options() {
    let _options = Options::try_parse_from([
        "backend",
        "--storage",
        "filesystem",
        "--storage-filesystem-path",
        "/tmp",
        "--database",
        "postgres",
    ])
    .unwrap();
}
