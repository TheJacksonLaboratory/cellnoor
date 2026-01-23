use axum::{extract::State, http::StatusCode};
use cellnoor_models::specimen::{Specimen, SpecimenId};
use cellnoor_schema::specimens::dsl::{id, project_id};
use diesel::prelude::*;

use crate::{
    api::{
        auth::{self, AuthorizationData},
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, handle_api_request},
    },
    db::{self, DbConnection},
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
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _state: &DbConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        Ok(())
    }

    fn execute(
        self,
        authorization: AuthorizationData,
        _validation_data: &(),
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Specimen, db::Error> {
        let q = Specimen::query().filter(id.eq(self));

        let specimen = match authorization.authorized_projects(None) {
            Some(projects) => q.filter(project_id.eq_any(projects)).first(db_conn)?,
            None => q.first(db_conn)?,
        };

        Ok(specimen)
    }
}
