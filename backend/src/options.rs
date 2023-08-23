use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(short, long, env = "BUILDSRS_DATABASE")]
    pub database: String,

    #[clap(short, long, env = "BUILDSRS_LISTEN", default_value = "127.0.0.1:8000")]
    pub listen: SocketAddr,
}
