use super::*;
use clap::Parser;

#[derive(Parser, Clone, Debug, PartialEq)]
pub struct CacheOptions {
    /// Enables storage cache
    #[clap(long, env)]
    pub storage_cache: bool,

    /// Storage cache capacity (in bytes)
    #[clap(long, requires("storage_cache"), env, default_value = "16000000")]
    pub storage_cache_capacity: u64,

    /// Timeout for package missing entries in the cache.
    #[clap(long, requires("storage_cache"), env, default_value = "60")]
    pub storage_cache_missing_timeout: u64,
}

impl CacheOptions {
    pub fn maybe_cache(&self, storage: AnyStorage) -> AnyStorage {
        if self.storage_cache {
            let config = CacheConfig {
                capacity: self.storage_cache_capacity,
                timeout_missing: Duration::from_secs(self.storage_cache_missing_timeout),
            };
            let cache = Cache::new(storage, config);
            Arc::new(cache)
        } else {
            storage
        }
    }
}
