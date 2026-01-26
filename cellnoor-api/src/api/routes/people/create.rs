use crate::api::auth::{self};
use crate::api::extract::Json;
use crate::api::{self};
use crate::{api::extract::auth::AuthenticatedUser, db, state::AppState};
use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::person::PersonUpdate;
use cellnoor_models::person::{Person, PersonCreation, PersonId};
use cellnoor_schema::people::dsl::{id, people};
use diesel::{
    prelude::*,
    sql_types::{Array, Text},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use regex::Regex;
use std::sync::LazyLock;

impl api::AuthorizedRequest<Person> for PersonCreation {
    type ValidationData = String;

    fn validate(&self, email: String) -> Result<(), api::DataError> {
        Ok(validate_email(&email)?)
    }

    async fn handle(self, mut db_conn: &AsyncPgConnection) -> Result<Person, api::Error> {
        // Get the ID of the inserted person first, then return the full `Person` struct
        let created_id: PersonId = diesel::insert_into(people)
            .values(self)
            .returning(id)
            .get_result(&mut db_conn)
            .await?;

        created_id.handle(&mut db_conn).await
    }
}

impl api::Request<Person> for PersonCreation {
    type Authorized = Self;
    type ValidationData = String;

    async fn fetch_validation_data(
        &self,
        _db_conn: &AsyncPgConnection,
    ) -> Result<String, db::Error> {
        // Ideally we could return a reference but that doesn't work (unless I don't know what I'm doing)
        Ok(self.email().to_owned())
    }

    fn authorize(self, user: AuthenticatedUser) -> Result<Self, auth::Error> {
        user.authorize_admin_only()?;

        Ok(self)
    }
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "PersonValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error("{email} invalid: {message}")]
pub enum Error {
    Email { email: String, message: String },
}

// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

pub(super) fn validate_email(email: &str) -> Result<(), Error> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(Error::Email {
            email: email.to_owned(),
            message: "invalid email".to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::EMAIL_REGEX;

    #[rstest]
    fn valid_email() {
        assert!(EMAIL_REGEX.is_match("peter.parker@spiderman.avengers"))
    }

    #[rstest]
    fn email_has_no_domain() {
        assert!(!EMAIL_REGEX.is_match("SpongeBob"))
    }

    #[rstest]
    fn email_contains_space() {
        assert!(!EMAIL_REGEX.is_match("Harry Potter"))
    }
}
