use crate::Backend;
use axum::{
    extract::{Path, State},
    http::header::{self, HeaderMap},
    routing::get,
    Router,
};
use bytes::Bytes;
use std::path::PathBuf;

const DEFAULT_FILE: &str = "index.html";

async fn frontend_file(
    State(state): State<Backend>,
    Path(mut path): Path<Vec<String>>,
) -> Result<(HeaderMap, Bytes), ()> {
    if path.is_empty() {
        path.push(DEFAULT_FILE.into());
    }

    tracing::debug!("{path:?}");

    let path = PathBuf::from(path.join("/"));
    let Some(bytes) = state.frontend().get(&path) else {
        return Err(());
    };

    let mut headers = HeaderMap::default();

    if let Some(mime) = mime_guess::from_path(path).first() {
        headers.insert(header::CONTENT_TYPE, mime.to_string().try_into().unwrap());
    }

    Ok((headers, bytes.clone()))
}

pub fn routes() -> Router<Backend> {
    Router::new()
        .route("/*path", get(frontend_file))
        .route("/", get(frontend_file))
}
