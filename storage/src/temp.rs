use std::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
};

/// Boxed future to run cleanup tasks.
pub type Cleanup = Pin<Box<dyn Future<Output = ()>>>;

/// Temporary value with associated cleanup method.
pub struct Temporary<T> {
    /// Instance.
    pub value: T,
    /// Future used to clean up the resources.
    pub cleanup: Cleanup,
}

impl<T> Deref for Temporary<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Temporary<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Temporary<T> {
    /// Create new [`Temporary`] from value and cleanup method.
    pub fn new(value: T, cleanup: Cleanup) -> Self {
        Self { value, cleanup }
    }

    /// Drop value and run cleanup.
    pub async fn cleanup(self) {
        let Self { value, cleanup } = self;
        drop(value);
        cleanup.await;
    }
}
