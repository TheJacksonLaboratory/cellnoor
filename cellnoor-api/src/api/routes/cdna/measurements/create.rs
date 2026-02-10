use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    cdna::measurement::{CdnaMeasurement, NewCdnaMeasurement},
    nucleic_acid_measurement::NucleicAcidMeasurementData,
};
use cellnoor_schema::{cdna, cdna_measurements};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::util::validate_timestamps,
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_cdna_measurement(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(measurement): Json<NewCdnaMeasurement>,
) -> Result<Json<CdnaMeasurement>, db::Error> {
    validate_electrophoretic_measurement(measurement.data())?;

    let prepared_at = cdna_prepared_at(id, &db_conn).await?;

    validate_timestamps(
        (prepared_at, "cdna_prepared_at"),
        (measurement.measured_at(), "measurement_made_at"),
    )?;

    insert_cdna_measurement(id, measurement, &db_conn)
        .await
        .map(Json)
}

async fn cdna_prepared_at(
    cdna_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Timestamp, db::Error> {
    Ok(cdna::table
        .select(cdna::prepared_at)
        .find(cdna_id)
        .first(&mut db_conn)
        .await
        .map(jiff_diesel::Timestamp::to_jiff)?)
}

pub async fn insert_cdna_measurement(
    cdna_id: Uuid,
    measurement: NewCdnaMeasurement,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<CdnaMeasurement, db::Error> {
    Ok(diesel::insert_into(cdna_measurements::table)
        .values((cdna_measurements::cdna_id.eq(cdna_id), measurement))
        .returning(CdnaMeasurement::as_returning())
        .get_result(&mut db_conn)
        .await?)
}

pub fn validate_electrophoretic_measurement(
    measurement_data: &NucleicAcidMeasurementData,
) -> Result<(), db::DataError> {
    match measurement_data {
        NucleicAcidMeasurementData::Electrophoretic {
            instrument_name: _,
            mean_size_bp: _,
            sizing_range: (min, max),
            concentration: _,
        } => {
            if min > max {
                return Err(db::DataError::new_other(
                    "sizing range minimum must be less than sizing range maximum, but found \
                     ({min}, {max})",
                ))?;
            }
            Ok(())
        }
        NucleicAcidMeasurementData::Fluorometric {
            instrument_name: _,
            concentration: _,
        } => Ok(()),
    }
}
