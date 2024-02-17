use super::*;
use docker_api::{conn::TtyChunk, Docker};
use futures::StreamExt;
use std::path::{Path, PathBuf};
use tracing::info;

const DEFAULT_DOCKER_SOCKET: &str = "unix:///var/run/docker.sock";

#[cfg(feature = "options")]
pub(crate) mod options;

#[cfg(test)]
mod tests;

/// Build strategy that uses Docker.
#[derive(Clone, Debug)]
pub struct DockerStrategy {
    docker: Docker,
}

#[async_trait]
impl Strategy for DockerStrategy {
    async fn builder_from_path(&self, path: &Path) -> Result<DynBuilder> {
        Ok(Box::new(DockerBuilder::new(self.docker.clone(), path)))
    }

    async fn builder_from_crate(&self, _krate: &Path) -> Result<DynBuilder> {
        todo!()
    }

    async fn builder_from_url(&self, _url: &str, _checksum: &str) -> Result<DynBuilder> {
        todo!()
    }
}

/// Crate builder that uses Docker to execute Cargo commands.
#[derive(Clone, Debug)]
pub struct DockerBuilder {
    docker: Docker,
    folder: PathBuf,
}

impl DockerBuilder {
    /// Create new Docker builer from Docker handle and path.
    pub fn new<P: Into<PathBuf>>(docker: Docker, path: P) -> Self {
        Self {
            docker,
            folder: path.into(),
        }
    }

    /// Get reference to Docker handle.
    pub fn docker(&self) -> &Docker {
        &self.docker
    }

    /// Get path that this crate is extracted at.
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// Delete this crate.
    pub async fn delete(self) -> Result<()> {
        tokio::fs::remove_dir_all(&self.folder).await?;
        Ok(())
    }
}

#[async_trait]
impl Builder for DockerBuilder {
    async fn metadata(&self) -> Result<Metadata> {
        let containers = self.docker.containers();
        let opts = docker_api::opts::ContainerCreateOpts::builder()
            .attach_stdout(true)
            .auto_remove(true)
            .command(["cargo", "metadata", "--no-deps"])
            .image("docker.io/library/rust")
            .volumes([format!("{}:/crates:ro", self.folder.display())])
            .working_dir("/crates")
            .build();
        let container = containers.create(&opts).await?;

        info!("Created docker container");

        let mut output = container.attach().await?;
        container.start().await?;

        info!("Launched docker container");

        let mut stderr = vec![];
        let mut stdout = vec![];

        while let Some(chunk) = output.next().await {
            match chunk? {
                TtyChunk::StdErr(mut out) => stderr.append(&mut out),
                TtyChunk::StdOut(mut out) => stdout.append(&mut out),
                TtyChunk::StdIn(_) => {}
            }
        }

        let _stderr = String::from_utf8_lossy(&stderr);
        let stdout = String::from_utf8_lossy(&stdout);

        Ok(serde_json::from_str(&stdout)?)
    }
}
