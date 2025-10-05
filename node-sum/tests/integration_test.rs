// node-sum/tests/integration_test.rs
use axum::{body::Body, http::{Request, StatusCode}};
use ndnm_core;
use tower::ServiceExt;
use serde_json::json;

// Padrão para testar um binário: incluímos o `main.rs` como um módulo.
mod server {
    #![allow(dead_code)]
    include!("../src/main.rs");
}

#[tokio::test]
async fn run_sum_ok() {
    let app = ndnm_core::router(server::SumNode::default());

    let body = serde_json::to_vec(&json!({"variables": [50, 25, 25]})).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    
    let expected_json = json!({"response": 100});
    let out_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(out_json, expected_json);
}

#[tokio::test]
async fn run_sum_bad_request_empty_list() {
    let app = ndnm_core::router(server::SumNode::default());

    // Dispara a validação (lista não pode ser vazia)
    let body = serde_json::to_vec(&json!({"variables": []})).unwrap();
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

    assert_eq!(
        v,
        json!({
          "status":"error",
          "error": { "code":"BAD_REQUEST", "message":"envie ao menos 1 número em 'variables'" }
        })
    );
}