use buildsrs_storage::*;
use std::{future::Future, sync::Arc, time::Duration};
use test_strategy::proptest;

#[cfg(feature = "filesystem")]
mod filesystem;
#[cfg(feature = "s3")]
mod s3;

/// Run a closure with a temporary instance and run cleanup afterwards.
pub async fn with<
    S: Storage,
    O1: Future<Output = Temporary<S>>,
    F1: Fn() -> O1,
    O2: Future<Output = ()>,
    F2: FnOnce(S) -> O2,
>(
    function: F1,
    closure: F2,
) {
    let storage = function().await;
    closure(storage.value).await;
    storage.cleanup.await;
}

#[cfg(feature = "cache")]
const TEST_CACHE_CONFIG: CacheConfig = CacheConfig {
    timeout_missing: Duration::from_secs(0),
    capacity: 16 * 1024 * 1024,
};

async fn create_temp_instances<
    S: Storage + 'static,
    O: Future<Output = Temporary<S>>,
    F: Fn() -> O,
>(
    storages: &mut Vec<AnyStorage>,
    cleanups: &mut Vec<Cleanup>,
    function: F,
) {
    // create instance
    let storage = function().await;
    storages.push(Arc::new(storage.value));
    cleanups.push(storage.cleanup);

    // create cached instance, if feature is enabled.
    #[cfg(feature = "cache")]
    {
        let storage = function().await;
        storages.push(Arc::new(Cache::new(
            Arc::new(storage.value),
            TEST_CACHE_CONFIG,
        )));
        cleanups.push(storage.cleanup);
    }
}

async fn temp_instances() -> (Vec<AnyStorage>, Cleanup) {
    let mut storage: Vec<AnyStorage> = vec![];
    let mut cleanup: Vec<Cleanup> = vec![];

    #[cfg(feature = "filesystem")]
    create_temp_instances(&mut storage, &mut cleanup, Filesystem::new_temp).await;

    #[cfg(feature = "s3")]
    create_temp_instances(&mut storage, &mut cleanup, S3::new_temp).await;

    let cleanup = Box::pin(async move {
        for c in cleanup.into_iter() {
            c.await;
        }
    });

    (storage, cleanup)
}

#[proptest(async = "tokio", cases = 10)]
async fn can_artifact_put(version: ArtifactId, contents: Vec<u8>) {
    let (instances, cleanup) = temp_instances().await;

    for storage in instances {
        // does not exist at first
        let result = storage.artifact_get(&version).await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));

        // we write it
        storage.artifact_put(&version, &contents).await.unwrap();

        // now it exists
        storage.artifact_get(&version).await.unwrap();
    }

    cleanup.await;
}

#[proptest(async = "tokio", cases = 10)]
async fn can_package_put_many(packages: Vec<(ArtifactId, Vec<u8>)>) {
    let (instances, cleanup) = temp_instances().await;

    for storage in instances {
        println!("Testing {storage:?}");

        for (version, bytes) in &packages {
            storage.artifact_put(&version, &bytes).await.unwrap();

            let result = storage.artifact_get(&version).await.unwrap();
            assert_eq!(result.bytes().unwrap(), bytes);
        }
    }

    cleanup.await;
}
