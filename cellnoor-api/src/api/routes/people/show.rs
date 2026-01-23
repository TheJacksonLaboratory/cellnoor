use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::person::{Person, PersonId};
use cellnoor_schema::people::dsl::id;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::{
    api::{
        self,
        auth::{self, Authorization},
        extract::auth::AuthenticatedUser,
    },
    db,
    state::AppState,
};

impl api::AuthorizedRequest<Person> for PersonId {
    type ValidationData = ();

    fn validate(&self, _validation_data: ()) -> Result<(), api::DataError> {
        Ok(())
    }

    async fn handle(self, mut db_conn: &AsyncPgConnection) -> Result<Person, api::Error> {
        Ok(Person::query()
            .filter(id.eq(self))
            .first(&mut db_conn)
            .await?)
    }
}

impl api::Request<Person> for PersonId {
    type Authorized = Self;
    type ValidationData = ();

    async fn fetch_validation_data(
        &self,
        _db_conn: &AsyncPgConnection,
    ) -> Result<Self::ValidationData, db::Error> {
        Ok(())
    }

    fn authorize(self, _authorization: Authorization) -> Result<Self::Authorized, auth::Error> {
        Ok(self)
    }
}
