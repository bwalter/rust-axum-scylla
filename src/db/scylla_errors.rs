use std::sync::Arc;

use crate::error::AppError;
use crate::register_db_error;

register_db_error!(scylla::transport::errors::NewSessionError);
register_db_error!(scylla::transport::errors::QueryError);
register_db_error!(Arc<scylla::transport::errors::QueryError>);
register_db_error!(scylla::cql_to_rust::FromRowError);
