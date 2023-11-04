use anyhow::Result;
use clap::Parser;
use buildsrs_backend::*;

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();
    let backend = Backend::new(&options).await?;

    backend
        .bucket()
        .put("hello_there", "hello there!".as_bytes().into())
        .await?;

    dbg!("Should upload.");

    backend.listen(options.listen).await?;
    Ok(())
}
