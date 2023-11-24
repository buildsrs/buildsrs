use buildsrs_storage::{Storage, Temporary};
use std::future::Future;

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
