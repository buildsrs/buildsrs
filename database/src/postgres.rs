#![allow(missing_docs)]

use super::*;
use deadpool::unmanaged::{Object, Pool as Deadpool};
use futures::Stream;
use ssh_key::{HashAlg, PublicKey};
use std::{collections::BTreeSet, ops::Deref, pin::Pin, sync::Arc};
use tokio::task::JoinHandle;
use tokio_postgres::{connect, AsyncMessage, Client, GenericClient, NoTls, Statement};
pub use tokio_postgres::{Error, Transaction};
use uuid::Uuid;

#[macro_use]
mod macros;
pub mod entity;
#[cfg(feature = "temp")]
mod temp;
mod util;

use entity::*;
#[cfg(feature = "temp")]
pub use temp::*;

statements!(
    /// Register new builder by SSH pubkey and comment.
    fn builder_register(uuid: Uuid, pubkey: i64) {
        "INSERT INTO builders(uuid, pubkey)
        VALUES ($1, $2)"
    }

    /// Add a fingerprint to a registered builder.
    fn fingerprint_add(pubkey: i64, fingerprint: &str) {
        "INSERT INTO pubkey_fingerprints(pubkey, fingerprint)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING"
    }

    /// Set builder enabled
    fn builder_set_enabled(uuid: Uuid, enabled: bool) {
        "UPDATE builders
        SET enabled = $2
        WHERE uuid = $1"
    }

    /// Set builder comment
    fn builder_set_comment(uuid: Uuid, commend: &str) {
        "UPDATE builders
        SET comment = $2
        WHERE uuid = $1"
    }

    /// Add an allowed triple for a builder
    fn builder_triple_add(builder: Uuid, triple: &str) {
        "INSERT INTO builder_triples(builder, triple)
        VALUES (
            (SELECT id FROM builders WHERE uuid = $1),
            (SELECT id FROM triples WHERE name = $2)
        )
        ON CONFLICT DO NOTHING"
    }

    /// Remove an allowed triple for a builder
    fn builder_triple_remove(builder: Uuid, triple: &str) {
        "DELETE FROM builder_triples
        WHERE builder = (SELECT id FROM builders WHERE uuid = $1)
        AND triple = (SELECT id FROM triples WHERE name = $2)"
    }

    /// Request a job for a builder.
    fn builder_request(builder: Uuid, triple: &str) {
        "SELECT 1"
    }

    /// Create a new triple
    fn triple_add(name: &str) {
        "INSERT INTO triples(name) VALUES ($1)
        ON CONFLICT DO NOTHING"
    }

    /// Remove a triple
    fn triple_remove(name: &str) {
        "DELETE FROM triples
        WHERE name = $1"
    }

    /// Set triple enabled or disabled
    fn triple_enabled(name: &str, enabled: bool) {
        "UPDATE triples
        SET enabled = $2
        WHERE name = $1"
    }

    /// Set triple enabled or disabled
    fn triple_rename(triple: &str, name: &str) {
        "UPDATE triples
        SET name = $2
        WHERE name = $1"
    }

    /// Add a crate to the database.
    fn crate_add(name: &str) {
        "INSERT INTO crates(name) VALUES ($1)
        ON CONFLICT DO NOTHING"
    }

    /// Add a crate to the database.
    fn tasks_create_all(kind: &str, triple: &str) {
        "INSERT INTO tasks(version, kind, triple)
        SELECT
            id,
            (SELECT id FROM task_kinds WHERE name = $1),
            (SELECT id FROM triples WHERE name = $2)
        FROM crate_versions
        ON CONFLICT DO NOTHING"
    }

    /// Add a crate version to the database.
    fn crate_version_add(krate: &str, version: &str, checksum: &str, yanked: bool) {
        "INSERT INTO crate_versions(crate, version, checksum, yanked)
        VALUES (
            (SELECT id FROM crates WHERE name = $1),
            $2, $3, $4
        )
        ON CONFLICT (version) DO UPDATE SET yanked = $4"
    }

    /// Set the job's current stage.
    fn job_stage(job: Uuid, stage: &str) {
        "UPDATE jobs
        SET stage = (SELECT id FROM job_stages WHERE name = $2)
        WHERE uuid = $1"
    }

    /// Add a log message for the job.
    fn job_log(job: Uuid, line: &str) {
        "INSERT INTO job_logs(job, stage, line)
        VALUES (
            (SELECT id FROM jobs WHERE uuid = $1),
            (SELECT stage FROM jobs WHERE uuid = $1),
            $2
        )"
    }

    let builder_by_fingerprint = "
        SELECT uuid
        FROM builders
        JOIN pubkey_fingerprints_view
        ON builders.pubkey = pubkey_fingerprints_view.id
        WHERE fingerprint = $1
    ";

    let builder_get = "
        SELECT *
        FROM builders_view
        WHERE uuid = $1
    ";

    let builder_list = "
        SELECT uuid
        FROM builders
    ";

    let builder_triples = "
        SELECT triple_name
        FROM builder_triples_view
        WHERE builder_uuid = $1
    ";

    let triple_list = "
        SELECT name
        FROM triples
    ";

    let triple_info = "
        SELECT *
        FROM triples
        WHERE name = $1
    ";

    let crate_list = "
        SELECT name
        FROM crates
        WHERE name % $1
    ";

    let crate_info = "
        SELECT *
        FROM crates
        WHERE name = $1
    ";

    let crate_versions = "
        SELECT version
        FROM crate_versions_view
        WHERE name = $1
    ";

    let version_info = "
        SELECT *
        FROM crate_versions_view
        WHERE name = $1
        AND version = $2
    ";

    let task_list = "
        SELECT *
        FROM tasks_view
        WHERE coalesce(crate = $1, true)
        AND coalesce(version = $2, true)
        AND coalesce(kind = $3, true)
        AND coalesce(triple = $4, true)
    ";

    let job_create = "
        INSERT INTO jobs(uuid, builder, task, stage)
        VALUES (
            $3,
            (SELECT id FROM builders WHERE uuid = $1),
            (SELECT id FROM tasks WHERE
                triple = (SELECT id FROM triples WHERE name = $2)
            ),
            (SELECT id FROM job_stages WHERE name = 'init')
        )
        RETURNING (uuid)
    ";

    let job_info = "
        SELECT *
        FROM jobs_view
        WHERE uuid = $1
    ";

    let pubkey_add = "
        INSERT INTO pubkeys (encoded)
        VALUES ($1)
        ON CONFLICT (encoded)
        DO NOTHING
        RETURNING id;
    ";
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
    statements: Arc<Statements>,
    /// Connection to database
    connection: T,
}

impl<T: GenericClient> Database<T> {
    pub async fn builder_lookup(&self, fingerprint: &str) -> Result<Uuid, Error> {
        let row = self
            .connection
            .query_one(&self.statements.builder_by_fingerprint, &[&fingerprint])
            .await?;
        row.try_get("uuid")
    }

    pub async fn builder_get(&self, builder: Uuid) -> Result<Builder, Error> {
        let row = self
            .connection
            .query_one(&self.statements.builder_get, &[&builder])
            .await?;
        Ok(Builder {
            uuid: builder,
            public_key: {
                let pubkey: &str = row.try_get("pubkey")?;
                PublicKey::from_openssh(pubkey).unwrap()
            },
            comment: row.try_get("comment")?,
            enabled: row.try_get("enabled")?,
        })
    }

    pub async fn builder_list(&self) -> Result<Vec<Uuid>, Error> {
        let rows = self
            .connection
            .query(&self.statements.builder_list, &[])
            .await?;
        rows.into_iter().map(|row| row.try_get("uuid")).collect()
    }

    pub async fn builder_triples(&self, builder: Uuid) -> Result<BTreeSet<String>, Error> {
        let rows = self
            .connection
            .query(&self.statements.builder_triples, &[&builder])
            .await?;
        rows.into_iter()
            .map(|row| row.try_get("triple_name"))
            .collect()
    }

    pub async fn triple_list(&self) -> Result<BTreeSet<String>, Error> {
        let rows = self
            .connection
            .query(&self.statements.triple_list, &[])
            .await?;
        rows.into_iter().map(|row| row.try_get("name")).collect()
    }

    pub async fn triple_info(&self, triple: &str) -> Result<TargetInfo, Error> {
        let row = self
            .connection
            .query_one(&self.statements.triple_info, &[&triple])
            .await?;
        Ok(TargetInfo {
            name: row.try_get("name")?,
            enabled: row.try_get("enabled")?,
        })
    }

    pub async fn task_list(
        &self,
        krate: Option<&str>,
        version: Option<&str>,
        task: Option<&str>,
        triple: Option<&str>,
    ) -> Result<Vec<Task>, Error> {
        let rows = self
            .connection
            .query(
                &self.statements.task_list,
                &[&krate, &version, &task, &triple],
            )
            .await?;
        rows.into_iter()
            .map(|row| {
                Ok(Task {
                    krate: row.try_get("crate")?,
                    version: row.try_get("version")?,
                    kind: { row.try_get::<&str, &str>("kind")?.parse().unwrap() },
                    triple: row.try_get("triple")?,
                })
            })
            .collect()
    }

    pub async fn job_request(&self, builder: Uuid, triple: &str) -> Result<Uuid, Error> {
        let row = self
            .connection
            .query_one(
                &self.statements.job_create,
                &[&builder, &triple, &Uuid::new_v4()],
            )
            .await?;
        row.try_get("uuid")
    }

    pub async fn job_info(&self, job: Uuid) -> Result<JobInfo, Error> {
        let row = self
            .connection
            .query_one(&self.statements.job_info, &[&job])
            .await?;
        Ok(JobInfo {
            uuid: row.try_get("uuid")?,
            version: row.try_get("crate_version_version")?,
            name: row.try_get("crate_name")?,
            builder: row.try_get("builder_uuid")?,
            triple: row.try_get("triple_name")?,
        })
    }

    /// Get info on a crate
    pub async fn crate_list(&self, name: &str) -> Result<Vec<String>, Error> {
        let rows = self
            .connection
            .query(&self.statements.crate_list, &[&name])
            .await?;
        rows.into_iter().map(|row| row.try_get("name")).collect()
    }

    /// Get info on a crate
    pub async fn crate_info(&self, name: &str) -> Result<CrateInfo, Error> {
        let info = self
            .connection
            .query_one(&self.statements.crate_info, &[&name])
            .await?;
        Ok(CrateInfo {
            name: info.try_get("name")?,
            enabled: info.try_get("enabled")?,
        })
    }

    /// Get a list of versions for a crate
    pub async fn crate_versions(&self, name: &str) -> Result<Vec<String>, Error> {
        let rows = self
            .connection
            .query(&self.statements.crate_versions, &[&name])
            .await?;
        rows.into_iter().map(|row| row.try_get("version")).collect()
    }

    /// Get info on a crate version
    pub async fn crate_version_info(
        &self,
        name: &str,
        version: &str,
    ) -> Result<VersionInfo, Error> {
        let info = self
            .connection
            .query_one(&self.statements.version_info, &[&name, &version])
            .await?;
        Ok(VersionInfo {
            name: info.try_get("name")?,
            version: info.try_get("version")?,
            checksum: info.try_get("checksum")?,
            yanked: info.try_get("yanked")?,
        })
    }
}

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
        let (client, connection) = connect(database, NoTls).await?;
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

    /// Add a builder
    pub async fn builder_add(
        &self,
        uuid: Uuid,
        key: &PublicKey,
        comment: &str,
    ) -> Result<(), Error> {
        let key = self.pubkey_add(key).await?;
        self.builder_register(uuid, key).await?;
        self.builder_set_comment(uuid, comment).await?;
        Ok(())
    }

    /// Add a pubkey.
    async fn pubkey_add(&self, pubkey: &PublicKey) -> Result<i64, Error> {
        let row = self
            .connection
            .query_one(
                &self.statements.pubkey_add,
                &[&pubkey.to_openssh().unwrap()],
            )
            .await?;
        let id = row.try_get("id")?;
        for alg in [HashAlg::Sha256, HashAlg::Sha512] {
            self.fingerprint_add(id, &pubkey.fingerprint(alg).to_string())
                .await?;
        }
        Ok(id)
    }
}

#[derive(Debug)]
pub struct DatabaseConnection {
    database: Database,
    connection: Option<JoinHandle<Result<(), Error>>>,
}

impl DatabaseConnection {
    pub fn new(database: Database, connection: Option<JoinHandle<Result<(), Error>>>) -> Self {
        Self {
            database,
            connection,
        }
    }

    pub async fn close(self) -> Result<(), BoxError> {
        let Self {
            database,
            connection,
        } = self;
        drop(database);
        if let Some(connection) = connection {
            connection.await??;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Pool {
    pool: Deadpool<DatabaseConnection>,
}

impl Pool {
    pub async fn new(database: &str, count: usize) -> Result<Self, Error> {
        let mut databases = vec![];
        for _ in 0..count {
            let (client, connection) = connect(database, NoTls).await?;
            databases.push(DatabaseConnection {
                connection: Some(tokio::spawn(connection)),
                database: Database::new(client).await?,
            });
        }

        Ok(Pool {
            pool: Deadpool::from(databases),
        })
    }

    pub async fn close(self) {
        self.pool.close();
        while let Ok(conn) = self.pool.remove().await {
            let _ = conn.close().await;
        }
    }

    pub async fn read(&self) -> Result<Handle, BoxError> {
        Ok(Handle {
            object: self.pool.get().await?,
        })
    }

    pub async fn write(&self) -> Result<Writer, BoxError> {
        let object = self.pool.get().await?;

        let mut tx = Writer {
            object: Box::new(object),
            transaction: None,
        };

        let transaction = tx.object.database.transaction().await?;

        // we are doing something naughty here.. this is only safe
        // because we know the client is boxed and pinned in place.
        let transaction = unsafe { std::mem::transmute::<_, _>(transaction) };

        tx.transaction = Some(transaction);

        Ok(tx)
    }
}

impl From<DatabaseConnection> for Pool {
    fn from(conn: DatabaseConnection) -> Self {
        Pool {
            pool: Deadpool::from(vec![conn]),
        }
    }
}

impl From<Vec<DatabaseConnection>> for Pool {
    fn from(conns: Vec<DatabaseConnection>) -> Self {
        Pool {
            pool: Deadpool::from(conns),
        }
    }
}

#[derive(Debug)]
pub struct Handle {
    object: Object<DatabaseConnection>,
}

impl Deref for Handle {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        &self.object.database
    }
}

trait AsDatabase {
    type Client: GenericClient;

    fn database(&self) -> &Database<Self::Client>;
}

impl AsDatabase for Handle {
    type Client = Client;

    fn database(&self) -> &Database<Self::Client> {
        &self.object.database
    }
}

impl AsDatabase for Writer {
    type Client = Transaction<'static>;

    fn database(&self) -> &Database<Self::Client> {
        Writer::database(self)
    }
}

#[derive()]
pub struct Writer {
    object: Box<Object<DatabaseConnection>>,
    transaction: Option<Database<Transaction<'static>>>,
}

impl Writer {
    pub async fn commit(mut self) -> Result<(), Error> {
        let transaction = std::mem::take(&mut self.transaction);
        let transaction = transaction.unwrap();
        transaction.commit().await?;
        Ok(())
    }

    fn database(&self) -> &Database<Transaction<'static>> {
        self.transaction.as_ref().unwrap()
    }
}

impl Deref for Writer {
    type Target = Database<Transaction<'static>>;

    fn deref(&self) -> &Self::Target {
        Writer::database(self)
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        // drop transaction
        self.transaction = None;
    }
}

#[async_trait::async_trait]
impl Metadata for Pool {
    /// Get a read handle to use for reading.
    async fn read(&self) -> Result<Box<dyn ReadHandle>, BoxError> {
        Pool::read(self)
            .await
            .map(|x| Box::new(x) as Box<dyn ReadHandle>)
    }

    /// Get a write handle to use for writing.
    async fn write(&self) -> Result<Box<dyn WriteHandle>, BoxError> {
        Pool::write(self)
            .await
            .map(|x| Box::new(x) as Box<dyn WriteHandle>)
    }
}

#[async_trait::async_trait]
impl<T: AsDatabase + Send + Sync> ReadHandle for T
where
    <T as AsDatabase>::Client: Send + Sync,
{
    async fn builder_lookup(&self, fingerprint: &str) -> Result<Uuid, Error> {
        self.database().builder_lookup(fingerprint).await
    }

    async fn builder_get(&self, builder: Uuid) -> Result<Builder, Error> {
        self.database().builder_get(builder).await
    }

    async fn builder_list(&self) -> Result<Vec<Uuid>, Error> {
        self.database().builder_list().await
    }

    async fn crate_list(&self, name: &str) -> Result<Vec<String>, Error> {
        self.database().crate_list(name).await
    }

    async fn crate_info(&self, name: &str) -> Result<CrateInfo, Error> {
        self.database().crate_info(name).await
    }

    async fn crate_versions(&self, name: &str) -> Result<Vec<String>, Error> {
        self.database().crate_versions(name).await
    }

    async fn crate_version_info(&self, name: &str, version: &str) -> Result<VersionInfo, Error> {
        self.database().crate_version_info(name, version).await
    }
}

#[async_trait::async_trait]
impl WriteHandle for Writer {
    async fn crate_add(&self, name: &str) -> Result<(), BoxError> {
        self.database().crate_add(name).await?;
        Ok(())
    }

    async fn crate_version_add(
        &self,
        name: &str,
        version: &str,
        checksum: &str,
        yanked: bool,
    ) -> Result<(), BoxError> {
        self.database()
            .crate_version_add(name, version, checksum, yanked)
            .await?;
        Ok(())
    }

    async fn tasks_create_all(&self, kind: &str, triple: &str) -> Result<(), BoxError> {
        self.database().tasks_create_all(kind, triple).await?;
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), BoxError> {
        let writer: Writer = *self;
        writer.commit().await?;
        Ok(())
    }
}
