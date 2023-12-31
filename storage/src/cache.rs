//! # Storage Cache
//!
//! This implements a layer that can be used with any storage provider which caches responses for a
//! specific amount of time.

use super::*;
use moka::{future::Cache as MokaCache, Expiry};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[cfg(feature = "options")]
mod options;

#[cfg(feature = "options")]
pub(crate) use options::CacheOptions;

/// Cached error.
///
/// This type adds context to the error it contains to communicate to users that this error may
/// have been cached.
#[derive(Debug, thiserror::Error)]
#[error("cached error")]
struct CachedError(#[from] SharedError);

/// Cache entry for one crate lookup.
#[derive(Clone, Debug)]
enum Entry {
    /// Crate is missing.
    Missing(Arc<CachedError>),
    /// Crate exists,
    Data(ArtifactData),
}

impl Entry {
    /// Determine the weight of this entry.
    fn weight(&self) -> usize {
        match self {
            Self::Data(ArtifactData::Data { bytes }) => bytes.len(),
            _ => 1,
        }
    }
}

/// Configuration for storage [`Cache`].
///
/// This allows you to override the behaviour of the cache. In general, you should use the
/// [`CacheConfig::default()`] implementation to create a default configuration. The value
/// that you should consider tweaking is the `capacity`, which you should set to however much
/// memory you are willing to throw at the cache.
#[derive(Clone, Copy, Debug)]
pub struct CacheConfig {
    /// Capacity of cache, in bytes.
    ///
    /// You should set this to however much memory you are willing to use for the cache. In
    /// general, the higher you set this to, the better. The default is set to 16MB.
    pub capacity: u64,

    /// Timeout for missing crate entries.
    ///
    /// Packages are immutable once published, so we can cache them forever. However,
    /// we are also caching negative lookup results, but these can change as artifacts are
    /// published. For that reason, negative lookups have a dedicated cache duration that should be
    /// set to a low value.
    pub timeout_missing: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capacity: 16 * 1024 * 1024,
            timeout_missing: Duration::from_secs(60),
        }
    }
}

impl Expiry<ArtifactId, Entry> for CacheConfig {
    fn expire_after_create(
        &self,
        _key: &ArtifactId,
        value: &Entry,
        _created_at: Instant,
    ) -> Option<Duration> {
        match value {
            Entry::Missing(_) => Some(self.timeout_missing),
            Entry::Data(ArtifactData::Redirect { validity, .. }) => Some(*validity),
            Entry::Data(_) => None,
        }
    }
}

/// Storage caching layer.
///
/// This is a layer you can use to wrap an existing storage provider to add an in-memory cache.
/// This allows you to serve commonly requested artifacts more efficiently.
///
/// The cache is implemented using the moka crate, which is optimized for highly concurrent,
/// lock-free access.
#[derive(Clone, Debug)]
pub struct Cache {
    /// Underlying storage implementation
    storage: AnyStorage,
    /// Cache used for artifact sources
    cache: MokaCache<ArtifactId, Entry>,
}

impl Cache {
    /// Create new caching layer on top of a storage.
    ///
    /// You need to create a [`CacheConfig`] to create the cache, which specifies some important
    /// metrics such as the capacity of the cache.
    /// You can use [`CacheConfig::default()`] to use defaults, which should be sane. Read the
    /// documentation on [`CacheConfig`] for more information on what can be tuned.
    pub fn new(storage: AnyStorage, config: CacheConfig) -> Self {
        // we use a custom weigher to ensure that entries are weighed by their size in bytes.
        // unfortunately, the weigher only supports u32 values, so when our entry is too big (more
        // than 4GB) we will fall back to using the maximum value.
        let cache = MokaCache::builder()
            .weigher(|_key, value: &Entry| -> u32 { value.weight().try_into().unwrap_or(u32::MAX) })
            .max_capacity(config.capacity)
            .expire_after(config)
            .build();

        Self { storage, cache }
    }

    /// Get a reference to the underlying storage.
    pub fn storage(&self) -> &AnyStorage {
        &self.storage
    }

    /// Clear the cache.
    ///
    /// This will invalidate all cache entries.
    pub fn clear(&self) {
        self.cache.invalidate_all();
    }
}

#[async_trait::async_trait]
impl Storage for Cache {
    async fn artifact_put(&self, version: &ArtifactId, data: &[u8]) -> Result<(), StorageError> {
        // we cannot cache mutable operations.
        self.storage().artifact_put(version, data).await
    }

    async fn artifact_get(&self, version: &ArtifactId) -> Result<ArtifactData, StorageError> {
        let storage = self.storage.clone();
        // using try_get_with here ensures that if we concurrently request the same artifact version
        // twice, only one lookup will be made.
        let result = self
            .cache
            .try_get_with(version.clone(), async move {
                match storage.artifact_get(version).await {
                    Ok(bytes) => Ok(Entry::Data(bytes)),
                    Err(StorageError::NotFound(error)) => {
                        // we save the error, but wrap it in a CachedError, to communicate to
                        // the caller that this error may have been cached.
                        Ok(Entry::Missing(Arc::new(CachedError(error))))
                    }
                    Err(error) => Err(error),
                }
            })
            .await;

        // depending on what the entry is, we construct the right response.
        match result {
            Ok(Entry::Data(bytes)) => Ok(bytes),
            Ok(Entry::Missing(error)) => Err(StorageError::NotFound(error)),
            Err(error) => Err((*error).clone()),
        }
    }
}
