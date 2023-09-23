use futures::{Stream, StreamExt};
use ssh_key::{HashAlg, PublicKey};
use std::{collections::BTreeSet, pin::Pin, sync::Arc};
use tokio::select;
use tokio_postgres::{connect, types::Json, AsyncMessage, Client, GenericClient, NoTls, Statement};
pub use tokio_postgres::{Error, Transaction};
use uuid::Uuid;

#[macro_use]
mod macros;
#[cfg(any(feature = "temp", test))]
mod temp;
#[cfg(test)]
mod tests;
mod util;

#[derive(Clone, Debug)]
pub struct Builder {
    pub uuid: Uuid,
    pub public_key: PublicKey,
    pub comment: String,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct TargetInfo {
    pub name: String,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct CrateInfo {
    pub name: String,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct VersionInfo {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub yanked: bool,
}

#[derive(Clone, Debug)]
pub struct JobInfo {
    pub uuid: Uuid,
    pub builder: Uuid,
    pub name: String,
    pub version: String,
    pub target: String,
}

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

    /// Add an allowed target for a builder
    fn builder_target_add(builder: Uuid, target: &str) {
        "INSERT INTO builder_targets(builder, target)
        VALUES (
            (SELECT id FROM builders WHERE uuid = $1),
            (SELECT id FROM targets WHERE name = $2)
        )
        ON CONFLICT DO NOTHING"
    }

    /// Remove an allowed target for a builder
    fn builder_target_remove(builder: Uuid, target: &str) {
        "DELETE FROM builder_targets
        WHERE builder = (SELECT id FROM builders WHERE uuid = $1)
        AND target = (SELECT id FROM targets WHERE name = $2)"
    }

    /// Request a job for a builder.
    fn builder_request(builder: Uuid, target: &str) {
        "SELECT 1"
    }

    /// Create a new target
    fn target_add(name: &str) {
        "INSERT INTO targets(name) VALUES ($1)
        ON CONFLICT DO NOTHING"
    }

    /// Remove a target
    fn target_remove(name: &str) {
        "DELETE FROM targets
        WHERE name = $1"
    }

    /// Set target enabled or disabled
    fn target_enabled(name: &str, enabled: bool) {
        "UPDATE targets
        SET enabled = $2
        WHERE name = $1"
    }

    /// Set target enabled or disabled
    fn target_rename(target: &str, name: &str) {
        "UPDATE targets
        SET name = $2
        WHERE name = $1"
    }

    /// Add a crate to the database.
    fn crate_add(name: &str) {
        "INSERT INTO crates(name) VALUES ($1)
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

    let builder_targets = "
        SELECT target_name
        FROM builder_targets_view
        WHERE builder_uuid = $1
    ";

    let target_list = "
        SELECT name
        FROM targets
    ";

    let target_info = "
        SELECT *
        FROM targets
        WHERE name = $1
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

    let job_create = "
        INSERT INTO jobs(uuid, builder, target, crate_version, stage)
        VALUES (
            $3,
            (SELECT id FROM builders WHERE uuid = $1),
            (SELECT target FROM builder_targets_view
                WHERE builder_uuid = $1
                AND target_name = $2),
            (SELECT version_id FROM build_queue WHERE target_name = $2),
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
    pub statements: Arc<Statements>,
    /// Connection to database
    pub connection: T,
}

impl<T: GenericClient> Database<T> {
    pub async fn builder_lookup(&self, fingerprint: &str) -> Result<Uuid, Error> {
        let row = self
            .connection
            .query_one(&self.statements.builder_by_fingerprint, &[&fingerprint])
            .await?;
        Ok(row.try_get("uuid")?)
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

    pub async fn builder_targets(&self, builder: Uuid) -> Result<BTreeSet<String>, Error> {
        let rows = self
            .connection
            .query(&self.statements.builder_targets, &[&builder])
            .await?;
        rows.into_iter()
            .map(|row| row.try_get("target_name"))
            .collect()
    }

    pub async fn target_list(&self) -> Result<BTreeSet<String>, Error> {
        let rows = self
            .connection
            .query(&self.statements.target_list, &[])
            .await?;
        rows.into_iter().map(|row| row.try_get("name")).collect()
    }

    pub async fn target_info(&self, target: &str) -> Result<TargetInfo, Error> {
        let row = self
            .connection
            .query_one(&self.statements.target_info, &[&target])
            .await?;
        Ok(TargetInfo {
            name: row.try_get("name")?,
            enabled: row.try_get("enabled")?,
        })
    }

    pub async fn job_request(&self, builder: Uuid, target: &str) -> Result<Uuid, Error> {
        let row = self
            .connection
            .query_one(
                &self.statements.job_create,
                &[&builder, &target, &Uuid::new_v4()],
            )
            .await?;
        Ok(row.try_get("uuid")?)
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
            target: row.try_get("target_name")?,
        })
    }

    pub async fn job_list(
        &self,
        builder: Option<Uuid>,
        target: Option<&str>,
        active: Option<bool>,
    ) -> Result<Vec<Uuid>, Error> {
        todo!()
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
