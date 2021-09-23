use async_trait::async_trait;
use scylla::cql_to_rust::{FromCqlVal, FromRow};
use scylla::{prepared_statement::PreparedStatement, IntoTypedRows, Session};
use std::convert::TryFrom;
use std::str::FromStr;
use std::{string::ToString, sync::Arc};

use crate::{
    db::queries::Queries,
    error::AppError,
    result::AppResult,
    vehicle::{Engine, EvData, Vehicle},
};

pub struct ScyllaQueries {
    session: Arc<Session>,
    insert_vehicle_statement: PreparedStatement,
    select_vehicle_statement: PreparedStatement,
}

impl std::fmt::Debug for ScyllaQueries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScyllaQueries").finish()
    }
}

impl ScyllaQueries {
    pub async fn try_new(session: Arc<Session>, keyspace: String) -> AppResult<Self> {
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
        session.use_keyspace(&keyspace, false).await?;

        // Prepare "insert vehicle" statement
        let cql = format!(
            "INSERT INTO vehicles ({}) VALUES (?, ?, ?) IF NOT EXISTS",
            VehicleRow::FIELDS.join(",")
        );
        let insert_vehicle_statement = session.prepare(cql).await?;

        // Prepare "select vehicle" statement
        let cql = "SELECT * from vehicles where vin = ?";
        let select_vehicle_statement = session.prepare(cql).await?;

        Ok(ScyllaQueries {
            session,
            insert_vehicle_statement,
            select_vehicle_statement,
        })
    }
}

#[async_trait]
impl Queries for ScyllaQueries {
    async fn create_vehicle(&self, vehicle: &Vehicle) -> AppResult<()> {
        let row = VehicleRow::from(vehicle);

        // TODO: check if the insert query has been applied, instead, as soon as lightweight transactions are supported
        // -> see https://github.com/scylladb/scylla-rust-driver/issues/100
        if self.find_one_vehicle(&vehicle.vin).await.is_ok() {
            return Err(AppError::AlreadyExists("Vehicle"));
        }

        self.session
            .execute(&self.insert_vehicle_statement, &row)
            .await?;

        Ok(())
    }

    async fn find_one_vehicle(&self, vin: &str) -> Result<Vehicle, AppError> {
        let rows = self
            .session
            .execute(&self.select_vehicle_statement, (vin,))
            .await?
            .rows
            .ok_or(AppError::NotFound("Vehicle"))?;

        let first_vehicle_row = rows
            .into_typed::<VehicleRow>()
            .next()
            .ok_or(AppError::NotFound("Vehicle"))??;

        Vehicle::try_from(&first_vehicle_row)
    }
}

#[derive(scylla::FromRow, scylla::ValueList, field_names::FieldNames, Debug)]
struct VehicleRow {
    vin: String,
    engine_type: String,
    ev_data: Option<EvDataUserType>,
}

#[derive(scylla::FromUserType, scylla::IntoUserType, Debug)]
struct EvDataUserType {
    pub battery_capacity_in_kwh: i32,
    pub soc_in_percent: i32,
}

// Vehicle -> VehicleRow
impl From<&Vehicle> for VehicleRow {
    fn from(vehicle: &Vehicle) -> Self {
        let ev_data = vehicle.ev_data.as_ref().map(EvDataUserType::from);

        VehicleRow {
            vin: vehicle.vin.clone(),
            engine_type: vehicle.engine.to_string(),
            ev_data,
        }
    }
}

// VehicleRow -> Vehicle
impl TryFrom<&VehicleRow> for Vehicle {
    type Error = AppError;

    fn try_from(vehicle_row: &VehicleRow) -> Result<Self, Self::Error> {
        let engine = Engine::from_str(&vehicle_row.engine_type)
            .map_err(|_| AppError::ConversionError("VehicleRow to Vehicle"))?;

        let ev_data = vehicle_row
            .ev_data
            .as_ref()
            .map(EvData::try_from)
            .transpose()?;

        Ok(Vehicle {
            vin: vehicle_row.vin.clone(),
            engine,
            ev_data,
        })
    }
}

// EvData -> EvDataUserType
impl From<&EvData> for EvDataUserType {
    fn from(ev_data: &EvData) -> Self {
        EvDataUserType {
            battery_capacity_in_kwh: ev_data.battery_capacity_in_kwh,
            soc_in_percent: ev_data.soc_in_percent,
        }
    }
}

// EvDataUserType -> EvData
impl TryFrom<&EvDataUserType> for EvData {
    type Error = AppError;

    fn try_from(ev_data_user_type: &EvDataUserType) -> Result<Self, Self::Error> {
        Ok(EvData {
            battery_capacity_in_kwh: ev_data_user_type.battery_capacity_in_kwh,
            soc_in_percent: ev_data_user_type.soc_in_percent,
        })
    }
}
