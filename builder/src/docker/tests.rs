use super::*;
use docker_api::opts::PullOpts;

#[tokio::test]
async fn test_metadata() {
    let host =
        std::env::var("DOCKER_HOST").unwrap_or_else(|_| "unix:///var/run/docker.sock".into());
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
