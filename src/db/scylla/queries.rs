use anyhow::Result;
use std::sync::Arc;

use crate::db::queries::Queries;
use crate::db::scylla::vehicle_queries::ScyllaVehicleQueries;
use crate::error::AppError;

pub struct ScyllaQueries {
    vehicle_queries: ScyllaVehicleQueries,

    #[allow(dead_code)]
    session: Arc<scylla::Session>,
}

impl ScyllaQueries {
    pub async fn new(session: scylla::Session, keyspace: &str) -> Result<ScyllaQueries, AppError> {
        let session = Arc::new(session);

        // Create keyspace, user types and tables
        let cql_array = [
            format!("CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{'class' : 'SimpleStrategy', 'replication_factor' : 1}}", keyspace),
            format!("CREATE TYPE IF NOT EXISTS {}.ev_data (battery_capacity_in_kwh int, soc_in_percent int)", keyspace),
            format!("CREATE TABLE IF NOT EXISTS {}.vehicles (vin text primary key, engine_type text, ev_data ev_data)", keyspace),
        ];
        for cql in cql_array.iter() {
            session.query(cql.as_ref(), &[]).await?;
        }

        // Use keyspace
        session.use_keyspace(keyspace, false).await?;

        // Create (lazily-prepared) vehicle queries
        let vehicle_queries = ScyllaVehicleQueries::try_new(session.clone()).await?;

        Ok(ScyllaQueries {
            vehicle_queries,
            session,
        })
    }
}

impl Queries for ScyllaQueries {
    type VQ = ScyllaVehicleQueries;

    fn vehicle_queries(&self) -> &Self::VQ {
        &self.vehicle_queries
    }
}

impl std::fmt::Debug for ScyllaQueries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScyllaQueries")
            //.field("session", &self.session)
            .field("vehicle_queries", &self.vehicle_queries)
            .finish()
    }
}
