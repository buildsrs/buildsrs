use crate::{temp::TempDatabase, Database};
use std::future::Future;

fn decompress(mut data: &[u8]) -> Vec<u8> {
    let mut output = vec![];
    lzma_rs::xz_decompress(&mut data, &mut output).unwrap();
    output
}

async fn with_database<O: Future<Output = ()>, F: FnOnce(Database) -> O>(f: F) {
    let host = std::env::var("DATABASE").expect("DATABASE env var must be present to run tests");
    let (temp_database, database) = TempDatabase::create(&host, None).await.unwrap();
    f(database).await;
    temp_database.delete().await.unwrap();
}

async fn with_database_from_dump<O: Future<Output = ()>, F: FnOnce(Database) -> O>(dump: &str, f: F) {
    let host = std::env::var("DATABASE").expect("DATABASE env var must be present to run tests");
    let (temp_database, database) = TempDatabase::create(&host, Some(dump)).await.unwrap();
    f(database).await;
    temp_database.delete().await.unwrap();
}

#[tokio::test]
async fn test_dump_2023_09_13() {
    let dump = decompress(include_bytes!("../dumps/2023-09-13.sql.xz"));
    let dump = std::str::from_utf8(&dump[..]).unwrap();
    with_database_from_dump(&dump[..], |database: Database| async move {
    })
    .await;
}

#[tokio::test]
async fn test_statements() {
    with_database(|_database: Database| async move {}).await;
}

#[tokio::test]
async fn test_registry() {
    with_database(|database: Database| async move {
        database.crate_add("serde").await.unwrap();
    })
    .await;
}
