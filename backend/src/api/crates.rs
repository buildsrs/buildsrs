use crate::Backend;
use axum::{routing::get, Router};

async fn crate_version() {}

async fn crate_versions() {}

pub fn routes() -> Router<Backend> {
    Router::new()
        .route("/crate/:crate", get(crate_versions))
        .route("/crate/:crate/:version", get(crate_version))
}
