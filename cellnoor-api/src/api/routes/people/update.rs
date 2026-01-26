use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::person::{Person, PersonId, PersonUpdate};
use diesel::{
    prelude::*,
    sql_types::{Array, Text},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use super::create::validate_email;
use crate::{
    api::{
        self,
        auth::{self},
        extract::{Json, auth::AuthenticatedUser},
    },
    db,
    state::AppState,
};

impl api::AuthorizedRequest<Person> for (PersonId, PersonUpdate) {
    type ValidationData = Option<String>;

    fn validate(&self, email: Option<String>) -> Result<(), api::DataError> {
        let Some(email) = email else {
            return Ok(());
        };

        Ok(validate_email(&email)?)
    }

    async fn handle(self, mut db_conn: &AsyncPgConnection) -> Result<Person, api::Error> {
        let (person_id, mut update) = self;
        update.set_id(person_id.0);

        diesel::update(&update)
            .set(&update)
            .execute(&mut db_conn)
            .await?;

        person_id.handle(db_conn).await
    }
}

impl api::Request<Person> for (PersonId, PersonUpdate) {
    type Authorized = Self;
    type ValidationData = Option<String>;

    async fn fetch_validation_data(
        &self,
        _db_conn: &AsyncPgConnection,
    ) -> Result<Option<String>, db::Error> {
        let (_person_id, update_data) = &self;
        // Ideally we could return a reference but that doesn't work (unless I don't know what I'm doing)
        Ok(update_data.email().map(str::to_owned))
    }

    fn authorize(self, user: AuthenticatedUser) -> Result<Self, auth::Error> {
        user.authorize_admin_only()?;

        Ok(self)
    }
}
