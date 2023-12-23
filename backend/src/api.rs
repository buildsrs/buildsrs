use crate::Backend;
use anyhow::Result;
use axum::{serve, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod crates;
#[cfg(feature = "frontend")]
mod frontend;
mod jobs;

fn routes() -> Router<Backend> {
    let api = Router::new().merge(crates::routes()).merge(jobs::routes());
    let router = Router::new().nest("/api/v1", api);
    #[cfg(feature = "frontend")]
    let router = router.nest("/", frontend::routes());
    router.layer(TraceLayer::new_for_http())
}

impl Backend {
    /// Get router for REST API.
    pub fn router(&self) -> Router {
        routes().with_state(self.clone())
    }

    /// Launch REST API, listening on the given address.
    pub async fn listen(&self, addr: SocketAddr) -> Result<()> {
        let router = self.router();
        let listener = TcpListener::bind(&addr).await?;
        serve(listener, router).await?;
        Ok(())
    }
}
