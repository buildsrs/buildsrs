use super::*;
use docker_api::opts::PullOpts;

fn docker_host() -> String {
    std::env::var("DOCKER_HOST").unwrap_or_else(|_| "unix:///var/run/docker.sock".into())
}

#[cfg(feature = "options")]
#[tokio::test]
async fn test_options() {
    use clap::Parser;
    let options = vec![
        "builder".into(),
        "--strategy".into(),
        "docker".into(),
        "--docker".into(),
        docker_host(),
    ];
    let options = crate::StrategyOptions::try_parse_from(options).unwrap();
    let _builder = options.build().await.unwrap();
}

#[tokio::test]
async fn test_metadata() {
    let host = docker_host();
    let docker = Docker::new(&host).unwrap();

    let images = docker.images();
    let mut stream = images.pull(&PullOpts::builder().image("docker.io/library/rust").build());
    while let Some(item) = stream.next().await {
        let _item = item.unwrap();
    }

    let path = tokio::fs::canonicalize("./tests/crates/no_deps")
        .await
        .unwrap();
    let builder = DockerBuilder::new(docker, path);
    let metadata = builder.metadata().await.unwrap();

    let expected: Metadata = serde_json::from_str(
        &tokio::fs::read_to_string("./tests/crates/no_deps/metadata.json")
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(metadata.packages, expected.packages);
}
