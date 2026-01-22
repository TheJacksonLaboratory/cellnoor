use axum::{extract::State, http::StatusCode};
use cellnoor_models::institution::{Institution, InstitutionId};
use cellnoor_schema::institutions::dsl::id;
use diesel::{PgConnection, prelude::*};

use crate::{
    api::{
        self,
        auth::{self, AuthorizationData},
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, handle_api_request},
    },
    db::{self},
    state::AppState,
};

pub(super) async fn fetch_institution(
    request: InstitutionId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Institution> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Institution> for InstitutionId {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _db_conn: db::DbConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        Ok(())
    }

    fn authorize(self, _authorization_data: AuthorizationData) -> Result<Self, auth::Error> {
        Ok(self)
    }

    fn validate(_authorized_request: &Self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    fn execute(
        institution_id: Self,
        db_conn: &mut PgConnection,
    ) -> Result<Institution, api::Error> {
        Ok(Institution::query()
            .filter(id.eq(institution_id))
            .first(db_conn)?)
    }
}
