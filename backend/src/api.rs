use crate::Backend;
use anyhow::Result;
use axum::{serve, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod crates;
mod jobs;

fn routes() -> Router<Backend> {
    let router = Router::new().merge(crates::routes()).merge(jobs::routes());
    Router::new().nest("/api/v1", router)
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
