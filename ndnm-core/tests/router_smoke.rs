use axum::{body::Body, http::{Request, StatusCode}};
use ndnm_core::{self as core, AppError, Node};
use serde::{Deserialize, Serialize};
use tower::ServiceExt; // for `oneshot`

// Node de teste bem simples
#[derive(Clone, Default)]
struct DummyNode;

#[derive(Debug, Deserialize)]
struct In { x: i64, y: i64 }

#[derive(Debug, Serialize, PartialEq)]
struct Out { sum: i64 }

impl Node for DummyNode {
    type Input = In;
    type Output = Out;

    fn validate(&self, input: &Self::Input) -> Result<(), AppError> {
        if input.x == 0 && input.y == 0 {
            return Err(AppError::bad("x e y não podem ser ambos zero"));
        }
        Ok(())
    }

    fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        Ok(Out { sum: input.x + input.y })
    }
}

#[tokio::test]
async fn health_ok() {
    let app = core::router(DummyNode::default());
    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v, serde_json::json!({"status":"ok"}));
}

#[tokio::test]
async fn run_ok() {
    let app = core::router(DummyNode::default());

    let body = serde_json::to_vec(&serde_json::json!({"x": 10, "y": 32})).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let out: Out = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(out, Out { sum: 42 });
}

#[tokio::test]
async fn run_bad_request() {
    let app = core::router(DummyNode::default());

    // dispara a validação (x=y=0)
    let body = serde_json::to_vec(&serde_json::json!({"x": 0, "y": 0})).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    // confere envelope de erro do AppError::bad(...)
    assert_eq!(
        v,
        serde_json::json!({
          "status":"error",
          "error": { "code":"BAD_REQUEST", "message":"x e y não podem ser ambos zero" }
        })
    );
}
