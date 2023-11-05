use anyhow::Result;
use buildsrs_backend::*;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();
    let backend = Backend::new(&options).await?;

    backend.listen(options.listen).await?;
    Ok(())
}
