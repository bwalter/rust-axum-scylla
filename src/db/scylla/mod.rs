use crate::result::AppResult;

pub mod errors;
pub mod queries;
pub mod vehicle_queries;

pub async fn create_session(addr: &str, port: u16) -> AppResult<scylla::Session> {
    // Database session
    let session = scylla::SessionBuilder::new()
        .known_node(format!("{}:{}", addr, port))
        .build()
        .await?;

    Ok(session)
}
