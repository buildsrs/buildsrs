use crate::Backend;
use anyhow::Result;
use axum::{Router, Server};
use std::net::SocketAddr;

mod crates;

fn routes() -> Router<Backend> {
    Router::new().nest("/api/v1", crates::routes())
}

impl Backend {
    pub fn router(&self) -> Router {
        routes().with_state(self.clone())
    }

    pub async fn listen(&self, addr: SocketAddr) -> Result<()> {
        let router = self.router();
        Server::bind(&addr)
            .serve(router.into_make_service())
            .await?;
        Ok(())
    }
}
