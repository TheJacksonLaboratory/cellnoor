use axum::{extract::State, http::StatusCode};
use cellnoor_models::institution::{Institution, InstitutionCreation};
use cellnoor_schema::institutions::dsl::institutions;
use diesel::prelude::*;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn create_institution(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<InstitutionCreation>,
) -> ApiResponse<Institution> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Institution> for InstitutionCreation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Institution, db::Error> {
        Ok(diesel::insert_into(institutions)
            .values(self)
            .returning(Institution::as_returning())
            .get_result(db_conn)?)
    }
}
