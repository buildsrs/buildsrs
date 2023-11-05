use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(short, long, env = "BUILDSRS_DATABASE")]
    pub database: String,

    #[clap(short, long, env = "BUILDSRS_LISTEN", default_value = "127.0.0.1:8000")]
    pub listen: SocketAddr,
}

#[derive(Parser, Debug)]
pub struct BucketOptions {
    #[clap(
        short,
        long,
        env = "BUILDSRS_BUCKET_NAME",
        default_value = "prod.builds.rs"
    )]
    pub name: String,

    #[clap(
        short,
        long,
        env = "BUILDSRS_BUCKET_REGION",
        default_value = "eu-central-2"
    )]
    pub region: String,

    #[clap(short, long, env = "BUILDSRS_BUCKET_ACCESS_KEY")]
    pub access_key: String,

    #[clap(short, long, env = "BUILDSRS_BUCKET_SECRET_KEY")]
    pub secret_key: String,
}
