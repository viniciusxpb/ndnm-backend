// ndnm-core/src/server/router.rs
use crate::error::AppError;
use crate::node::Node;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::sync::Arc;

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
    let out = node.process(input).await?;
    Ok((StatusCode::OK, Json(out)))
}