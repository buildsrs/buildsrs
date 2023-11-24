use super::*;
use aws_config::SdkConfig;
use aws_credential_types::Credentials;
use aws_sdk_s3::Config;
use aws_types::region::Region;
use url::Url;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "options", derive(clap::Parser))]
pub struct S3Options {
    #[cfg_attr(feature = "options", clap(long, env, required_if_eq("storage", "s3")))]
    pub storage_s3_endpoint: Option<Url>,

    #[cfg_attr(feature = "options", clap(long, env, required_if_eq("storage", "s3")))]
    pub storage_s3_access_key_id: Option<String>,

    #[cfg_attr(feature = "options", clap(long, env, required_if_eq("storage", "s3")))]
    pub storage_s3_secret_access_key: Option<String>,

    #[cfg_attr(feature = "options", clap(long, env, required_if_eq("storage", "s3")))]
    pub storage_s3_region: Option<String>,

    #[cfg_attr(feature = "options", clap(long, env))]
    pub storage_s3_path_style: bool,

    #[cfg_attr(
        feature = "options",
        clap(long, env, default_value = "buildsrs", required_if_eq("storage", "s3"))
    )]
    pub storage_s3_bucket: Option<String>,
}

impl S3Options {
    fn credentials(&self) -> Credentials {
        Credentials::new(
            self.storage_s3_access_key_id.as_ref().unwrap(),
            self.storage_s3_secret_access_key.as_ref().unwrap(),
            None,
            None,
            "S3Options",
        )
    }

    async fn config(&self) -> SdkConfig {
        aws_config::from_env()
            .endpoint_url(self.storage_s3_endpoint.as_ref().unwrap().as_str())
            .region(Region::new(
                self.storage_s3_region.as_ref().unwrap().clone(),
            ))
            .credentials_provider(self.credentials())
            .load()
            .await
    }

    pub async fn build(&self) -> S3 {
        let config = Config::from(&self.config().await)
            .to_builder()
            .force_path_style(self.storage_s3_path_style)
            .build();
        let client = Client::from_conf(config);
        S3::new(client, self.storage_s3_bucket.clone().unwrap())
    }
}
