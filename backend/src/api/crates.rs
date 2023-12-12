use crate::Backend;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use buildsrs_common::api::*;

async fn crate_list(
    State(backend): State<Backend>,
    Query(query): Query<CratesQuery>,
) -> Result<Json<CratesResponse>, ()> {
    let database = backend.database().read().await.unwrap();
    let crates = database.crate_list(&query.name).await.unwrap();
    Ok(Json(CratesResponse { crates }))
}

async fn crate_info(
    State(backend): State<Backend>,
    Path(name): Path<String>,
) -> Result<Json<CrateResponse>, ()> {
    let database = backend.database().read().await.unwrap();
    let info = database.crate_info(&name).await.unwrap();
    let versions = database.crate_versions(&name).await.unwrap();
    Ok(Json(CrateResponse {
        name: info.name,
        versions: versions.into_iter().collect(),
    }))
}

async fn crate_version(
    State(backend): State<Backend>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Json<CrateVersionResponse>, ()> {
    let database = backend.database().read().await.unwrap();
    let info = database.crate_version_info(&name, &version).await.unwrap();
    Ok(Json(CrateVersionResponse {
        name: info.name,
        version: info.version,
        checksum: info.checksum,
        yanked: info.yanked,
        artifacts: Default::default(),
    }))
}

async fn crate_artifact(
    State(_backend): State<Backend>,
    Path((_name, _version, _target, _artifact)): Path<(String, String, String, String)>,
) -> Result<Json<()>, ()> {
    todo!()
}

pub fn routes() -> Router<Backend> {
    Router::new()
        .route("/crates", get(crate_list))
        .route("/crates/:crate", get(crate_info))
        .route("/crates/:crate/:version", get(crate_version))
        .route(
            "/crates/:crate/:version/:target/:artifact",
            get(crate_artifact),
        )
}
