use async_trait::async_trait;
use scylla::cql_to_rust::{FromCqlVal, FromRow};
use scylla::{prepared_statement::PreparedStatement, IntoTypedRows, Session};
use std::convert::TryFrom;
use std::str::FromStr;
use std::{string::ToString, sync::Arc};

use crate::{
    db::queries::VehicleQueries,
    error::AppError,
    model::vehicle::{Engine, EvData, Vehicle},
    result::AppResult,
};

pub struct ScyllaVehicleQueries {
    session: Arc<Session>,
    insert_vehicle_statement: PreparedStatement,
    select_vehicle_statement: PreparedStatement,
    delete_vehicle_statement: PreparedStatement,
}

impl std::fmt::Debug for ScyllaVehicleQueries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScyllaQueries").finish()
    }
}

impl ScyllaVehicleQueries {
    pub async fn try_new(session: Arc<Session>) -> AppResult<Self> {
        // Prepare "insert vehicle" statement
        let cql = format!(
            "INSERT INTO vehicles ({}) VALUES (?, ?, ?) IF NOT EXISTS",
            VehicleRow::FIELDS.join(",")
        );
        let insert_vehicle_statement = session.prepare(cql).await?;

        // Prepare "select vehicle" statement
        let cql = "SELECT * from vehicles where vin = ?";
        let select_vehicle_statement = session.prepare(cql).await?;

        // Prepare "delete vehicle" statement
        let cql = "DELETE from vehicles where vin = ?";
        let delete_vehicle_statement = session.prepare(cql).await?;

        Ok(ScyllaVehicleQueries {
            session,
            insert_vehicle_statement,
            select_vehicle_statement,
            delete_vehicle_statement,
        })
    }
}

#[async_trait]
impl VehicleQueries for ScyllaVehicleQueries {
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

    async fn delete_one_vehicle(&self, vin: &str) -> AppResult<()> {
        // Ensure that the vehicle can be found
        // TODO: check if the insert query has been applied, instead, as soon as lightweight transactions are supported
        // -> see https://github.com/scylladb/scylla-rust-driver/issues/100
        let _ = self.find_one_vehicle(vin).await?;

        self.session
            .execute(&self.delete_vehicle_statement, (vin,))
            .await
            .map_err(|_| AppError::NotFound("Vehicle"))?;

        Ok(())
    }
}

#[derive(PartialEq, scylla::FromRow, scylla::ValueList, field_names::FieldNames, Debug)]
struct VehicleRow {
    vin: String,
    engine_type: String,
    ev_data: Option<EvDataUserType>,
}

#[derive(PartialEq, scylla::FromUserType, scylla::IntoUserType, Debug)]
struct EvDataUserType {
    pub battery_capacity_in_kwh: i32,
    pub soc_in_percent: i32,
}

// Vehicle -> VehicleRow
impl From<Vehicle> for VehicleRow {
    fn from(vehicle: Vehicle) -> Self {
        let ev_data = vehicle.ev_data.as_ref().map(EvDataUserType::from);

        VehicleRow {
            vin: vehicle.vin,
            engine_type: vehicle.engine.to_string(),
            ev_data,
        }
    }
}

// &Vehicle -> VehicleRow
impl From<&Vehicle> for VehicleRow {
    fn from(vehicle: &Vehicle) -> Self {
        VehicleRow::from(vehicle.clone())
    }
}

// &VehicleRow -> Vehicle
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

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use crate::model::vehicle;

    use super::*;

    fn vehicle1() -> Vehicle {
        Vehicle {
            vin: "vin".to_string(),
            engine: vehicle::Engine::Combustion,
            ev_data: None,
        }
    }

    fn vehicle1_row() -> VehicleRow {
        VehicleRow {
            vin: "vin".to_string(),
            engine_type: "Combustion".to_string(),
            ev_data: None,
        }
    }

    fn vehicle2() -> Vehicle {
        Vehicle {
            vin: "vin".to_string(),
            engine: vehicle::Engine::Combustion,
            ev_data: Some(vehicle::EvData {
                battery_capacity_in_kwh: 69,
                soc_in_percent: 12,
            }),
        }
    }

    fn vehicle2_row() -> VehicleRow {
        VehicleRow {
            vin: "vin".to_string(),
            engine_type: "Combustion".to_string(),
            ev_data: Some(EvDataUserType {
                battery_capacity_in_kwh: 69,
                soc_in_percent: 12,
            }),
        }
    }

    fn invalid_vehicle_row() -> VehicleRow {
        VehicleRow {
            vin: "vin".to_string(),
            engine_type: "Invalid".to_string(),
            ev_data: None,
        }
    }

    #[tokio::test]
    async fn model_to_row() {
        assert_eq!(VehicleRow::from(vehicle1()), vehicle1_row());
        assert_eq!(VehicleRow::from(&vehicle1()), vehicle1_row());

        assert_eq!(VehicleRow::from(vehicle2()), vehicle2_row());
        assert_eq!(VehicleRow::from(&vehicle2()), vehicle2_row());
    }

    #[tokio::test]
    async fn row_to_model_ok() -> anyhow::Result<()> {
        assert_eq!(Vehicle::try_from(&vehicle1_row())?, vehicle1());
        assert_eq!(Vehicle::try_from(&vehicle2_row())?, vehicle2());

        Ok(())
    }

    #[tokio::test]
    async fn row_to_model_error() {
        // TODO: user assert_matches! when stable
        match Vehicle::try_from(&invalid_vehicle_row()) {
            Err(AppError::ConversionError(_)) => (),
            _ => assert!(false),
        }
    }
}
