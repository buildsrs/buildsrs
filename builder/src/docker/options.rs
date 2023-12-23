use super::*;

#[derive(clap::Parser, PartialEq, Debug)]
pub(crate) struct DockerBuilderOptions {
    /// Docker daemon to connect to
    #[clap(long, short, env, default_value = DEFAULT_DOCKER_SOCKET)]
    pub docker: String,
}

impl DockerBuilderOptions {
    pub(crate) async fn build(&self) -> Result<DockerStrategy> {
        Ok(DockerStrategy {
            docker: Docker::new(&self.docker)?,
        })
    }
}
