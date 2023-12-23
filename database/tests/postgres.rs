use buildsrs_database::{
    entity::{ArtifactKind, Task},
    Pool, TempDatabase,
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

async fn with_database<O: Future<Output = ()>, F: FnOnce(Pool) -> O>(f: F) {
    let host = std::env::var("DATABASE").expect("DATABASE env var must be present to run tests");
    let temp_database = TempDatabase::create(&host, None).await.unwrap();
    f(temp_database.pool().clone()).await;
    temp_database.delete().await.unwrap();
}

async fn with_database_from_dump<O: Future<Output = ()>, F: FnOnce(Pool) -> O>(dump: &str, f: F) {
    let host = std::env::var("DATABASE").expect("DATABASE env var must be present to run tests");
    let temp_database = TempDatabase::create(&host, Some(dump)).await.unwrap();
    f(temp_database.pool().clone()).await;
    temp_database.delete().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_dump_2023_09_17() {
    let dump = decompress(include_bytes!("../dumps/2023-09-17.sql.xz"));
    let dump = std::str::from_utf8(&dump[..]).unwrap();
    with_database_from_dump(dump, |_pool: Pool| async move {}).await;
}

#[tokio::test]
async fn test_statements() {
    with_database(|_pool: Pool| async move {}).await;
}

#[tokio::test]
async fn can_add_crate() {
    with_database(|pool: Pool| async move {
        let name = "serde";
        let writer = pool.write().await.unwrap();
        writer.crate_add(name).await.unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();

        let info = reader.crate_info(name).await.unwrap();
        assert_eq!(info.name, name);
        assert!(info.enabled);
    })
    .await;
}

#[tokio::test]
async fn can_add_crate_version() {
    with_database(|pool: Pool| async move {
        let name = "serde";
        let version = "0.1.0";

        let writer = pool.write().await.unwrap();
        writer.crate_add(name).await.unwrap();
        writer
            .crate_version_add(name, version, "abcdef", false)
            .await
            .unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();
        let info = reader.crate_version_info(name, version).await.unwrap();

        assert_eq!(info.name, name);
        assert_eq!(info.version, version);
        assert_eq!(info.checksum, "abcdef");
        assert!(!info.yanked);
    })
    .await;
}

#[tokio::test]
async fn can_add_crate_version_task() {
    with_database(|pool: Pool| async move {
        let name = "serde";
        let version = "0.1.0";

        let writer = pool.write().await.unwrap();
        writer.crate_add(name).await.unwrap();
        writer
            .crate_version_add(name, version, "abcdef", false)
            .await
            .unwrap();
        writer
            .tasks_create_all("metadata", "generic")
            .await
            .unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();
        let tasks = reader.task_list(None, None, None, None).await.unwrap();
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
    with_database(|pool: Pool| async move {
        let name = "serde";
        let version = "0.1.0";

        let writer = pool.write().await.unwrap();
        writer.crate_add(name).await.unwrap();
        writer
            .crate_version_add(name, version, "abcdef", false)
            .await
            .unwrap();
        writer.commit().await.unwrap();

        for yanked in [true, false] {
            let writer = pool.write().await.unwrap();
            writer
                .crate_version_add(name, version, "abcdef", yanked)
                .await
                .unwrap();
            writer.commit().await.unwrap();

            let reader = pool.read().await.unwrap();
            let info = reader.crate_version_info(name, version).await.unwrap();

            assert_eq!(info.yanked, yanked);
        }
    })
    .await;
}

#[tokio::test]
async fn can_add_builder() {
    with_database(|pool: Pool| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        let writer = pool.write().await.unwrap();

        // make sure we can add a builder
        writer
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();

        // get builder
        let builder = reader.builder_get(uuid).await.unwrap();
        assert_eq!(builder.uuid, uuid);
        assert_eq!(&builder.public_key, private_key.public_key());
        assert_eq!(builder.comment, "comment");
    })
    .await;
}

#[tokio::test]
async fn can_lookup_builder() {
    with_database(|pool: Pool| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        // make sure we can add a builder
        let writer = pool.write().await.unwrap();
        writer
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();
        // make sure we can look it up
        for alg in [HashAlg::Sha256, HashAlg::Sha512] {
            assert_eq!(
                reader
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
    with_database(|pool: Pool| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        let writer = pool.write().await.unwrap();
        writer
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();
        writer.commit().await.unwrap();

        for comment in ["this", "that", "other"] {
            let writer = pool.write().await.unwrap();
            // set comment
            writer.builder_set_comment(uuid, comment).await.unwrap();
            writer.commit().await.unwrap();

            // check comment
            let reader = pool.read().await.unwrap();
            let builder = reader.builder_get(uuid).await.unwrap();
            assert_eq!(builder.comment, comment);
        }
    })
    .await;
}

#[tokio::test]
async fn can_set_builder_enabled() {
    with_database(|pool: Pool| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        let writer = pool.write().await.unwrap();
        writer
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();
        writer.commit().await.unwrap();

        for enabled in [false, true] {
            // set comment
            let writer = pool.write().await.unwrap();
            writer.builder_set_enabled(uuid, enabled).await.unwrap();
            writer.commit().await.unwrap();

            // check comment
            let reader = pool.read().await.unwrap();
            let builder = reader.builder_get(uuid).await.unwrap();
            assert_eq!(builder.enabled, enabled);
        }
    })
    .await;
}

#[tokio::test]
async fn can_add_builder_triple() {
    with_database(|pool: Pool| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();
        let triple = "x86_64-unknown-unknown";

        let writer = pool.write().await.unwrap();
        writer
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();
        writer.triple_add(triple).await.unwrap();
        writer.builder_triple_add(uuid, triple).await.unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();

        let triples = reader.builder_triples(uuid).await.unwrap();
        assert!(triples.contains(triple));
    })
    .await;
}

#[tokio::test]
async fn can_remove_builder_triple() {
    with_database(|pool: Pool| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();
        let triple = "x86_64-unknown-unknown";

        let writer = pool.write().await.unwrap();
        writer
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();

        writer.triple_add(triple).await.unwrap();
        writer.builder_triple_add(uuid, triple).await.unwrap();

        //writer.commit().await.unwrap();

        let triples = writer.builder_triples(uuid).await.unwrap();
        assert!(triples.contains(triple));

        writer.builder_triple_remove(uuid, triple).await.unwrap();

        let triples = writer.builder_triples(uuid).await.unwrap();
        assert!(!triples.contains(triple));
    })
    .await;
}

#[tokio::test]
async fn can_list_builder_triple() {
    with_database(|pool: Pool| async move {
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let uuid = Uuid::new_v4();

        let writer = pool.write().await.unwrap();
        writer
            .builder_add(uuid, private_key.public_key(), "comment")
            .await
            .unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();

        // defaults to empty
        let triples = reader.builder_triples(uuid).await.unwrap();
        assert!(triples.is_empty());

        drop(reader);

        let triples = ["x86_64-unknown-unknown", "arm64-unknown-unknown"];

        for triple in triples {
            let writer = pool.write().await.unwrap();
            writer.triple_add(triple).await.unwrap();
            writer.builder_triple_add(uuid, triple).await.unwrap();
            writer.commit().await.unwrap();

            let reader = pool.read().await.unwrap();
            let triples = reader.builder_triples(uuid).await.unwrap();
            assert!(triples.contains(triple));
        }
    })
    .await;
}

#[tokio::test]
async fn can_triple_add() {
    with_database(|pool: Pool| async move {
        let writer = pool.write().await.unwrap();
        let triple = "x86_64-unknown-unknown";
        writer.triple_add(triple).await.unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();
        let info = reader.triple_info(triple).await.unwrap();
        assert_eq!(info.name, triple);
        assert!(!info.enabled);
    })
    .await;
}

#[tokio::test]
async fn can_triple_set_enabled() {
    with_database(|pool: Pool| async move {
        let triple = "x86_64-unknown-unknown";
        let writer = pool.write().await.unwrap();
        writer.triple_add(triple).await.unwrap();
        let info = writer.triple_info(triple).await.unwrap();
        assert!(!info.enabled);
        for enabled in [true, false] {
            writer.triple_enabled(triple, enabled).await.unwrap();
            let info = writer.triple_info(triple).await.unwrap();
            assert_eq!(info.enabled, enabled);
        }
    })
    .await;
}

#[tokio::test]
async fn can_triple_list() {
    with_database(|pool: Pool| async move {
        let triples = ["x86_64-unknown-unknown", "arm64-unknown-musl"];

        let writer = pool.write().await.unwrap();

        // add triples
        for triple in triples {
            writer.triple_add(triple).await.unwrap();
        }

        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();

        // make sure they are all there
        let list = reader.triple_list().await.unwrap();
        for triple in triples {
            assert!(list.contains(triple));
        }
    })
    .await;
}

#[tokio::test]
async fn can_triple_remove() {
    with_database(|pool: Pool| async move {
        let triples = ["x86_64-unknown-unknown", "arm64-unknown-musl"];

        let writer = pool.write().await.unwrap();
        for triple in &triples {
            writer.triple_add(triple).await.unwrap();
        }

        writer.triple_remove(triples[0]).await.unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();
        let list = reader.triple_list().await.unwrap();
        assert!(!list.contains(triples[0]));
    })
    .await;
}

#[tokio::test]
async fn can_triple_rename() {
    with_database(|pool: Pool| async move {
        let triple_original = "x86_64-unknown";
        let triple_renamed = "x86_64-unknown-unknown";

        let writer = pool.write().await.unwrap();
        writer.triple_add(triple_original).await.unwrap();
        writer
            .triple_rename(triple_original, triple_renamed)
            .await
            .unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();
        let info = reader.triple_info(triple_renamed).await.unwrap();
        assert_eq!(info.name, triple_renamed);
    })
    .await;
}

#[tokio::test]
async fn can_job_create() {
    with_database(|pool: Pool| async move {
        let writer = pool.write().await.unwrap();

        // add triple
        let triple = "x86_64-unknown-unknown";
        writer.triple_add(triple).await.unwrap();

        // add crate and version
        let name = "serde";
        let version = "0.1.0";
        writer.crate_add(name).await.unwrap();
        writer
            .crate_version_add(name, version, "abcdef", false)
            .await
            .unwrap();

        // add builder
        let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let builder = Uuid::new_v4();
        writer
            .builder_add(builder, private_key.public_key(), "comment")
            .await
            .unwrap();
        writer.builder_triple_add(builder, triple).await.unwrap();

        writer.tasks_create_all("metadata", triple).await.unwrap();

        // add job
        let job = writer.job_request(builder, triple).await.unwrap();

        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();
        // get job info
        let info = reader.job_info(job).await.unwrap();

        assert_eq!(info.builder, builder);
        assert_eq!(info.triple, triple);
        assert_eq!(info.name, name);
        assert_eq!(info.version, version);
    })
    .await;
}
