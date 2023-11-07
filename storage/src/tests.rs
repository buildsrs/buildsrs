use super::*;
use std::{future::Future, pin::Pin};
pub use test_strategy::proptest;

/// Generic future used for cleanup tasks.
pub type Cleanup = Pin<Box<dyn Future<Output = ()>>>;

/// Run a closure with a temporary instance and run cleanup afterwards.
pub async fn with<
    S: Storage,
    O1: Future<Output = (S, Cleanup)>,
    F1: Fn() -> O1,
    O2: Future<Output = ()>,
    F2: FnOnce(S) -> O2,
>(
    function: F1,
    closure: F2,
) {
    let (storage, cleanup) = function().await;
    closure(storage).await;
    cleanup.await;
}
