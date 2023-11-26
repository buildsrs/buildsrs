use buildsrs_database::AnyMetadata;
use buildsrs_storage::AnyStorage;

/// Backend state.
///
/// This struct contains all shared state that is needed to implement the backend service.
#[derive(Clone, Debug)]
pub struct Backend {
    database: AnyMetadata,
    storage: AnyStorage,
}

impl Backend {
    /// Create new backend state from a database connection and storage instance.
    pub fn new(database: AnyMetadata, storage: AnyStorage) -> Self {
        Backend { database, storage }
    }

    /// Return a reference to the database.
    pub fn database(&self) -> &AnyMetadata {
        &self.database
    }

    /// Return a reference to the storage.
    pub fn storage(&self) -> &AnyStorage {
        &self.storage
    }
}
