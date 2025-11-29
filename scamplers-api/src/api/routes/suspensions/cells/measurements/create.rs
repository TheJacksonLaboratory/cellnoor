use axum_extra::routing::TypedPath;
use diesel::prelude::*;
use scamplers_models::suspension::measurements::{
    CellSuspensionMeasurementCreation, SuspensionMeasurement,
};
use scamplers_schema::suspension_measurements::dsl::*;

use crate::{api::extract::auth::AuthenticatedUser, db};

pub(super) async fn create_cell_suspension_measurement(
    _: MeasurementEndpoint,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<CellSuspensionMeasurementCreation>,
) -> ApiResponse<SuspensionMeasurement> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<SuspensionMeasurement> for CellSuspensionMeasurementCreation {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<SuspensionMeasurement, db::Error> {
        Ok(diesel::insert_into(suspension_measurements)
            .values(self.0)
            .returning(SuspensionMeasurement::as_returning())
            .get_result(db_conn))
    }
}
