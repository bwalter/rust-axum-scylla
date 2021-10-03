use anyhow::Result;
use reqwest::StatusCode;
use serde_json::json;
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use hello::{
    self,
    db::queries::VehicleQueries,
    model::vehicle::{Engine, Vehicle},
};

#[tokio::test]
async fn test_post_vehicle() -> Result<()> {
    let ctx = Context::try_new().await?;

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
        .post(format!("http://{}/vehicle", ctx.addr))
        .json(&vehicle_json)
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::CREATED);

    // Check returned vehicle
    let body = res.text().await.unwrap();
    assert_eq!(json_value(&body)?, vehicle_json);

    // Ensure that it has been added to the database
    assert_eq!(
        ctx.queries.find_one_vehicle("vin1").await.ok(),
        Some(serde_json::from_value(vehicle_json.clone())?),
    );

    // Insert the same vehicle again => CONFLICT
    let res = client
        .post(format!("http://{}/vehicle", ctx.addr))
        .json(&vehicle_json)
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::CONFLICT);

    Ok(())
}

#[tokio::test]
async fn test_get_vehicle() -> Result<()> {
    let ctx = Context::try_new().await?;

    let client = reqwest::Client::new();

    // Get non-existing vehicle => NOT_FOUND
    let res = client
        .get(format!("http://{}/vehicle/vin1", ctx.addr))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // Add vehicle to database
    let vehicle = Vehicle {
        vin: "vin1".to_string(),
        engine: Engine::Combustion,
        ev_data: None,
    };
    ctx.queries.create_vehicle(&vehicle).await?;

    // Get existing vehicle => OK
    let res = client
        .get(format!("http://{}/vehicle/vin1", ctx.addr))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::OK);

    // Check returned vehicle
    let body = res.text().await.unwrap();
    assert_eq!(json_value(&body)?, serde_json::to_value(&vehicle)?);

    Ok(())
}

#[tokio::test]
async fn test_delete_vehicle() -> Result<()> {
    let ctx = Context::try_new().await?;

    let client = reqwest::Client::new();

    // Delete non-existing vehicle => NOT_FOUND
    let res = client
        .delete(format!("http://{}/vehicle/vin1", ctx.addr))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // Add vehicle to database
    let vehicle = Vehicle {
        vin: "vin1".to_string(),
        engine: Engine::Combustion,
        ev_data: None,
    };
    ctx.queries.create_vehicle(&vehicle).await?;

    // Delete existing vehicle => OK
    let res = client
        .delete(format!("http://{}/vehicle/vin1", ctx.addr))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::OK);

    // Ensure that it is not in the database anymore
    assert!(ctx.queries.find_one_vehicle("vin1").await.is_err());

    Ok(())
}

fn json_value(s: &str) -> Result<serde_json::Value> {
    Ok(serde_json::from_str::<serde_json::Value>(s)?)
}

struct Context {
    queries: Arc<dyn VehicleQueries>,
    addr: SocketAddr,
}

impl Context {
    async fn try_new() -> Result<Self> {
        let queries = Arc::new(create_test_queries().await?);
        let addr = serve(queries.clone()).await?;

        Ok(Self { queries, addr })
    }
}

impl Drop for Context {
    fn drop(&mut self) {}
}
async fn create_test_queries() -> Result<impl VehicleQueries> {
    use hello::db::scylla::vehicle_queries::ScyllaVehicleQueries;
    use scylla::SessionBuilder;

    let uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    let session = SessionBuilder::new().known_node(uri).build().await.unwrap();

    // First, delete test keyspace to have a fresh test data
    session
        .query("DROP KEYSPACE IF EXISTS hello_test", &[])
        .await
        .unwrap_or_default();

    let queries =
        ScyllaVehicleQueries::try_new(Arc::new(session), "hello_test".to_string()).await?;
    Ok(queries)
}

async fn serve(queries: Arc<dyn VehicleQueries>) -> Result<SocketAddr> {
    // TCP listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(&addr)?;
    let addr = listener.local_addr()?;

    // Router
    let router = hello::router::create_router(queries);

    // Run our app
    tracing::debug!("listening on {:?}", listener);
    let server = axum::Server::from_tcp(listener)?.serve(router.into_make_service());
    tokio::spawn(async move { server.await });

    Ok(addr)
}
