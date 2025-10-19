// ndnm-core/src/server/mod.rs
pub mod router;

pub use router::router;

use crate::error::AppError;
use crate::node::Node;
use serde::{de::DeserializeOwned, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ServerOpts {
    pub port: u16,
}

pub async fn serve<N>(opts: ServerOpts, node: N) -> Result<(), AppError>
where
    N: Node + Send + Sync + 'static,
    N::Input: DeserializeOwned + Send + 'static,
    N::Output: Serialize + Send + 'static,
{
    let app = router(node);
    let addr: SocketAddr = format!("0.0.0.0:{}", opts.port).parse().unwrap();
    println!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|_| AppError::Internal)
}