// ndnm-core/src/server.rs
use crate::{AppError, Node};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};

#[derive(Debug, Clone)]
pub struct ServerOpts {
    pub port: u16,
}

/// Monta um Router genérico para um Node qualquer.
/// Rotas: GET /health, POST /run
pub fn router<N>(node: N) -> Router
where
    N: Node + Send + Sync + 'static,
    N::Input: DeserializeOwned + Send + 'static,
    N::Output: Serialize + Send + 'static,
{
    let state = Arc::new(node);
    Router::new()
        .route("/health", get(health))
        .route("/run", post(run::<N>))
        .with_state(state)
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

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

async fn run<N>(
    State(node): State<Arc<N>>,
    Json(input): Json<N::Input>,
) -> Result<impl IntoResponse, AppError>
where
    N: Node + Send + Sync + 'static,
    N::Input: DeserializeOwned + Send + 'static,
    N::Output: Serialize + Send + 'static,
{
    node.validate(&input)?;
    // Corrigido: Agora usamos .await para chamar a função assíncrona.
    let out = node.process(input).await?;
    Ok((StatusCode::OK, Json(out)))
}