use super::Credentials;
use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_s3::{client::Client, types::ObjectCannedAcl};
use aws_types::region::Region;
use axum::body::Bytes;

#[derive(Debug)]
pub struct WasabiBucket {
    bucket: Client,
    name: String,
    region: String,
}

impl WasabiBucket {
    pub fn new(bucket_name: &str, credentials: Credentials, region: &str) -> Result<Self> {
        // Make aws credentials from our own type which implements the ProvideCredentials trait.
        let credentials: aws_credential_types::Credentials = credentials.into();

        // Since we're only using the storage service and it is not even aws we don't need a shared config.
        let bucket_config = aws_sdk_s3::config::Config::builder()
            .region(Region::new(region.to_owned()))
            .credentials_provider(credentials)
            .endpoint_url(format!("https://s3.{region}.wasabisys.com"))
            .force_path_style(true)
            .build();

        Ok(Self {
            bucket: Client::from_conf(bucket_config),
            name: bucket_name.to_owned(),
            region: region.to_owned(),
        })
    }
}

#[async_trait]
impl crate::bucket::Bucket for WasabiBucket {
    async fn put(&self, path: &str, content: Bytes) -> Result<()> {
        self.bucket
            .put_object()
            .bucket(self.name.to_owned())
            .key(path)
            // If necessary we can expose a way to set the content type in the trait method signature.
            .content_type("application/octet-stream")
            .content_length(content.len() as i64)
            // If necessary we can expose a generic way to set the ACL in the trait method signature.
            .acl(ObjectCannedAcl::PublicRead)
            .body(content.into())
            // Setting metadata for objects might interest us later on.
            // .set_metadata()
            .send()
            .await?;

        Ok(())
    }

    fn gen_url(&self, path: &str) -> String {
        format!(
            "https://s3.{region}.wasabisys.com/{bucket}/{path}",
            region = self.region,
            bucket = self.name
        )
    }
}
