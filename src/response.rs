use axum::{
    body::{Bytes, Full},
    http,
};

use crate::error::AppError;

// e.g.: Json<User>
pub type AppResponseBody = Full<Bytes>;

// e.g.: (StatusCode::Ok, Json(user)).into_response()
pub type AppResponse = http::Response<AppResponseBody>;

// Return value of route handlers
pub type AppResponseResult = std::result::Result<AppResponse, AppError>;
