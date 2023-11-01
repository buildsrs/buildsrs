pub mod wasabi;

use crate::options::BucketOptions;
use anyhow::Result;
use async_trait::async_trait;
use axum::body::Bytes;
use std::fmt::{Debug, Formatter};

pub type BucketTraitObject = Box<dyn Bucket + Send + Sync>;

#[async_trait]
pub trait Bucket {
    async fn put(&self, name: &str, content: Bytes) -> Result<()>;
    fn gen_url(&self, path: &str) -> String;
}

impl Debug for BucketTraitObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bucket").finish()
    }
}

pub struct Credentials {
    pub access_key: String,
    pub secret_key: String,
}

impl From<&BucketOptions> for Credentials {
    fn from(options: &BucketOptions) -> Self {
        Self {
            access_key: options.access_key.clone(),
            secret_key: options.secret_key.clone(),
        }
    }
}

impl From<Credentials> for aws_credential_types::Credentials {
    fn from(credentials: Credentials) -> Self {
        aws_credential_types::Credentials::new(
            credentials.access_key.clone(),
            credentials.secret_key.clone(),
            None,
            None,
            "wasabi",
        )
    }
}
