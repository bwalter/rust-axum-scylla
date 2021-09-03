use anyhow::Result;
use reqwest::StatusCode;
use serde_json::json;
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use hello::{
    self,
    db::queries::Queries,
    vehicle::{Engine, Vehicle},
};

use crate::support::mocked_queries::MockedQueries;

mod support;

#[tokio::test]
async fn test_create_vehicle() -> Result<()> {
    // Start server with mock queries
    let queries = Arc::new(MockedQueries::new());
    let addr = serve(queries.clone()).await?;

    let vehicle_json = json!({
        "vin": "vin1",
        "engine": { "type": "Combustion" }
    });

    let client = reqwest::Client::new();

    // Insert vehicle
    let res = client
        .post(format!("http://{}/vehicle", addr))
        .json(&vehicle_json)
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::CREATED);

    // Check returned vehicle
    let body = res.text().await.unwrap();
    assert_eq!(json_value(&body)?, vehicle_json);

    // Ensure that it has been added to the database
    assert_eq!(
        queries.get_vehicle("vin1"),
        Some(serde_json::from_value(vehicle_json.clone())?),
    );

    // Insert vehicle again
    let res = client
        .post(format!("http://{}/vehicle", addr))
        .json(&vehicle_json)
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::CONFLICT);
    Ok(())
}

#[tokio::test]
async fn test_get_vehicle() -> Result<()> {
    // Start server with mock queries
    let queries = Arc::new(MockedQueries::new());
    let addr = serve(queries.clone()).await?;

    let client = reqwest::Client::new();

    // Get non-existing vehicle
    let res = client
        .get(format!("http://{}/vehicle", addr))
        .query(&[("vin", "vin1")])
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // Add vehicle to database
    let vehicle = Vehicle {
        vin: "vin1".to_string(),
        engine: Engine::Combustion,
        ev_data: None,
    };
    queries.insert_vehicle(vehicle.clone());

    // Get existing vehicle
    let res = client
        .get(format!("http://{}/vehicle", addr))
        .query(&[("vin", "vin1")])
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::OK);

    // Check returned vehicle
    let body = res.text().await.unwrap();
    assert_eq!(json_value(&body)?, serde_json::to_value(&vehicle)?);

    Ok(())
}

fn json_value(s: &str) -> Result<serde_json::Value> {
    Ok(serde_json::from_str::<serde_json::Value>(s)?)
}

async fn serve(queries: Arc<dyn Queries>) -> Result<SocketAddr> {
    //tracing_subscriber::fmt::init();

    // TCP listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(&addr)?;
    let addr = listener.local_addr()?;

    // Start the server and wait until ready
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move { hello::start(listener, queries, Some(tx)).await });
    rx.await?;

    Ok(addr)
}
