use axum::{extract::State, http::StatusCode};
use cellnoor_models::specimen::{Specimen, SpecimenCreation, SpecimenId};
use cellnoor_schema::specimens::dsl::{id, specimens};
use diesel::prelude::*;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, handle_request},
    },
    db,
    state::AppState,
};

pub(super) async fn create_specimen(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<SpecimenCreation>,
) -> ApiResponse<Specimen> {
    let item = handle_request(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Specimen> for SpecimenCreation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Specimen, db::Error> {
        let split = match self {
            Self::Block(s) => s.split_for_insertion(),
            Self::Suspension(s) => s.split_for_insertion(),
            Self::Tissue(s) => s.split_for_insertion(),
        };

        let created_id = diesel::insert_into(specimens)
            .values(split)
            .returning(id)
            .get_result(db_conn)?;

        SpecimenId(created_id).execute(db_conn)
    }
}
