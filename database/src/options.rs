use crate::{AnyMetadata, BoxError, Pool};
use clap::{Parser, ValueEnum};
use std::sync::Arc;

#[derive(ValueEnum, Clone, Copy, PartialEq, Debug)]
enum DatabaseKind {
    Postgres,
}

#[derive(Parser, Debug, Clone, PartialEq)]
pub struct DatabaseOptions {
    #[clap(long, env = "DATABASE_KIND")]
    database: DatabaseKind,

    #[clap(flatten)]
    postgres: PostgresOptions,
}

impl DatabaseOptions {
    pub async fn build(&self) -> Result<AnyMetadata, BoxError> {
        match self.database {
            DatabaseKind::Postgres => self.postgres.build().await,
        }
    }
}

#[derive(Parser, Debug, Clone, PartialEq)]
struct PostgresOptions {
    #[clap(long, env)]
    database_postgres: Option<String>,

    #[clap(long, default_value = "16", env)]
    database_postgres_connections: usize,
}

impl PostgresOptions {
    async fn build(&self) -> Result<AnyMetadata, BoxError> {
        let pool = Pool::new(
            self.database_postgres.as_ref().unwrap(),
            self.database_postgres_connections,
        )
        .await?;
        Ok(Arc::new(pool) as AnyMetadata)
    }
}
