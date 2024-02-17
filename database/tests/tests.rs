use buildsrs_database::{AnyMetadata, TempDatabase};
use std::{future::Future, sync::Arc};
use test_strategy::*;

const NUM_CASES: u32 = 10;

async fn with_database<O: Future<Output = ()>, F: Fn(AnyMetadata) -> O>(f: F) {
    let host = std::env::var("DATABASE").expect("DATABASE env var must be present to run tests");
    let temp_database = TempDatabase::create(&host, None).await.unwrap();
    f(Arc::new(temp_database.pool().clone())).await;
    temp_database.delete().await.unwrap();
}

#[proptest(async = "tokio", cases = NUM_CASES)]
async fn can_add_crate(name: String) {
    let name = name.as_str();
    with_database(|metadata| async move {
        // add crate
        let writer = metadata.write().await.unwrap();
        writer.crate_add(&name).await.unwrap();
        writer.commit().await.unwrap();

        // verify presence
        let reader = metadata.read().await.unwrap();
        let info = reader.crate_info(&name).await.unwrap();
        assert_eq!(info.name, name);
        assert!(info.enabled);
    })
    .await;
}

#[proptest(async = "tokio", cases = NUM_CASES)]
async fn can_add_crates(names: Vec<String>) {
    let names = &names;
    with_database(|metadata| async move {
        // add crate
        let writer = metadata.write().await.unwrap();
        for name in names.iter() {
            writer.crate_add(name).await.unwrap();
        }
        writer.commit().await.unwrap();

        // verify presence
        let reader = metadata.read().await.unwrap();
        for name in names.iter() {
            let info = reader.crate_info(&name).await.unwrap();
            assert_eq!(&info.name, name);
            assert!(info.enabled);
        }
    })
    .await;
}

#[proptest(async = "tokio", cases = NUM_CASES)]
async fn can_add_crate_version(name: String, version: String, checksum: String, yanked: bool) {
    let name = name.as_str();
    let version = version.as_str();
    let checksum = checksum.as_str();
    with_database(|pool| async move {
        let writer = pool.write().await.unwrap();
        writer.crate_add(name).await.unwrap();
        writer
            .crate_version_add(name, version, checksum, yanked)
            .await
            .unwrap();
        writer.commit().await.unwrap();

        let reader = pool.read().await.unwrap();
        let info = reader.crate_version_info(name, version).await.unwrap();

        assert_eq!(info.name, name);
        assert_eq!(info.version, version);
        assert_eq!(info.checksum, checksum);
        assert_eq!(info.yanked, yanked);
    })
    .await;
}
