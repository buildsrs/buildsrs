use buildsrs_database::Database;
use buildsrs_storage::AnyStorage;
use std::sync::Arc;

/// Backend state.
///
/// This struct contains all shared state that is needed to implement the backend service.
#[derive(Clone, Debug)]
pub struct Backend {
    database: Arc<Database>,
    storage: AnyStorage,
}

impl Backend {
    /// Create new backend state from a database connection and storage instance.
    pub fn new(database: Arc<Database>, storage: AnyStorage) -> Self {
        Backend { database, storage }
    }

    /// Return a reference to the database.
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Return a reference to the storage.
    pub fn storage(&self) -> &AnyStorage {
        &self.storage
    }
}
