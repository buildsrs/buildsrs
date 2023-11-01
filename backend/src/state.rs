use super::Options;
use crate::bucket::{wasabi::WasabiBucket, BucketTraitObject};
use anyhow::Result;
use apply::Apply;
use buildsrs_database::Database;
use std::{ops::Deref, sync::Arc};

#[derive(Debug)]
pub struct Shared {
    database: Database,
    bucket: BucketTraitObject,
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
        let Options {
            database,
            bucket: bucket_options,
            ..
        } = options;

        Backend {
            shared: Shared {
                database: Database::connect(database).await?,
                bucket: Box::new(WasabiBucket::new(
                    &bucket_options.name,
                    bucket_options.into(),
                    &bucket_options.region,
                )?) as BucketTraitObject,
            }
            .apply(Arc::new),
        }
        .apply(Ok)
    }

    pub fn database(&self) -> &Database {
        &self.shared.database
    }

    pub fn bucket(&self) -> &BucketTraitObject {
        &self.shared.bucket
    }
}
