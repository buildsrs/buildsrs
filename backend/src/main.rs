use anyhow::Result;
use buildsrs_backend::*;
use clap::Parser;

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
