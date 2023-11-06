use anyhow::Result;
use buildsrs_backend::*;
use clap::Parser;

mod options;

#[tokio::main]
async fn main() -> Result<()> {
    let options = options::Options::parse();
    let backend = options.build().await?;

    backend.listen(options.listen).await?;
    Ok(())
}
