pub mod queries;
pub mod scylla;

pub use crate::db::scylla::start_db_session_and_create_queries;
