use std::{convert::Infallible, sync::Arc};

use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("DB session error")]
    DbSessionError(#[from] scylla::transport::errors::NewSessionError),
    #[error("DB query error")]
    DbQueryError(#[from] scylla::transport::errors::QueryError),
    #[error("DB query error")]
    DbQueryArcError(#[from] Arc<scylla::transport::errors::QueryError>),
    #[error("DB row error")]
    DbFromRowError(#[from] scylla::cql_to_rust::FromRowError),
    #[error("Not found")]
    NotFound(),
    #[error("Timeout error")]
    TimeoutError(#[from] Box<tower::timeout::error::Elapsed>),
    #[error("Any error")]
    AnyHowError(#[from] anyhow::Error),
    #[error("Standard error")]
    StdError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl IntoResponse for AppError {
    type Body = axum::body::Full<axum::body::Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let (status, msg_opt) = match self {
            AppError::DbSessionError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppError::DbQueryError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppError::DbQueryArcError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppError::DbFromRowError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppError::NotFound() => (StatusCode::NOT_FOUND, None),
            AppError::TimeoutError(ref e) => (StatusCode::REQUEST_TIMEOUT, Some(e.to_string())),
            AppError::AnyHowError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppError::StdError(ref e) => (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string())),
        };

        let msg = if let Some(msg) = msg_opt {
            format!("{} ({})", self, msg)
        } else {
            self.to_string()
        };

        let body = axum::Json(json!({
            "error": msg,
        }));

        (status, body).into_response()
    }
}
