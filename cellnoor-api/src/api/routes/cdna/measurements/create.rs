use axum::{extract::State, http::StatusCode};
use cellnoor_models::cdna::{
    CdnaIdMeasurements,
    measurement::{CdnaMeasurement, CdnaMeasurementCreation},
};
use cellnoor_schema::cdna_measurements;
use diesel::{RunQueryDsl, prelude::*};

use crate::{
    api::{
        extract::{ValidPathJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_measurement(
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidPathJson(cdna_id, measurement): ValidPathJson<CdnaIdMeasurements, CdnaMeasurementCreation>,
) -> ApiResponse<CdnaMeasurement> {
    let item = inner_handler(state, user, (cdna_id, measurement)).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<CdnaMeasurement> for (CdnaIdMeasurements, CdnaMeasurementCreation) {
    fn execute(self, db_conn: &mut PgConnection) -> Result<CdnaMeasurement, db::Error> {
        let (cdna_id, measurement) = self;

        Ok(diesel::insert_into(cdna_measurements::table)
            .values((cdna_measurements::cdna_id.eq(cdna_id), measurement))
            .returning(CdnaMeasurement::as_returning())
            .get_result(db_conn)?)
    }
}
