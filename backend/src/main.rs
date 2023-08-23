use anyhow::Result;
use apply::Apply;
use axum::Router;
use buildsrs_database::Database;
use clap::Parser;
use std::sync::Arc;

mod api;
mod options;
mod state;

pub use crate::{options::Options, state::Backend};

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();
    let backend = Backend::new(&options).await?;
    backend.listen(options.listen).await?;
    Ok(())
}
