use super::*;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Clone, Debug, PartialEq)]
pub struct FilesystemOptions {
    #[clap(long, env, required_if_eq("storage", "filesystem"))]
    storage_filesystem_path: Option<PathBuf>,
}

impl FilesystemOptions {
    pub async fn build(&self) -> Filesystem {
        Filesystem::new(self.storage_filesystem_path.clone().unwrap())
    }
}
