use super::{Database, DatabaseConnection, Pool};
use rand::{thread_rng, Rng};
use tokio::task::JoinHandle;
use tokio_postgres::{connect, Client, Error, NoTls};

/// Generate a random sequence suitable for use as a Postgres database name.
fn random_database_name(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length).map(|_| rng.gen_range('a'..='z')).collect()
}

/// Temporary database handle.
///
/// This is used to generate a temporary database during testing.
pub struct TempDatabase {
    database_name: String,
    inner_host: String,
    outer_client: Client,
    outer_handle: JoinHandle<Result<(), Error>>,
    pool: Pool,
}

impl TempDatabase {
    /// Connection string of temporary database.
    pub fn database_string(&self) -> &str {
        &self.inner_host
    }

    /// Create new temporary database.
    pub async fn create(database: &str, dump: Option<&str>) -> Result<Self, Error> {
        // connect to database
        let (outer_client, connection) = connect(database, NoTls).await?;
        let outer_handle = tokio::spawn(connection);

        // create new, empty, random database
        let database_name = format!("test_{}", random_database_name(15));
        println!("=> Creating database {database_name:?}");
        println!("=> Run `just database-repl {database_name:?}` to inspect database");
        outer_client
            .execute(&format!("CREATE DATABASE {database_name}"), &[])
            .await?;

        // connect to new, empty database
        let inner_host = format!("{database} dbname={database_name}");
        let (mut inner_client, inner_connection) = connect(&inner_host, NoTls).await.unwrap();
        let inner_handle = tokio::spawn(inner_connection);

        if let Some(dump) = dump {
            inner_client.batch_execute(dump).await.unwrap();
            // https://dba.stackexchange.com/questions/106057/error-no-schema-has-been-selected-to-create-in
            inner_client
                .execute(
                    "SELECT pg_catalog.set_config('search_path', 'public', false);",
                    &[],
                )
                .await
                .unwrap();
        }

        crate::migrations::runner()
            .run_async(&mut inner_client)
            .await
            .unwrap();
        let database = Database::new(inner_client).await.unwrap();
        let database = DatabaseConnection::new(database, Some(inner_handle));
        let pool = Pool::from(database);

        //let pool = Pool::new(&inner_host, 1).await?;

        Ok(TempDatabase {
            database_name,
            inner_host,
            outer_client,
            outer_handle,
            pool,
        })
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Delete temporary database.
    pub async fn delete(self) -> Result<(), Error> {
        let Self {
            database_name,
            outer_client,
            outer_handle,
            pool,
            ..
        } = self;
        pool.close().await;

        // drop database
        outer_client
            .execute(&format!("DROP DATABASE {database_name}"), &[])
            .await?;
        drop(outer_client);
        outer_handle.await.unwrap().unwrap();

        Ok(())
    }
}
