use axum::{extract::State, http::StatusCode};
use cellnoor_models::specimen::{Specimen, SpecimenId};
use cellnoor_schema::specimens::dsl::id;
use diesel::prelude::*;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, handle_api_request},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_specimen(
    specimen_id: SpecimenId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Specimen> {
    let item = handle_api_request(state, user, specimen_id).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Specimen> for SpecimenId {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Specimen, db::Error> {
        Ok(Specimen::query().filter(id.eq(self)).first(db_conn)?)
    }
}
