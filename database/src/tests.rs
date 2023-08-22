use crate::{temp::TempDatabase, Database};
use std::future::Future;

async fn with_database<O: Future<Output = ()>, F: FnOnce(Database) -> O>(f: F) {
    let host = std::env::var("DATABASE").expect("DATABASE env var must be present to run tests");
    let (temp_database, database) = TempDatabase::create(&host).await.unwrap();
    f(database).await;
    temp_database.delete().await.unwrap();
}

#[tokio::test]
async fn test_statements() {
    with_database(|_database: Database| async move {}).await;
}
