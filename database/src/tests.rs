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
async fn can_add_crate() {
    with_database(|database: Database| async move {
        let name = "serde";
        database.crate_add(name).await.unwrap();
        let info = database.crate_info(name).await.unwrap();
        assert_eq!(info.name, name);
        assert_eq!(info.enabled, true);
    })
    .await;
}

#[tokio::test]
async fn can_add_crate_version() {
    with_database(|database: Database| async move {
        let name = "serde";
        let version = "0.1.0";
        database.crate_add(name).await.unwrap();
        database.crate_version_add(name, version, "abcdef", false).await.unwrap();
        let info = database.crate_version_info(name, version).await.unwrap();
        assert_eq!(info.name, name);
        assert_eq!(info.version, version);
        assert_eq!(info.checksum, "abcdef");
        assert_eq!(info.yanked, false);
    })
    .await;
}

#[tokio::test]
async fn can_yank_crate_version() {
    with_database(|database: Database| async move {
        let name = "serde";
        let version = "0.1.0";
        database.crate_add(name).await.unwrap();
        database.crate_version_add(name, version, "abcdef", false).await.unwrap();

        for yanked in [true, false] {
            database.crate_version_add(name, version, "abcdef", yanked).await.unwrap();
            let info = database.crate_version_info(name, version).await.unwrap();
            assert_eq!(info.yanked, yanked);
        }
    })
    .await;
}

#[tokio::test]
async fn can_add_builder() {
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

        // get builder
        let builder = database.builder_get(uuid).await.unwrap();
        assert_eq!(builder.uuid, uuid);
        assert_eq!(&builder.public_key, private_key.public_key());
        assert_eq!(builder.comment, "comment");
    })
    .await;
}

#[tokio::test]
async fn can_lookup_builder() {
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
    })
    .await;
}

#[tokio::test]
async fn can_set_builder_comment() {
    with_database(|mut database: Database| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(uuid, &private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        for comment in ["this", "that", "other"] {
            // set comment
            database.builder_set_comment(uuid, comment).await.unwrap();

            // check comment
            let builder = database.builder_get(uuid).await.unwrap();
            assert_eq!(builder.comment, comment);
        }
    })
    .await;
}

#[tokio::test]
async fn can_set_builder_enabled() {
    with_database(|mut database: Database| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(uuid, &private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        for enabled in [false, true] {
            // set comment
            database.builder_set_enabled(uuid, enabled).await.unwrap();

            // check comment
            let builder = database.builder_get(uuid).await.unwrap();
            assert_eq!(builder.enabled, enabled);
        }
    })
    .await;
}

#[tokio::test]
async fn can_add_builder_target() {
    with_database(|mut database: Database| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();
        let target = "x86_64-unknown-unknown";

        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(uuid, &private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        database.target_add(target).await.unwrap();
        database.builder_target_add(uuid, target).await.unwrap();

        let targets = database.builder_targets(uuid).await.unwrap();
        assert!(targets.contains(target));
    })
    .await;
}

#[tokio::test]
async fn can_remove_builder_target() {
    with_database(|mut database: Database| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();
        let target = "x86_64-unknown-unknown";

        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(uuid, &private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        database.target_add(target).await.unwrap();
        database.builder_target_add(uuid, target).await.unwrap();

        let targets = database.builder_targets(uuid).await.unwrap();
        assert!(targets.contains(target));

        database.builder_target_remove(uuid, target).await.unwrap();

        let targets = database.builder_targets(uuid).await.unwrap();
        assert!(!targets.contains(target));
    })
    .await;
}

#[tokio::test]
async fn can_list_builder_target() {
    with_database(|mut database: Database| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(uuid, &private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        // defaults to empty
        let targets = database.builder_targets(uuid).await.unwrap();
        assert!(targets.is_empty());

        let targets = ["x86_64-unknown-unknown", "arm64-unknown-unknown"];

        for target in targets {
            database.target_add(target).await.unwrap();
            database.builder_target_add(uuid, target).await.unwrap();
            let targets = database.builder_targets(uuid).await.unwrap();
            assert!(targets.contains(target));
        }
    })
    .await;
}

#[tokio::test]
async fn can_target_add() {
    with_database(|mut database: Database| async move {
        let target = "x86_64-unknown-unknown";
        database.target_add(target).await.unwrap();
        let info = database.target_info(target).await.unwrap();
        assert_eq!(info.name, target);
        assert_eq!(info.enabled, false);
    })
    .await;
}

#[tokio::test]
async fn can_target_set_enabled() {
    with_database(|mut database: Database| async move {
        let target = "x86_64-unknown-unknown";
        database.target_add(target).await.unwrap();
        let info = database.target_info(target).await.unwrap();
        assert_eq!(info.enabled, false);
        for enabled in [true, false] {
            database.target_enabled(target, enabled).await.unwrap();
            let info = database.target_info(target).await.unwrap();
            assert_eq!(info.enabled, enabled);
        }
    })
    .await;
}

#[tokio::test]
async fn can_target_list() {
    with_database(|mut database: Database| async move {
        let targets = ["x86_64-unknown-unknown", "arm64-unknown-musl"];

        // add targets
        for target in targets {
            database.target_add(target).await.unwrap();
        }

        // make sure they are all there
        let list = database.target_list().await.unwrap();
        for target in targets {
            assert!(list.contains(target));
        }
    })
    .await;
}

#[tokio::test]
async fn can_target_remove() {
    with_database(|mut database: Database| async move {
        let targets = ["x86_64-unknown-unknown", "arm64-unknown-musl"];
        for target in &targets {
            database.target_add(target).await.unwrap();
        }

        database.target_remove(targets[0]).await.unwrap();
        let list = database.target_list().await.unwrap();
        assert!(!list.contains(targets[0]));
    })
    .await;
}
