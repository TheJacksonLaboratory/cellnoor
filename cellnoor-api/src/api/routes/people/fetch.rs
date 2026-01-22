use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::person::{Person, PersonId};
use cellnoor_schema::people::dsl::id;
use diesel::{PgConnection, prelude::*};

use super::{ApiResponse, handle_api_request};
use crate::{
    api::{
        self,
        auth::{self, AuthorizationData},
        extract::auth::AuthenticatedUser,
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_person(
    request: PersonId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Person> {
    let item = handle_api_request(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<Person> for PersonId {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _db_conn: db::DbConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        Ok(())
    }

    fn authorize(
        self,
        _authorization_data: AuthorizationData,
    ) -> Result<Self::Authorized, auth::Error> {
        Ok(self)
    }

    fn validate(_authorized_request: &Self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    fn execute(person_id: Self, db_conn: &mut PgConnection) -> Result<Person, api::Error> {
        Ok(Person::query().filter(id.eq(person_id)).first(db_conn)?)
    }
}
