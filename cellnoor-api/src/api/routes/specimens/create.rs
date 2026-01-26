use axum::{extract::State, http::StatusCode};
use cellnoor_models::specimen::{Specimen, SpecimenCreation, SpecimenId};
use cellnoor_schema::specimens::dsl::{id, specimens};
use diesel::prelude::*;

use crate::{
    api::{
        self,
        auth::{self, AuthorizationData},
        extract::{Json, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, handle_api_request},
    },
    db,
    state::AppState,
};

pub(super) async fn create_specimen(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<SpecimenCreation>,
) -> ApiResponse<Specimen> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Specimen> for SpecimenCreation {
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _db_conn: &db::DbConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        Ok(())
    }

    fn execute(
        self,
        user: AuthenticatedUserData,
        _validation_data: (),
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Specimen, api::Error> {
        if !authorization.is_admin() {
            return Err(auth::Error::PermissionDenied)?;
        }

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
