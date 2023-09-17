use crate::{temp::TempDatabase, Database};
use rand_core::OsRng;
use ssh_key::{Algorithm, HashAlg, PrivateKey};
use std::future::Future;
use uuid::Uuid;

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

async fn with_database_from_dump<O: Future<Output = ()>, F: FnOnce(Database) -> O>(
    dump: &str,
    f: F,
) {
    let host = std::env::var("DATABASE").expect("DATABASE env var must be present to run tests");
    let (temp_database, database) = TempDatabase::create(&host, Some(dump)).await.unwrap();
    f(database).await;
    temp_database.delete().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_dump_2023_09_17() {
    let dump = decompress(include_bytes!("../dumps/2023-09-17.sql.xz"));
    let dump = std::str::from_utf8(&dump[..]).unwrap();
    with_database_from_dump(&dump[..], |database: Database| async move {}).await;
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

#[tokio::test]
async fn test_builders() {
    with_database(|mut database: Database| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        // make sure we can add a builder
        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(uuid, &private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        // make sure we can look it up
        for alg in [HashAlg::Sha256, HashAlg::Sha512] {
            assert_eq!(
                database
                    .builder_lookup(&private_key.public_key().fingerprint(alg).to_string())
                    .await
                    .unwrap(),
                uuid
            );
        }

        let builder = database.builder_get(uuid).await.unwrap();
        assert_eq!(builder.uuid, uuid);
        assert_eq!(builder.public_key, private_key.public_key().to_openssh().unwrap());
        assert_eq!(builder.comment, "comment");
    })
    .await;
}
