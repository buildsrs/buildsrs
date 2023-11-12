use super::*;
use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::sync::Arc;

#[derive(ValueEnum, PartialEq, Debug, Clone, Copy)]
enum StrategyName {
    #[cfg(feature = "docker")]
    Docker,
}

/// Command-line options for builder strategy.
#[derive(Parser, PartialEq, Debug)]
pub struct StrategyOptions {
    #[clap(long, default_value = "docker")]
    builder: StrategyName,

    #[cfg(feature = "docker")]
    #[clap(flatten)]
    docker: docker::options::DockerBuilderOptions,
}

impl StrategyOptions {
    /// Build strategy
    pub async fn build(&self) -> Result<DynStrategy> {
        let strategy: DynStrategy = match self.builder {
            #[cfg(feature = "docker")]
            StrategyName::Docker => Arc::new(self.docker.build().await?),
        };

        Ok(strategy)
    }
}
