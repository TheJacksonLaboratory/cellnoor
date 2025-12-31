use axum::extract::State;
use cellnoor_models::cdna::{CdnaIdMeasurements, measurement::CdnaMeasurement};
use cellnoor_schema::cdna_measurements;
use diesel::prelude::*;
use reqwest::StatusCode;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db::{self},
    state::AppState,
};

pub async fn list_measurements(
    cdna_id: CdnaIdMeasurements,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<CdnaMeasurement>> {
    let item = inner_handler(state, user, cdna_id).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Vec<CdnaMeasurement>> for CdnaIdMeasurements {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<CdnaMeasurement>, db::Error> {
        Ok(CdnaMeasurement::query()
            .order_by(cdna_measurements::measured_at)
            .filter(cdna_measurements::cdna_id.eq(self))
            .load(db_conn)?)
    }
}
