use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal error")]
    Internal,
}

#[derive(Serialize)]
struct ErrEnvelope<'a> {
    status: &'a str,
    error: ErrDetail<'a>,
}

#[derive(Serialize)]
struct ErrDetail<'a> {
    code: &'a str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::BadRequest(msg) => {
                let body = ErrEnvelope {
                    status: "error",
                    error: ErrDetail { code: "BAD_REQUEST", message: msg },
                };
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }
            AppError::Internal => {
                let body = ErrEnvelope {
                    status: "error",
                    error: ErrDetail { code: "INTERNAL", message: "internal error".into() },
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
            }
        }
    }
}

impl AppError {
    pub fn bad<S: Into<String>>(s: S) -> Self { AppError::BadRequest(s.into()) }
}
