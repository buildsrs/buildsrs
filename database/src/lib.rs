use futures::{Stream, StreamExt};
use std::{pin::Pin, sync::Arc};
use tokio::select;
use tokio_postgres::{
    connect, types::Json, AsyncMessage, Client, Error, GenericClient, NoTls, Statement, Transaction,
};

#[macro_use]
mod macros;
#[cfg(any(feature = "temp", test))]
mod temp;
#[cfg(test)]
mod tests;
mod util;

statements!(
    /// Register new builder by SSH pubkey.
    fn builder_add(pubkey: &str, fingerprint_sha256: &str, fingerprint_sha512: &str) {
        "INSERT INTO
            builders(
                builder_pubkey,
                builder_fingerprint_sha256,
                builder_fingerprint_sha512
            )
        VALUES ($1, $2, $3)
        ON CONFLICT DO NOTHING"
    }

    /// Create a new target
    fn target_add(name: &str) {
        "INSERT INTO targets(target_name) VALUES ($1)
        ON CONFLICT DO NOTHING"
    }

    /// Add a crate to the database.
    fn crate_add(name: &str) {
        "INSERT INTO registry_crates(crate_name) VALUES ($1)
        ON CONFLICT DO NOTHING"
    }

    /// Add a crate version to the database.
    fn crate_version_add(krate: &str, version: &str, checksum: &str, yanked: bool) {
        "INSERT INTO registry_versions(crate_id, version, checksum, yanked)
        VALUES (
            (SELECT crate_id FROM registry_crates WHERE crate_name = $1),
            $2, $3, $4
        )
        ON CONFLICT (version) DO UPDATE SET yanked = $4"
    }

    let crate_versions = "
        SELECT version
        FROM registry_versions_view
        WHERE crate_name = $1
    ";
    let version_info = "
        SELECT
            yanked
        FROM registry_versions_view
        WHERE
            crate_name = $1
            AND version = $2
    ";
    let job_create = "
        INSERT INTO build_jobs(builder_id, target_id, version_id)
        VALUES (
            $1,
            $2,
            (SELECT version_id FROM build_queue WHERE target_id = $2)
        )
        RETURNING (job_id)
    ";
    let crate_list = "SELECT 1";
    let crate_query = "SELECT 1";
);

#[cfg(any(feature = "migrations", test))]
refinery::embed_migrations!("migrations");

/// Database wrapper
///
/// This precompiles statements and offers wrappers for all mutations and queries. The wrappers are
/// partly automatically generated by the `statements!` macro.
#[derive(Clone, Debug)]
pub struct Database<T = Client> {
    /// Precompiled statements
    pub statements: Arc<Statements>,
    /// Connection to database
    pub connection: T,
}

impl<T: GenericClient> Database<T> {}

pub type ConnectionStream = Pin<Box<dyn Stream<Item = Result<AsyncMessage, Error>> + Send>>;

impl Database<Client> {
    /// Create new [`Database`] from Postgres [`Client`].
    ///
    /// This will prepare all of the statements that are used.
    pub async fn new(connection: Client) -> Result<Self, Error> {
        Ok(Database {
            statements: Arc::new(Statements::prepare(&connection).await?),
            connection,
        })
    }

    /// Connect to database.
    pub async fn connect(database: &str) -> Result<Self, Error> {
        let (client, mut connection) = connect(database, NoTls).await?;
        tokio::spawn(connection);
        Database::new(client).await
    }

    /// Create transaction.
    pub async fn transaction(&mut self) -> Result<Database<Transaction<'_>>, Error> {
        Ok(Database {
            statements: self.statements.clone(),
            connection: self.connection.transaction().await?,
        })
    }
}

impl Database<Transaction<'_>> {
    /// Commit this transaction.
    pub async fn commit(self) -> Result<(), Error> {
        self.connection.commit().await
    }
}
