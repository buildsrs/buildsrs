use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use url::Url;
use ssh_key::{PrivateKey, HashAlg};
use tracing::*;
use tokio_tungstenite::connect_async;

#[derive(Parser, Debug, Clone)]
pub struct Options {
    #[clap(long, short, env)]
    pub private_key_file: PathBuf,

    #[clap(long, short, env)]
    pub websocket: Url,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = Options::parse();
    tracing_subscriber::fmt::init();

    let private_key = PrivateKey::read_openssh_file(&options.private_key_file)?;
    info!("Read private key {}", private_key.fingerprint(HashAlg::Sha512));

    let _connection = connect_async(options.websocket.as_str()).await?;
    info!("Connected to {}", options.websocket);

    Ok(())
}
