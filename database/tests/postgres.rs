use buildsrs_database::{
    entity::{ArtifactKind, Task},
    Database, TempDatabase,
};
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
    with_database_from_dump(dump, |_database: Database| async move {}).await;
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
        assert!(info.enabled);
    })
    .await;
}

#[tokio::test]
async fn can_add_crate_version() {
    with_database(|database: Database| async move {
        let name = "serde";
        let version = "0.1.0";
        database.crate_add(name).await.unwrap();
        database
            .crate_version_add(name, version, "abcdef", false)
            .await
            .unwrap();
        let info = database.crate_version_info(name, version).await.unwrap();
        assert_eq!(info.name, name);
        assert_eq!(info.version, version);
        assert_eq!(info.checksum, "abcdef");
        assert!(!info.yanked);
    })
    .await;
}

#[tokio::test]
async fn can_add_crate_version_task() {
    with_database(|database: Database| async move {
        let name = "serde";
        let version = "0.1.0";
        database.crate_add(name).await.unwrap();
        database
            .crate_version_add(name, version, "abcdef", false)
            .await
            .unwrap();
        database
            .tasks_create_all("metadata", "generic")
            .await
            .unwrap();
        let tasks = database.task_list(None, None, None, None).await.unwrap();
        assert_eq!(
            tasks,
            [Task {
                krate: name.into(),
                version: version.into(),
                kind: ArtifactKind::Metadata,
                triple: "generic".into(),
            }]
        );
    })
    .await;
}

#[tokio::test]
async fn can_yank_crate_version() {
    with_database(|database: Database| async move {
        let name = "serde";
        let version = "0.1.0";
        database.crate_add(name).await.unwrap();
        database
            .crate_version_add(name, version, "abcdef", false)
            .await
            .unwrap();

        for yanked in [true, false] {
            database
                .crate_version_add(name, version, "abcdef", yanked)
                .await
                .unwrap();
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
            .builder_add(uuid, private_key.public_key(), "comment")
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
            .builder_add(uuid, private_key.public_key(), "comment")
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
            .builder_add(uuid, private_key.public_key(), "comment")
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
            .builder_add(uuid, private_key.public_key(), "comment")
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
async fn can_add_builder_triple() {
    with_database(|mut database: Database| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();
        let triple = "x86_64-unknown-unknown";

        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        database.triple_add(triple).await.unwrap();
        database.builder_triple_add(uuid, triple).await.unwrap();

        let triples = database.builder_triples(uuid).await.unwrap();
        assert!(triples.contains(triple));
    })
    .await;
}

#[tokio::test]
async fn can_remove_builder_triple() {
    with_database(|mut database: Database| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();
        let triple = "x86_64-unknown-unknown";

        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        database.triple_add(triple).await.unwrap();
        database.builder_triple_add(uuid, triple).await.unwrap();

        let triples = database.builder_triples(uuid).await.unwrap();
        assert!(triples.contains(triple));

        database.builder_triple_remove(uuid, triple).await.unwrap();

        let triples = database.builder_triples(uuid).await.unwrap();
        assert!(!triples.contains(triple));
    })
    .await;
}

#[tokio::test]
async fn can_list_builder_triple() {
    with_database(|mut database: Database| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();

        // defaults to empty
        let triples = database.builder_triples(uuid).await.unwrap();
        assert!(triples.is_empty());

        let triples = ["x86_64-unknown-unknown", "arm64-unknown-unknown"];

        for triple in triples {
            database.triple_add(triple).await.unwrap();
            database.builder_triple_add(uuid, triple).await.unwrap();
            let triples = database.builder_triples(uuid).await.unwrap();
            assert!(triples.contains(triple));
        }
    })
    .await;
}

#[tokio::test]
async fn can_triple_add() {
    with_database(|database: Database| async move {
        let triple = "x86_64-unknown-unknown";
        database.triple_add(triple).await.unwrap();
        let info = database.triple_info(triple).await.unwrap();
        assert_eq!(info.name, triple);
        assert!(!info.enabled);
    })
    .await;
}

#[tokio::test]
async fn can_triple_set_enabled() {
    with_database(|database: Database| async move {
        let triple = "x86_64-unknown-unknown";
        database.triple_add(triple).await.unwrap();
        let info = database.triple_info(triple).await.unwrap();
        assert!(!info.enabled);
        for enabled in [true, false] {
            database.triple_enabled(triple, enabled).await.unwrap();
            let info = database.triple_info(triple).await.unwrap();
            assert_eq!(info.enabled, enabled);
        }
    })
    .await;
}

#[tokio::test]
async fn can_triple_list() {
    with_database(|database: Database| async move {
        let triples = ["x86_64-unknown-unknown", "arm64-unknown-musl"];

        // add triples
        for triple in triples {
            database.triple_add(triple).await.unwrap();
        }

        // make sure they are all there
        let list = database.triple_list().await.unwrap();
        for triple in triples {
            assert!(list.contains(triple));
        }
    })
    .await;
}

#[tokio::test]
async fn can_triple_remove() {
    with_database(|database: Database| async move {
        let triples = ["x86_64-unknown-unknown", "arm64-unknown-musl"];
        for triple in &triples {
            database.triple_add(triple).await.unwrap();
        }

        database.triple_remove(triples[0]).await.unwrap();
        let list = database.triple_list().await.unwrap();
        assert!(!list.contains(triples[0]));
    })
    .await;
}

#[tokio::test]
async fn can_triple_rename() {
    with_database(|database: Database| async move {
        let triple_original = "x86_64-unknown";
        let triple_renamed = "x86_64-unknown-unknown";
        database.triple_add(triple_original).await.unwrap();
        database
            .triple_rename(triple_original, triple_renamed)
            .await
            .unwrap();
        let info = database.triple_info(triple_renamed).await.unwrap();
        assert_eq!(info.name, triple_renamed);
    })
    .await;
}

#[tokio::test]
async fn can_job_create() {
    with_database(|mut database: Database| async move {
        // add triple
        let triple = "x86_64-unknown-unknown";
        database.triple_add(triple).await.unwrap();

        // add crate and version
        let name = "serde";
        let version = "0.1.0";
        database.crate_add(name).await.unwrap();
        database
            .crate_version_add(name, version, "abcdef", false)
            .await
            .unwrap();

        // add builder
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let builder = Uuid::new_v4();
        let transaction = database.transaction().await.unwrap();
        transaction
            .builder_add(builder, private_key.public_key(), "comment")
            .await
            .unwrap();
        transaction.commit().await.unwrap();
        database.builder_triple_add(builder, triple).await.unwrap();

        database.tasks_create_all("metadata", triple).await.unwrap();

        // add job
        let job = database.job_request(builder, triple).await.unwrap();

        // get job info
        let info = database.job_info(job).await.unwrap();

        assert_eq!(info.builder, builder);
        assert_eq!(info.triple, triple);
        assert_eq!(info.name, name);
        assert_eq!(info.version, version);
    })
    .await;
}
