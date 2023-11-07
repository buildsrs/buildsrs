use buildsrs_database::Database;
use buildsrs_storage::AnyStorage;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Backend {
    database: Arc<Database>,
    storage: AnyStorage,
}

impl Backend {
    pub fn new(database: Arc<Database>, storage: AnyStorage) -> Self {
        Backend { database, storage }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn storage(&self) -> &AnyStorage {
        &self.storage
    }
}
