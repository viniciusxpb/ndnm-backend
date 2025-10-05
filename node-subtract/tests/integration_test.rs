// node-subtract/tests/integration_test.rs
use axum::{body::Body, http::{Request, StatusCode}};
use ndnm_core;
use serde_json::json;
// Corrigido: O trait `ServiceExt` é importado diretamente de `tower`.
use tower::ServiceExt;

// Padrão para testar um binário: incluímos o `main.rs` como um módulo.
mod server {
    // Silencia warnings que podem aparecer ao incluir o main aqui.
    #![allow(dead_code)]

    // Corrigido: Removemos os `use` duplicados daqui.
    // O `include!` trará as importações de `main.rs` para este escopo.
    include!("../src/main.rs");
}

#[tokio::test]
async fn run_subtract_ok() {
    let app = ndnm_core::router(server::SubtractNode::default());

    let body = serde_json::to_vec(&json!({"variables": [100, 20, 8]})).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();

    let expected_json = json!({"response": 72});
    let out_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(out_json, expected_json);
}

#[tokio::test]
async fn run_subtract_bad_request_not_enough_numbers() {
    let app = ndnm_core::router(server::SubtractNode::default());

    let body = serde_json::to_vec(&json!({"variables": [100]})).unwrap();
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
          "error": { "code":"BAD_REQUEST", "message":"envie ao menos 2 números em 'variables' para subtrair" }
        })
    );
}