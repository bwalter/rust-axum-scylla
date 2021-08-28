use axum::{
    body::{Bytes, Full},
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::{convert::Infallible, result::Result};

// e.g.: Json<User>
pub type AppResponseBody = Full<Bytes>;

// e.g.: (StatusCode::Ok, Json(user)).into_response()
pub type AppResponse = Response<AppResponseBody>;

// Return value of route handlers
pub type AppResponseResult = Result<AppResponse, AppResponseError>;

#[derive(thiserror::Error, Debug)]
pub enum AppResponseError {
    #[error("DB session error")]
    DbSessionError(#[from] scylla::transport::errors::NewSessionError),
    #[error("DB query error")]
    DbQueryError(#[from] scylla::transport::errors::QueryError),
    #[error("DB row error")]
    DbFromRowError(#[from] scylla::cql_to_rust::FromRowError),
    #[error("Not found")]
    NotFound(),
    #[error("Timeout error")]
    TimeoutError(#[from] Box<tower::timeout::error::Elapsed>),
    #[error("Any error")]
    AnyHowError(#[from] anyhow::Error),
    #[error("Standard error")]
    StdError(#[from] Box<dyn std::error::Error>),
}

impl IntoResponse for AppResponseError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let (status, msg_opt) = match self {
            AppResponseError::DbSessionError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppResponseError::DbQueryError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppResponseError::DbFromRowError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppResponseError::NotFound() => (StatusCode::NOT_FOUND, None),
            AppResponseError::TimeoutError(ref e) => {
                (StatusCode::REQUEST_TIMEOUT, Some(e.to_string()))
            }
            AppResponseError::AnyHowError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
            AppResponseError::StdError(ref e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Some(e.to_string()))
            }
        };

        let msg = if let Some(msg) = msg_opt {
            format!("{} ({})", self, msg)
        } else {
            self.to_string()
        };

        let body = Json(json!({
            "error": msg,
        }));

        (status, body).into_response()
    }
}
