use axum::{extract::State, http::StatusCode};
use diesel::{RunQueryDsl, prelude::*};
use scamplers_models::cdna::{
    CdnaIdMeasurements,
    measurement::{CdnaMeasurement, CdnaMeasurementCreation},
};
use scamplers_schema::cdna_measurements;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn create_measurement(
    specimen_id: CdnaIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<CdnaMeasurementCreation>,
) -> ApiResponse<CdnaMeasurement> {
    let item = inner_handler(state, user, (specimen_id, request)).await?;
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
