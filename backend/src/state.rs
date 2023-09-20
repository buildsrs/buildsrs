use super::Options;
use anyhow::Result;
use apply::Apply;
use buildsrs_database::Database;
use std::{ops::Deref, sync::Arc};

#[derive(Debug)]
pub struct Shared {
    database: Database,
}

#[derive(Clone, Debug)]
pub struct Backend {
    shared: Arc<Shared>,
}

impl Deref for Backend {
    type Target = Shared;

    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}

impl Backend {
    pub async fn new(options: &Options) -> Result<Self> {
        Backend {
            shared: Shared {
                database: Database::connect(&options.database).await?,
            }
            .apply(Arc::new),
        }
        .apply(Ok)
    }

    pub fn database(&self) -> &Database {
        &self.shared.database
    }
}
