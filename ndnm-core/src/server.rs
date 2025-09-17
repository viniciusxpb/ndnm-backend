use crate::{AppError, Node};
use axum::{extract::State, routing::{get, post}, Json, Router};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};

#[derive(Debug, Clone)]
pub struct ServerOpts {
    pub port: u16,
}

/// Monta um Router genérico para um Node qualquer.
/// Rotas: GET /health, POST /run
pub fn router<N: Node>(node: N) -> Router<Arc<N>> {
    let state = Arc::new(node);
    Router::new()
        .route("/health", get(health))
        .route("/run", post(run::<N>))
        .with_state(state)
}

pub async fn serve<N: Node>(opts: ServerOpts, node: N) -> Result<(), AppError> {
    let app = router(node);
    let addr: SocketAddr = format!("0.0.0.0:{}", opts.port).parse().unwrap();
    println!("listening on http://{addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .map_err(|_| AppError::Internal)
}

async fn health() -> impl axum::response::IntoResponse {
    (axum::http::StatusCode::OK, Json(json!({"status":"ok"})))
}

async fn run<N: Node>(
    State(node): State<Arc<N>>,
    Json(input): Json<N::Input>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    node.validate(&input)?;
    let out = node.process(input)?;
    Ok((axum::http::StatusCode::OK, Json(out)))
}
