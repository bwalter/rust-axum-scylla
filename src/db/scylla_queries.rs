use async_oncecell::OnceCell;
use async_trait::async_trait;
use scylla::{
    frame::value::MaybeUnset, prepared_statement::PreparedStatement, transport::errors::QueryError,
    IntoTypedRows, Session,
};
use std::sync::Arc;

use crate::{
    db::queries::Queries,
    error::AppError,
    result::AppResult,
    vehicle::{Vehicle, VehicleRow},
};

type PreparedQueryResult = Result<Arc<PreparedStatement>, Arc<QueryError>>;

pub struct ScyllaQueries {
    session: Arc<Session>,
    pub create_vehicle: OnceCell<PreparedQueryResult>,
    pub find_vehicles: OnceCell<PreparedQueryResult>,
}

impl ScyllaQueries {
    pub fn new(session: Arc<Session>) -> Self {
        ScyllaQueries {
            session,
            create_vehicle: OnceCell::new(),
            find_vehicles: OnceCell::new(),
        }
    }
}

#[async_trait]
impl Queries for ScyllaQueries {
    async fn create_tables_if_not_exist(&self) -> AppResult<()> {
        self.session
            .query(
                "CREATE KEYSPACE IF NOT EXISTS hello WITH REPLICATION = \
            {'class' : 'SimpleStrategy', 'replication_factor' : 1}",
                &[],
            )
            .await?;

        self.session.use_keyspace("hello", false).await?;

        self.session
        .query(
            "CREATE TYPE IF NOT EXISTS ev_data (battery_capacity_in_kwh int, soc_in_percent int)",
            &[],
        )
        .await?;

        self.session
        .query(
            "CREATE TABLE IF NOT EXISTS vehicles (vin text primary key, engine_type text, ev_data ev_data)",
            &[],
        )
        .await?;

        Ok(())
    }

    async fn create_vehicle(&self, vehicle: &Vehicle) -> AppResult<()> {
        let prepared_statement = self
            .get_prepared_statement_helper(
                &self.create_vehicle,
                "INSERT INTO vehicles (vin, engine_type, ev_data) VALUES (?, ?, ?) IF NOT EXISTS",
            )
            .await?;

        let row = VehicleRow::from_vehicle(vehicle.clone());
        let ev_data = if let Some(ref d) = vehicle.ev_data {
            MaybeUnset::Set(d)
        } else {
            MaybeUnset::Unset
        };

        // TODO: check if query has been applied (lightweight transaction support)
        // if not, return QueryResult::
        // -> see https://github.com/scylladb/scylla-rust-driver/issues/100
        self.session
            .execute(&prepared_statement, (&row.vin, &row.engine_type, &ev_data))
            .await?;

        Ok(())
    }

    async fn find_one_vehicle(&self, vin: &str) -> Result<Vehicle, AppError> {
        let prepared_statement = self
            .get_prepared_statement_helper(
                &self.create_vehicle,
                "SELECT * FROM vehicles WHERE vin = ?",
            )
            .await?;

        let rows = self
            .session
            .execute(&prepared_statement, (vin,))
            .await?
            .rows
            .ok_or_else(|| AppError::NotFound())?;

        let first_vehicle_row = rows
            .into_typed::<VehicleRow>()
            .next()
            .ok_or_else(|| AppError::NotFound())??;

        let vehicle = first_vehicle_row
            .to_vehicle()
            .ok_or_else(|| AppError::NotFound())?;

        Ok(vehicle)
    }
}

impl ScyllaQueries {
    async fn get_prepared_statement_helper<'a>(
        &'a self,
        query_once_cell: &'a OnceCell<PreparedQueryResult>,
        query_str: &'static str,
    ) -> Result<&'a PreparedStatement, AppError> {
        query_once_cell
            .get_or_init(async {
                let statement = self
                    .session
                    .prepare(query_str)
                    .await
                    .map_err(|e| Arc::new(e))?;
                Ok(Arc::new(statement))
            })
            .await
            .as_deref()
            .map_err(|e| Arc::clone(e).into())
    }
}
