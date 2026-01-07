use axum::{extract::State, http::StatusCode};
use cellnoor_models::specimen::{Specimen, SpecimenCreation, SpecimenId};
use cellnoor_schema::specimens::dsl::{id, specimens};
use diesel::prelude::*;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
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
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Specimen> for SpecimenCreation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Specimen, db::Error> {
        let split = match self {
            Self::FixedBlock(s) => s.split_for_insertion(),
            Self::FlashFrozenBlock(s) => s.split_for_insertion(),
            Self::CryopreservedSuspension(s) => s.split_for_insertion(),
            Self::NonCryopreservedSuspsension(s) => s.split_for_insertion(),
            Self::CryopreservedTissue(s) => s.split_for_insertion(),
            Self::NonCryopreservedTissue(s) => s.split_for_insertion(),
        };

        let created_id = diesel::insert_into(specimens)
            .values(split)
            .returning(id)
            .get_result(db_conn)?;

        SpecimenId(created_id).execute(db_conn)
    }
}
