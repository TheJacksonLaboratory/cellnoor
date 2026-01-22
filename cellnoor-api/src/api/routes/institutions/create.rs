use axum::{extract::State, http::StatusCode};
use cellnoor_models::institution::{Institution, InstitutionCreation};
use cellnoor_schema::institutions::dsl::institutions;
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

#[axum::debug_handler]
pub(super) async fn create_institution(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<InstitutionCreation>,
) -> ApiResponse<Institution> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<Institution> for InstitutionCreation {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(&self, _db_conn: db::DbConnection) -> Result<(), db::Error> {
        Ok(())
    }

    fn authorize(self, authorization_data: AuthorizationData) -> Result<Self, auth::Error> {
        if !authorization_data.is_admin() {
            return Err(auth::Error::PermissionDenied);
        }

        Ok(self)
    }

    fn validate(_authorized_request: &Self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    fn execute(
        authorized_request: Self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Institution, api::Error> {
        Ok(diesel::insert_into(institutions)
            .values(authorized_request)
            .returning(Institution::as_returning())
            .get_result(db_conn)?)
    }
}
