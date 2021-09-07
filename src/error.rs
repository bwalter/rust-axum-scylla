use std::{convert::Infallible, sync::Arc};

use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    // Scylla errors
    #[error("DB session error")]
    DbSessionError(#[from] scylla::transport::errors::NewSessionError),
    #[error("DB query error")]
    DbQueryError(#[from] scylla::transport::errors::QueryError),
    #[error("DB query error")]
    DbQueryArcError(#[from] Arc<scylla::transport::errors::QueryError>),
    #[error("DB row error")]
    DbFromRowError(#[from] scylla::cql_to_rust::FromRowError),

    // Tower errors
    #[error("Timeout error")]
    TimeoutError(#[from] Box<tower::timeout::error::Elapsed>),

    // App-specific errors
    #[error("Not found")]
    NotFound(),
    #[error("Already exists")]
    AlreadyExists(),
    #[error("Conversion error")]
    ConversionError(&'static str),

    // Generic errors (standard, anyhow)
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
            // Scylla errors
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

            // Tower errors
            AppError::TimeoutError(ref e) => (StatusCode::REQUEST_TIMEOUT, Some(e.to_string())),

            // App-specific errors
            AppError::NotFound() => (StatusCode::NOT_FOUND, None),
            AppError::AlreadyExists() => (StatusCode::CONFLICT, None),
            AppError::ConversionError(s) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(s.to_string()))
            }

            // Generic errors (standard, anyhow)
            AppError::AnyHowError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppError::StdError(ref e) => (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string())),
        };

        let msg = if let Some(msg) = msg_opt {
            // AppError description + message of the source error
            format!("{} ({})", self, msg)
        } else {
            // Only AppError description
            self.to_string()
        };

        let body = axum::Json(json!({
            "error": msg,
        }));

        (status, body).into_response()
    }
}
