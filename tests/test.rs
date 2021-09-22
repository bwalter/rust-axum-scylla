use anyhow::Result;
use reqwest::StatusCode;
use serde_json::json;
use std::{
    net::{SocketAddr, TcpListener},
    sync::{Arc, RwLock},
};

use hello::{
    self,
    db::queries::Queries,
    state::State,
    vehicle::{Engine, Vehicle},
};

use crate::support::test_queries::TestQueries;

mod support;

#[tokio::test]
async fn test_post_vehicle() -> Result<()> {
    // Start server with mock queries
    let queries = Arc::new(create_test_queries());
    let addr = serve(queries.clone()).await?;

    let vehicle_json = json!({
        "vin": "vin1",
        "engine_type": "Ev",
        "ev_data": {
            "battery_capacity_in_kwh": 12,
            "soc_in_percent": 76,
        }
    });

    let client = reqwest::Client::new();

    // Insert vehicle => CREATED
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
        queries.find_one_vehicle("vin1").await.ok(),
        Some(serde_json::from_value(vehicle_json.clone())?),
    );

    // Insert the same vehicle again => CONFLICT
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
    let queries = Arc::new(create_test_queries());
    let addr = serve(queries.clone()).await?;

    let client = reqwest::Client::new();

    // Get non-existing vehicle => NOT_FOUND
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
    queries.create_vehicle(&vehicle).await?;

    // Get existing vehicle => OK
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

    // Shared state
    let shared_state = Arc::new(RwLock::new(State {}));

    // App
    let app = hello::app(shared_state, queries);

    // Run our app with hyper
    tracing::debug!("listening on {:?}", listener);
    let server = axum::Server::from_tcp(listener)?.serve(app.into_make_service());
    tokio::spawn(async move { server.await });

    Ok(addr)
}

fn create_test_queries() -> impl Queries {
    // Note: returning TestQueries, could be also the real DB queries, though...
    TestQueries::new()
}
