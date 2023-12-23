use super::*;
use anyhow::Result;
use clap::{Parser, ValueEnum};
#[cfg(feature = "docker")]
use docker::options::DockerBuilderOptions;
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
    strategy: StrategyName,

    #[cfg(feature = "docker")]
    #[clap(flatten)]
    docker: DockerBuilderOptions,
}

#[test]
fn can_parse_default_options() {
    use std::iter::empty;

    #[cfg(feature = "docker")]
    assert_eq!(
        StrategyOptions::parse_from(empty::<String>()),
        StrategyOptions {
            strategy: StrategyName::Docker,
            docker: DockerBuilderOptions::parse_from(empty::<String>()),
        }
    );
}

impl StrategyOptions {
    /// Build strategy
    pub async fn build(&self) -> Result<DynStrategy> {
        let strategy: DynStrategy = match self.strategy {
            #[cfg(feature = "docker")]
            StrategyName::Docker => Arc::new(self.docker.build().await?),
        };

        Ok(strategy)
    }
}
