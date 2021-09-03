use axum::{
    body::{Bytes, Full},
    http::Response,
};
use std::result::Result;

use crate::error::AppError;

// e.g.: Json<User>
pub type AppResponseBody = Full<Bytes>;

// e.g.: (StatusCode::Ok, Json(user)).into_response()
pub type AppResponse = Response<AppResponseBody>;

// Return value of route handlers
pub type AppResponseResult = Result<AppResponse, AppError>;
