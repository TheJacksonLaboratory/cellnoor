use axum::extract::State;
use cellnoor_models::specimen::{Specimen, SpecimenId};
use cellnoor_schema::specimens::dsl::id;
use diesel::prelude::*;
use reqwest::StatusCode;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_specimen(
    specimen_id: SpecimenId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Specimen> {
    let item = inner_handler(state, user, specimen_id).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Specimen> for SpecimenId {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Specimen, db::Error> {
        Ok(Specimen::query().filter(id.eq(self)).first(db_conn)?)
    }
}
