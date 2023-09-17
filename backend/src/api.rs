use crate::Backend;
use anyhow::Result;
use axum::{Router, Server};
use std::net::SocketAddr;

mod crates;
mod jobs;

fn routes() -> Router<Backend> {
    let router = Router::new().merge(crates::routes()).merge(jobs::routes());
    Router::new().nest("/api/v1", router)
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
