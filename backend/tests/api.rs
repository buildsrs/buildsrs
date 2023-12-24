use axum::{body::Body, http::Request};
use buildsrs_backend::*;
use buildsrs_database::*;
use buildsrs_storage::*;
use http_body_util::BodyExt;
use std::{future::Future, path::Path, sync::Arc};
use tower::ServiceExt;

async fn with_backend<O: Future<Output = ()>, F: FnOnce(Backend) -> O>(f: F) {
    let storage = S3::new_temp().await;
    let host = std::env::var("DATABASE").expect("DATABASE env var must be present to run tests");
    let temp_database = TempDatabase::create(&host, None).await.unwrap();

    let backend = Backend::new(
        Arc::new(temp_database.pool().clone()),
        Arc::new((&*storage).clone()),
    );

    f(backend).await;

    temp_database.delete().await.unwrap();
    storage.cleanup().await;
}

#[cfg(feature = "frontend")]
#[tokio::test]
async fn can_get_frontend_file() {
    with_backend(|backend| async move {
        let backend = backend.with_frontend(Arc::new(
            [("file".into(), (b"content" as &[u8]).into())].into(),
        ));
        let request = Request::builder().uri("/file").body(Body::empty()).unwrap();
        let response = backend.router().oneshot(request).await.unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body, &b"content"[..]);
    })
    .await;
}

#[cfg(feature = "frontend-vendor")]
#[tokio::test]
async fn can_get_frontend_vendored() {
    with_backend(|backend| async move {
        let frontend = Arc::new(frontend());
        let backend = backend.with_frontend(frontend.clone());

        // make sure that getting the root path works correctly
        let request = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = backend.router().oneshot(request).await.unwrap();
        assert_eq!(response.headers().get("Content-Type").unwrap(), "text/html");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body, frontend.get(Path::new("index.html")).unwrap());
    })
    .await;
}
