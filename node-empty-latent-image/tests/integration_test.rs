// C:/Projetos/ndnm/ndnm-backend/node-empty-latent-image/tests/integration_test.rs
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
async fn run_empty_latent_image_ok() {
    let app = ndnm_core::router(server::EmptyLatentImageNode::default());

    let body = serde_json::to_vec(&json!({
        "width": 512,
        "height": 512,
        "batch_size": 1
    })).unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let out_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    
    assert_eq!(out_json["status"], "Empty Latent Created");
    assert_eq!(out_json["latent_width"], 64);
    assert_eq!(out_json["latent_height"], 64);
}

#[tokio::test]
async fn run_bad_request_not_divisible_by_8() {
    let app = ndnm_core::router(server::EmptyLatentImageNode::default());

    let body = serde_json::to_vec(&json!({
        "width": 512,
        "height": 511, // <-- Inválido
        "batch_size": 1
    })).unwrap();
    
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
          "error": { "code":"BAD_REQUEST", "message":"Width and height must be divisible by 8" }
        })
    );
}

#[tokio::test]
async fn run_bad_request_too_large() {
    let app = ndnm_core::router(server::EmptyLatentImageNode::default());

    let body = serde_json::to_vec(&json!({
        "width": 99999, // <-- Inválido
        "height": 512,
        "batch_size": 1
    })).unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}