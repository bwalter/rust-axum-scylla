use crate::error::AppError;

pub type AppResult<T> = anyhow::Result<T, AppError>;
