use std::convert::Infallible;

use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

// TODO: display sources, backtrace, ...
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    // Tower errors
    #[error("Timeout error ({0})")]
    TimeoutError(#[from] Box<tower::timeout::error::Elapsed>),

    // App-specific errors
    #[error("DB error ({0})")]
    DatabaseError(anyhow::Error),
    #[error("Not found ({0})")]
    NotFound(&'static str),
    #[error("Already exists ({0})")]
    AlreadyExists(&'static str),
    #[error("Conversion error ({0})")]
    ConversionError(&'static str),

    // Generic errors (standard, anyhow)
    #[error(transparent)]
    StdError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    AnyHowError(#[from] anyhow::Error),
}

impl From<&'static str> for AppError {
    fn from(f: &'static str) -> AppError {
        AppError::AnyHowError(anyhow::anyhow!(f))
    }
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::TimeoutError(_) => StatusCode::REQUEST_TIMEOUT,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::AlreadyExists(_) => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    type Body = axum::body::Full<axum::body::Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let body = axum::Json(json!({
            "error": self.to_string(),
        }));

        (self.status_code(), body).into_response()
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! register_db_error {
    ($error_type:ty) => {
        impl From<$error_type> for AppError {
            fn from(e: $error_type) -> Self {
                AppError::DatabaseError(e.into())
            }
        }
    };
}
